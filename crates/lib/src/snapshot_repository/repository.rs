use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    #[serde(rename = "default_snapshot_repo")]
    pub default_snapshot_repo: Option<DefaultSnapshotRepo>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultSnapshotRepo {
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub uuid: Option<String>,
    pub settings: Option<Settings>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub readonly: Option<String>,
    pub location: Option<String>,
}
