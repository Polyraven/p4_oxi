//! Crate-level error type for failures that prevent a command from producing output.

use std::fmt;

use crate::p4_out::P4Out;

/// Represents errors from the p4_oxi crate itself, not from the P4 server.
///
/// Server-side messages (warnings, info, failures) are represented by [`P4Out`] variants
/// and are yielded during iteration. `P4Error` covers everything that prevents
/// iteration from happening at all.
#[derive(Debug)]
pub enum P4Error {
    /// Failed to spawn, communicate with, or wait on the P4 process.
    ProcessError(String, Option<std::io::Error>),
    /// The API was used incorrectly (e.g. calling `.run()` when an input form is set).
    InvalidUsage(&'static str),
    /// A command that expected at least one result got none.
    NoResults,
    /// Expected content but received a non-content message.
    UnexpectedOutput(P4Out<()>),
    /// Failed to parse expected data (e.g. version string format).
    ParseError(String),
    /// The installed P4 client version does not meet minimum requirements.
    VersionMismatch(String),
}

impl fmt::Display for P4Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            P4Error::ProcessError(msg, Some(e)) => {
                write!(f, "p4-oxi: process error: {msg}: {e}")
            }
            P4Error::ProcessError(msg, None) => write!(f, "p4-oxi: process error: {msg}"),
            P4Error::InvalidUsage(msg) => write!(f, "p4-oxi: invalid usage: {msg}"),
            P4Error::NoResults => write!(f, "p4-oxi: expected a result but got none"),
            P4Error::UnexpectedOutput(out) => write!(f, "p4-oxi: unexpected output: {out:?}"),
            P4Error::ParseError(msg) => write!(f, "p4-oxi: parse error: {msg}"),
            P4Error::VersionMismatch(msg) => write!(f, "p4-oxi: version mismatch: {msg}"),
        }
    }
}

impl std::error::Error for P4Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            P4Error::ProcessError(_, Some(e)) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for P4Error {
    fn from(e: std::io::Error) -> Self {
        P4Error::ProcessError(e.to_string(), Some(e))
    }
}
