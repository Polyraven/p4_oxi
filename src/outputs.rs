/// Holds the structs of deserialized output responses of the commands.
use serde::Deserialize;

#[allow(non_snake_case, dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct Info {
    pub clientHost: String,
    pub clientCwd: String,
    pub clientName: String,
    pub clientRoot: String,
    pub clientStream: String,
    pub serverAddress: String,
    // Fixme: not complete
}

#[allow(non_snake_case, dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct FStatOut {
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

#[allow(non_snake_case, dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct FileAction {
    pub action: String,
    pub clientFile: String,
    pub depotFile: String,
    pub fileSize: String,
    pub rev: String,
    // Fixme: not complete
}

#[allow(non_snake_case, dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct Workspace {
    pub Backup: String,
    pub Access: String,
    pub Description: String,
    pub Host: String,
    pub LineEnd: String,
    pub Options: String,
    pub Owner: String,
    pub Root: String,
    pub Stream: Option<String>,
    pub SubmitOptions: String,
    pub Update: String,
    pub client: String,
    // Fixme: not complete
}
