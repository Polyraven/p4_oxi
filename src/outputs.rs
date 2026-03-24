//! Common structs for deserializable output responses of the commands.
//! TODO: It should probably be moved to cmd sub-crate in the future.

use serde::Deserialize;
/// TODO: FileAction is meant to be the output of a number of different formats,
/// like:
/// - p4 sync
/// - p4 clean
/// - p4 flush (sync without file transfer)
/// - p4 reconcile (though reconcile adds extra fields like fromFile)
/// - p4 populate
#[allow(non_snake_case, dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct FileAction {
    pub action: String,
    pub clientFile: String,
    pub depotFile: String,
    pub fileSize: String,
    pub rev: String,
    // FIXME: not complete
}

/// TODO: Should be an output of P4Clients / P4Workspaces in the future
/// Not to confuse it with the form of P4Workspace and P4Client
/// conversion may be desired, unclear if it can be shared by multiple commands
#[allow(non_snake_case, dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct Workspace {
    pub Backup: String,
    pub Access: String,
    pub Description: String,
    pub Host: String,
    pub LineEnd: String,
    pub Options: String,
    pub Owner: String,
    pub Root: String,
    pub Stream: Option<String>,
    pub SubmitOptions: String,
    pub Update: String,
    pub client: String,
    // FIXME: not complete
}
