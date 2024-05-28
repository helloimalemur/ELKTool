use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissingQuery {
    pub query: Query,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Query {
    pub bool: Bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bool {
    #[serde(rename = "must_not")]
    pub must_not: MustNot,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MustNot {
    pub exists: Exists,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exists {
    pub field: String,
}

pub fn missing_query<T: ToString>(val: T) -> MissingQuery {
    MissingQuery {
        query: Query {
            bool: Bool {
                must_not: MustNot {
                    exists: Exists {
                        field: val.to_string(),
                    },
                },
            },
        },
    }
}
