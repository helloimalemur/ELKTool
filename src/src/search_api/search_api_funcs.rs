use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::collections::HashMap;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsyncSearchSize {
    pub transient: Transient,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transient {
    #[serde(rename = "search.max_async_search_response_size")]
    pub search_max_async_search_response_size: String,
}

pub async fn max_async_search_response_size(
    settings_map: HashMap<String, String>,
    _policies_map: HashMap<String, String>,
) {
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
    let max_async_search_response_size = settings_map
        .get("max_async_search_response_size")
        .expect("COULD NOT GET max_async_search_response_size");

    let transient = Transient {
        search_max_async_search_response_size: max_async_search_response_size.to_string(),
    };

    let search_size = AsyncSearchSize { transient };
    let json = serde_json::to_string(&search_size).unwrap();

    let full_url = format!("{}{}", elastic_url, "/_cluster/settings");
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
        .put(full_url)
        .basic_auth(elastic_user, Some(elastic_pass))
        .body(json)
        .header("Cache-Control", "max-age=0")
        .header("Content-Type", "application/json")
        .header("Accept", "text/html")
        .header("Accept-Encoding", "gzip, deflate")
        .send()
        .await;

    let res = client;
    println!(
        "Setting max Async search response size: {}",
        max_async_search_response_size
    );
    if res.is_ok() {
        let res_text = res.unwrap().text().await;
        if res_text.is_ok() {
            println!("{}", res_text.unwrap());
        } else {
            println!("unable to apply ilm setting");
        }
    } else {
        println!("unable to apply ilm setting");
    }
}
