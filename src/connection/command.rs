use serde::{Deserialize, Serialize};
use thiserror::Error;


/// Represent XTB API command
#[derive(Clone, Serialize)]
pub struct Command<T: Serialize> {
    /// Command name
    /// See http://developers.xstore.pro/documentation/ for available commands
    pub command: String,

    /// Arguments
    pub arguments: Option<T>,

    /// Custom tag used to identify responses
    pub custom_tag: Option<String>,

    /// For streaming commands only
    /// See http://developers.xstore.pro/documentation/#available-streaming-commands
    pub stream_session_id: Option<String>,
}


/// Type alias for command responses
pub type CommandResult<T: Deserialize> = Result<CommandOk<T>, CommandError>;


/// Represent response when command was successful
#[derive(Clone, Deserialize)]
pub struct CommandOk<T: Deserialize> {
    /// Returned data
    return_data: Option<T>,

    /// Custom tag used for response identification
    custom_tag: Option<String>,
}


/// Represent error when command fails
#[derive(Clone, Deserialize)]
pub struct CommandError {
    /// Error code
    /// See http://developers.xstore.pro/documentation/#error-messages
    pub error_code: XtbErrorCode,

    /// Description of the error
    pub error_description: String,
}


/// Error codes of XTB API
/// See http://developers.xstore.pro/documentation/#error-messages
#[derive(Debug, Error, Deserialize)]
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
