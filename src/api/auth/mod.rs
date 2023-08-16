use async_trait::async_trait;
use thiserror::Error;

pub use login::{LoginArgs, LoginArgsBuilder};

use crate::connection::XtbServerConnectionError;

mod login;

#[async_trait]
pub trait AuthApi {

    type Error;

    /// Login user
    async fn login(&mut self, args: LoginArgs) -> Result<(), Self::Error>;

    ///  Logout user
    async fn logout(&mut self) -> Result<(), Self::Error>;
}


#[derive(Debug, Error)]
pub enum AuthApiError {
    #[error("Error on connection layer")]
    ConnectionError(XtbServerConnectionError),
    #[error("Login failed")]
    LoginFailed,
}
