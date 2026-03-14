use crate::cmd::info::P4Info;
use crate::cmd::version::{P4ClientVersion, P4Version};
use crate::error::P4Error;
use crate::p4_cmd::P4Cmd;

const MIN_JSON_VERSION: P4ClientVersion = P4ClientVersion {
    major: 2016,
    minor: 1,
};

fn check_p4_client_available_and_compatible() -> Result<(), P4Error> {
    let client_version = P4Version.execute()?;

    if client_version.major > MIN_JSON_VERSION.major {
        return Ok(());
    } else if client_version.major == MIN_JSON_VERSION.major
        && client_version.minor >= MIN_JSON_VERSION.minor
    {
        return Ok(());
    } else {
        return Err(P4Error::VersionMismatch(format!(
            "Minimum P4-Cli client version not satisfied: {}.{}",
            MIN_JSON_VERSION.major, MIN_JSON_VERSION.minor
        )));
    }
}

fn check_connection() -> Result<(), P4Error> {
    P4Info::default().single()?;
    // TODO: there is a bit more that can be done here,
    // running p4 info on itself does not mean that the client can run commands against the database.
    Ok(())
}

/// Check if P4 client is available and if a connection to the server can be established.
pub fn p4_can_run_commands() -> Result<(), P4Error> {
    check_p4_client_available_and_compatible()?;
    check_connection()?;
    Ok(())
}
