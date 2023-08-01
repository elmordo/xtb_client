use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
pub struct Command<T: Serialize> {
    pub command: String,
    pub arguments: T,
    pub custom_tag: Option<String>,
    pub stream_session_id: Option<String>,
}


#[derive(Clone, Deserialize)]
pub struct CommandResponse<T: Deserialize> {
    return_data: T,
    custom_tag: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct CommandErrorResponse {
    pub error_code: XtbErrorCode,
    pub error_description: String,
}

pub enum XtbErrorCode {

}
