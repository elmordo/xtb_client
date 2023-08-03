pub use login::{LoginArgs};
mod login;
use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait AuthApi {
    /// Login user
    async fn login(&mut self, args: LoginArgs) -> Result<bool, AuthApiError>;

    ///  Logout user
    async fn logout(&mut self) -> Result<(), AuthApiError>;
}


#[derive(Debug, Error)]
pub enum AuthApiError {
    #[error("Error on connection layer")]
    ConnectionError
}
