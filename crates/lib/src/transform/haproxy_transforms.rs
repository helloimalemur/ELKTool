use crate::common::haproxy_index::get_haproxy_index_missing_field;
use crate::common::haproxy_index_entities::HAProxyIndexUpdate;
use crate::common::{escape_special, today_index_name, yesterday_index_name};
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use substring::Substring;

pub async fn start_haproxy_transforms(settings_map: HashMap<String, String>) {
    #[allow(unused)]
    let mut metrics_req_per_sec: Vec<u64> = vec![];
    let transforms = get_haproxy_transform_config();
    for transform in transforms.iter() {
        let mut ind_name = transform.index_prefix.to_string();
        if transform.index_prefix.contains("TODAY") {
            ind_name = today_index_name(transform.index_prefix.to_string());
        }
        if transform.index_prefix.contains("YESTERDAY") {
            ind_name = yesterday_index_name(transform.index_prefix.to_string());
        }

        // 2024.05.14
        // println!("{}", ind_name);
        for _i in 0..transform.multiplier {
            parse_to_new_haproxy_field(
                ind_name.clone(),
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
}

pub async fn parse_to_new_haproxy_field(
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
    let index_data = get_haproxy_index_missing_field(
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
    let mut changes: Vec<HAProxyIndexUpdate> = vec![];

    // prepare index update script (changes)
    if let Ok(i_d) = index_data {
        for index in i_d.iter() {
            let message = index.clone().source.unwrap().message.unwrap();
            let _original = index
                .clone()
                .source
                .unwrap()
                .event
                .unwrap()
                .original
                .unwrap();

            // todo()! custom 2nd needle in config
            if message.contains(&"name=\"loginId\"#015#012#015#012") {
                rcount += 1;
                if let Some(split_by_needle0) = message.split(&"name=\"loginId\"#015#012#015#012".to_string()).last() {
                    // println!("{:?}", index);
                    let split_by_param0: Vec<&str> = split_by_needle0.split("#015#012").collect();
                    // index name
                    let index_name0 = index.clone().index.unwrap();
                    // index id
                    let index_id0 = index.clone().id.unwrap();
                    // param value
                    if let Some(split_by_param_res0) = split_by_param0.first() {
                        let split_by_ws_res0: Vec<&str> = split_by_param_res0.split_whitespace().collect();
                        let mut new_field_value0 = String::new();
                        // println!("{}", new_field_value);
                        if let Some(s0) = split_by_ws_res0.first() {
                            new_field_value0 = s0.to_string();
                        }

                        changes.push(HAProxyIndexUpdate {
                            index_name: index_name0,
                            index_id: index_id0,
                            new_field_name: "loginId".to_string(),
                            new_field_value: new_field_value0,
                        })
                    }
                }
            }

            if message.contains(&"name='loginId'") {
                rcount += 1;
                if let Some(split_by_needle1) = message.split(&"name='loginId'".to_string()).last() {
                    // println!("{:?}", index);
                    let split_by_param1: Vec<&str> = split_by_needle1.split("-----").collect();
                    // index name
                    let index_name1 = index.clone().index.unwrap();
                    // index id
                    let index_id1 = index.clone().id.unwrap();
                    // param value
                    if let Some(split_by_param_res1) = split_by_param1.first() {
                        let split_by_ws_res1: Vec<&str> = split_by_param_res1.split_whitespace().collect();
                        let mut new_field_value1 = String::new();
                        // println!("{}", new_field_value);
                        if let Some(s1) = split_by_ws_res1.first() {
                            new_field_value1 = s1.to_string();
                        }

                        changes.push(HAProxyIndexUpdate {
                            index_name: index_name1,
                            index_id: index_id1,
                            new_field_name: "loginId".to_string(),
                            new_field_value: new_field_value1,
                        })
                    }
                }
            }

            // add endpoint field
            if message.contains(&"\"GET") {
                rcount += 1;
                if let Some(split_by_needle2) = message.split(&"\"GET".to_string()).last() {
                    // println!("{:?}", index);
                    let split_by_param2: Vec<&str> = split_by_needle2.split('?').collect();
                    // index name
                    let index_name2 = index.clone().index.unwrap();
                    // index id
                    let index_id2 = index.clone().id.unwrap();
                    // param value
                    if let Some(split_by_param_res2) = split_by_param2.first() {
                        let split_by_ws_res2: Vec<&str> = split_by_param_res2.split_whitespace().collect();
                        let mut new_field_value2 = String::new();
                        // println!("{}", new_field_value);
                        if let Some(s2) = split_by_ws_res2.first() {
                            new_field_value2 = s2.to_string();
                        }

                        changes.push(HAProxyIndexUpdate {
                            index_name: index_name2,
                            index_id: index_id2,
                            new_field_name: "url_endpoint".to_string(),
                            new_field_value: new_field_value2,
                        })
                    }
                }
            }



            if message.contains(&"\"POST") {
                rcount += 1;
                if let Some(split_by_needle3) = message.split(&"\"POST".to_string()).last() {
                    // println!("{:?}", index);
                    let split_by_param3: Vec<&str> = split_by_needle3.split('?').collect();
                    // index name
                    let index_name3 = index.clone().index.unwrap();
                    // index id
                    let index_id3 = index.clone().id.unwrap();
                    // param value
                    if let Some(split_by_param_res3) = split_by_param3.first() {
                        let split_by_ws_res3: Vec<&str> = split_by_param_res3.split_whitespace().collect();
                        let mut new_field_value3 = String::new();
                        // println!("{}", new_field_value);
                        if let Some(s3) = split_by_ws_res3.first() {
                            new_field_value3 = s3.to_string();
                        }

                        changes.push(HAProxyIndexUpdate {
                            index_name: index_name3,
                            index_id: index_id3,
                            new_field_name: "url_endpoint".to_string(),
                            new_field_value: new_field_value3,
                        })
                    }
                }
            }


            if message.contains(&needle) {
                rcount += 1;
                if let Some(split_by_needle4) = message.split(&needle.to_string()).last() {
                    // println!("{:?}", index);
                    let split_by_param4: Vec<&str> = split_by_needle4.split('&').collect();
                    // index name
                    let index_name4 = index.clone().index.unwrap();
                    // index id
                    let index_id4 = index.clone().id.unwrap();
                    // param value
                    if let Some(split_by_param_res4) = split_by_param4.first() {
                        let split_by_ws_res4: Vec<&str> = split_by_param_res4.split_whitespace().collect();
                        let mut new_field_value4 = String::new();
                        // println!("{}", new_field_value);
                        if let Some(s4) = split_by_ws_res4.first() {
                            new_field_value4 = s4.to_string();
                        }

                        changes.push(HAProxyIndexUpdate {
                            index_name: index_name4,
                            index_id: index_id4,
                            new_field_name: destination_field.to_string(),
                            new_field_value: new_field_value4,
                        })
                    }
                }
            } else {
                let index_name5 = index.clone().index.unwrap();
                let index_id5 = index.clone().id.unwrap();

                changes.push(HAProxyIndexUpdate {
                    index_name: index_name5,
                    index_id: index_id5,
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
    send_bulk_haproxy_update_index_add_field(changes, elastic_url, elastic_user, elastic_pass)
        .await;
}

// https://www.elastic.co/guide/en/elasticsearch/reference/current/docs-bulk.html#bulk-update

pub async fn send_bulk_haproxy_update_index_add_field(
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
            // println!("{}", res);
            if res.contains("x_content_parse_exception") {
                println!("{}", res);
            }
        } else {
            println!(
                "WARNING: REQUEST MAY HAVE FAILED :: {}",
                json.substring(0, 120)
            );
        }
    } else {
        println!(
            "WARNING: REQUEST FAILED :: {}, {}",
            json.substring(0, 120),
            client.err().unwrap()
        );
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

fn bulk_haproxy_index_update_json(changes: Vec<HAProxyIndexUpdate>) -> String {
    let mut full_string = String::new();
    // { "update" : {"_id" : "1", "_index" : "test"} }
    // { "doc" : {"field2" : "value2"} }
    // let bkslsh = r#"\u0027"#;
    // format!(
    //     "{{\"script\" : \"ctx._source.{} = {}{}{}\"}}",
    //     new_field, bkslsh, value, bkslsh
    // )

    // println!("CHANGES LEN: {}", changes.len());

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

        new_field = escape_special(ch.new_field_name.clone());
        new_value = escape_special(ch.new_field_value.clone());

        // println!("Before: {}", ch.new_field_value);
        // println!("After: {}", new_value);

        // Remove control characters
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

        new_value = new_value.replace('\\', ""); // bugfix
        new_value = new_value.replace("HTTP/1.1\"", ""); // bugfix

        // new_value = new_value.replace("/0", ""); // bugfix
        new_value = new_value.replace("\"", ""); // bugfix

        // if new_field.contains("event.original") {
        //     let doc = format!(
        //         // { "script" : { "source": "ctx._source.counter += params.param1", "lang" : "painless", "params" : {"param1" : 1}}, "upsert" : {"counter" : 1}}
        //         // { "script": { "source": "ctx._source.userName=new_user_name", "lang": "painless"}
        //         // "{{ \"script\" : {{\"source\" : \"ctx._source.{}={}\", \"lang\": \"painless\"}} }}\n",
        //         "{{\"script\" : \"ctx._source.{} = {}\"}}",
        //         // { "doc" : {"field2" : "value2"} }
        //         // "{{ \"doc\" : {{\"{}\" : \"{}\"}} }}\n",
        //         new_field, new_value
        //     );
        //
        //     full_string.push_str(doc.as_str());
        // } else {
        //     let doc = format!(
        //         "{{ \"doc\" : {{\"{}\" : \"{}\"}} }}\n",
        //         new_field, new_value
        //     );
        //
        //     full_string.push_str(doc.as_str());
        // }
        let doc = format!(
            "{{ \"doc\" : {{\"{}\" : \"{}\"}} }}\n",
            new_field, new_value
        );
        full_string.push_str(doc.as_str());

        // // add event.original and haproxy. overwrite
        // let update = format!(
        //     "{{ \"update\" : {{\"_id\" : \"{}\", \"_index\" : \"{}\"}} }}\n",
        //     ch.index_id, ch.index_name
        // );
        // full_string.push_str(update.as_str());
        // let update = index_update_script_json("event.original".to_string(), "redacted".to_string());
        // full_string.push_str(update.as_str());
        // // add sanitized field
        // let update = format!(
        //     "{{ \"update\" : {{\"_id\" : \"{}\", \"_index\" : \"{}\"}} }}\n",
        //     ch.index_id, ch.index_name
        // );
        // full_string.push_str(update.as_str());
        // let doc = format!(
        //     "{{ \"doc\" : {{\"{}\" : \"{}\"}} }}\n",
        //     "sanitized", "true"
        // );
        // full_string.push_str(doc.as_str());
    }
    full_string.push_str("\n");
    // println!("{full_string}");
    full_string
}

#[derive(Debug, Deserialize)]
struct TransformOuter {
    haproxy: Vec<Transform>,
}

#[derive(Debug, Deserialize)]
struct Transform {
    index_prefix: String,
    source_field: String,
    destination_field: String,
    transform_type: String,
    needle: String,
    total_to_process: u16,
    multiplier: u16,
}

fn get_haproxy_transform_config() -> Vec<Transform> {
    let toml_str = fs::read_to_string("config/Transforms.toml").unwrap();
    match toml::from_str::<TransformOuter>(&toml_str) {
        Ok(a) => a.haproxy,
        Err(e) => {
            println!("{e}");
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sanitize::haproxy_sanitize::{sanitize_haproxy_field, start_sanitize_haproxy};
    use crate::transform::haproxy_transforms::{
        get_haproxy_transform_config, parse_to_new_haproxy_field,
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
    fn test_transform_toml() {
        let entries = get_haproxy_transform_config();
        for i in entries.iter() {
            println!("{:?}", i);
        }
        assert!(!entries.is_empty())
    }

    #[test]
    #[ignore]
    fn test_sanitize_haproxy_field() {
        let settings = Config::builder()
            .add_source(config::File::with_name("../../config/Settings"))
            .build()
            .unwrap();
        let settings_map = settings
            .try_deserialize::<HashMap<String, String>>()
            .unwrap();

        let rt = tokio::runtime::Runtime::new();
        let out = rt.unwrap().block_on(sanitize_haproxy_field(
            "haproxy-files-*".to_string(),
            "testcheckfield1".to_string(),
            settings_map,
            100,
        ));

        // assert!(!out.unwrap().is_empty())
    }

    #[test]
    // #[ignore]
    fn test_start_sanitize() {
        let settings = Config::builder()
            .add_source(config::File::with_name("../../config/Settings"))
            .build()
            .unwrap();
        let settings_map = settings
            .try_deserialize::<HashMap<String, String>>()
            .unwrap();

        let rt = tokio::runtime::Runtime::new();
        let out = rt
            .unwrap()
            .block_on(start_sanitize_haproxy(settings_map.clone()));

        // println!("{:?}", cl);
    }
}
