//! Crate-level error type for failures that prevent a command from producing output.

use std::fmt;

use crate::p4_out::P4Out;

/// Represents errors from the p4_oxi crate itself, not from the P4 server.
///
/// Server-side messages (warnings, info, failures) are represented by [`P4Out`] variants
/// and are yielded during iteration. `P4Error` covers everything that prevents
/// iteration from happening at all.
#[derive(Debug)]
pub enum P4OxiError {
    /// Failed to spawn the p4 process or encountered an IO error.
    ProcessError(String),
    /// The API was used incorrectly (e.g. calling `.run()` when an input spec is set).
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

impl fmt::Display for P4OxiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            P4OxiError::ProcessError(msg) => write!(f, "process error: {msg}"),
            P4OxiError::InvalidUsage(msg) => write!(f, "invalid usage: {msg}"),
            P4OxiError::NoResults => write!(f, "expected a result but got none"),
            P4OxiError::UnexpectedOutput(out) => write!(f, "unexpected output: {out:?}"),
            P4OxiError::ParseError(msg) => write!(f, "parse error: {msg}"),
            P4OxiError::VersionMismatch(msg) => write!(f, "version mismatch: {msg}"),
        }
    }
}

impl std::error::Error for P4OxiError {}
