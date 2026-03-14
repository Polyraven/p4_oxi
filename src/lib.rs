//! Work in progress crate to interact with P4 server.\
//! Mostly planned/intended to be used fro client side applications.

mod check_connection;
pub mod cmd;
pub mod global_options;
pub mod iter_cmd;
pub mod outputs;
pub mod p4_cmd;
mod p4_running_cmd;
pub mod prelude;
pub mod warnings;
