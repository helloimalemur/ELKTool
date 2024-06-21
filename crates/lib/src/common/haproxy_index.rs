use crate::common::haproxy_index_search_result_entities::{
    HAProxyIndexSearchResult, HAProxySearchResult,
};

use crate::common::missing_query::{missing_but_has_query, missing_query};
use anyhow::{anyhow, Error};
use std::time::Duration;
use substring::Substring;

pub async fn get_haproxy_index_missing_field<T: ToString>(
    index_name: T,
    missing_field: T,
    elastic_url: T,
    elastic_user: T,
    elastic_pass: T,
    total: u16,
) -> Result<Vec<HAProxySearchResult>, Error> {
    let full_url = format!(
        "{}{}{}{}{}",
        elastic_url.to_string(),
        "/",
        index_name.to_string(),
        "/_search?size=",
        total
    );

    let json = serde_json::to_string(&missing_query(missing_field))?;

    // println!("{}", json);

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?
        .get(full_url)
        .basic_auth(elastic_user.to_string(), Some(elastic_pass.to_string()))
        .header("Content-Type", "application/json")
        .header("Cache-Control", "max-age=0")
        .header("Accept", "application/json")
        .header("Accept-Encoding", "gzip, deflate")
        .body(json)
        .timeout(Duration::new(30, 0))
        .send()
        .await;

    if let Ok(cl) = client {
        // get indicies
        if let Ok(res) = cl.text().await {
            // deserialize from json to Vec of ElasticSearch Index obj
            // println!("{}", res); // troubleshooting
            let res: HAProxyIndexSearchResult = match serde_json::from_str(res.clone().as_str()) {
                Ok(r) => r,
                // Err(e) => panic!("{}", e.to_string()),
                Err(e) => panic!("{} - {}", e.to_string(), res.substring(0, 920)), // troubleshooting
            };

            return if let Some(hits) = res.hits {
                if let Some(h) = hits.hits {
                    Ok(h.to_vec())
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

pub async fn get_haproxy_index_missing_field_but_has<T: ToString>(
    index_name: T,
    missing_field: T,
    has_field: T,
    elastic_url: T,
    elastic_user: T,
    elastic_pass: T,
    total: u16,
) -> Result<Vec<HAProxySearchResult>, Error> {
    let full_url = format!(
        "{}{}{}{}{}",
        elastic_url.to_string(),
        "/",
        index_name.to_string(),
        "/_search?size=",
        total
    );

    let json = serde_json::to_string(&missing_but_has_query(missing_field, has_field))?;

    // println!("{}", json);

    // exit(0);

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?
        .get(full_url)
        .basic_auth(elastic_user.to_string(), Some(elastic_pass.to_string()))
        .header("Content-Type", "application/json")
        .header("Cache-Control", "max-age=0")
        .header("Accept", "application/json")
        .header("Accept-Encoding", "gzip, deflate")
        .body(json)
        .timeout(Duration::new(30, 0))
        .send()
        .await;

    if let Ok(cl) = client {
        // get indicies
        if let Ok(res) = cl.text().await {
            // deserialize from json to Vec of ElasticSearch Index obj
            // println!("{}", res); // troubleshooting
            let res: HAProxyIndexSearchResult = match serde_json::from_str(res.clone().as_str()) {
                Ok(r) => r,
                // Err(e) => panic!("{}", e.to_string()),
                Err(e) => panic!("{} - {}", e.to_string(), res.substring(0, 920)), // troubleshooting
            };

            return if let Some(hits) = res.hits {
                if let Some(h) = hits.hits {
                    Ok(h.to_vec())
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
