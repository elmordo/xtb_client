pub use command::{Command, CommandError, CommandOk, CommandResult};
pub use connection::{Response, ResponseError, XtbServerConnection, XtbServerConnectionError};

mod command;
mod connection;
