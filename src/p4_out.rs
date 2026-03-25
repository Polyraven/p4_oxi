//! Holds the messages of info and warnings that come up during command execution.

use serde::Deserialize;

/// A non-fatal warning from the P4 server, such as `file(s) up-to-date`.
///
/// For any command that has any kind of errors it will respond with a message as such:
/// ```json
/// {"data":"<wildcard> - file(s) up-to-date.\n","generic":17,"severity":2}
/// ```
/// These messages still let p4 continue its execution.
/// Listening to these gives the opportunity to react to them.
#[derive(Debug, Deserialize)]
pub struct P4WarningMessage {
    /// will contain the wildcard and the reason in failed
    pub data: String,
    /// the function of this is unknown
    pub generic: i32,
    /// irrelevant to us (actually severity 3 means it could not run)
    pub severity: i32,
}

/// An informational message from the P4 server, not an error or warning.
///
/// When P4 won't do something with a file yet it's not really a problem it will output a message like so:
/// ```json
/// {"data":"<file_path> - must resolve #3 before submitting","level":1}
/// ```
#[derive(Debug, Deserialize)]
pub struct P4InfoMessage {
    pub data: String,
    pub level: i32,
}

/// A deserialization failure when parsing P4 server output into the expected type.
#[derive(Debug)]
pub struct P4ParseError {
    pub data: String,
    pub error: String,
}

/// A fatal error from the P4 server or the subprocess.
#[derive(Debug)]
pub struct P4Failure {
    pub data: String,
}

/// A single item yielded by a running P4 command. Categorizes server output into content, warnings, info, failures, and parse errors.
#[derive(Debug)]
pub enum P4Out<T> {
    Content(T),
    Warning(P4WarningMessage),
    Info(P4InfoMessage),
    Failure(P4Failure),
    ParseError(P4ParseError),
}

impl std::fmt::Display for P4WarningMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for P4InfoMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


impl std::fmt::Display for P4ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for P4Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<T: std::fmt::Debug> std::fmt::Display for P4Out<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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

    pub fn take_warning(self) -> Option<P4WarningMessage> {
        match self {
            P4Out::Warning(v) => Some(v),
            _ => None,
        }
    }

    pub fn take_info(self) -> Option<P4InfoMessage> {
        match self {
            P4Out::Info(v) => Some(v),
            _ => None,
        }
    }

    pub fn take_failure(self) -> Option<P4Failure> {
        match self {
            P4Out::Failure(v) => Some(v),
            _ => None,
        }
    }

    pub fn take_parse_error(self) -> Option<P4ParseError> {
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
