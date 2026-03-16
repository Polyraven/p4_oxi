//! Create, edit, switch, or delete a workspace (client).
//!
//! <https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/p4_client.html>
//!
//! This command works by reading and writing a P4 Form through std out and std in.

use serde::{Deserialize, Serialize};

use crate::{
    P4FormCmd, P4RawCmd,
    error::P4Error,
    p4_cmd::{P4Cmd, P4Form},
    p4_lensed_cmd::{P4CmdMode, P4LensedCmd, P4RunEager, P4RunRead, P4RunSubmit},
};

// Command struct //////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Default)]
pub struct P4Workspace {
    pub name: Option<String>,                  // positional [name]
    pub delete: bool,                          // -d
    pub force: bool,                           // -f
    pub switch: bool,                          // -s
    pub template: Option<String>,              // -t template
    pub workspace_type: Option<WorkspaceType>, // -T type
    pub stream: Option<String>,                // -S stream (supports stream@change)
    pub at_change: Option<u64>,                // -c change
    pub server_id: Option<String>,             // --serverid=X
    pub force_shelved_keep: bool,              // -Fs
    pub force_shelved_delete: bool,            // -Fd
}

// Form ////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "PascalCase")]
pub struct P4WorkspaceForm {
    #[serde(rename = "Client")]
    pub workspace: String,
    pub owner: String,
    pub host: String,
    pub description: String,
    pub root: String,
    pub alt_roots: String,
    pub options: String,
    pub submit_options: String,
    pub line_end: String,
    pub stream: String,
    pub stream_at_change: String,
    #[serde(rename = "ServerID")]
    pub server_id: String,
    pub view: String,
    pub change_view: String,
    pub update: String,
    pub access: String,
    #[serde(rename = "Type")]
    pub workspace_type: String,
    pub backup: String,
    pub limit_view: String,
}

impl P4WorkspaceForm {
    pub fn blank() -> Result<Self, P4Error> {
        P4Workspace::for_read(None).read()
    }
}

impl P4Form for P4WorkspaceForm {
    fn multiline_fields(&self) -> &'static [&'static str] {
        &["Description", "View", "ChangeView", "AltRoots", "LimitView"]
    }
}

// Command Specific Types //////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub enum WorkspaceType {
    Writeable,
    Readonly,
    Partitioned,
    PartitionedJnl,
    Graph,
}

