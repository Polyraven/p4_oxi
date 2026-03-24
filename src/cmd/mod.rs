//! P4 CLI Command implementations.
//!
//! All valid P4 commands (not all of them are implemented):\
//! <https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/commands.html>
//!
//! Most of the commands are implemented in their own file,
//! with a few exceptions in special cases.

pub(crate) fn parse_or_default<T: Default + std::str::FromStr>(s: Option<String>) -> T {
    s.and_then(|s| s.parse().ok()).unwrap_or_default()
}

pub mod change;
pub mod fstat;
pub mod info;
pub mod workspace;

pub use change::{P4Change, P4ChangeForm, P4ChangeVisibility};

pub use workspace::{
    P4Client, P4ClientForm, P4Workspace, P4WorkspaceForm, P4WorkspaceShelvedPolicy, P4WorkspaceType,
};

pub use fstat::{P4FStat, P4FStatOut, P4FStatOptionalOutput, P4FStatRestrict, P4FStatSort};
pub use info::{P4Info, P4InfoOut};

pub mod client {
    pub use super::workspace::{
        P4Client, P4ClientForm, P4Workspace, P4WorkspaceForm, P4WorkspaceShelvedPolicy,
        P4WorkspaceType,
    };
}
