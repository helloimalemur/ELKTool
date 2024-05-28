use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JDBCIndexSearchResult {
    pub took: Option<i64>,
    #[serde(rename = "timed_out")]
    pub timed_out: Option<bool>,
    #[serde(rename = "_shards")]
    pub shards: Option<Shards>,
    pub hits: Option<Hits>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shards {
    pub total: Option<i64>,
    pub successful: Option<i64>,
    pub skipped: Option<i64>,
    pub failed: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hits {
    pub total: Option<Total>,
    #[serde(rename = "max_score")]
    pub max_score: Option<f64>,
    pub hits: Option<Vec<JDBCSearchResult>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Total {
    pub value: Option<i64>,
    pub relation: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JDBCSearchResult {
    #[serde(rename = "_index")]
    pub index: Option<String>,
    #[serde(rename = "_id")]
    pub id: Option<String>,
    #[serde(rename = "_score")]
    pub score: Option<f64>,
    #[serde(rename = "_ignored")]
    pub ignored: Option<Vec<String>>,
    #[serde(rename = "_source")]
    pub source: Option<Source>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub jsonrequest: Option<Value>,
    pub request_body: Option<String>,
    pub requestmethod: Option<String>,
    #[serde(rename = "@version")]
    pub version: Option<String>,
    pub konnektype: Option<String>,
    pub datecreated: Option<String>,
    pub responsetime: Option<f64>,
    pub companyid: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub response_body: Option<String>,
    pub remoteurl: Option<String>,
    #[serde(rename = "@timestamp")]
    pub timestamp: Option<String>,
    pub request_params: Option<String>,
    pub id: Option<i64>,
}