impl WorkspaceType {
    fn as_str(&self) -> &str {
        match self {
            WorkspaceType::Writeable => "writeable",
            WorkspaceType::Readonly => "readonly",
            WorkspaceType::Partitioned => "partitioned",
            WorkspaceType::PartitionedJnl => "partitioned-jnl",
            WorkspaceType::Graph => "graph",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ShelvedPolicy {
    Keep,
    Delete,
}

// Builder methods on P4Workspace //////////////////////////////////////////////

impl P4Workspace {
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_force(mut self) -> Self {
        self.force = true;
        self
    }

    pub fn with_template(mut self, template: impl Into<String>) -> Self {
        self.template = Some(template.into());
        self
    }

    pub fn with_workspace_type(mut self, wt: WorkspaceType) -> Self {
        self.workspace_type = Some(wt);
        self
    }

    pub fn with_stream(mut self, stream: impl Into<String>) -> Self {
        self.stream = Some(stream.into());
        self
    }

    pub fn with_at_change(mut self, change: u64) -> Self {
        self.at_change = Some(change);
        self
    }

    pub fn with_server_id(mut self, id: impl Into<String>) -> Self {
        self.server_id = Some(id.into());
        self
    }

    pub fn with_shelved_policy(mut self, policy: ShelvedPolicy) -> Self {
        self.force = true;
        match policy {
            ShelvedPolicy::Keep => self.force_shelved_keep = true,
            ShelvedPolicy::Delete => self.force_shelved_delete = true,
        }
        self
    }

    pub fn with_force_shelved_keep(mut self) -> Self {
        self.force_shelved_keep = true;
        self
    }
}

// Factory methods /////////////////////////////////////////////////////////////

impl P4Workspace {
    /// p4 client -o [-t template] [-T type] [-S stream [-c change]] [name]
    pub fn for_read(name: Option<&str>) -> P4WorkspaceRead {
        P4LensedCmd::new(P4Workspace {
            name: name.map(str::to_string),
            ..Default::default()
        })
    }

    /// p4 client -i [-f]
    pub fn for_submit() -> P4WorkspaceSubmit {
        P4LensedCmd::new(P4Workspace::default())
    }

    /// p4 client -d [-f [-Fs|-Fd]] name
    pub fn for_delete(name: impl Into<String>) -> P4WorkspaceDelete {
        P4LensedCmd::new(P4Workspace {
            name: Some(name.into()),
            delete: true,
            ..Default::default()
        })
    }

    /// p4 client -s [-f] -S stream [name]
    pub fn for_switch_to_stream(stream: impl Into<String>) -> P4WorkspaceSwitchStream {
        P4LensedCmd::new(P4Workspace {
            switch: true,
            stream: Some(stream.into()),
            ..Default::default()
        })
    }

    /// p4 client -s [-f] -t template [name]
    pub fn for_switch_to_template(template: impl Into<String>) -> P4WorkspaceSwitchTemplate {
        P4LensedCmd::new(P4Workspace {
            switch: true,
            template: Some(template.into()),
            ..Default::default()
        })
    }

    /// p4 client -d -f --serverid=X [-Fs] name
    pub fn for_cross_server_delete(
        name: impl Into<String>,
        server_id: impl Into<String>,
    ) -> P4WorkspaceCrossServerDelete {
        P4LensedCmd::new(P4Workspace {
            name: Some(name.into()),
            delete: true,
            force: true,
            server_id: Some(server_id.into()),
            ..Default::default()
        })
    }
}

// Modes & Lensed builder passthroughs per mode ////////////////////////////////

// p4 client -o [-t template] [-T type] [-S stream [-c change]] [name]
pub struct WorkspaceRead;
impl P4CmdMode for WorkspaceRead {}
pub type P4WorkspaceRead = P4LensedCmd<P4Workspace, WorkspaceRead, P4RunRead>;

// p4 client -i [-f]
pub struct WorkspaceSubmit;
impl P4CmdMode for WorkspaceSubmit {}
pub type P4WorkspaceSubmit = P4LensedCmd<P4Workspace, WorkspaceSubmit, P4RunSubmit>;

// p4 client -d [-f [-Fs|-Fd]] name
pub struct WorkspaceDelete;
impl P4CmdMode for WorkspaceDelete {}
pub type P4WorkspaceDelete = P4LensedCmd<P4Workspace, WorkspaceDelete, P4RunEager>;

// p4 client -d -f [-Fs|-Fd] name
pub struct WorkspaceForcedDelete;
impl P4CmdMode for WorkspaceForcedDelete {}
pub type P4WorkspaceForcedDelete = P4LensedCmd<P4Workspace, WorkspaceForcedDelete, P4RunEager>;

// p4 client -s [-f] -S stream [name]
pub struct WorkspaceSwitchStream;
impl P4CmdMode for WorkspaceSwitchStream {}
pub type P4WorkspaceSwitchStream = P4LensedCmd<P4Workspace, WorkspaceSwitchStream, P4RunEager>;

// p4 client -s [-f] -t template [name]
pub struct WorkspaceSwitchTemplate;
impl P4CmdMode for WorkspaceSwitchTemplate {}
pub type P4WorkspaceSwitchTemplate = P4LensedCmd<P4Workspace, WorkspaceSwitchTemplate, P4RunEager>;

// p4 client -d -f --serverid=X [-Fs] name
pub struct WorkspaceCrossServerDelete;
impl P4CmdMode for WorkspaceCrossServerDelete {}
pub type P4WorkspaceCrossServerDelete =
    P4LensedCmd<P4Workspace, WorkspaceCrossServerDelete, P4RunEager>;

impl P4WorkspaceRead {
    p4_expose!(with_template, template: String);
    p4_expose!(with_workspace_type, wt: WorkspaceType);
    p4_expose!(with_stream, stream: String);
    p4_expose!(with_at_change, change: u64);
}

impl P4WorkspaceSubmit {
    p4_expose!(with_force);
}

impl P4WorkspaceDelete {
    /// -f: transitions to P4WorkspaceForcedDelete, unlocking [-Fs|-Fd]
    pub fn with_force(mut self) -> P4WorkspaceForcedDelete {
        self.inner_cmd.force = true;
        P4LensedCmd::new(self.inner_cmd)
    }
}

impl P4WorkspaceForcedDelete {
    /// [-Fs|-Fd]
    pub fn with_shelved_policy(mut self, policy: ShelvedPolicy) -> Self {
        match policy {
            ShelvedPolicy::Keep => self.inner_cmd.force_shelved_keep = true,
            ShelvedPolicy::Delete => self.inner_cmd.force_shelved_delete = true,
        }
        self
    }
}

impl P4WorkspaceSwitchStream {
    p4_expose!(with_force);
    p4_expose!(with_name, name: String);
}

impl P4WorkspaceSwitchTemplate {
    p4_expose!(with_force);
    p4_expose!(with_name, name: String);
}

impl P4WorkspaceCrossServerDelete {
    p4_expose!(with_force_shelved_keep);
}

// Trait impls /////////////////////////////////////////////////////////////////

impl From<P4Workspace> for P4RawCmd {
    fn from(value: P4Workspace) -> Self {
        let mut raw = P4RawCmd::new("client");

        if value.delete {
            raw = raw.with_arg("-d");
        }
        if value.force {
            raw = raw.with_arg("-f");
        }
        if value.switch {
            raw = raw.with_arg("-s");
        }

        if value.force_shelved_keep {
            raw = raw.with_arg("-Fs");
        }
        if value.force_shelved_delete {
            raw = raw.with_arg("-Fd");
        }

        if let Some(stream) = &value.stream {
            raw = raw.with_arg_value("-S", stream);
        }
        if let Some(change) = value.at_change {
            raw = raw.with_arg_value("-c", change);
        }
        if let Some(template) = &value.template {
            raw = raw.with_arg_value("-t", template);
        }
        if let Some(wt) = &value.workspace_type {
            raw = raw.with_arg_value("-T", wt.as_str());
        }
        if let Some(server_id) = &value.server_id {
            raw = raw.with_arg(format!("--serverid={server_id}"));
        }

        // name is positional, goes last
        if let Some(name) = &value.name {
            raw = raw.with_arg(name);
        }

        raw
    }
}

impl P4Cmd for P4Workspace {
    type Output = ();
}

impl P4FormCmd for P4Workspace {
    type FormSubmitOutput = String;
    type Form = P4WorkspaceForm;

    fn parse_form_submit_raw_output(raw: String) -> Result<Self::FormSubmitOutput, P4Error> {
        Ok(raw)
    }
}

////////////////////////////////////////////////////////////////////////////////
