use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HAProxyIndexSearchResult {
    pub took: Option<i64>,
    #[serde(rename = "timed_out")]
    pub timed_out: Option<bool>,
    #[serde(rename = "_shards")]
    #[serde(skip)]
    pub shards: Option<Shards>,
    pub hits: Option<Hits>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shards {
    #[serde(skip)]
    pub total: Option<i64>,
    #[serde(skip)]
    pub successful: Option<i64>,
    #[serde(skip)]
    pub skipped: Option<i64>,
    #[serde(skip)]
    pub failed: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hits {
    #[serde(skip)]
    pub total: Option<Total>,
    #[serde(rename = "max_score")]
    #[serde(skip)]
    pub max_score: Option<f64>,
    pub hits: Option<Vec<HAProxySearchResult>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Total {
    #[serde(skip)]
    pub value: Option<i64>,
    #[serde(skip)]
    pub relation: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HAProxySearchResult {
    #[serde(rename = "_index")]
    pub index: Option<String>,
    #[serde(rename = "_id")]
    pub id: Option<String>,
    #[serde(rename = "_score")]
    #[serde(skip)]
    pub score: Option<f64>,
    #[serde(rename = "_ignored")]
    #[serde(skip)]
    pub ignored: Option<Vec<String>>,
    // do not skip
    #[serde(rename = "_source")]
    pub source: Option<HAProxySource>,
    //
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HAProxySource {
    // haproxy
    // #[serde(skip)]
    pub event: Option<HAProxyEvent>,
    #[serde(rename = "server_name")]
    #[serde(skip)]
    pub server_name: Option<String>,
    #[serde(rename = "client_ip")]
    #[serde(skip)]
    pub client_ip: Option<String>,
    #[serde(skip)]
    pub ecs: Option<Ecs>,
    #[serde(rename = "syslog_server")]
    #[serde(skip)]
    pub syslog_server: Option<String>,
    #[serde(rename = "time_backend_response")]
    #[serde(skip)]
    pub time_backend_response: Option<String>,
    #[serde(skip)]
    pub agent: Option<Agent>,
    #[serde(rename = "syslog_timestamp")]
    #[serde(skip)]
    pub syslog_timestamp: Option<String>,
    #[serde(skip)]
    pub beconn: Option<String>,
    #[serde(rename = "time_duration")]
    #[serde(skip)]
    pub time_duration: Option<String>,
    #[serde(rename = "@version")]
    #[serde(skip)]
    pub version: Option<String>,
    #[serde(rename = "termination_state")]
    #[serde(skip)]
    pub termination_state: Option<String>,
    #[serde(rename = "time_backend_connect")]
    #[serde(skip)]
    pub time_backend_connect: Option<String>,
    #[serde(skip)]
    pub log: Option<Log>,
    #[serde(skip)]
    pub input: Option<Input>,
    #[serde(skip)]
    pub tags: Option<Vec<String>>,
    #[serde(skip)]
    pub retries: Option<String>,
    #[serde(skip)]
    #[serde(rename = "srv_queue")]
    pub srv_queue: Option<String>,
    #[serde(rename = "captured_response_cookie")]
    #[serde(skip)]
    pub captured_response_cookie: Option<String>,
    #[serde(skip)]
    pub process: Option<Process>,
    #[serde(skip)]
    pub feconn: Option<String>,
    // do not skip
    pub message: Option<String>,
    //
    #[serde(rename = "frontend_name")]
    #[serde(skip)]
    pub frontend_name: Option<String>,
    #[serde(skip)]
    #[serde(rename = "backend_name")]
    pub backend_name: Option<String>,
    #[serde(skip)]
    pub host: Option<Host>,
    #[serde(rename = "time_queue")]
    #[serde(skip)]
    pub time_queue: Option<String>,
    #[serde(skip)]
    pub haproxy: Option<Haproxy>,

    #[serde(rename = "@timestamp")]
    pub timestamp: Option<String>,

    #[serde(rename = "accept_date")]
    pub accept_date: Option<String>,

    #[serde(rename = "captured_request_cookie")]
    #[serde(skip)]
    pub captured_request_cookie: Option<String>,
    #[serde(rename = "time_request")]
    #[serde(skip)]
    pub time_request: Option<String>,
    #[serde(rename = "client_port")]
    #[serde(skip)]
    pub client_port: Option<String>,
    #[serde(rename = "backend_queue")]
    #[serde(skip)]
    pub backend_queue: Option<String>,
    #[serde(skip)]
    pub srvconn: Option<String>,
    #[serde(rename = "bytes_read")]
    #[serde(skip)]
    pub bytes_read: Option<String>,
    #[serde(skip)]
    pub actconn: Option<String>,
    #[serde(rename = "http_status_code")]
    #[serde(skip)]
    pub http_status_code: Option<String>,
    // Jdbc
    #[serde(skip)]
    pub jsonrequest: Option<Value>,
    #[serde(skip)]
    pub request_body: Option<String>,
    #[serde(skip)]
    pub requestmethod: Option<String>,
    // #[serde(rename = "@version")]
    // pub version: String,
    #[serde(skip)]
    pub konnektype: Option<String>,
    #[serde(skip)]
    pub datecreated: Option<String>,
    #[serde(skip)]
    pub responsetime: Option<f64>,
    #[serde(skip)]
    pub companyid: Option<i64>,
    // pub tags: Vec<String>,
    #[serde(skip)]
    pub response_body: Option<String>,
    #[serde(skip)]
    pub remoteurl: Option<String>,
    // #[serde(rename = "@timestamp")]
    // pub timestamp: String,
    #[serde(skip)]
    pub request_params: Option<String>,
    #[serde(skip)]
    pub id: Option<i64>,

    #[serde(rename = "loginId")]
    pub login_id: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HAProxyEvent {
    pub original: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ecs {
    pub version: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub version: Option<String>,
    #[serde(rename = "ephemeral_id")]
    pub ephemeral_id: Option<String>,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub name: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    pub file: Option<File>,
    pub offset: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    #[serde(skip)]
    pub inode: Option<String>,
    pub path: Option<String>,
    #[serde(rename = "device_id")]
    #[serde(skip)]
    pub device_id: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    #[serde(rename = "type")]
    pub type_field: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Process {
    pub name: Option<String>,
    pub pid: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Host {
    pub name: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Haproxy {
    pub http: Option<Http>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Http {
    pub request: Option<Request>,
    pub response: Option<Response>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    #[serde(rename = "captured_headers")]
    pub captured_headers: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    #[serde(rename = "captured_headers")]
    pub captured_headers: Option<String>,
}
