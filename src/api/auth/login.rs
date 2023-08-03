use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
pub struct LoginArgs {
    user_id: String,
    password: String,
}
