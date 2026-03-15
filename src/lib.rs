//! Work in progress crate to interact with P4 server.\
//! Mostly planned/intended to be used fro client side applications.

#[macro_use]
mod p4_lensed_cmd;

mod check_connection;
mod client_version;
pub mod cmd;
mod error;
mod global_options;
mod iter_cmd;
pub mod ops;
mod outputs;
mod p4_cmd;
mod p4_out;
mod p4_running_cmd;

pub use crate::check_connection::p4_can_run_commands;
pub use crate::client_version::P4ClientVersion;
pub use crate::error::P4Error;
pub use crate::p4_cmd::{P4Cmd, P4Form, P4FormCmd, P4RawCmd};
pub use crate::p4_out::P4Out;
pub use crate::p4_running_cmd::{P4RunningCmd, P4TypedRunningCmd};
