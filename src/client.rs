use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use futures_util::stream::{SplitSink, SplitStream};
use serde::Serialize;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::spawn;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::{Error as WSError, Message};
use url::Url;

use crate::api::{AccountApi, ApiCommand, CommandFailed, CommandResult, LoginArg, ParseResponseError, ResponseChannel, ResponseInfo, ResponseSink, ResponseStream};

type ResponseSinkLookup = Arc<Mutex<HashMap<String, ResponseChannel<ResponseSink>>>>;

pub struct XtbClient {
    api: ApiWrapper,
    stream_api: ApiWrapper,

    stream_session_id: Option<String>,
    next_id: u64,
    sink_by_tag: ResponseSinkLookup,
}


impl XtbClient {
    pub fn new(
        api_socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
        stream_api_socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    ) -> Self {
        let sink_by_tag = Arc::new(Mutex::new(HashMap::new()));

        Self {
            stream_session_id: None,
            api: ApiWrapper::new(api_socket, sink_by_tag.clone()),
            stream_api: ApiWrapper::new(stream_api_socket, sink_by_tag.clone()),
            next_id: 1,
            sink_by_tag,
        }
    }

    async fn send_api_command<A: Serialize>(&mut self, command: &str, args: Option<A>) -> Result<ResponseChannel<ResponseStream>, XtbClientError> {
        let cmd = self.build_command(command, args, false);
        let tag = cmd.custom_tag.clone();
        let response_channel = self.make_response_channel(&tag.unwrap()).await;
        let message = Self::build_message(cmd)?;
        self.api.sink.send(message).await.map_err(|_| XtbClientError::MessageCannotBeSend(ApiType::Api))?;
        Ok(response_channel)
    }

    fn build_command<A>(&mut self, command: &str, args: Option<A>, is_streaming: bool) -> ApiCommand<A> {
        ApiCommand::builder()
            .command(command.to_string())
            .arguments(args)
            .custom_tag(Some(self.generate_unique_custom_tag()))
            .stream_session_id(if is_streaming { self.stream_session_id.clone() } else { None })
            .build()
    }

    fn build_message<A: Serialize>(command: ApiCommand<A>) -> Result<Message, XtbClientError> {
        let payload = serde_json::to_string(&command).map_err(|_| XtbClientError::SerializationFailed)?;
        let message = Message::text(payload);
        Ok(message)
    }

    async fn make_response_channel(&mut self, custom_tag: &str) -> ResponseChannel<ResponseStream> {
        let (mut response_sink, response_stream) = ResponseChannel::<ResponseSink>::new();
        let lookup = self.sink_by_tag.clone();
        let custom_tag_clone = custom_tag.to_string();
        response_sink.add_after_close_callback(Box::new(|| {
            async fn remove_tag(lookup: ResponseSinkLookup, tag: String) {
                lookup.lock().await.remove(&tag);
            }
            spawn(remove_tag(lookup, custom_tag_clone));
        })).await;
        self.sink_by_tag.lock().await.insert(custom_tag.to_owned(), response_sink);
        response_stream
    }

    fn generate_unique_custom_tag(&mut self) -> String {
        let id = self.next_id.to_string();
        self.next_id += 1;
        format!("cmd_{}", id)
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
        let stream = self.send_api_command("login", Some(args)).await?;
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
        let result: CommandResult<()> = self.send_api_command::<()>("logout", None).await?.first().await.ok_or_no_response()?.try_into()?;
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


struct ApiWrapper {
    sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    join_handle: JoinHandle<()>,
}


impl ApiWrapper {
    fn new(socket: WebSocketStream<MaybeTlsStream<TcpStream>>, response_sink_lookup: ResponseSinkLookup) -> Self {
        let (sink, stream) = socket.split();
        async fn receiver(
            stream_: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
            lookup: ResponseSinkLookup,
        ) {
            let mut stream_ = stream_;
            stream_.for_each(|payload| async {
                match payload {
                    Ok(message) => {
                        let result = ResponseInfo::try_from(message);
                        match result {
                            Ok(response_info) => {
                                let lookup = lookup.clone();
                                let tag = response_info.custom_tag.clone();
                                let mut guard = lookup.lock().await;
                                match tag.and_then(|tag| (*guard).get_mut(&tag)) {
                                    Some(sink) => sink.write(response_info).await,
                                    _ => ()
                                };
                            }
                            Err(_) => {
                                // TODO: log error
                                ()
                            }
                        }
                    }
                    Err(err) => {
                        // TODO: log error
                        ()
                    }
                }
            }).await;
        }

        let join_handle = spawn(receiver(stream, response_sink_lookup));
        Self {
            sink,
            join_handle,
        }
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
    #[error("Message cannot be sent")]
    MessageCannotBeSend(ApiType),
    #[error("Unable to serialize data")]
    SerializationFailed,
}


#[derive(Debug)]
pub enum ApiType {
    Api,
    StreamApi,
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
