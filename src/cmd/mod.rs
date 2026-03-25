//! P4 CLI Command implementations.
//!
//! All valid P4 commands (not all of them are implemented):
//! <https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/commands.html>
//!
//! Each commands is implemented in its own file named after the command.

pub(crate) fn parse_or_default<T: Default + std::str::FromStr>(s: Option<String>) -> T {
    s.and_then(|s| s.parse().ok()).unwrap_or_default()
}

mod change;
mod fstat;
mod info;
mod workspace;

pub use change::{P4Change, P4ChangeForm, P4ChangeVisibility, P4Changelist, P4ChangelistForm};

pub use workspace::{
    P4Client, P4ClientForm, P4Workspace, P4WorkspaceForm, P4WorkspaceShelvedPolicy, P4WorkspaceType,
};

pub use fstat::{
    P4FStat, P4FStatOptionalOutput, P4FStatOut, P4FStatRestrict, P4FStatSort, P4FstatOutHead,
};
pub use info::{P4Info, P4InfoOut};
