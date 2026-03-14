//! Gets file informations of each file.
//!
//! <https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/p4_fstat.html#p4_fstat>
//!

use serde::Deserialize;

use crate::prelude::*;

#[derive(Default, Debug)]
/// Argument builder for the fstat command.
///
/// Example:
///
/// ```rust, no_run
/// let fstats = P4Fstat::of_one(wildcard).execute()?
/// ```
pub struct P4FStat {
    #[doc = include_str!("wildcards.md")]
    pub wildcards: Vec<String>,
    /// Produce fstat output for only the first max files.
    pub max: Option<i32>,
}

#[allow(non_snake_case, dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct P4FStatOut {
    /// !
    pub clientFile: String,
    pub depotFile: String,
    pub effectiveComponentType: String,
    pub haveRev: Option<String>,
    pub headRev: String,
    pub headAction: String,
    pub headChange: String,
    pub headModTime: String,
    pub headTime: String,
    pub isMapped: String,
    pub pathPermission: Option<String>,
    pub pathSource: String,
    pub pathType: String,
    // Fixme: not complete
}

impl P4CmdArgs<Vec<P4FStatOut>> for P4FStat {
    fn to_p4_cmd(&self) -> P4Cmd {
        let mut p4_cmd = P4Cmd::raw("fstat");

        if let Some(v) = self.max {
            p4_cmd = p4_cmd.with_arg_value("-m", v.to_string());
        };

        p4_cmd.wildcards = self.wildcards.clone();

        p4_cmd
    }
}

// Constructors
impl P4FStat {
    pub fn of_one(wildcard: &str) -> Self {
        Self {
            wildcards: vec![wildcard.to_string()],
            ..Default::default()
        }
    }

    pub fn of_many(wildcards: Vec<&str>) -> Self {
        Self {
            wildcards: wildcards.iter().map(|w| w.to_string()).collect(),
            ..Default::default()
        }
    }
}

// Convinience functions:
pub fn get_list_of_files(wildcards: Vec<&str>) -> Result<Vec<String>, String> {
    Ok(P4FStat::of_many(wildcards)
        .execute()?
        .iter()
        .map(|r| r.clientFile.clone())
        .collect())
}
