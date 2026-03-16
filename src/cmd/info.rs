//! Display information about the current P4 Server application and the shared versioning service.
//!
//! <https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/p4_info.html#p4_info>

use serde::Deserialize;

use crate::*;

// Command struct //////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Default)]
pub struct P4Info {
    shortened: bool, // -s
}

// Output //////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct P4InfoOut {
    pub user_name: String,
    pub client_name: String,
    pub client_host: String,
    pub client_cwd: String,
    pub client_root: String,
    pub client_stream: String,
    pub peer_address: String,
    pub client_address: String,
    pub server_address: String,
    pub server_root: String,
    pub server_date: String,
    pub server_uptime: String,
    pub server_version: String,
    pub server_license: String,
    pub server_services: String,
    pub case_handling: String,
    pub client_version: String,
}

// Trait impls /////////////////////////////////////////////////////////////////

impl From<P4Info> for P4RawCmd {
    fn from(value: P4Info) -> Self {
        let mut raw = P4RawCmd::new("info");

        if value.shortened {
            raw = raw.with_arg("-s");
        }

        raw
    }
}

impl P4Cmd for P4Info {
    type Output = P4InfoOut;
}

// Command Builders ////////////////////////////////////////////////////////////

impl P4Info {
    pub fn with_shortened(mut self) -> Self {
        self.shortened = true;
        self
    }
}

////////////////////////////////////////////////////////////////////////////////
