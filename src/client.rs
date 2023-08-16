use async_trait::async_trait;
use futures_util::TryFutureExt;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;
use tokio::spawn;
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tokio::task::JoinHandle;

use crate::api::{AuthApi, LoginArgs};
use crate::connection::{Command, MessageSink, MessageStream, Response, ResponseError, XtbErrorCode, XtbServerConnection, XtbServerConnectionError, XtbSink, XtbStream};

pub struct XtbClient {
    api_sink: XtbSink,
    api_stream_handle: JoinHandle<()>,
    api_channel: (Sender<Response>, Receiver<Response>),
    stream_api_sink: XtbSink,
    stream_api_stream_handle: JoinHandle<()>,
    stream_api_channel: (Sender<Response>, Receiver<Response>),
    last_idx: u64,
    stream_session_id: Option<String>
}


impl XtbClient {
    pub async fn new(api_connection: XtbServerConnection, stream_api_connection: XtbServerConnection) -> Self {
        let (api_sink, api_stream) = api_connection.split();
        let (stream_api_sink, stream_api_stream) = stream_api_connection.split();

        let api_channel = channel::<Response>(256);
        let stream_api_channel = channel::<Response>(256);

        let api_stream_handle = spawn(receive(api_stream, api_channel.0.clone()));
        let stream_api_stream_handle = spawn(receive(stream_api_stream, stream_api_channel.0.clone()));

        Self {
            api_sink,
            api_stream_handle,
            api_channel,
            stream_api_sink,
            stream_api_stream_handle,
            stream_api_channel,
            last_idx: 0,
            stream_session_id: None,
        }
    }

    pub fn generate_unique_tag(&mut self) -> String {
        self.last_idx += 1;
        format!("xtb_client_tag_{}", self.last_idx)
    }


    pub async fn send_command<T: Serialize + Send>(&mut self, cmd: Command<T>) -> Result<(), XtbClientError> {
        self.api_sink
            .send(cmd)
            .await
            .map_err(|err| XtbClientError::UnableToSendCommand(err))
    }

    pub async fn wait_for_response(&self, custom_tag: String) -> Result<Response, XtbClientError> {
        let mut receiver = self.api_channel.0.subscribe();
        while let Ok(resp) = receiver.recv().await {
            if let Some(resp_custom_tag) = resp.get_custom_tag()? {
                if resp_custom_tag != custom_tag {
                    continue;
                }
                return Ok(resp);
            }
        }
        Err(XtbClientError::CannotReceiveResponse)
    }
}


async fn receive(stream: XtbStream, sender: Sender<Response>) {
    let mut stream = stream;
    while let Ok(resp) = stream.receive().await {
        sender.send(resp).unwrap();
    }
}


#[async_trait]
impl AuthApi for XtbClient {
    type Error = XtbClientError;

    async fn login(&mut self, args: LoginArgs) -> Result<(), XtbClientError> {
        let mut cmd = Command::new("login", Some(args));
        let tag = self.generate_unique_tag();
        cmd.custom_tag = Some(tag.clone());
        self.send_command(cmd).await?;
        let response = self.wait_for_response(tag).await?;

        if response.is_ok()? {
            let value: Value = response.into();
            let value = value.as_object().unwrap().get("stream_session_id").unwrap();
            self.stream_session_id = Some(value.as_str().unwrap().to_owned());
            Ok(())
        } else {
            let error_response = response.unpack_command_result::<()>().map_err(|err| XtbClientError::ResponseError(err))?.err().unwrap();
            Err(XtbClientError::CommandFailed(error_response.error_code, error_response.error_description))
        }
    }

    async fn logout(&mut self) -> Result<(), XtbClientError> {
        todo!()
    }
}


#[derive(Debug, Error)]
pub enum XtbClientError {
    #[error("unable to send command")]
    UnableToSendCommand(XtbServerConnectionError),
    #[error("Response cannot be received")]
    CannotReceiveResponse,
    #[error("Command failed to be done")]
    CommandFailed(XtbErrorCode, String),
    #[error("Invalid response format")]
    ResponseError(ResponseError),
}


impl From<ResponseError> for XtbClientError {
    fn from(value: ResponseError) -> Self {
        Self::ResponseError(value)
    }
}
