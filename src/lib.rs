//! Work in progress crate to interact with P4 server.\
//! Mostly planned/intended to be used fro client side applications.

pub mod cmd;
pub mod global_options;
mod handled_command;
pub mod handlers;
pub mod outputs;
pub mod p4cmd;
pub mod prelude;
pub mod warnings;
