use serde::Deserialize;

use crate::api::XtbErrorCode;

pub type CommandResult<D> = Result<CommandSuccess<D>, CommandFailed>;


#[derive(Clone, Deserialize)]
pub struct CommandSuccess<D> {
    /// Returned data
    return_data: Option<D>,

    /// Custom tag used for response identification
    custom_tag: Option<String>,
}


#[derive(Clone, Deserialize)]
pub struct CommandFailed {
    /// Error code
    /// See http://developers.xstore.pro/documentation/#error-messages
    pub error_code: XtbErrorCode,

    /// Description of the error
    pub error_description: String,
}
