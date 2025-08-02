use crate::{handlers::OutputHandler, outputs::FileAction, p4cmd::P4Cmd};

pub use crate::p4cmd::P4Execute;

/// https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/p4_fstat.html#p4_fstat
#[derive(Default, Debug)]
pub struct FStat {
    pub wildcards: Vec<String>,
}

impl From<FStat> for P4Cmd {
    fn from(fstat: FStat) -> Self {
        let handler = OutputHandler::<FileAction>::new();
        let mut p4_cmd = P4Cmd::new("fstat").with_output_handler(Box::new(handler));
        p4_cmd.wildcards = fstat.wildcards;
        p4_cmd
    }
}

impl P4Execute for FStat {
    fn execute(self) -> Result<(), String> {
        P4Cmd::from(self).execute()
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
