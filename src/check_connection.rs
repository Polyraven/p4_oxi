use crate::P4Out;
use crate::client_version::P4ClientVersion;
use crate::cmd::P4Info;
use crate::error::P4Error;
use crate::p4_cmd::P4Cmd;

const MIN_JSON_VERSION: P4ClientVersion = P4ClientVersion {
    major: 2016,
    minor: 1,
};

fn check_p4_client_available_and_compatible() -> Result<(), P4Error> {
    let client_version = P4ClientVersion::get()?;

    if client_version < MIN_JSON_VERSION {
        Err(P4Error::VersionMismatch(format!(
            "Minimum P4-Cli client version not satisfied: {}.{}",
            MIN_JSON_VERSION.major, MIN_JSON_VERSION.minor
        )))
    } else {
        Ok(())
    }
}

fn check_connection() -> Result<(), P4Error> {
    match P4Info::default().run()?.first()? {
        P4Out::Content(_) => Ok(()),
        other => Err(P4Error::UnexpectedOutput(other.erase_content_type())),
    }
}

/// Verify that the p4 client is installed, meets the minimum version, and can reach the server.
pub fn p4_check_ready() -> Result<(), P4Error> {
    check_p4_client_available_and_compatible()?;
    check_connection()?;
    Ok(())
}
