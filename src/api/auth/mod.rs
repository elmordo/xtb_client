use async_trait::async_trait;
use thiserror::Error;

pub use login::LoginArgs;

use crate::connection::XtbServerConnectionError;

mod login;

#[async_trait]
pub trait AuthApi {
    /// Login user
    async fn login(&mut self, args: LoginArgs) -> Result<(), AuthApiError>;

    ///  Logout user
    async fn logout(&mut self) -> Result<(), AuthApiError>;
}


#[derive(Debug, Error)]
pub enum AuthApiError {
    #[error("Error on connection layer")]
    ConnectionError(XtbServerConnectionError),
    #[error("Login failed")]
    LoginFailed,
}
