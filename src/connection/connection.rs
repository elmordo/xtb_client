use futures_util::{Sink, SinkExt};
use http::Uri;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_websockets::{ClientBuilder, Error as WSError, MaybeTlsStream, Message, WebsocketStream};
use tokio_websockets::proto::ProtocolError;

use crate::connection::command::{Command, CommandError, CommandOk, CommandResult};

type WSStream = WebsocketStream<MaybeTlsStream<TcpStream>>;


/// Wrap connection (normal or streaming) to the XTB server
pub struct XtbServerConnection {
    stream: WSStream,
}


/// Connection to the XTB trading server
impl XtbServerConnection {
    /// Create new connection based on uri
    pub async fn new(uri: Uri) -> Result<Self, XtbServerConnectionError> {
        let stream = ClientBuilder::from_uri(uri).connect().await.map_err(|err| XtbServerConnectionError::UnableToConnect(err))?;
        Ok(Self {
            stream
        })
    }

    /// Send command to the server
    pub async fn send<T: Serialize>(&mut self, command: Command<T>) -> Result<(), XtbServerConnectionError> {
        let payload = serde_json::to_string(&command)?;
        self.stream.send(Message::text(payload)).await.map_err(|err| XtbServerConnectionError::UnableToSendMessage(err))?;
        Ok(())
    }

    /// Read response from the server
    pub async fn read(&mut self) -> Result<Response, XtbServerConnectionError> {
        let body = self
            .stream
            .next()
            .await
            .unwrap()
            .map_err(|err| XtbServerConnectionError::UnableToReceiveMessage(err))?
            .as_text()
            .map_err(|err| XtbServerConnectionError::UnableToDecodeMessage(err))?
            .to_owned();
        Ok(Response::from(body))
    }
}


/// The command response with unknown type
pub struct Response {
    value: Value,
}


impl Response {
    /// Return true, if response `status` field  has value `true`. Return `false` otherwise.
    pub fn is_ok(&self) -> Result<bool, ResponseError> {
        let main_obj = self.get_main_object()?;
        main_obj.get("status")
            .ok_or_else(|| ResponseError::InvalidFormat)
            .and_then(|val| {
                match val {
                    Value::Bool(is_ok) => Ok(is_ok.clone()),
                    _ => Err(ResponseError::InvalidFormat)
                }
            })
    }

    /// Return value of the `custom_tag` field or None if no `custom_tag` field was not found
    pub fn get_custom_tag(&self) -> Result<Option<String>, ResponseError> {
        let main_obj = self.get_main_object()?;
        match main_obj.get("custom_tag") {
            Some(val) => match val {
                Value::Null => Ok(None),
                Value::String(tag) => Ok(Some(tag.clone())),
                _ => Err(ResponseError::InvalidFormat),
            },
            None => Ok(None),
        }
    }

    /// Consume `Response` and return `CommandResult` constructed from the response
    pub fn unpack_command_result<'de, T: Deserialize<'de>>(self) -> Result<CommandResult<T>, ResponseError> {
        if self.is_ok()? {
            Ok(Ok(CommandOk::deserialize(self.value).map_err(|err| ResponseError::DeserializationError(err))?))
        } else {
            Ok(Err(CommandError::deserialize(self.value).map_err(|err| ResponseError::DeserializationError(err))?))
        }
    }

    /// Get main object of the response
    fn get_main_object(&self) -> Result<&Map<String, Value>, ResponseError> {
        match &self.value {
            Value::Object(obj) => Ok(obj),
            _ => Err(ResponseError::InvalidFormat)
        }
    }
}


impl From<String> for Response {
    fn from(value: String) -> Self {
        Self {
            value: serde_json::from_str(&value).unwrap()
        }
    }
}


#[derive(Debug, Error)]
pub enum XtbServerConnectionError {
    #[error("Unable to connect to the server")]
    UnableToConnect(WSError),
    #[error("Unable to send message")]
    UnableToSendMessage(WSError),
    #[error("Unable to receive message")]
    UnableToReceiveMessage(WSError),
    #[error("Unable to serialize value")]
    SerializationError(serde_json::Error),
    #[error("Unable to decode message")]
    UnableToDecodeMessage(ProtocolError),
}


impl From<serde_json::Error> for XtbServerConnectionError {
    fn from(value: serde_json::Error) -> Self {
        Self::SerializationError(value)
    }
}


#[derive(Debug, Error)]
pub enum ResponseError {
    #[error("Response object is in invalid format")]
    InvalidFormat,
    #[error("Unable to deserialize")]
    DeserializationError(serde_json::Error),
}
