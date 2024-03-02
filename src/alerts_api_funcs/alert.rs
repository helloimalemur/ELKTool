use std::collections::HashMap;
use std::fmt::Error;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElasticAlert {
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
    pub total: Total,
    #[serde(rename = "max_score")]
    pub max_score: Option<f64>,
    pub hits: Option<Vec<Hit>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Total {
    pub value: Option<i64>,
    pub relation: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hit {
    #[serde(rename = "_index")]
    pub index: Option<String>,
    #[serde(rename = "_id")]
    pub id: Option<String>,
    #[serde(rename = "_score")]
    pub score: Option<f64>,
    #[serde(rename = "_source")]
    pub source: Option<Source>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    #[serde(rename = "an error")]
    pub an_error: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
}


pub async fn get_alert_indicies(
    elastic_url: &str,
    elastic_user: &str,
    elastic_pass: &str,
    settings_map: HashMap<String, String>,
) -> Result<HashMap<String, String>, Error> {
    // curl -X GET "https://stats.konnektive.com:9200/alert-index/_search" -H $'Authorization: Basic ZWxhc3RpYzpPVkIzN3dzUD1EUXlDU3J4U1hGVA=='

    let mut results_hashmap: HashMap<String, String> = HashMap::new();

    let full_url = format!("{}{}", elastic_url, "/alert-index/_search");

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
        .get(full_url)
        .basic_auth(elastic_user, Some(elastic_pass))
        .header("Cache-Control", "max-age=0")
        .header("Accept", "application/json")
        .header("Accept-Encoding", "gzip, deflate")
        .send()
        .await;

    // get indicies
    let client_ok = client.is_ok();
    if client_ok {
        let res = client.unwrap().text().await.unwrap();

        // deserialize from json to Vec of ElasticSearch Index obj
        let res: ElasticAlert = match serde_json::from_str(res.clone().as_str()) {
            Ok(r) => r,
            Err(e) => panic!("{}\n{:?}", e, res),
        };

        // println!("{:?}", res);

        if res.hits.is_some() {
            for hit in res.hits.unwrap().hits.unwrap().iter() {
                let mut title = String::new();
                let mut content = String::new();
                
                if let Some(title_some) = hit.source.clone().unwrap().title {
                    title = title_some;
                }
                if let Some(content_some) = hit.source.clone().unwrap().content {
                    content = content_some;
                }

                // for (ind, source) in hit.source.
                if !title.is_empty() && !content.is_empty() {
                    println!("{}", hit.source.clone().unwrap().title.unwrap());
                    println!("{}", hit.source.clone().unwrap().content.unwrap());
                    results_hashmap.insert(
                        hit.source.clone().unwrap().title.expect("alert-index rule connector format error").to_string(),
                        hit.source.clone().unwrap().content.expect("alert-index rule connector format error").to_string(),
                    );
                } else {
                    crate::alerts_api_funcs::alert_api_funcs::send_alerts("unable to retrieve alert-index check rules".to_string(), settings_map.clone()).await;
                }
            }
        }
    }

    Ok(results_hashmap)
}

pub async fn delete_alert_indicies(
    elastic_url: &str,
    elastic_user: &str,
    elastic_pass: &str,
) -> HashMap<String, String> {
    // curl -X DELETE "https://stats.konnektive.com:9200/alert-index" -H $'Authorization: Basic ZWxhc3RpYzpPVkIzN3dzUD1EUXlDU3J4U1hGVA=='

    let results_hashmap: HashMap<String, String> = HashMap::new();

    let full_url = format!("{}{}", elastic_url, "/alert-index");

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
        .delete(full_url)
        .basic_auth(elastic_user, Some(elastic_pass))
        .header("Cache-Control", "max-age=0")
        .header("Accept", "application/json")
        .header("Accept-Encoding", "gzip, deflate")
        .send()
        .await;

    // get indicies
    if client.is_ok() {
        let res = client.unwrap().text().await.unwrap();

        // deserialize from json to Vec of ElasticSearch Index obj
        let _res: ElasticAlert = match serde_json::from_str(res.clone().as_str()) {
            Ok(r) => r,
            Err(e) => panic!("{}\n{:?}", e, res),
        };
    }

    // println!("{:?}", res);

    results_hashmap
}
