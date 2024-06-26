use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HAProxyIndexUpdate {
    pub index_name: String,
    pub index_id: String,
    pub new_field_name: String,
    pub new_field_value: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct HAProxyIndexUpdateScript {
    pub script: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElasticIndex {
    pub health: Option<String>,
    pub status: Option<String>,
    pub index: Option<String>,
    pub uuid: Option<String>,
    pub pri: Option<String>,
    pub rep: Option<String>,
    // #[serde(rename = "docs.count")]
    pub docs_count: Option<String>,
    // #[serde(rename = "docs.deleted")]
    pub docs_deleted: Option<String>,
    // #[serde(rename = "store.size")]
    pub store_size: Option<String>,
    // #[serde(rename = "pri.store.size")]
    pub pri_store_size: Option<String>,
}
