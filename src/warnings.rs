use serde::Deserialize;

/// Deserializes and utility functions to help deal with p4 command warinngs
///
/// For any command that has any kind of errors it will respond with a message as such:
/// {"data":"<wildcard> - file(s) up-to-date.\n","generic":17,"severity":2}
///
/// These messages still let p4 continue it's execution.
/// But listening to these gives the oportunity to react to these.
/// Thus is why this implementation listen to these in the handler
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct WarningMessage {
    /// will contain the wildcard and the reason in failed
    pub data: String,
    /// the furnction of this is unknown
    pub generic: i32,
    /// irrelevant to us (actually severity 3 means it could not run)
    pub severity: i32,
}

impl WarningMessage {}

/// When P4 won't do something with a file yet it's not really a problem it will output a message like so:
/// {"data":"<file_path> - must resolve #3 before submitting","level":1}
/// There is an option to register a handler for these as well
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct InfoMessage {
    pub data: String,
    pub level: i32,
}
