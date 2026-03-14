//! P4 CLI Command implementations.
//!
//! All valid P4 commands (not all of them are implemented):\
//! <https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/commands.html>
//!
//! Most of the commands are implemented in their own file,
//! with a few exceptions in special cases.
pub mod change;
pub mod fstat;
pub mod info;

pub use change::*;
pub use fstat::*;
pub use info::*;
