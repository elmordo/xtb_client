use std::net::TcpStream;
use thiserror::Error;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, WebSocket, Error as TungsteniteError};
use url::{Url, ParseError};

/// Hold configuration for Socket
pub struct SocketConfig {
    /// Should be always wss
    pub protocol: String,
    /// Server host
    pub host: String,
    /// Endpoint on the host where to connect
    pub endpoint: String,
}


pub struct Socket {
    /// Underlying tungstenite socket connection
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
}


impl Socket {
    pub fn new(config: SocketConfig) -> Result<Self, SocketError> {
        let url = Self::build_url(config)?;
        let (socket, _) = connect(url).map_err(|err| SocketError::ConnectionError(err))?;
        Ok(Self {
            socket
        })
    }

    fn build_url(config: SocketConfig) -> Result<Url, SocketError> {
        let url_base = format!("{}://{}/{}", config.protocol, config.host, config.endpoint);
        Ok(Url::parse(&url_base)?)
    }
}

#[derive(Debug, Error)]
pub enum SocketError {
    #[error("URL is malformed")]
    InvalidUrl(ParseError),
    #[error("Unable to connect")]
    ConnectionError(TungsteniteError)
}


impl From<ParseError> for SocketError {
    fn from(value: ParseError) -> Self {
        Self::InvalidUrl(value)
    }
}
