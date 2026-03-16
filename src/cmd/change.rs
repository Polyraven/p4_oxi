//! Create or edit a changelist.
//!
//! <https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/p4_change.html#p4_change>
//!
//! This command works by reading and writing a P4 Form through std out and std in.

use serde::{Deserialize, Serialize};

use crate::{p4_lensed_cmd::*, *};

// Aliases /////////////////////////////////////////////////////////////////////

pub type P4Changelist = P4Change;
pub type P4ChangelistForm = P4ChangeForm;

// Command struct //////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Default)]
pub struct P4Change {
    changelist_id: Option<u64>,              // positional
    delete: bool,                            // -d
    force: bool,                             // -f
    update: bool,                            // -u
    arbitrary_job_status: bool,              // -s
    as_pre_submit_changelist_id: bool,       // -O
    as_identity_changelist_id: bool,         // -I
    change_type: Option<P4ChangeVisibility>, // -t restricted|public
    user: Option<String>,                    // -U user
    server_id: Option<String>,               // --serverid=X
}

// Form / Output ///////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "PascalCase")]
/// Used when creating a changelist but can be also a return value.
pub struct P4ChangeForm {
    pub change: String,
    #[serde(rename = "Client")]
    pub workspace: String,
    pub user: String,
    pub status: String,
    pub description: String,
    pub date: String,
    #[serde(rename = "Type")]
    pub change_type: String,
    pub imported_by: String,
    pub identity: String,
    pub stream: String,
    pub jobs: String,
    pub files: String,
}

impl P4ChangeForm {
    pub fn blank() -> Result<Self, P4Error> {
        P4Change::for_read(None).read()
    }

    /// Creates a new changelist form with the given description.
    ///
    /// ```rust, no_run
    /// let form = P4ChangeForm::new("my_changelist")?;
    /// let changelist_id = P4Change::for_submit().submit(form)?;
    /// ```
    pub fn new(description: impl ToString) -> Result<Self, P4Error> {
        let mut change_form = Self::blank()?;
        change_form.description = description.to_string();

        Ok(change_form)
    }
}

impl P4Form for P4ChangeForm {
    fn multiline_fields(&self) -> &'static [&'static str] {
        &["Description", "Jobs", "Files"]
    }
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

// Command Builders ////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub enum P4ChangeVisibility {
    Restricted,
    Public,
}

impl P4ChangeVisibility {
    fn as_str(&self) -> &str {
        match self {
            P4ChangeVisibility::Restricted => "restricted",
            P4ChangeVisibility::Public => "public",
        }
    }
}

impl P4Change {
    fn with_changelist_id(mut self, id: u64) -> Self {
        self.changelist_id = Some(id);
        self
    }

    fn with_force(mut self) -> Self {
        self.force = true;
        self
    }

    fn with_update(mut self) -> Self {
        self.update = true;
        self
    }

    fn with_arbitrary_job_status(mut self) -> Self {
        self.arbitrary_job_status = true;
        self
    }

    fn with_as_pre_submit_changelist_id(mut self) -> Self {
        self.as_pre_submit_changelist_id = true;
        self
    }

    fn with_as_identity_changelist_id(mut self) -> Self {
        self.as_identity_changelist_id = true;
        self
    }

    fn with_change_type(mut self, vis: P4ChangeVisibility) -> Self {
        self.change_type = Some(vis);
        self
    }

    fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    fn with_server_id(mut self, id: impl Into<String>) -> Self {
        self.server_id = Some(id.into());
        self
    }
}

// Modes & Lensed builder passthroughs per mode ////////////////////////////////

// p4 change -d [-f -s -O] changelist
p4_lens!(P4ChangeDelete, ChangeDelete, P4Change, P4RunEager);
impl P4Change {
    pub fn for_delete(changelist_id: u64) -> P4ChangeDelete {
        P4LensedCmd::new(P4Change {
            changelist_id: Some(changelist_id),
            delete: true,
            ..Default::default()
        })
    }
}
impl P4ChangeDelete {
    p4_expose_many!(
        with_force;
        with_arbitrary_job_status;
        with_as_pre_submit_changelist_id;
    );
}

// p4 change -o [-s] [-f] [[-O|-I] changelist]
p4_lens!(P4ChangeRead, ChangeRead, P4Change, P4RunRead);
impl P4Change {
    pub fn for_read(changelist_id: Option<u64>) -> P4ChangeRead {
        P4LensedCmd::new(P4Change {
            changelist_id,
            ..Default::default()
        })
    }
}
impl P4ChangeRead {
    p4_expose_many!(
        with_arbitrary_job_status;
        with_force;
        with_as_pre_submit_changelist_id;
        with_as_identity_changelist_id;
    );
}

// p4 change -i [-s] [-f | -u]
p4_lens!(P4ChangeSubmit, ChangeSubmit, P4Change, P4RunSubmit);
impl P4Change {
    pub fn for_submit() -> P4ChangeSubmit {
        P4LensedCmd::new(P4Change::default())
    }
}
impl P4ChangeSubmit {
    p4_expose_many!(
        with_arbitrary_job_status;
        with_force;
        with_update;
    );
}

// p4 change -t restricted|public [-U user] [-f|-u|-O|-I] changelist
p4_lens!(P4ChangeSetType, ChangeSetType, P4Change, P4RunEager);
impl P4Change {
    pub fn for_set_type(changelist_id: u64, vis: P4ChangeVisibility) -> P4ChangeSetType {
        P4LensedCmd::new(P4Change {
            changelist_id: Some(changelist_id),
            change_type: Some(vis),
            ..Default::default()
        })
    }
}
impl P4ChangeSetType {
    p4_expose_many!(
        with_user, user: String;
        with_force;
        with_update;
        with_as_pre_submit_changelist_id;
        with_as_identity_changelist_id;
    );
}

// p4 change -U user [-t restricted|public] [-f] changelist
p4_lens!(P4ChangeSetUser, ChangeSetUser, P4Change, P4RunEager);
impl P4Change {
    pub fn for_set_user(changelist_id: u64, user: impl Into<String>) -> P4ChangeSetUser {
        P4LensedCmd::new(P4Change {
            changelist_id: Some(changelist_id),
            user: Some(user.into()),
            ..Default::default()
        })
    }
}
impl P4ChangeSetUser {
    p4_expose_many!(
        with_change_type, vis: P4ChangeVisibility;
        with_force;
    );
}

// p4 change -d -f --serverid=X changelist
p4_lens!(
    P4ChangeCrossServerDelete,
    ChangeCrossServerDelete,
    P4Change,
    P4RunEager
);
impl P4Change {
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
// nothing to expose here, all variables already set.

// Unrestricted builder -- use .into_cmd() to execute via P4Cmd / P4FormCmd traits.
p4_lens!(P4ChangeWild, ChangeWild, P4Change, P4RunManual);
impl P4Change {
    pub fn wild_west() -> P4ChangeWild {
        P4LensedCmd::new(P4Change::default())
    }
}
impl P4ChangeWild {
    p4_expose_many!(
        with_changelist_id, id: u64;
        with_force;
        with_update;
        with_arbitrary_job_status;
        with_as_pre_submit_changelist_id;
        with_as_identity_changelist_id;
        with_change_type, vis: P4ChangeVisibility;
        with_user, user: String;
        with_server_id, id: String;
    );
}

////////////////////////////////////////////////////////////////////////////////
