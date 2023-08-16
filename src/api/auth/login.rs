use derive_builder::Builder;
use serde::Serialize;

#[derive(Clone, Default, Serialize, Builder)]
pub struct LoginArgs {
    pub user_id: String,
    pub password: String,
}


pub struct LoginResponse {

}
