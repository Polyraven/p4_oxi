/// Used to add the g-options in front of the commands.
///
/// Modifies a lot how the command is actually ran, including:
///
/// - output format
/// - workspace
/// - cwd
/// etc.
///
/// Docs:
/// https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/global.options.html
#[derive(Debug)]
pub struct GlobalOptions {
    /// If this is not set to true, this library won't be able to deserialize the returned values
    /// There still might be benefit in running without this sometimes.
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
    pub fn to_arg_list(&self) -> Vec<&str> {
        let mut args: Vec<&str> = Vec::new();

        if self.format_as_json {
            args.extend(vec!["-ztag", "-Mj"]);
        }

        args
    }
}
