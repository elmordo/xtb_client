use serde::Deserialize;
use serde_json::{Error as SerdeJsonError, Value};
use thiserror::Error;

use crate::api::XtbErrorCode;

pub type CommandResult<D> = Result<CommandSuccess<D>, CommandFailed>;


pub fn parse_response<D>(value: Value) -> Result<CommandResult<D>, ParseResponseError>
    where D: for<'de> Deserialize<'de>
{
    let status = get_response_status_from_raw_body(&value)?;
    let return_value = match status {
        true => {
            Ok(serde_json::from_value::<CommandSuccess<D>>(value).map_err(|err| ParseResponseError::DeserializationError(err))?)
        }
        false => {
            Err(serde_json::from_value::<CommandFailed>(value).map_err(|err| ParseResponseError::DeserializationError(err))?)
        }
    };
    Ok(return_value)
}

fn get_response_status_from_raw_body(value: &Value) -> Result<bool, ParseResponseError> {
    value
        .as_object()
        .ok_or_else(|| ParseResponseError::InvalidDataFormat(InvalidFormatErrorInfo::NotAnObject))
        .and_then(|data| {
            data
                .get("status")
                .ok_or_else(|| ParseResponseError::InvalidDataFormat(InvalidFormatErrorInfo::StatusFieldMissing))
        })
        .and_then(|raw_status| {
            raw_status
                .as_bool()
                .ok_or_else(|| ParseResponseError::InvalidDataFormat(InvalidFormatErrorInfo::InvalidStatusType))
        })
}


#[derive(Debug, Error)]
pub enum ParseResponseError {
    #[error("Data is in invalid format")]
    InvalidDataFormat(InvalidFormatErrorInfo),
    #[error("Unable to deserialize response")]
    DeserializationError(SerdeJsonError),
}


#[derive(Debug)]
pub enum InvalidFormatErrorInfo {
    NotAnObject,
    StatusFieldMissing,
    InvalidStatusType,
}


#[derive(Clone, Deserialize)]
pub struct CommandSuccess<D> {
    /// Returned data
    pub return_data: Option<D>,

    /// Custom tag used for response identification
    pub custom_tag: Option<String>,
}


#[derive(Clone, Deserialize)]
pub struct CommandFailed {
    /// Error code
    /// See http://developers.xstore.pro/documentation/#error-messages
    pub error_code: XtbErrorCode,

    /// Description of the error
    pub error_description: String,
}
