use async_trait::async_trait;
use tokio::spawn;
use tokio::task::JoinHandle;
use crate::api::{AuthApi, AuthApiError, LoginArgs};
use crate::connection::{XtbServerConnection, XtbSink, XtbStream};

pub struct XtbClient {
    api_sink: XtbSink,
    stream_api_sink: XtbSink,
    api_stream_handle: JoinHandle<()>,
    stream_api_stream_handle: JoinHandle<()>,
}


impl XtbClient {
    pub async fn new(api_connection: XtbServerConnection, stream_api_connection: XtbServerConnection) -> Self {
        let (api_sink, api_stream) = api_connection.split();
        let (stream_api_sink, stream_api_stream) = stream_api_connection.split();

        let api_stream_handle = spawn(receive(api_stream));
        let stream_api_stream_handle = spawn(receive(stream_api_stream));

        Self {
            api_sink,
            stream_api_sink,
            api_stream_handle,
            stream_api_stream_handle,
        }
    }
}


async fn receive(stream: XtbStream) {

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
