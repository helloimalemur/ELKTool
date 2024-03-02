use serde::{Deserialize, Serialize};

// use serde_derive::Deserialize;
// use serde_derive::Serialize;

// pub type Root = Vec<ElasticIndex>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ILMIndex {
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
