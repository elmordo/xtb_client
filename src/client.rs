use std::fmt::Error;

use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Error as WSError;
use url::Url;

use crate::api::{AccountApi, CommandResult};

pub struct XtbClient {}


impl XtbClient {
    pub fn new(
        api_socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
        stream_api_socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    ) -> Self {
        todo!()
    }
}


impl AccountApi for XtbClient {
    type Error = XtbClientError;

    fn login(&mut self, account_id: &str, password: &str) -> Result<CommandResult<()>, Error> {
        todo!()
    }

    fn logout(&mut self) -> Result<CommandResult<()>, Error> {
        todo!()
    }
}


pub struct XtbClientBuilder {
    protocol: Option<String>,
    host: String,
    port: Option<u32>,
    api_endpoint: String,
    stream_api_endpoint: String,
}


impl XtbClientBuilder {
    pub fn xtb_demo(self) -> Self {
        self
            .protocol(None)
            .host("ws.xtb.com".to_owned())
            .port(None)
            .api_endpoint("/demo".to_owned())
            .stream_api_endpoint("/demoStream".to_owned())
    }

    pub fn xtb_real(self) -> Self {
        self
            .xtb_demo()
            .api_endpoint("/real".to_owned())
            .stream_api_endpoint("/realStream".to_owned())
    }

    pub fn protocol(mut self, val: Option<String>) -> Self {
        self.protocol = val;
        self
    }

    pub fn host(mut self, val: String) -> Self {
        self.host = val;
        self
    }

    pub fn port(mut self, val: Option<u32>) -> Self {
        self.port = val;
        self
    }

    pub fn api_endpoint(mut self, val: String) -> Self {
        self.api_endpoint = val;
        self
    }

    pub fn stream_api_endpoint(mut self, val: String) -> Self {
        self.stream_api_endpoint = val;
        self
    }

    pub async fn build(self) -> Result<XtbClient, XtbClientBuilderError> {
        let protocol = match self.protocol {
            None => "wss".to_owned(),
            Some(v) => v,
        };
        let base_url = format!("{}://{}", protocol, self.host);
        let base_url = match self.port {
            None => base_url,
            Some(port) => format!("{}:{}", base_url, port),
        };

        let api_url = format!("{}{}", base_url, self.api_endpoint);
        let stream_api_url = format!("{}{}", base_url, self.stream_api_endpoint);

        let api_socket = Self::open_websocket(api_url).await?;
        let stream_api_socket = Self::open_websocket(stream_api_url).await?;
        Ok(XtbClient::new(api_socket, stream_api_socket))
    }

    async fn open_websocket(url: String) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, XtbClientBuilderError> {
        let url = Url::parse(&url).map_err(|_| XtbClientBuilderError::InvalidUrl(url))?;
        let (ws, _) = connect_async(url).await.map_err(|err| XtbClientBuilderError::ConnectionError(err))?;
        Ok(ws)
    }
}


#[derive(Debug, Error)]
pub enum XtbClientError {}


#[derive(Debug, Error)]
pub enum XtbClientBuilderError {
    #[error("Built url is in invalid format")]
    InvalidUrl(String),
    #[error("Connection failed")]
    ConnectionError(WSError),
}
