use crate::index_api::replicassetting::ReplicasSetting;
use serde_json::json;
use std::collections::HashMap;

pub async fn stop_ilm_service(
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
    let _discord_webhook_url = settings_map
        .get("discord_webhook_url")
        .expect("COULD NOT GET discord_webhook_url")
        .as_str();

    let full_url = format!("{}{}", elastic_url, "/_ilm/stop");
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
        .post(full_url)
        .basic_auth(elastic_user, Some(elastic_pass))
        .header("Cache-Control", "max-age=0")
        .header("Accept", "text/html")
        .header("Accept-Encoding", "gzip, deflate")
        .send()
        .await;

    let res = client;
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

pub async fn set_number_of_replicas_to_zero(
    elastic_url: &str,
    elastic_user: &str,
    elastic_pass: &str,
) {
    // close unassigned shards
    let full_url = format!("{}{}", elastic_url, "/*/_settings");

    let json = json!({
         "index" : {
          "number_of_replicas":0
         }
    });

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
        .put(full_url)
        .basic_auth(elastic_user, Some(elastic_pass))
        .header("Cache-Control", "max-age=0")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Accept-Encoding", "gzip, deflate")
        .json(&json)
        .send()
        .await;

    let res = client.unwrap().text().await.unwrap();

    // deserialize from json to Vec of ElasticSearch Index obj
    let res: ReplicasSetting = match serde_json::from_str(res.clone().as_str()) {
        Ok(r) => r,
        Err(e) => panic!("{}", e.to_string()),
    };

    println!("{:?}", res);
}
