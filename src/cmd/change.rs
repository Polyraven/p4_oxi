//! Create or edit a changelist.
//!
//! <https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/p4_change.html#p4_change>
//!
//! This command works by reading and writing a P4 Form through std out and std in.

use serde::{Deserialize, Serialize};

use crate::{
    P4FormCmd, P4RawCmd,
    cmd::info::P4Info,
    error::P4Error,
    p4_cmd::{P4Cmd, P4Form},
    p4_lensed_cmd::{P4CmdMode, P4LensedCmd, P4RunEager, P4RunRead, P4RunSubmit},
};

// Command struct //////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Default)]
pub struct P4Change {
    pub changelist_id: Option<u64>,            // positional
    pub delete: bool,                          // -d
    pub force: bool,                           // -f
    pub update: bool,                          // -u
    pub arbitrary_job_status: bool,            // -s
    pub as_pre_submit_changelist_id: bool,     // -O
    pub as_identity_changelist_id: bool,       // -I
    pub change_type: Option<ChangeVisibility>, // -t restricted|public
    pub user: Option<String>,                  // -U user
    pub server_id: Option<String>,             // --serverid=X
}

// Form and/or Output /////////////////////////////////////////////////////////

#[allow(non_snake_case, dead_code)]
#[derive(Debug, Deserialize, Serialize, Default)]
/// Used when creating a changelist but can be also a return value.
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
    /// Creates a new changelist form with the given description.
    ///
    /// ```rust, no_run
    /// let form = P4ChangeForm::new("my_changelist")?;
    /// let changelist_id = P4Change::for_submit().submit(form)?;
    /// ```
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

// Command Specific Types /////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub enum ChangeVisibility {
    Restricted,
    Public,
}

impl ChangeVisibility {
    fn as_str(&self) -> &str {
        match self {
            ChangeVisibility::Restricted => "restricted",
            ChangeVisibility::Public => "public",
        }
    }
}

// Builder methods on P4Change /////////////////////////////////////////////////

impl P4Change {
    pub fn with_changelist_id(mut self, id: u64) -> Self {
        self.changelist_id = Some(id);
        self
    }

    pub fn with_force(mut self) -> Self {
        self.force = true;
        self
    }

    pub fn with_update(mut self) -> Self {
        self.update = true;
        self
    }

    pub fn with_arbitrary_job_status(mut self) -> Self {
        self.arbitrary_job_status = true;
        self
    }

    pub fn with_as_pre_submit_changelist_id(mut self) -> Self {
        self.as_pre_submit_changelist_id = true;
        self
    }

    pub fn with_as_identity_changelist_id(mut self) -> Self {
        self.as_identity_changelist_id = true;
        self
    }

    pub fn with_change_type(mut self, vis: ChangeVisibility) -> Self {
        self.change_type = Some(vis);
        self
    }

    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    pub fn with_server_id(mut self, id: impl Into<String>) -> Self {
        self.server_id = Some(id.into());
        self
    }
}

// Factory methods /////////////////////////////////////////////////////////////

impl P4Change {
    /// p4 change -d [-f -s -O] changelist
    pub fn for_delete(changelist_id: u64) -> P4ChangeDelete {
        P4LensedCmd::new(P4Change {
            changelist_id: Some(changelist_id),
            delete: true,
            ..Default::default()
        })
    }

    /// p4 change -o [-s] [-f] [[-O|-I] changelist]
    pub fn for_read(changelist_id: Option<u64>) -> P4ChangeRead {
        P4LensedCmd::new(P4Change {
            changelist_id,
            ..Default::default()
        })
    }

    /// p4 change -i [-s] [-f | -u]
    pub fn for_submit() -> P4ChangeSubmit {
        P4LensedCmd::new(P4Change::default())
    }

    /// p4 change -t restricted|public [-U user] [-f|-u|-O|-I] changelist
    pub fn for_set_type(changelist_id: u64, vis: ChangeVisibility) -> P4ChangeSetType {
        P4LensedCmd::new(P4Change {
            changelist_id: Some(changelist_id),
            change_type: Some(vis),
            ..Default::default()
        })
    }

