//! Used to add the g-options in front of the commands.
//!
//! <https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/global.options.html>
//!
//! Modifies a lot how the command is actually ran, including:
//!
//! - output format
//! - workspace
//! - cwd
//! etc.
//!
//! Docs:

#[derive(Debug)]
/// Structure to hold the g-options in a human readable way.
pub struct GlobalOptions {
    /// If this is not set to true, this library won't be able to deserialize the returned values/
    /// There is still benefit in running without this, for example when sending a spec./
    /// The value of this is handled automatically in most cases.
    pub format_as_json: bool,
    // TODO: list is not exhaustive
}

impl Default for GlobalOptions {
    fn default() -> Self {
        Self {
            format_as_json: true,
        }
    }
}

impl GlobalOptions {
    /// Convert the GlobalOptions to valid p4 g-options arguments.
    pub fn to_arg_list(&self) -> Vec<&str> {
        let mut args: Vec<&str> = Vec::new();

        if self.format_as_json {
            args.extend(vec!["-ztag", "-Mj"]);
        }

        args
    }
}
