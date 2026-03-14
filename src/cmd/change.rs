//! Create or edit a changelist.
//!
//! <https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/p4_change.html#p4_change>
//!
//! This command works by reading and writing a P4 Spec throught std out and std in.

use serde::{Deserialize, Serialize};

use crate::{
    cmd::info::P4Info,
    p4_cmd::{P4CmdArgs, P4Specification},
};

#[allow(non_snake_case, dead_code)]
#[derive(Debug, Deserialize, Serialize)]
/// Used when creating a changelist but can be also a return value.
///
/// Example when used to create a changelist (raw command)
/// ```rust, no_run
///
/// let output = P4Cmd::raw("change")
///    .with_arg("-i")
///    .with_input_spec(changelist_spec)
///    .execute()
///    .expect("changelist failed to be created");
///
/// ```
pub struct P4ChangelistSpec {
    Change: String,
    Client: String,
    User: String,
    Status: String,
    Description: String,
}

impl P4ChangelistSpec {
    pub fn new(description: &str) -> Result<Box<Self>, String> {
        let info = P4Info::default().execute()?;

        Ok(Box::new(Self {
            Change: "new".to_string(),
            Client: info.clientName,
            User: info.userName,
            Status: "new".to_string(),
            Description: description.to_string(),
        }))
    }
}

impl P4Specification for P4ChangelistSpec {
    fn convert(&self) -> String {
        format!(
            "Change: {}\nClient: {}\nUser: {}\nStatus: {}\nDescription:\n\t{}\n",
            self.Change, self.Client, self.User, self.Status, self.Description
        )
    }
}
