//! Create, edit, switch, or delete a workspace (client).
//!
//! <https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/p4_client.html>
//!
//! This command works by reading and writing a P4 Form through std out and std in.

use serde::{Deserialize, Serialize};

use crate::{p4_lensed_cmd::*, *};

// Aliases /////////////////////////////////////////////////////////////////////

pub type P4Client = P4Workspace;
pub type P4ClientForm = P4WorkspaceForm;

// Command struct //////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Default)]
pub struct P4Workspace {
    name: Option<String>,                    // positional [name]
    delete: bool,                            // -d
    force: bool,                             // -f
    switch: bool,                            // -s
    template: Option<String>,                // -t template
    workspace_type: Option<P4WorkspaceType>, // -T type
    stream: Option<String>,                  // -S stream (supports stream@change)
    at_change: Option<u64>,                  // -c change
    server_id: Option<String>,               // --serverid=X
    force_shelved_keep: bool,                // -Fs
    force_shelved_delete: bool,              // -Fd
}

// Form / Output ///////////////////////////////////////////////////////////////

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

// Command Builders ////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub enum P4WorkspaceType {
    Writeable,
    Readonly,
    Partitioned,
    PartitionedJnl,
    Graph,
}

impl P4WorkspaceType {
    fn as_str(&self) -> &str {
        match self {
            P4WorkspaceType::Writeable => "writeable",
            P4WorkspaceType::Readonly => "readonly",
            P4WorkspaceType::Partitioned => "partitioned",
            P4WorkspaceType::PartitionedJnl => "partitioned-jnl",
            P4WorkspaceType::Graph => "graph",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum P4WorkspaceShelvedPolicy {
    Keep,
    Delete,
}

impl P4Workspace {
    fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    fn with_force(mut self) -> Self {
        self.force = true;
        self
    }

    fn with_template(mut self, template: impl Into<String>) -> Self {
        self.template = Some(template.into());
        self
    }

    fn with_workspace_type(mut self, wt: P4WorkspaceType) -> Self {
        self.workspace_type = Some(wt);
        self
    }

    fn with_stream(mut self, stream: impl Into<String>) -> Self {
        self.stream = Some(stream.into());
        self
    }

    fn with_at_change(mut self, change: u64) -> Self {
        self.at_change = Some(change);
        self
    }

    fn with_server_id(mut self, id: impl Into<String>) -> Self {
        self.server_id = Some(id.into());
        self
    }

    fn with_shelved_policy(mut self, policy: P4WorkspaceShelvedPolicy) -> Self {
        self.force = true;
        match policy {
            P4WorkspaceShelvedPolicy::Keep => self.force_shelved_keep = true,
            P4WorkspaceShelvedPolicy::Delete => self.force_shelved_delete = true,
        }
        self
    }

    fn with_force_shelved_keep(mut self) -> Self {
        self.force_shelved_keep = true;
        self
    }
}

// Modes & Lensed builder passthroughs per mode ////////////////////////////////

// Unrestricted builder -- use .into_cmd() to execute via P4Cmd / P4FormCmd traits.
p4_lens!(P4WorkspaceWild, P4Workspace, WorkspaceWild, P4RunNone);
impl P4Workspace {
    pub fn wild_west() -> P4WorkspaceWild {
        P4LensedCmd::new(P4Workspace::default())
    }
}
impl P4WorkspaceWild {
    p4_expose!(with_name, name: String);
    p4_expose!(with_force);
    p4_expose!(with_template, template: String);
    p4_expose!(with_workspace_type, wt: P4WorkspaceType);
    p4_expose!(with_stream, stream: String);
    p4_expose!(with_at_change, change: u64);
    p4_expose!(with_server_id, id: String);
    p4_expose!(with_shelved_policy, policy: P4WorkspaceShelvedPolicy);
    p4_expose!(with_force_shelved_keep);
}

// p4 client -o [-t template] [-T type] [-S stream [-c change]] [name]
p4_lens!(P4WorkspaceRead, P4Workspace, WorkspaceRead, P4RunRead);
impl P4Workspace {
    pub fn for_read(name: Option<&str>) -> P4WorkspaceRead {
        P4LensedCmd::new(P4Workspace {
            name: name.map(str::to_string),
            ..Default::default()
        })
    }
}
impl P4WorkspaceRead {
    p4_expose!(with_template, template: String);
    p4_expose!(with_workspace_type, wt: P4WorkspaceType);
    p4_expose!(with_stream, stream: String);
    p4_expose!(with_at_change, change: u64);
}

// p4 client -i [-f]
p4_lens!(P4WorkspaceSubmit, P4Workspace, WorkspaceSubmit, P4RunSubmit);
impl P4Workspace {
    pub fn for_submit() -> P4WorkspaceSubmit {
        P4LensedCmd::new(P4Workspace::default())
    }
}
impl P4WorkspaceSubmit {
    p4_expose!(with_force);
}

// p4 client -d [-f [-Fs|-Fd]] name
p4_lens!(P4WorkspaceDelete, P4Workspace, WorkspaceDelete, P4RunEager);
impl P4Workspace {
    pub fn for_delete(name: impl Into<String>) -> P4WorkspaceDelete {
        P4LensedCmd::new(P4Workspace {
            name: Some(name.into()),
            delete: true,
            ..Default::default()
        })
    }
}
impl P4WorkspaceDelete {
    /// -f: transitions to P4WorkspaceForcedDelete, unlocking [-Fs|-Fd]
    pub fn with_force(self) -> P4WorkspaceForcedDelete {
        let inner = self.inner_cmd.with_force();
        P4LensedCmd::new(inner)
    }
}

// p4 client -d -f [-Fs|-Fd] name
p4_lens!(
    P4WorkspaceForcedDelete,
    P4Workspace,
    WorkspaceForcedDelete,
    P4RunEager
);
impl P4WorkspaceForcedDelete {
    p4_expose!(with_shelved_policy, policy: P4WorkspaceShelvedPolicy);
}

// p4 client -s [-f] -S stream [name]
p4_lens!(
    P4WorkspaceSwitchStream,
    P4Workspace,
    WorkspaceSwitchStream,
    P4RunEager
);
impl P4Workspace {
    pub fn for_switch_to_stream(stream: impl Into<String>) -> P4WorkspaceSwitchStream {
        P4LensedCmd::new(P4Workspace {
            switch: true,
            stream: Some(stream.into()),
            ..Default::default()
        })
    }
}
impl P4WorkspaceSwitchStream {
    p4_expose!(with_force);
    p4_expose!(with_name, name: String);
}

// p4 client -s [-f] -t template [name]
p4_lens!(
    P4WorkspaceSwitchTemplate,
    P4Workspace,
    WorkspaceSwitchTemplate,
    P4RunEager
);
impl P4Workspace {
    pub fn for_switch_to_template(template: impl Into<String>) -> P4WorkspaceSwitchTemplate {
        P4LensedCmd::new(P4Workspace {
            switch: true,
            template: Some(template.into()),
            ..Default::default()
        })
    }
}
impl P4WorkspaceSwitchTemplate {
    p4_expose!(with_force);
    p4_expose!(with_name, name: String);
}

// p4 client -d -f --serverid=X [-Fs] name
p4_lens!(
    P4WorkspaceCrossServerDelete,
    P4Workspace,
    WorkspaceCrossServerDelete,
    P4RunEager
);
impl P4Workspace {
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
impl P4WorkspaceCrossServerDelete {
    p4_expose!(with_force_shelved_keep);
}

////////////////////////////////////////////////////////////////////////////////
