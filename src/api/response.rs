use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::Arc;

use serde::Deserialize;
use serde_json::{Error as SerdeJsonError, Map, Value};
use thiserror::Error;
use tokio::sync::{Mutex, Notify};

/// Result of the XTB API command
/// * Ok variant represents success
/// * Err variant represents error returned by remote API
pub type CommandResult<D> = Result<CommandSuccess<D>, CommandFailed>;


/// Data passed when command was success
#[derive(Clone, Deserialize)]
pub struct CommandSuccess<D> {
    /// Returned data
    #[serde(default = "Option::default")]
    pub return_data: Option<D>,

    /// Returned data
    #[serde(default = "Option::default")]
    pub stream_session_id: Option<String>,

    /// Custom tag used for response identification
    #[serde(default)]
    pub custom_tag: Option<String>,
}


/// Data passed when command failed
#[derive(Clone, Debug, Deserialize)]
pub struct CommandFailed {
    /// Error code
    /// See http://developers.xstore.pro/documentation/#error-messages
    pub error_code: XtbErrorCode,

    /// Description of the error
    pub error_description: String,
}


/// Contains basic info about response for response dispatching process
#[derive(Debug, Clone)]
pub struct ResponseInfo {
    pub status: bool,
    pub custom_tag: Option<String>,
    value: Value,
}


impl ResponseInfo {
    /// Construct new `ResponseInfo` from JSON value
    pub fn new(value: Value) -> Result<Self, ParseResponseError> {
        let top_level = get_response_top_level_object(&value)?;
        let status = get_response_status_from_top_level_object(top_level)?;
        let custom_tag = get_custom_tag_from_top_level_object(top_level)?;
        Ok(Self {
            status,
            custom_tag,
            value,
        })
    }
}

impl<D> TryInto<CommandResult<D>> for ResponseInfo where D: for<'de> Deserialize<'de> {
    type Error = ParseResponseError;

    /// Convert response info into typed response
    fn try_into(self) -> Result<CommandResult<D>, ParseResponseError> {
        Ok(match self.status {
            true => Ok(serde_json::from_value::<CommandSuccess<D>>(self.value).map_err(|err| ParseResponseError::DeserializationError(err))?),
            false => Err(serde_json::from_value::<CommandFailed>(self.value).map_err(|err| ParseResponseError::DeserializationError(err))?),
        })
    }
}


/// Shared state of response channel
struct ResponseSharedState {
    queue: VecDeque<ResponseInfo>,
    closed: bool,
}


impl ResponseSharedState {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            closed: false,
        }
    }
}


pub struct ResponseStream;

pub struct ResponseSink;


/// Consumer endpoint of response channel
pub struct ResponseChannel<T> {
    state: Arc<Mutex<ResponseSharedState>>,
    notify: Arc<Notify>,
    endpoint: PhantomData<T>,
}


impl<T> ResponseChannel<T> {
    pub fn new() -> (ResponseChannel<ResponseSink>, ResponseChannel<ResponseStream>) {
        let state = Arc::new(Mutex::new(ResponseSharedState::new()));
        let notify = Arc::new(Notify::new());

        let sink = ResponseChannel {
            state: state.clone(),
            notify: notify.clone(),
            endpoint: PhantomData::<ResponseSink>,
        };
        let stream = ResponseChannel {
            state: state.clone(),
            notify: notify.clone(),
            endpoint: PhantomData::<ResponseStream>,
        };
        (sink, stream)
    }

    pub async fn status(&self) -> ResponseStreamStatus {
        if self.closed().await {
            ResponseStreamStatus::Closed
        } else if self.queue_size().await > 0 {
            ResponseStreamStatus::Ready
        } else {
            ResponseStreamStatus::Pending
        }
    }

    pub async fn close(self) {
        (*self.state.lock().await).closed = true;
    }

    pub async fn queue_size(&self) -> usize {
        (*self.state.lock().await).queue.len()
    }

    pub async fn closed(&self) -> bool {
        (*self.state.lock().await).closed
    }
}


/// Implementation of the response stream (message consumer)
impl ResponseChannel<ResponseStream> {
    pub async fn read(&mut self) -> Option<ResponseInfo> {
        loop {
            {
                let mut state = self.state.lock().await;
                if state.closed {
                    break None;
                }
                if state.queue.len() > 0 {
                    break state.queue.pop_front();
                }
            }
            self.notify.notified().await;
        }
    }

    pub async fn first(mut self) -> Option<ResponseInfo> {
        let msg = self.read().await;
        self.close().await;
        msg
    }
}


/// Implementation of response sink (producer)
impl ResponseChannel<ResponseSink> {
    /// Send new response into channel
    pub async fn write(&mut self, info: ResponseInfo) {
        (*self.state.lock().await).queue.push_back(info);
        self.notify.notify_waiters();
    }
}


/// Status of the response channel
pub enum ResponseStreamStatus {
    /// Channel is open but no response is in queue
    Pending,
    /// Channel is open and some response is in queue
    Ready,
    /// Channel is closed
    Closed,
}


