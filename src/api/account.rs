use std::fmt::Error;

use async_trait::async_trait;

use crate::api::CommandResult;

#[async_trait]
pub trait AccountApi {
    type Error;

    /// Login user
    fn login(&mut self, account_id: &str, password: &str) -> Result<CommandResult<()>, Error>;

    /// Logout user
    fn logout(&mut self) -> Result<CommandResult<()>, Error>;
}
