use serde::{Deserialize, Serialize};

// use serde_derive::Deserialize;
// use serde_derive::Serialize;

// pub type Root = Vec<ElasticIndex>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheck {
    #[serde(rename = "cluster_name")]
    pub cluster_name: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "timed_out")]
    pub timed_out: Option<bool>,
    #[serde(rename = "number_of_nodes")]
    pub number_of_nodes: Option<i128>,
    #[serde(rename = "number_of_data_nodes")]
    pub number_of_data_nodes: Option<i128>,
    #[serde(rename = "active_primary_shards")]
    pub active_primary_shards: Option<i128>,
    #[serde(rename = "active_shards")]
    pub active_shards: Option<i128>,
    #[serde(rename = "relocating_shards")]
    pub relocating_shards: Option<i128>,
    #[serde(rename = "initializing_shards")]
    pub initializing_shards: Option<i128>,
    #[serde(rename = "unassigned_shards")]
    pub unassigned_shards: Option<i128>,
    #[serde(rename = "delayed_unassigned_shards")]
    pub delayed_unassigned_shards: Option<i128>,
    #[serde(rename = "number_of_pending_tasks")]
    pub number_of_pending_tasks: Option<i128>,
    #[serde(rename = "number_of_in_flight_fetch")]
    pub number_of_in_flight_fetch: Option<i128>,
    #[serde(rename = "task_max_waiting_in_queue_millis")]
    pub task_max_waiting_in_queue_millis: Option<i128>,
    #[serde(rename = "active_shards_percent_as_number")]
    pub active_shards_percent_as_number: Option<f64>,
}
