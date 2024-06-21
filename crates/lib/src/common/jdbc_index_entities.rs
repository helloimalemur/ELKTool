use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JDBCIndexUpdate {
    pub index_name: String,
    pub index_id: String,
    pub new_field_name: String,
    pub new_field_value: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct JDBCIndexUpdateScript {
    pub script: String,
}
