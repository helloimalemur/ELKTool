use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshots {
    pub snapshots: Option<Vec<Snapshot>>,
    pub total: Option<i128>,
    pub remaining: Option<i128>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub snapshot: Option<String>,
    pub uuid: Option<String>,
    pub repository: Option<String>,
    #[serde(rename = "version_id")]
    pub version_id: Option<f64>,
    pub version: Option<String>,
    pub indices: Option<Vec<Value>>,
    #[serde(rename = "data_streams")]
    pub data_streams: Option<Vec<Value>>,
    #[serde(rename = "feature_states")]
    pub feature_states: Option<Vec<Value>>,
    #[serde(rename = "include_global_state")]
    pub include_global_state: Option<bool>,
    pub state: Option<String>,
    #[serde(rename = "start_time")]
    pub start_time: Option<String>,
    #[serde(rename = "start_time_in_millis")]
    pub start_time_in_millis: Option<i128>,
    #[serde(rename = "end_time")]
    pub end_time: Option<String>,
    #[serde(rename = "end_time_in_millis")]
    pub end_time_in_millis: Option<i128>,
    #[serde(rename = "duration_in_millis")]
    pub duration_in_millis: Option<i128>,
    pub failures: Option<Vec<Value>>,
    pub shards: Option<Shards>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shards {
    pub total: Option<i128>,
    pub failed: Option<i128>,
    pub successful: Option<i128>,
}

// Snapshot creation confirmation
//
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapShotMetadata {
    pub taken_by: String,
    pub taken_because: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotCreationConfirmation {
    pub indices: String,
    pub ignore_unavailable: bool,
    pub include_global_state: bool,
    pub expand_wildcards: String,
}