    /// p4 change -U user [-t restricted|public] [-f] changelist
    pub fn for_set_user(changelist_id: u64, user: impl Into<String>) -> P4ChangeSetUser {
        P4LensedCmd::new(P4Change {
            changelist_id: Some(changelist_id),
            user: Some(user.into()),
            ..Default::default()
        })
    }

    /// p4 change -d -f --serverid=X changelist
    pub fn for_cross_server_delete(
        changelist_id: u64,
        server_id: impl Into<String>,
    ) -> P4ChangeCrossServerDelete {
        P4LensedCmd::new(P4Change {
            changelist_id: Some(changelist_id),
            delete: true,
            force: true,
            server_id: Some(server_id.into()),
            ..Default::default()
        })
    }
}

// Modes & Lensed builder passthroughs per mode ////////////////////////////////

p4_cmd_modes!(
    P4Change,
    [
        // p4 change -d [-f -s -O] changelist
        (P4ChangeDelete, ChangeDelete, P4RunEager),
        // p4 change -o [-s] [-f] [[-O|-I] changelist]
        (P4ChangeRead, ChangeRead, P4RunRead),
        // p4 change -i [-s] [-f | -u]
        (P4ChangeSubmit, ChangeSubmit, P4RunSubmit),
        // p4 change -t restricted|public [-U user] [-f|-u|-O|-I] changelist
        (P4ChangeSetType, ChangeSetType, P4RunEager),
        // p4 change -U user [-t restricted|public] [-f] changelist
        (P4ChangeSetUser, ChangeSetUser, P4RunEager),
        // p4 change -d -f --serverid=X changelist
        (
            P4ChangeCrossServerDelete,
            ChangeCrossServerDelete,
            P4RunEager
        ),
    ]
);

impl P4ChangeDelete {
    p4_expose!(with_force);
    p4_expose!(with_arbitrary_job_status);
    p4_expose!(with_as_pre_submit_changelist_id);
}

impl P4ChangeRead {
    p4_expose!(with_arbitrary_job_status);
    p4_expose!(with_force);
    p4_expose!(with_as_pre_submit_changelist_id);
    p4_expose!(with_as_identity_changelist_id);
}

impl P4ChangeSubmit {
    p4_expose!(with_arbitrary_job_status);
    p4_expose!(with_force);
    p4_expose!(with_update);
}

impl P4ChangeSetType {
    p4_expose!(with_user, user: String);
    p4_expose!(with_force);
    p4_expose!(with_update);
    p4_expose!(with_as_pre_submit_changelist_id);
    p4_expose!(with_as_identity_changelist_id);
}

impl P4ChangeSetUser {
    p4_expose!(with_change_type, vis: ChangeVisibility);
    p4_expose!(with_force);
}

impl P4ChangeCrossServerDelete {
    // nothing to expose here, all variables already set.
}

// Trait impls /////////////////////////////////////////////////////////////////

impl From<P4Change> for P4RawCmd {
    fn from(value: P4Change) -> Self {
        let mut raw = P4RawCmd::new("change");

        if value.delete {
            raw = raw.with_arg("-d");
        }
        if value.force {
            raw = raw.with_arg("-f");
        }
        if value.update {
            raw = raw.with_arg("-u");
        }
        if value.arbitrary_job_status {
            raw = raw.with_arg("-s");
        }
        if value.as_pre_submit_changelist_id {
            raw = raw.with_arg("-O");
        }
        if value.as_identity_changelist_id {
            raw = raw.with_arg("-I");
        }

        if let Some(vis) = &value.change_type {
            raw = raw.with_arg_value("-t", vis.as_str());
        }
        if let Some(user) = &value.user {
            raw = raw.with_arg_value("-U", user);
        }
        if let Some(server_id) = &value.server_id {
            raw = raw.with_arg(format!("--serverid={server_id}"));
        }

        // changelist_id is positional, goes last
        if let Some(changelist_id) = value.changelist_id {
            raw = raw.with_arg(changelist_id);
        }

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

////////////////////////////////////////////////////////////////////////////////
