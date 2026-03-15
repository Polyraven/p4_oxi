//! Create or edit a changelist.
//!
//! <https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/p4_change.html#p4_change>
//!
//! This command works by reading and writing a P4 Form throught std out and std in.

use serde::{Deserialize, Serialize};

use crate::{
    P4FormCmd, P4RawCmd,
    cmd::info::P4Info,
    error::P4Error,
    p4_cmd::{P4Cmd, P4Form},
    p4_lensed_cmd::{P4LensedCmd, P4RunEager, P4RunRead},
};

#[allow(non_snake_case, dead_code)]
#[derive(Debug, Deserialize, Serialize, Default)]
/// Used when creating a changelist but can be also a return value.
///
/// Example when used to create a changelist:
/// ```rust, no_run
/// let form = P4ChangeForm::new("my_changelist")?;
/// let changelist_id = P4Change::default().submit_form(form)?;
/// ```
pub struct P4ChangeForm {
    pub Change: String,
    pub Client: String,
    pub User: String,
    pub Status: String,
    pub Description: String,
    ////////////////////
    // post-creation fields:
    pub Date: String,
    pub Type: String,
}

impl P4ChangeForm {
    pub fn new(description: &str) -> Result<Self, P4Error> {
        let info = P4Info::default().first_content()?;

        Ok(Self {
            Change: "new".to_string(),
            Client: info.clientName,
            User: info.userName,
            Status: "new".to_string(),
            Description: description.to_string(),
            ..Default::default()
        })
    }
}

impl P4Form for P4ChangeForm {
    fn convert(&self) -> String {
        format!(
            "Change: {}\nClient: {}\nUser: {}\nStatus: {}\nDescription:\n\t{}\n",
            self.Change, self.Client, self.User, self.Status, self.Description
        )
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct P4Change {
    pub changelist_id: Option<u64>,
}

impl P4Change {
    pub fn with_changelist_number(mut self, changelist_id: u64) -> Self {
        self.changelist_id = Some(changelist_id);
        self
    }

    pub fn for_read_form(changelist_id: u64) -> P4ChangeRead {
        P4LensedCmd::new(P4Change {
            changelist_id: Some(changelist_id),
            ..Default::default()
        })
    }
}

use crate::p4_lensed_cmd::P4CmdMode;

p4_cmd_modes!(
    P4Change,
    [
        (P4ChangeDelete, ChangeDelete, P4RunEager),
        (P4ChangeRead, ChangeRead, P4RunRead)
    ]
);

impl P4ChangeRead {
    p4_expose!(with_changelist_number, changelist_id: u64);
}

impl From<P4Change> for P4RawCmd {
    fn from(value: P4Change) -> Self {
        let mut raw = P4RawCmd::new("change");
        if let Some(changelist_id) = value.changelist_id {
            raw = raw.with_arg(changelist_id)
        };
        raw
    }
}

impl P4Cmd for P4Change {
    type Output = ();
}

impl P4FormCmd for P4Change {
    type FormSubmitOutput = u64;
    type Form = P4ChangeForm;

    fn parse_form_submit_raw_output(raw: String) -> Result<Self::FormSubmitOutput, P4Error> {
        // raw output example: "Change 1146494 created.\r\n"
        raw.split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .ok_or(P4Error::ParseError(raw))
    }
}
