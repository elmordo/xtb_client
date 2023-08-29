use async_trait::async_trait;
use serde::Serialize;

#[async_trait]
pub trait AccountApi {
    type Error;

    /// Login user
    async fn login(&mut self, user_id: &str, password: &str) -> Result<(), Self::Error>;

    /// Logout user
    async fn logout(&mut self) -> Result<(), Self::Error>;
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct LoginArg {
    pub user_id: String,
    pub password: String,
}