/// Extract top level object of the json `Value`
/// If `Value` is not type of `Object` return error
fn get_response_top_level_object<'v, 'm>(value: &'v Value) -> Result<&'m Map<String, Value>, ParseResponseError> where 'v: 'm {
    value
        .as_object()
        .ok_or_else(|| ParseResponseError::InvalidDataFormat(InvalidFormatErrorInfo::NotAnObject))
}


/// Extract the `status` field from map representing the top level response object.
fn get_response_status_from_top_level_object(value: &Map<String, Value>) -> Result<bool, ParseResponseError> {
    value.get("status")
        .ok_or_else(|| ParseResponseError::InvalidDataFormat(InvalidFormatErrorInfo::StatusFieldMissing))
        .and_then(|raw_status| {
            raw_status
                .as_bool()
                .ok_or_else(|| ParseResponseError::InvalidDataFormat(InvalidFormatErrorInfo::InvalidStatusType))
        })
}


/// Extract the `custom_tag` field from map representing the top level response object.
fn get_custom_tag_from_top_level_object(value: &Map<String, Value>) -> Result<Option<String>, ParseResponseError> {
    // the unwrap is safe at this point
    match value.get("custom_tag").or_else(|| Some(&Value::Null)).unwrap() {
        Value::Null => Ok(None),
        Value::String(val) => Ok(Some(val.to_string())),
        _ => Err(ParseResponseError::InvalidDataFormat(InvalidFormatErrorInfo::InvalidCustomTagType))
    }
}


/// Error states for response parsing
#[derive(Debug, Error)]
pub enum ParseResponseError {
    #[error("Data is in invalid format")]
    InvalidDataFormat(InvalidFormatErrorInfo),
    #[error("Unable to deserialize response")]
    DeserializationError(SerdeJsonError),
}


/// Detail of the response data content errors
#[derive(Debug)]
pub enum InvalidFormatErrorInfo {
    NotAnObject,
    StatusFieldMissing,
    InvalidStatusType,
    InvalidCustomTagType,
}

/// Error codes of XTB API
/// See http://developers.xstore.pro/documentation/#error-messages
#[derive(Clone, Debug, Error, Deserialize)]
pub enum XtbErrorCode {
    #[error("Invalid price")]
    BE001,
    #[error("Invalid StopLoss or TakeProfit")]
    BE002,
    #[error("Invalid volume")]
    BE003,
    #[error("Login disabled")]
    BE004,
    #[error("userPasswordCheck: Invalid login or password.")]
    BE005,
    #[error("Market for instrument is closed")]
    BE006,
    #[error("Mismatched parameters")]
    BE007,
    #[error("Modification is denied")]
    BE008,
    #[error("Not enough money on account to perform trade")]
    BE009,
    #[error("Off quotes")]
    BE010,
    #[error("Opposite positions prohibited")]
    BE011,
    #[error("Short positions prohibited")]
    BE012,
    #[error("Price has changed")]
    BE013,
    #[error("Request too frequent")]
    BE014,
    #[error("Too many trade requests")]
    BE016,
    #[error("Too many trade requests")]
    BE017,
    #[error("Trading on instrument disabled")]
    BE018,
    #[error("Trading timeout")]
    BE019,
    #[error("Other error")]
    BE020,
    #[error("Other error")]
    BE021,
    #[error("Other error")]
    BE022,
    #[error("Other error")]
    BE023,
    #[error("Other error")]
    BE024,
    #[error("Other error")]
    BE025,
    #[error("Other error")]
    BE026,
    #[error("Other error")]
    BE027,
    #[error("Other error")]
    BE028,
    #[error("Other error")]
    BE029,
    #[error("Other error")]
    BE030,
    #[error("Other error")]
    BE031,
    #[error("Other error")]
    BE032,
    #[error("Other error")]
    BE033,
    #[error("Other error")]
    BE034,
    #[error("Other error")]
    BE035,
    #[error("Other error")]
    BE036,
    #[error("Other error")]
    BE037,
    #[error("Other error")]
    BE099,
    #[error("Symbol does not exist for given account")]
    BE094,
    #[error("Account cannot trade on given symbol")]
    BE095,
    #[error("Pending order cannot be closed. Pending order must be deleted")]
    BE096,
    #[error("Cannot close already closed order")]
    BE097,
    #[error("No such transaction")]
    BE098,
    #[error("Unknown instrument symbol")]
    BE101,
    #[error("Unknown transaction type")]
    BE102,
    #[error("User is not logged")]
    BE103,
    #[error("Method does not exist")]
    BE104,
    #[error("Incorrect period given")]
    BE105,
    #[error("Missing data")]
    BE106,
    #[error("Incorrect command format")]
    BE110,
    #[error("Symbol does not exist")]
    BE115,
    #[error("Symbol does not exist")]
    BE116,
    #[error("Invalid token")]
    BE117,
    #[error("User already logged")]
    BE118,
    #[error("Session timed out.")]
    BE200,
    #[error("Invalid parameters")]
    EX000,
    #[error("Internal error, in case of such error, please contact support")]
    EX001,
    #[error("Internal error, in case of such error, please contact support")]
    EX002,
    #[error("Internal error, in case of such error, please contact support")]
    BE000,
    #[error("Internal error, request timed out")]
    EX003,
    #[error("Login credentials are incorrect or this login is not allowed to use an application with this appId")]
    EX004,
    #[error("Internal error, system overloaded")]
    EX005,
    #[error("No access")]
    EX006,
    #[error("userPasswordCheck: Invalid login or password. This login/password is disabled for 10 minutes (the specific login and password pair is blocked after an unsuccessful login attempt).")]
    EX007,
    #[error("You have reached the connection limit. For details see the Connection validation section.")]
    EX008,
    #[error("Data limit potentially exceeded. Please narrow your request range. The potential data size is calculated by: (end_time - start_time) / interval. The limit is 50 000 candles")]
    EX009,
    #[error("Your login is on the black list, perhaps due to previous misuse. For details please contact support.")]
    EX010,
    #[error("You are not allowed to execute this command. For details please contact support.")]
    EX011,
}
