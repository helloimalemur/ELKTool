use crate::app_index::config::get_app_index_config;
use crate::notifications::discord::send_discord;
use anyhow::{anyhow, Error};
use config::Config;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use substring::Substring;

#[allow(unused)]
pub async fn process_app_index() {
    let settings = Config::builder()
        .add_source(config::File::with_name("../../config/Settings"))
        .build()
        .expect("COULD NOT LOAD SETTINGS");
    let settings_map = settings
        .try_deserialize::<HashMap<String, String>>()
        .expect("COULD NOT LOAD SETTINGS");
    let elastic_url = settings_map
        .get("elastic_url")
        .expect("COULD NOT GET elastic_url")
        .as_str();
    let elastic_user = settings_map
        .get("elastic_user")
        .expect("COULD NOT GET elastic_user")
        .as_str();
    let elastic_pass = settings_map
        .get("elastic_pass")
        .expect("COULD NOT GET elastic_pass")
        .as_str();

    let ind = get_app_index_config();

    for app_index_config in ind {
        if app_index_config.enabled.contains("true") {
            let index_prefix = app_index_config.index_prefix;

            if index_prefix.clone().contains("app-index") {
                println!("{:#?}", index_prefix);
                let x = get_app_index(
                    index_prefix.clone(),
                    elastic_url.to_string(),
                    elastic_user.to_string(),
                    elastic_pass.to_string(),
                    app_index_config.total_to_process,
                )
                .await;
                if let Ok(r) = x {
                    for app_index in r {
                        // println!("{}", app_index.title);
                        if app_index
                            .content
                            .contains(app_index_config.needle.clone().as_str())
                        {
                            if app_index_config.alert.contains("true") {
                                println!("Fire Alert: {}", app_index.title);
                                let message = format!(
                                    "Fire Alert: {} :: {}",
                                    app_index.title, app_index.content
                                );
                                // send_alerts(message, settings_map.clone())
                                send_discord(&settings_map, "CAP", message.as_str()).await;
                            }
                        }
                    }
                }
            }

            if index_prefix.clone().contains("jdbc") {
                println!("{:#?}", index_prefix);
                let x = get_app_index(
                    index_prefix.clone(),
                    elastic_url.to_string(),
                    elastic_user.to_string(),
                    elastic_pass.to_string(),
                    app_index_config.total_to_process,
                )
                .await;
                if let Ok(r) = x {
                    for app_index in r {
                        // println!("{}", app_index.title);
                        if app_index
                            .content
                            .contains(app_index_config.needle.clone().as_str())
                        {
                            if app_index_config.alert.contains("true") {
                                println!("Fire Alert: {}", app_index.title);
                                let message = format!(
                                    "Fire Alert: {} :: {}",
                                    app_index.title, app_index.content
                                );
                                // send_alerts(message, settings_map.clone())
                                send_discord(&settings_map, "CAP", message.as_str()).await;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[allow(unused)]
pub async fn get_app_index<T: ToString>(
    index_name: T,
    elastic_url: T,
    elastic_user: T,
    elastic_pass: T,
    total: u16,
) -> Result<Vec<AppIndex>, Error> {
    let full_url = format!(
        "{}{}{}{}{}",
        elastic_url.to_string(),
        "/",
        index_name.to_string(),
        "/_search?size=",
        total
    );

    // println!("{}", full_url);

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?
        .get(full_url)
        .basic_auth(elastic_user.to_string(), Some(elastic_pass.to_string()))
        .header("Content-Type", "application/json")
        .header("Cache-Control", "max-age=0")
        .header("Accept", "application/json")
        .header("Accept-Encoding", "gzip, deflate")
        .timeout(Duration::new(30, 0))
        .send()
        .await;

    if let Ok(cl) = client {
        // get indicies
        if let Ok(res) = cl.text().await {
            // deserialize from json to Vec of ElasticSearch Index obj
            // println!("{}", res); // troubleshooting
            let res: AppIndexSearchResult = match serde_json::from_str(res.clone().as_str()) {
                Ok(r) => r,
                // Err(e) => panic!("{}", e.to_string()),
                Err(e) => panic!("{} - {}", e.to_string(), res.substring(0, 920)), // troubleshooting
            };

            // println!("{:?}", res);

            let mut inds: Vec<AppIndex> = vec![];

            return if let Some(hits) = res.hits {
                if let Some(h) = hits.hits {
                    // println!("{:?}", h);
                    for entry in h {
                        inds.push(entry.source)
                    }

                    Ok(inds)
                } else {
                    // println!("no index updates");
                    Err(anyhow!("no hits"))
                }
            } else {
                // println!("no index updates");
                Err(anyhow!("no hits"))
            };
        } else {
            Err(anyhow!("no hits"))
        }
    } else {
        Err(anyhow!("no hits"))
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppIndexSearchResult {
    pub took: i64,
    #[serde(rename = "timed_out")]
    pub timed_out: bool,
    #[serde(rename = "_shards")]
    pub shards: Shards,
    pub hits: Option<Hits>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shards {
    pub total: i64,
    pub successful: i64,
    pub skipped: i64,
    pub failed: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hits {
    pub total: Total,
    #[serde(rename = "max_score")]
    pub max_score: f64,
    pub hits: Option<Vec<Hit>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Total {
    pub value: i64,
    pub relation: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hit {
    #[serde(rename = "_index")]
    pub index: String,
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_score")]
    pub score: f64,
    #[serde(rename = "_source")]
    pub source: AppIndex,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppIndex {
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    pub title: String,
    pub content: String,
    pub source: String,
}

#[cfg(test)]
mod tests {
    use crate::app_index::app_index::{get_app_index, process_app_index};
    use config::Config;
    use std::collections::HashMap;

    #[test]
    fn test_get_app_index() {
        let settings = Config::builder()
            .add_source(config::File::with_name("../../config/Settings"))
            .build()
            .expect("COULD NOT LOAD SETTINGS");
        let settings_map = settings
            .try_deserialize::<HashMap<String, String>>()
            .expect("COULD NOT LOAD SETTINGS");
        let elastic_url = settings_map
            .get("elastic_url")
            .expect("COULD NOT GET elastic_url")
            .as_str();
        let elastic_user = settings_map
            .get("elastic_user")
            .expect("COULD NOT GET elastic_user")
            .as_str();
        let elastic_pass = settings_map
            .get("elastic_pass")
            .expect("COULD NOT GET elastic_pass")
            .as_str();

        let rt = tokio::runtime::Runtime::new();
        let out = rt.unwrap().block_on(get_app_index(
            "test-app-index*",
            elastic_url,
            elastic_user,
            elastic_pass,
            10,
        ));
        if let Ok(_cl) = out {
            // println!("{:?}", cl);
            // assert!(!cl.is_empty())
        } else {
            println!("NO HITS")
        }
    }
    #[test]
    fn test_process_app_index() {
        let rt = tokio::runtime::Runtime::new();
        let _out = rt.unwrap().block_on(process_app_index());
        // if let Ok(cl) = out {
        //     // println!("{:?}", cl);
        //     assert!(!cl.is_empty())
        // } else {
        //     println!("NO HITS")
        // }
    }
}
