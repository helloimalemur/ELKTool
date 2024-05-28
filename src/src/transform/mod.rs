mod haproxy_index_search_result;
mod missing_query;

use crate::transform::haproxy_index_search_result::{
    HAProxyIndexSearchResult, HAProxySearchResult,
};
use crate::transform::missing_query::missing_query;
use anyhow::{anyhow, Error};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::ops::Not;
use std::process::exit;
use std::str::FromStr;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};
use chrono::Datelike;
use substring::Substring;
use urlencoding::decode;

pub async fn start_transforms(settings_map: HashMap<String, String>) {
    let mut metrics_req_per_sec: Vec<u64> = vec![];
    let transforms = get_transform_config();
    for transform in transforms.iter() {
        let mut ind_name = transform.index_prefix.to_string();

        let mut day_str = String::new();
        let mut month_str = String::new();
        let mut year_str = String::new();

        let tdy = chrono::Local::now();
        let day = tdy.day();
        if day < 10 {
            day_str = format!("{}{}", 0, day)
        } else {
            day_str = day.to_string()
        }
        let month = tdy.month();
        if month < 10 {
            month_str = format!("{}{}", 0, month)
        } else {
            month_str = month.to_string()
        }
        let year= tdy.year();
        year_str = year.to_string();

        if transform.index_prefix.contains("TODAY") {
            let date_str = format!("{}.{}.{}", year, month_str, day_str);
            ind_name = ind_name.replace("TODAY", date_str.as_str());
        }
        // 2024.05.14
        // println!("{}", ind_name);

        parse_to_new_haproxy_field(
            ind_name,
            transform.source_field.to_string(),
            transform.destination_field.to_string(),
            transform.transform_type.to_string(),
            transform.needle.to_string(),
            settings_map.clone(),
            transform.total_to_process,
        )
        .await
    }
    // if !metrics_req_per_sec.is_empty() {
    //     let mut sum = 0;
    //     metrics_req_per_sec.iter().for_each(|a| {sum += a});
    //     let met_len = metrics_req_per_sec.len() as u64;
    //     let avg_req_sec = sum / met_len;
    //     println!("Average Requests Per Second: {}/s", avg_req_sec);
    // }
}

pub async fn parse_to_new_haproxy_field(
    index_prefix: String,
    source_field: String,
    destination_field: String,
    needle_type: String,
    needle: String,
    settings_map: HashMap<String, String>,
    total: u16,
) {
    let start = SystemTime::now();
    let elastic_url = settings_map
        .get("elastic_url")
        .expect("COULD NOT GET elastic_url")
        .to_string();
    let elastic_user = settings_map
        .get("elastic_user")
        .expect("COULD NOT GET elastic_user")
        .to_string();
    let elastic_pass = settings_map
        .get("elastic_pass")
        .expect("COULD NOT GET elastic_pass")
        .to_string();
    let parallelism = settings_map
        .get("parallelism")
        .expect("COULD NOT GET parallelism")
        .to_string();

    let p_index_prefix = index_prefix.clone();
    let p_destination_field = destination_field.clone();

    // get list of indexes missing new field
    let index_data = get_index_missing_field(
        index_prefix,
        destination_field.clone(),
        elastic_url.to_string(),
        elastic_user.to_string(),
        elastic_pass.to_string(),
        total,
    )
    .await;

    // println!("{:?}", index_data);
    let mut rcount = 0;
    let mut changes: Vec<HAProxyIndexUpdate> = vec![];
    let mut handles: Vec<JoinHandle<()>> = vec![];

    // prepare index update script (changes)
    if let Ok(i_d) = index_data {
        for index in i_d.iter() {
            let message = index.clone().source.unwrap().message.unwrap();
            if message.contains(&needle) {
                rcount += 1;
                let split_by_needle = message.split(&needle.to_string()).last().unwrap();
                // println!("{:?}", index);
                let split_by_param: Vec<&str> = split_by_needle.split('&').collect();
                // index name
                let index_name = index.clone().index.unwrap();
                // index id
                let index_id = index.clone().id.unwrap();
                // param value
                let split_by_param_res = split_by_param.first().unwrap().to_string();

                let split_by_ws_res: Vec<&str> = split_by_param_res.split_whitespace().collect();
                let mut new_field_value = String::new();
                // println!("{}", new_field_value);
                if let Some(s) = split_by_ws_res.first() {
                    if let Ok(r) = decode(s) {
                        new_field_value = r.to_string();
                    }
                }
                // else {
                //     println!("{}", message);
                //     // exit(1);
                //     // new_field_value = split_by_ws_res
                // }

                changes.push(HAProxyIndexUpdate {
                    index_name,
                    index_id,
                    new_field_name: destination_field.to_string(),
                    new_field_value,
                })
            } else {
                let index_name = index.clone().index.unwrap();
                let index_id = index.clone().id.unwrap();

                changes.push(HAProxyIndexUpdate {
                    index_name,
                    index_id,
                    new_field_name: destination_field.to_string(),
                    new_field_value: "".to_string(),
                })
            }
        }
    }

    println!(
        "Index Updates: {} .. {} - {}",
        changes.len(),
        p_index_prefix,
        p_destination_field
    );
    bulk_haproxy_update_index_add_field(changes, elastic_url, elastic_user, elastic_pass).await;
}

