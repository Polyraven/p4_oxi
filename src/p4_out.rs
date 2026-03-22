//! Holds the messages of info and warnings that come up during command execution.

use serde::Deserialize;

/// Deserializes and utility functions to help deal with p4 command warnings
/// For any command that has any kind of errors it will respond with a message as such:
/// ```json
/// {"data":"<wildcard> - file(s) up-to-date.\n","generic":17,"severity":2}
/// ```
/// These messages still let p4 continue it's execution.
/// But listening to these gives the oportunity to react to these.
/// Thus is why this implementation listen to these in the handler
#[derive(Debug, Deserialize)]
pub struct WarningMessage {
    /// will contain the wildcard and the reason in failed
    pub data: String,
    /// the function of this is unknown
    pub generic: i32,
    /// irrelevant to us (actually severity 3 means it could not run)
    pub severity: i32,
}

impl WarningMessage {}

/// When P4 won't do something with a file yet it's not really a problem it will output a message like so:
/// ```json
/// {"data":"<file_path> - must resolve #3 before submitting","level":1}
/// ```
/// There is an option to register a handler for these as well
#[derive(Debug, Deserialize)]
pub struct InfoMessage {
    pub data: String,
    pub level: i32,
}

#[derive(Debug)]
pub struct ParseError {
    pub data: String,
    pub error: String,
}

#[derive(Debug)]
pub struct Failure {
    pub data: String,
}

#[derive(Debug)]
pub enum P4Out<T> {
    Content(T),
    Warning(WarningMessage),
    Info(InfoMessage),
    Failure(Failure),
    ParseError(ParseError),
}

impl<T> P4Out<T> {
    pub fn is_content(&self) -> bool {
        matches!(self, P4Out::Content(_))
    }

    pub fn is_warning(&self) -> bool {
        matches!(self, P4Out::Warning(_))
    }

    pub fn is_info(&self) -> bool {
        matches!(self, P4Out::Info(_))
    }

    pub fn is_failure(&self) -> bool {
        matches!(self, P4Out::Failure(_))
    }

    pub fn is_parse_error(&self) -> bool {
        matches!(self, P4Out::ParseError(_))
    }

    pub fn take_content(self) -> Option<T> {
        match self {
            P4Out::Content(v) => Some(v),
            _ => None,
        }
    }

    pub fn take_warning(self) -> Option<WarningMessage> {
        match self {
            P4Out::Warning(v) => Some(v),
            _ => None,
        }
    }

    pub fn take_info(self) -> Option<InfoMessage> {
        match self {
            P4Out::Info(v) => Some(v),
            _ => None,
        }
    }

    pub fn take_failure(self) -> Option<Failure> {
        match self {
            P4Out::Failure(v) => Some(v),
            _ => None,
        }
    }

    pub fn take_parse_error(self) -> Option<ParseError> {
        match self {
            P4Out::ParseError(v) => Some(v),
            _ => None,
        }
    }

    /// Discards the content type, used when storing non-content variants in [`P4Error`](crate::error::P4Error).
    pub fn erase_content_type(self) -> P4Out<()> {
        match self {
            P4Out::Content(_) => P4Out::Content(()),
            P4Out::Warning(w) => P4Out::Warning(w),
            P4Out::Info(i) => P4Out::Info(i),
            P4Out::Failure(f) => P4Out::Failure(f),
            P4Out::ParseError(e) => P4Out::ParseError(e),
        }
    }
}
