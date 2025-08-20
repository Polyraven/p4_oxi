//! Display information about the current P4 Server application and the shared versioning service.
//!
//! <https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/p4_info.html#p4_info>

use serde::Deserialize;

use crate::prelude::*;

#[derive(Default, Debug)]
/// Argument builder for the info command.
pub struct P4Info {
    /// exclude information (for example, the workspace root) that requires a database lookup.
    shortened: bool,
}

#[allow(non_snake_case, dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct P4InfoOut {
    pub clientHost: String,
    pub clientCwd: String,
    pub clientName: String,
    pub clientRoot: String,
    pub clientStream: String,
    pub serverAddress: String,
    pub userName: String,
    // Fixme: not complete
}

impl P4CmdArgs<P4InfoOut> for P4Info {
    fn to_p4_cmd(&self) -> P4Cmd {
        let mut p4_cmd = P4Cmd::raw("info");

        if self.shortened {
            p4_cmd = p4_cmd.with_arg("-s");
        };

        p4_cmd
    }

    fn execute(self) -> Result<P4InfoOut, String> {
        self.to_p4_cmd().execute()?.parse()?.single()
    }
}