pub async fn get_index_missing_field<T: ToString>(
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
        .timeout(Duration::new(6, 0))
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

pub async fn haproxy_update_index_add_field(
    index_name: String,
    index_id: String,
    new_field: String,
    new_field_value: String,
    elastic_url: String,
    elastic_user: String,
    elastic_pass: String,
) {
    let full_url = format!(
        "{}{}{}{}{}",
        elastic_url.to_string(),
        "/",
        index_name.to_string(),
        "/_update/",
        index_id.to_string()
    );

    let json = haproxy_index_update_json(new_field.to_string(), new_field_value.to_string());

    // println!("Index Update:{} ID:{} {}", index_name.to_string(), index_id.to_string(), json);

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
        .post(full_url)
        .basic_auth(elastic_user.to_string(), Some(elastic_pass.to_string()))
        .header("Content-Type", "application/json")
        .header("Cache-Control", "max-age=0")
        .header("Accept", "application/json")
        .header("Accept-Encoding", "gzip, deflate")
        .body(json)
        .timeout(Duration::new(2, 0))
        .send()
        .await;

    // get indicies
    // let res = client.unwrap().text().await.unwrap();

    // // deserialize from json to Vec of ElasticSearch Index obj
    // let res: IndexSearchResult = match serde_json::from_str(res.clone().as_str()) {
    //     Ok(r) => r,
    //     Err(e) => panic!("{}", e.to_string()),
    // };
    //
    // let vec = res.hits.unwrap().clone();
    // vec.hits.unwrap().to_vec()
}

// todo()! bulk update
// https://www.elastic.co/guide/en/elasticsearch/reference/current/docs-bulk.html#bulk-update

pub async fn bulk_haproxy_update_index_add_field(
    updates: Vec<HAProxyIndexUpdate>,
    elastic_url: String,
    elastic_user: String,
    elastic_pass: String,
) {
    // println!("{}", updates.len());

    let full_url = format!("{}{}{}", elastic_url.to_string(), "/", "_bulk/",);

    let json = bulk_haproxy_index_update_json(updates);

    // println!("{:?}", json.clone());

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
        .post(full_url)
        .basic_auth(elastic_user.to_string(), Some(elastic_pass.to_string()))
        .header("Content-Type", "application/json")
        .header("Cache-Control", "max-age=0")
        .header("Accept", "application/json")
        .header("Accept-Encoding", "gzip, deflate")
        .body(json.clone())
        .timeout(Duration::new(30, 0))
        .send()
        .await;

    // get indicies
    if let Ok(cl) = client {
        if let Ok(res) = cl.text().await {
            if res.contains("x_content_parse_exception") {
                println!("{}", res);
            }
        } else {
            eprintln!("WARNING: REQUEST FAILED :: {}", json.substring(0, 30));
        }
    } else {
        eprintln!("WARNING: REQUEST FAILED :: {}", json.substring(0, 30));
    }

    // // deserialize from json to Vec of ElasticSearch Index obj
    // let res: IndexSearchResult = match serde_json::from_str(res.clone().as_str()) {
    //     Ok(r) => r,
    //     Err(e) => panic!("{}", e.to_string()),
    // };
    //
    // let vec = res.hits.unwrap().clone();
    // vec.hits.unwrap().to_vec()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct HAProxyIndexUpdate {
    index_name: String,
    index_id: String,
    new_field_name: String,
    new_field_value: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct HAProxyIndexUpdateScript {
    script: String,
}

fn haproxy_index_update_json(new_field: String, value: String) -> String {
    let bkslsh = r#"\u0027"#;
    format!(
        "{{\"script\" : \"ctx._source.{} = {}{}{}\"}}",
        new_field, bkslsh, value, bkslsh
    )
}

fn bulk_haproxy_index_update_json(changes: Vec<HAProxyIndexUpdate>) -> String {
    let mut full_string = String::new();
    // { "update" : {"_id" : "1", "_index" : "test"} }
    // { "doc" : {"field2" : "value2"} }
    // let bkslsh = r#"\u0027"#;
    // format!(
    //     "{{\"script\" : \"ctx._source.{} = {}{}{}\"}}",
    //     new_field, bkslsh, value, bkslsh
    // )

    for ch in changes {
        let update = format!(
            "{{ \"update\" : {{\"_id\" : \"{}\", \"_index\" : \"{}\"}} }}\n",
            ch.index_id, ch.index_name
        );
        full_string.push_str(update.as_str());
        let mut new_field = String::new();
        let mut new_value = String::new();

        // println!("Before: {}", ch.new_field_value);

        new_field = escape_special(ch.new_field_name.clone());
        new_value = escape_special(ch.new_field_value.clone());

        // println!("Before: {}", ch.new_field_value);
        // println!("After: {}", new_value);

        new_value = new_value.replace('\\', ""); // bugfix
        new_value = new_value.replace("HTTP/1.1\"", ""); // bugfix


        let doc = format!(
            "{{ \"doc\" : {{\"{}\" : \"{}\"}} }}\n",
            new_field, new_value
        );
        full_string.push_str(doc.as_str());
    }
    full_string.push_str("\n");
    // println!("{full_string}");
    full_string
}

fn escape_special(input: String) -> String {
    // let output = String::new();

    // output
    escape_string::escape(input.as_str()).to_string()
}

#[derive(Debug, Deserialize)]
pub struct TransformOuter {
    entry: Vec<Transform>,
}

#[derive(Debug, Deserialize)]
pub struct Transform {
    index_prefix: String,
    source_field: String,
    destination_field: String,
    transform_type: String,
    needle: String,
    total_to_process: u16,
}

pub fn get_transform_config() -> Vec<Transform> {
    let toml_str = fs::read_to_string("config/Transforms.toml").unwrap();
    let entries: TransformOuter = toml::from_str(&toml_str).unwrap();
    entries.entry
}

#[cfg(test)]
mod tests {
    use crate::transform::{
        bulk_haproxy_update_index_add_field, get_index_missing_field, get_transform_config,
        haproxy_update_index_add_field, parse_to_new_haproxy_field,
    };
    use config::Config;
    use std::collections::HashMap;

    fn config() -> Config {
        Config::builder()
            .add_source(config::File::with_name("config/Settings.toml"))
            .build()
            .unwrap()
    }

    #[test]
    // #[ignore]
    fn test_transform() {
        let config = config();
        let settings_map = config.try_deserialize::<HashMap<String, String>>().unwrap();
        // get elastic user settings
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
        let run_lm_on_start = settings_map
            .get("run_lm_on_start")
            .expect("COULD NOT GET run_lm_on_start")
            .as_str();
        let alerting_enabled = settings_map
            .get("alerting_enabled")
            .expect("COULD NOT GET alerting_enabled")
            .as_str();

        let tk = tokio::runtime::Runtime::new();
        tk.unwrap().block_on(parse_to_new_haproxy_field(
            "haproxy-files-2024.05.14".to_string(),
            "message".to_string(),
            "funnelReferenceId".to_string(),
            "url_param".to_string(),
            "funnelReferenceId=".to_string(),
            settings_map.clone(),
            5000,
        ))
    }

    #[test]
    // #[ignore]
    fn test_update_index() {
        let config = config();
        let settings_map = config.try_deserialize::<HashMap<String, String>>().unwrap();
        // get elastic user settings
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
        let run_lm_on_start = settings_map
            .get("run_lm_on_start")
            .expect("COULD NOT GET run_lm_on_start")
            .as_str();
        let alerting_enabled = settings_map
            .get("alerting_enabled")
            .expect("COULD NOT GET alerting_enabled")
            .as_str();

        let rt = tokio::runtime::Runtime::new();
        let out = rt.unwrap().block_on(get_index_missing_field(
            "haproxy*",
            "new_field",
            elastic_url,
            elastic_user,
            elastic_pass,
            100,
        ));

        // let tk = tokio::runtime::Runtime::new();
        // tk.unwrap().block_on(parse_to_new_haproxy_field(
        //     "haproxy*".to_string(),
        //     "new_field".to_string(),
        //     elastic_url.to_string(),
        //     elastic_user.to_string(),
        //     elastic_pass.to_string(),
        //     100,
        // ))
    }

    #[test]
    // #[ignore]
    fn test_transform_toml() {
        let entries = get_transform_config();
        for i in entries.iter() {
            println!("{:?}", i);
        }
    }

    #[test]
    fn test_get_missing_field() {
        let settings = Config::builder()
            .add_source(config::File::with_name("config/Settings"))
            .build()
            .unwrap();
        let settings_map = settings
            .try_deserialize::<HashMap<String, String>>()
            .unwrap();
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
        let out = rt.unwrap().block_on(get_index_missing_field(
            "haproxy*",
            "new_field",
            elastic_url,
            elastic_user,
            elastic_pass,
            100,
        ));
        println!("INDEXES: {:?}", out.unwrap().len());
    }
}
