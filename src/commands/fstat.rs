use crate::handled_command::HandledCommand;

pub use crate::p4_command::P4Command;

/// https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/p4_fstat.html#p4_fstat
#[derive(Default, Debug)]
pub struct FStat {
    pub wildcards: Vec<String>,
}

impl P4Command for FStat {
    fn build(self) -> HandledCommand {
        self.prefix().arg("fstat").args(&self.wildcards)
    }

    fn execute(self) -> Result<Vec<String>, String> {
        self.build().execute()?;
        Ok(vec![]) // TODO: collect results and return them
    }
}

/// these short implementations should make it so that it is always easy to use.
impl FStat {
    pub fn of_wildcard(wildcard: &str) -> Self {
        Self {
            wildcards: vec![wildcard.to_string()],
        }
    }
}
