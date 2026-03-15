//! Gets file informations of each file.
//!
//! <https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/p4_fstat.html#p4_fstat>
//!

use serde::Deserialize;

use crate::*;

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

impl From<P4FStat> for P4RawCmd {
    fn from(value: P4FStat) -> Self {
        let mut p4_cmd = P4RawCmd::new("fstat");

        if let Some(v) = value.max {
            p4_cmd = p4_cmd.with_arg_value("-m", v);
        };

        p4_cmd.wildcards = value.wildcards.clone();

        p4_cmd
    }
}

impl P4Cmd for P4FStat {
    type Output = P4FStatOut;
}

// Constructors
impl P4FStat {
    pub fn of_one(wildcard: impl Into<String>) -> Self {
        Self {
            wildcards: vec![wildcard.into()],
            max: Some(1),
            ..Default::default()
        }
    }

    pub fn of_many(wildcards: &[impl ToString]) -> Self {
        Self {
            wildcards: wildcards.iter().map(|w| w.to_string()).collect(),
            ..Default::default()
        }
    }
}
