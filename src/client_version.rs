use std::process::Command;

use crate::error::P4Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct P4ClientVersion {
    pub major: u32,
    pub minor: u32,
}

impl P4ClientVersion {
    /// Checks the local p4 client version by running `p4 -V`.
    /// Does not require server connectivity.
    pub fn get() -> Result<Self, P4Error> {
        let output = Command::new("p4")
            .arg("-V")
            .output()
            .map_err(|e| P4Error::ProcessError("Failed to run p4 -V".to_string(), Some(e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        let rev_line = stdout
            .lines()
            .find(|l| l.starts_with("Rev."))
            .ok_or_else(|| P4Error::ParseError("No Rev. line found in p4 -V output".to_string()))?;

        // Format: Rev. P4/<platform>/<major>.<minor>/<changelist> (<date>).
        let parts: Vec<&str> = rev_line.split('/').collect();
        let version_str = parts
            .get(2)
            .ok_or_else(|| P4Error::ParseError(format!("Unexpected Rev. format: {rev_line}")))?;

        let mut ver_parts = version_str.splitn(2, '.');
        let major: u32 = ver_parts
            .next()
            .and_then(|s| s.trim().parse().ok())
            .ok_or_else(|| {
                P4Error::ParseError(format!("Could not parse major version from: {version_str}"))
            })?;
        let minor: u32 = ver_parts
            .next()
            .and_then(|s| s.trim().parse().ok())
            .ok_or_else(|| {
                P4Error::ParseError(format!("Could not parse minor version from: {version_str}"))
            })?;

        Ok(Self { major, minor })
    }
}
