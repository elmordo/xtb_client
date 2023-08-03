use async_trait::async_trait;
use crate::api::{AuthApi, AuthApiError, LoginArgs};
use crate::connection::XtbServerConnection;

pub struct XtbClient {
    api_connection: XtbServerConnection,
    stream_api_connection: XtbServerConnection,
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
