use crate::transform::jdbc_index_search_result::{JDBCIndexSearchResult, JDBCSearchResult};
use crate::transform::missing_query::missing_query;
use anyhow::{anyhow, Error};
use chrono::{Datelike, Days};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use substring::Substring;
use urlencoding::decode;

pub async fn start_jdbc_transforms(settings_map: HashMap<String, String>) {
    #[allow(unused)]
    let mut metrics_req_per_sec: Vec<u64> = vec![];
    let transforms = get_jdbc_transform_config();
    for transform in transforms.iter() {
        let mut ind_name = transform.index_prefix.to_string();

        #[allow(unused)]
        let mut day_str = String::new();
        #[allow(unused)]
        let mut month_str = String::new();

        let tdy = chrono::Local::now();

        if transform.index_prefix.contains("TODAY") {
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
            let year = tdy.year();

            let date_str = format!("{}.{}.{}", year, month_str, day_str);
            ind_name = ind_name.replace("TODAY", date_str.as_str());
        }
        if transform.index_prefix.contains("YESTERDAY") {
            let yest = tdy.checked_sub_days(Days::new(1)).unwrap();
            let day = yest.day();
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
            let year = tdy.year();

            let date_str = format!("{}.{}.{}", year, month_str, day_str);
            ind_name = ind_name.replace("YESTERDAY", date_str.as_str());
        }
        // 2024.05.14
        // println!("{}", ind_name);

        parse_to_new_jdbc_field(
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
}

pub async fn parse_to_new_jdbc_field(
    index_prefix: String,
    _source_field: String,
    destination_field: String,
    _needle_type: String,
    needle: String,
    settings_map: HashMap<String, String>,
    total: u16,
) {
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

    let p_index_prefix = index_prefix.clone();
    let p_destination_field = destination_field.clone();

    // get list of indexes missing new field
    let index_data = get_jdbc_index_missing_field(
        index_prefix,
        destination_field.clone(),
        elastic_url.to_string(),
        elastic_user.to_string(),
        elastic_pass.to_string(),
        total,
    )
    .await;

    // println!("{:?}", index_data);
    #[allow(unused)]
    let mut rcount = 0;
    let mut changes: Vec<JDBCIndexUpdate> = vec![];

    // prepare index update script (changes)
    if let Ok(i_d) = index_data {
        for index in i_d.iter() {
            if let Some(index_id) = index.clone().id {
                if let Some(index_name) = index.clone().index {
                    if let Some(response_body) = index.clone().source.unwrap().response_body {
                        if response_body.contains(&needle) {
                            rcount += 1;
                            let response_body = response_body.split(&needle.to_string()).last();
                            // println!("{:?}", index);
                            // let split_by_param: Vec<&str> = response_body.split('&').collect();
                            // // index name
                            // println!("{:?} - {:?}", index.index, index.id);
                            // println!("{:?}", response_body);
                            // let index_name = index.clone().index.unwrap();
                            // // index id
                            // let index_id = index.clone().id.unwrap();
                            // param value
                            // let split_by_param_res = split_by_param.first().unwrap().to_string();

                            // let split_by_ws_res: Vec<&str> = split_by_param_res.split_whitespace().collect();
                            let mut new_field_value = String::new();
                            // println!("{}", new_field_value);
                            if let Some(s) = response_body {
                                if let Ok(r) = decode(s) {
                                    new_field_value = r.to_string();
                                }
                            }

                            // else {
                            //     println!("{}", message);
                            //     // exit(1);
                            //     // new_field_value = split_by_ws_res
                            // }

                            changes.push(JDBCIndexUpdate {
                                index_name,
                                index_id,
                                new_field_name: destination_field.to_string(),
                                new_field_value,
                            })
                        } else {
                            let index_name = index.clone().index.unwrap();
                            let index_id = index.clone().id.unwrap();

                            changes.push(JDBCIndexUpdate {
                                index_name,
                                index_id,
                                new_field_name: destination_field.to_string(),
                                new_field_value: "".to_string(),
                            })
                        }
                    } else {
                        changes.push(JDBCIndexUpdate {
                            index_name,
                            index_id,
                            new_field_name: destination_field.to_string(),
                            new_field_value: "".to_string(),
                        })
                    }
                }
            }
        }
    }

    println!(
        "Index Updates: {} .. {} - {}",
        changes.len(),
        p_index_prefix,
        p_destination_field
    );
    bulk_jdbc_update_index_add_field(changes, elastic_url, elastic_user, elastic_pass).await;
}

pub async fn get_jdbc_index_missing_field<T: ToString>(
    index_name: T,
    missing_field: T,
    elastic_url: T,
    elastic_user: T,
    elastic_pass: T,
    total: u16,
) -> Result<Vec<JDBCSearchResult>, Error> {
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
            let res: JDBCIndexSearchResult = match serde_json::from_str(res.clone().as_str()) {
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

#[allow(unused)]
pub async fn jdbc_update_index_add_field(
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

    let json = jdbc_index_update_json(new_field.to_string(), new_field_value.to_string());

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
        .timeout(Duration::new(30, 0))
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

pub async fn bulk_jdbc_update_index_add_field(
    updates: Vec<JDBCIndexUpdate>,
    elastic_url: String,
    elastic_user: String,
    elastic_pass: String,
) {
    // println!("{}", updates.len());

    let full_url = format!("{}{}{}", elastic_url.to_string(), "/", "_bulk/",);

    let json = bulk_jdbc_index_update_json(updates);

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
                // println!("{}", res);
                // println!("{}\n\n{}", res, json.clone().substring(0,120));
                println!("{}\n\n{}", res, json.clone());
            }
            // println!("{}", res);
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
pub struct JDBCIndexUpdate {
    index_name: String,
    index_id: String,
    new_field_name: String,
    new_field_value: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct JDBCIndexUpdateScript {
    script: String,
}

#[allow(unused)]
fn jdbc_index_update_json(new_field: String, value: String) -> String {
    let bkslsh = r#"\u0027"#;
    format!(
        "{{\"script\" : \"ctx._source.{} = {}{}{}\"}}",
        new_field, bkslsh, value, bkslsh
    )
}

fn bulk_jdbc_index_update_json(changes: Vec<JDBCIndexUpdate>) -> String {
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

        #[allow(unused)]
        let mut new_field = String::new();
        #[allow(unused)]
        let mut new_value = String::new();

        // println!("Before: {}", ch.new_field_value);
        // new_field = ch.new_field_name;
        // new_value = ch.new_field_value;

        new_field = escape_special(ch.new_field_name.clone());
        new_value = escape_special(ch.new_field_value.clone());

        // // println!("Before: {}", ch.new_field_value);
        // // println!("After: {}", new_value);
        // new_value = new_value.replace("\\u", "");

        // Remove control characters
        // ctrl-@      0        00
        new_value = new_value.replace("\u{0000}", "");
        // ctrl-A      1        01
        new_value = new_value.replace("\u{0001}", "");
        // ctrl-B      2        02
        new_value = new_value.replace("\u{0002}", "");
        // ctrl-C      3        03
        new_value = new_value.replace("\u{0003}", "");
        // ctrl-D      4        04
        new_value = new_value.replace("\u{0004}", "");
        // ctrl-E      5        05
        new_value = new_value.replace("\u{0005}", "");
        // ctrl-F      6        06
        new_value = new_value.replace("\u{0006}", "");
        // ctrl-G      7        07
        new_value = new_value.replace("\u{0007}", "");
        // ctrl-H      8        08
        new_value = new_value.replace("\u{0008}", "");
        // ctrl-I      9        09
        new_value = new_value.replace("\u{0009}", "");
        // ctrl-J      10       0A
        new_value = new_value.replace("\u{000A}", "");
        // ctrl-K      11       0B
        new_value = new_value.replace("\u{000B}", "");
        // ctrl-L      12       0C
        new_value = new_value.replace("\u{000C}", "");
        // ctrl-M      13       0D
        new_value = new_value.replace("\u{000D}", "");
        // ctrl-N      14       0E
        new_value = new_value.replace("\u{000E}", "");
        // ctrl-O      15       0F
        new_value = new_value.replace("\u{000F}", "");
        // ctrl-P      16       10
        new_value = new_value.replace("\u{0010}", "");
        // ctrl-Q      17       11
        new_value = new_value.replace("\u{0011}", "");
        // ctrl-R      18       12
        new_value = new_value.replace("\u{0012}", "");
        // ctrl-S      19       13
        new_value = new_value.replace("\u{0013}", "");
        // ctrl-T      20       14
        new_value = new_value.replace("\u{0014}", "");
        // ctrl-U      21       15
        new_value = new_value.replace("\u{0015}", "");
        // ctrl-V      22       16
        new_value = new_value.replace("\u{0016}", "");
        // ctrl-W      23       17
        new_value = new_value.replace("\u{0017}", "");
        // ctrl-X      24       18
        new_value = new_value.replace("\u{0018}", "");
        // ctrl-Y      25       19
        new_value = new_value.replace("\u{0019}", "");
        // ctrl-Z      26       1A
        new_value = new_value.replace("\u{001A}", "");
        // ctrl-[      27       1B
        new_value = new_value.replace("\u{001B}", "");
        // ctrl-\      28       1C
        new_value = new_value.replace("\u{001C}", "");
        // ctrl-]      29       1D
        new_value = new_value.replace("\u{001D}", "");
        // ctrl-^      30       1E
        new_value = new_value.replace("\u{001E}", "");
        // ctrl-_      31       1F
        new_value = new_value.replace("\u{001F}", "");
        //

        new_value = new_value.replace("null", "..");
        new_value = new_value.replace("null", "..");
        //
        new_value = new_value.replace('"', "|");
        new_value = new_value.replace(',', "|");

        // new_value = new_value.replace('{', "|");
        // new_value = new_value.replace('}', "|");
        // new_value = new_value.replace('[', "|");
        // new_value = new_value.replace(']', "|");
        // new_value = new_value.replace("https://", "");
        // new_value = new_value.replace("http://", "");
        // new_value = new_value.replace("://", "");
        // new_value = new_value.replace("\\x", "");

        new_value = new_value.replace(r#""{\"#, r#""{\\"#);
        // println!("NEWVAL: {}", new_value);
        new_value = new_value.trim().to_string();

        // new_value = new_value.split("response=").last().unwrap().to_string().replace('"', "'");
        // new_value = new_value.split("response=").last().unwrap().to_string().replace(',', "|");
        // if new_value.contains("response=") {
        //     // new_value = "".to_string()
        // }

        new_value = new_value.replace("\u{000E}", ""); // bugfix
        new_value = new_value.replace('\\', ""); // bugfix
        new_value = new_value.replace("HTTP/1.1\"", ""); // bugfix
                                                         // new_value = new_value.replace('&', encode("&").as_ref()); // bugfix

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
    jdbc: Vec<Transform>,
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

pub fn get_jdbc_transform_config() -> Vec<Transform> {
    let toml_str = fs::read_to_string("config/Transforms.toml").unwrap();
    if let Ok(entries) = toml::from_str::<TransformOuter>(&toml_str) {
        entries.jdbc
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use crate::transform::jdbc_transforms::{
        get_jdbc_index_missing_field, get_jdbc_transform_config, parse_to_new_jdbc_field,
    };
    use config::Config;
    use std::collections::HashMap;

    fn config() -> Config {
        Config::builder()
            .add_source(config::File::with_name("../../config/Settings.toml"))
            .build()
            .unwrap()
    }

    #[test]
    // #[ignore]
    fn test_transform() {
        let config = config();
        let settings_map = config.try_deserialize::<HashMap<String, String>>().unwrap();

        let tk = tokio::runtime::Runtime::new();
        tk.unwrap().block_on(parse_to_new_jdbc_field(
            "jdbc_mysql-2024.05.21".to_string(),
            // "jdbc_mysql-TODAY".to_string(),
            // "jdbc_mysql-YESTERDAY".to_string(),
            "response_body".to_string(),
            "response_body_only".to_string(),
            "response_body_needle_type".to_string(),
            "\n".to_string(),
            settings_map.clone(),
            40,
        ))
    }

    #[test]
    #[ignore]
    fn test_transform_toml() {
        let entries = get_jdbc_transform_config();
        for i in entries.iter() {
            println!("{:?}", i);
        }
    }

    #[test]
    // #[ignore]
    fn test_get_missing_field() {
        let settings = Config::builder()
            .add_source(config::File::with_name("../../config/Settings"))
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
        let out = rt.unwrap().block_on(get_jdbc_index_missing_field(
            "jdbc*",
            "new_field",
            elastic_url,
            elastic_user,
            elastic_pass,
            10,
        ));
        let cl = out.unwrap().clone();
        // println!("{:?}", cl);
        assert!(!cl.is_empty())
    }
}
