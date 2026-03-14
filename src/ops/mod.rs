//! High-level operations composed of one or more P4 commands.
//!
//! Unlike [`cmd`](crate::cmd) which maps 1:1 to P4 CLI commands,
//! functions here express user intent and may change their underlying
//! implementation without affecting the public API.

mod files;

pub use files::*;
