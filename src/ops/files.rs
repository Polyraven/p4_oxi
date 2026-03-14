use crate::cmd::fstat::P4FStat;
use crate::error::P4Error;
use crate::p4_cmd::P4Cmd;
use crate::p4_out::P4Out;

/// Returns a list of file paths matching the given wildcards.
pub fn get_list_of_files(wildcards: &[&str]) -> Result<Vec<String>, P4Error> {
    Ok(P4FStat::of_many(wildcards)
        .run()?
        .filter_map(P4Out::take_content)
        .map(|v| v.clientFile)
        .collect())
}
