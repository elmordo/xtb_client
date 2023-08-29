use async_trait::async_trait;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Error as WSError;
use url::Url;

use crate::api::{AccountApi, CommandFailed, CommandResult, LoginArg, ParseResponseError, ResponseChannel, ResponseInfo, ResponseStream};

pub struct XtbClient {
    stream_session_id: Option<String>,
}


impl XtbClient {
    pub fn new(
        api_socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
        stream_api_socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    ) -> Self {
        todo!()
    }

    async fn send_command<A>(&mut self, command: &str, args: Option<A>) -> Result<ResponseChannel<ResponseStream>, XtbClientError> {
        todo!()
    }
}


#[async_trait]
impl AccountApi for XtbClient {
    type Error = XtbClientError;

    async fn login(&mut self, user_id: &str, password: &str) -> Result<(), Self::Error> {
        let args = LoginArg {
            user_id: user_id.to_string(),
            password: password.to_string(),
        };
        let stream = self.send_command("login", Some(args)).await?;
        let response_info = stream.first().await.ok_or_no_response()?;

        let command_result: CommandResult<()> = response_info.try_into()?;
        match command_result.clone() {
            Ok(resp) => {
                self.stream_session_id = resp.stream_session_id;
                Ok(())
            }
            Err(err) => Err(XtbClientError::CommandFailed(err))
        }
    }

    async fn logout(&mut self) -> Result<(), Self::Error> {
        let result: CommandResult<()> = self.send_command::<()>("logout", None).await?.first().await.ok_or_no_response()?.try_into()?;
        result.map(|_| ()).map_err(|err| XtbClientError::CommandFailed(err))
    }
}


trait OkOrNoResponse {
    fn ok_or_no_response(self) -> Result<ResponseInfo, XtbClientError>;
}

impl OkOrNoResponse for Option<ResponseInfo> {
    fn ok_or_no_response(self) -> Result<ResponseInfo, XtbClientError> {
        self.ok_or_else(|| XtbClientError::NoResponseReceived)
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
pub enum XtbClientError {
    #[error("No response was received")]
    NoResponseReceived,
    #[error("Cannot parse response")]
    ParseResponseError(ParseResponseError),
    #[error("CommandFailed")]
    CommandFailed(CommandFailed),
}


impl From<ParseResponseError> for XtbClientError {
    fn from(value: ParseResponseError) -> Self {
        Self::ParseResponseError(value)
    }
}


impl From<CommandFailed> for XtbClientError {
    fn from(value: CommandFailed) -> Self {
        Self::CommandFailed(value)
    }
}


#[derive(Debug, Error)]
pub enum XtbClientBuilderError {
    #[error("Built url is in invalid format")]
    InvalidUrl(String),
    #[error("Connection failed")]
    ConnectionError(WSError),
}
