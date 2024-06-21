use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissingQuery {
    pub query: Query,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissingButHasQuery {
    pub query: QueryMissingButHas,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryMissingButHas {
    pub bool: TwoBool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoolMissingButHas {
    #[serde(rename = "must_not")]
    pub must_not: MustNot,
    pub exists: Exists,
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
pub struct TwoBool {
    #[serde(rename = "must_not")]
    pub must_not: MustNot,
    pub must: Must,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MustNot {
    pub exists: Exists,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Must {
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

pub fn missing_but_has_query<T: ToString>(not_val: T, has_val: T) -> MissingButHasQuery {
    MissingButHasQuery {
        query: QueryMissingButHas {
            bool: TwoBool {
                must_not: MustNot {
                    exists: Exists {
                        field: not_val.to_string(),
                    },
                },
                must: Must {
                    exists: Exists {
                        field: has_val.to_string(),
                    },
                },
            },
        },
    }
}
