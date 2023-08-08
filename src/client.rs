use std::arch::x86_64::_mm256_broadcast_pd;
use async_trait::async_trait;
use tokio::spawn;
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tokio::task::JoinHandle;
use crate::api::{AuthApi, AuthApiError, LoginArgs};
use crate::connection::{MessageStream, Response, XtbServerConnection, XtbSink, XtbStream};

pub struct XtbClient {
    api_sink: XtbSink,
    api_stream_handle: JoinHandle<()>,
    api_channel: (Sender<Response>, Receiver<Response>),
    stream_api_sink: XtbSink,
    stream_api_stream_handle: JoinHandle<()>,
    stream_api_channel: (Sender<Response>, Receiver<Response>),
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
        }
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
    async fn login(&mut self, args: LoginArgs) -> Result<(), AuthApiError> {
        todo!()
    }

    async fn logout(&mut self) -> Result<(), AuthApiError> {
        todo!()
    }
}
