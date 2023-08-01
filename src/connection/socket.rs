use std::net::TcpStream;
use std::time::Duration;
use thiserror::Error;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, WebSocket, Error as TungsteniteError, Message};
use url::{Url, ParseError};
use rxrust::prelude::*;

/// Hold configuration for Socket
pub struct SocketConfig {
    /// Should be always wss
    pub protocol: String,
    /// Server host
    pub host: String,
    /// Endpoint on the host where to connect
    pub endpoint: String,
    /// Socket read timeout in ms
    pub read_timeout: Option<u64>,
}


pub struct Socket {
    /// Underlying tungstenite socket connection
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    message_feeder: Subject<'_, Message, SocketError>,
}


impl Socket {
    pub fn new(config: SocketConfig) -> Result<Self, SocketError> {
        let socket = Self::make_socket(config)?;
        let message_feeder: Subject<Message, SocketError> = Subject::default();
        Ok(Self {
            socket,
            message_feeder,
        })
    }

    pub fn send(&mut self, message: Message) -> Result<(), SocketError> {
        self.socket.send(message).map_err(|err| SocketError::UnableToSendMessage(err))
    }

    fn make_socket(config: SocketConfig) -> Result<WebSocket<MaybeTlsStream<TcpStream>>, SocketError> {
        let url = Self::build_url(&config)?;
        let (mut socket, _) = connect(url).map_err(|err| SocketError::UnableToConnect(err))?;
        if let Some(timeout) = config.read_timeout {
            Self::set_timeout(&mut socket, timeout)?;
        }
        Ok(socket)
    }

    fn build_url(config: &SocketConfig) -> Result<Url, SocketError> {
        let url_base = format!("{}://{}/{}", config.protocol, config.host, config.endpoint);
        Ok(Url::parse(&url_base)?)
    }

    fn set_timeout(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>, timeout: u64) -> Result<(), SocketError> {
        let mut socket = socket;
        let dur = Some(Duration::from_millis(timeout));
        match socket.get_mut() {
            MaybeTlsStream::Plain(s) => s.set_read_timeout(dur),
            MaybeTlsStream::Rustls(s) => s.get_mut().set_read_timeout(dur),
            _ => Ok(())
        }.map_err(|_| SocketError::UnableToSetTimeout)?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum SocketError {
    #[error("URL is malformed")]
    InvalidUrl(ParseError),
    #[error("Unable to connect")]
    UnableToConnect(TungsteniteError),
    #[error("Unable to connect")]
    UnableToSendMessage(TungsteniteError),
    #[error("Unable to set timeout")]
    UnableToSetTimeout,
}


impl From<ParseError> for SocketError {
    fn from(value: ParseError) -> Self {
        Self::InvalidUrl(value)
    }
}
