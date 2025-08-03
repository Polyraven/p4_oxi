pub mod change;
/// Plan / Intent:
/// This sub-crate is meant to hold all the p4 commands implementations:
/// https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/commands.html
/// Thise commands would than implement their own handlers and return types,
/// Not all command would perhaps end up as it's own file, will see how it turns out.
pub mod fstat;
pub mod info;
