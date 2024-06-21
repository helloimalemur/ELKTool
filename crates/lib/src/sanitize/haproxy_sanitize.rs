use crate::common::haproxy_index::get_haproxy_index_missing_field;
use crate::common::haproxy_index_entities::HAProxyIndexUpdate;
use crate::common::haproxy_index_search_result_entities::{HAProxyEvent, HAProxySource};
use crate::common::{escape_special, today_index_name, yesterday_index_name};
use crate::lifetime_api::lifetime_api_funcs::index_update_script_json;
use chrono::Datelike;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use substring::Substring;

pub async fn start_sanitize_haproxy(settings_map: HashMap<String, String>) {
    let san = get_haproxy_sanitize_config();
    for entry in san {
        for _i in 0..entry.multiplier {
            let mut ind_name = entry.index_prefix.to_string();
            if entry.index_prefix.contains("TODAY") {
                ind_name = today_index_name(entry.index_prefix.to_string());
            }
            if entry.index_prefix.contains("YESTERDAY") {
                ind_name = yesterday_index_name(entry.index_prefix.to_string());
            }

            sanitize_haproxy_field(
                ind_name,
                entry.check_field_value.to_string(),
                settings_map.clone(),
                entry.total_to_process,
            )
            .await;
        }
    }
}

fn get_haproxy_sanitize_config() -> Vec<Sanitize> {
    let toml_str = fs::read_to_string("config/Sanitize.toml").unwrap();
    match toml::from_str::<SanitizeOuter>(&toml_str) {
        Ok(a) => a.haproxy,
        Err(e) => {
            println!("{e}");
            vec![]
        }
    }
}

#[derive(Debug, Deserialize)]
struct SanitizeOuter {
    haproxy: Vec<Sanitize>,
}

#[derive(Debug, Deserialize)]
struct Sanitize {
    index_prefix: String,
    check_field_value: String,
    total_to_process: u16,
    multiplier: u16,
}

pub async fn sanitize_haproxy_field(
    index_prefix: String,
    check_field: String,
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
    let p_destination_field = check_field.clone();

    // println!("{:?}", index_data);
    #[allow(unused)]
    let mut rcount = 0;
    let mut changes: Vec<HAProxyIndexUpdate> = vec![];

    let mut ind_name = index_prefix.to_string();
    #[allow(unused)]
    let mut day_str = String::new();
    #[allow(unused)]
    let mut month_str = String::new();
    #[allow(unused)]
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
    let year = tdy.year();

    if index_prefix.contains("TODAY") {
        let date_str = format!("{}.{}.{}", year, month_str, day_str);
        ind_name = ind_name.replace("TODAY", date_str.as_str());
    }

    // println!("{}", ind_name);

    // get list of indexes missing new field
    let index_data = get_haproxy_index_missing_field(
        ind_name,
        check_field.clone(),
        elastic_url.to_string(),
        elastic_user.to_string(),
        elastic_pass.to_string(),
        total,
    )
    .await;

    // println!("{:?}", index_data);

    // prepare index update script (changes)
    if let Ok(i_d) = index_data {
        for index in i_d.iter() {
            let index_name = index.clone().index.unwrap_or("".to_string());
            let index_id = index.clone().id.unwrap_or("".to_string());

            // let message = index.clone().source.unwrap().message.unwrap();
            let original = index
                .clone()
                .source
                .unwrap_or(HAProxySource {
                    event: None,
                    server_name: None,
                    client_ip: None,
                    ecs: None,
                    syslog_server: None,
                    time_backend_response: None,
                    agent: None,
                    syslog_timestamp: None,
                    beconn: None,
                    time_duration: None,
                    version: None,
                    termination_state: None,
                    time_backend_connect: None,
                    log: None,
                    input: None,
                    tags: None,
                    retries: None,
                    srv_queue: None,
                    captured_response_cookie: None,
                    process: None,
                    feconn: None,
                    message: None,
                    frontend_name: None,
                    backend_name: None,
                    host: None,
                    time_queue: None,
                    haproxy: None,
                    timestamp: None,
                    accept_date: None,
                    captured_request_cookie: None,
                    time_request: None,
                    client_port: None,
                    backend_queue: None,
                    srvconn: None,
                    bytes_read: None,
                    actconn: None,
                    http_status_code: None,
                    jsonrequest: None,
                    request_body: None,
                    requestmethod: None,
                    konnektype: None,
                    datecreated: None,
                    responsetime: None,
                    companyid: None,
                    response_body: None,
                    remoteurl: None,
                    request_params: None,
                    id: None,
                    login_id: None,
                })
                .event
                .unwrap_or(HAProxyEvent { original: None })
                .original
                .unwrap_or("".to_string());
            // if message contains urlencoding decode it

            // sanitize!(changes, message, index.clone(), "message".to_string(), check_field);
            // sanitize!(changes, original, index.clone(), "original".to_string(), check_field);
            // sanitize!(original, index.clone(), "captured_headers".to_string());

            // let index_name = $b.clone().index.unwrap();
            // let index_id = $b.clone().id.unwrap();
            // println!("NAME: {:?}", index_name);
            // println!("ID: {:?}", index_id);

            // let mut msg = message.clone();
            let mut msg = original.clone();

            if msg.contains(&"cardNumber#22#0D#0A#0D#0A") {
                // println!("CONTAINTS CARDNUMBER");
                // println!("NAME: {:?}", index_name);
                // println!("ID: {:?}", index_id);
                // rcount += 1;

                if let Some(split_by_needle) =
                    msg.split(&"cardNumber#22#0D#0A#0D#0A".to_string()).last()
                {
                    // println!("{:?}", index);
                    let split_by_param0: Vec<&str> = split_by_needle.split("#0D#0A").collect();
                    // index name
                    // param value
                    if let Some(split_by_param0_res) = split_by_param0.first() {
                        let split_by_ws_res0: Vec<&str> =
                            split_by_param0_res.split_whitespace().collect();
                        let mut new_field_value0 = String::new();
                        // println!("{}", new_field_value0);
                        if let Some(s) = split_by_ws_res0.first() {
                            new_field_value0 = s.to_string();
                        }
                        // if found string replace it in the original index
                        let replacer_from0 = format!("{}{}", "cardNumber#22#0D#0A#0D#0A", new_field_value0);
                        let replacer_to0 = format!("{}{}", "cardNumber#22#0D#0A#0D#0A", "UNAVAILABLE_01");
                        msg = msg.replace(replacer_from0.as_str(), replacer_to0.as_str());
                    }
                }
            }

            if msg.contains(&"cardNumber\"#015#012#015#") {
                // println!("CONTAINTS CARDNUMBER");
                // println!("NAME: {:?}", index_name);
                // println!("ID: {:?}", index_id);
                // rcount += 1;

                if let Some(split_by_needle) =
                    msg.split(&"cardNumber\"#015#012#015#".to_string()).last()
                {
                    // println!("{:?}", index);
                    let split_by_param1: Vec<&str> = split_by_needle.split("#012").collect();
                    // index name
                    // param value
                    if let Some(split_by_param1_res) = split_by_param1.first() {
                        let split_by_ws_res1: Vec<&str> =
                            split_by_param1_res.split_whitespace().collect();
                        let mut new_field_value1 = String::new();
                        // println!("{}", new_field_value1);
                        if let Some(s) = split_by_ws_res1.first() {
                            new_field_value1 = s.to_string();
                        }
                        // if found string replace it in the original index
                        let replacer_from1 = format!("{}{}", "cardNumber\"#015#012#015#", new_field_value1);
                        let replacer_to1 = format!("{}{}", "cardNumber\"#015#012#015#", "UNAVAILABLE_02");
                        msg = msg.replace(replacer_from1.as_str(), replacer_to1.as_str());
                    }
                }
            }

            if msg.contains(&"cardNumber#015#012#015#0") {
                // println!("CONTAINTS CARDNUMBER");
                // println!("NAME: {:?}", index_name);
                // println!("ID: {:?}", index_id);
                // rcount += 1;

                if let Some(split_by_needle) =
                    msg.split(&"cardNumber#015#012#015#0".to_string()).last()
                {
                    // println!("{:?}", index);
                    let split_by_param2: Vec<&str> = split_by_needle.split("#012").collect();
                    // index name
                    // param value
                    if let Some(split_by_param2_res) = split_by_param2.first() {
                        let split_by_ws_res2: Vec<&str> =
                            split_by_param2_res.split_whitespace().collect();
                        let mut new_field_value2 = String::new();
                        // println!("{}", new_field_value2);
                        if let Some(s) = split_by_ws_res2.first() {
                            new_field_value2 = s.to_string();
                        }
                        // if found string replace it in the original index
                        let replacer_from2 = format!("{}{}", "cardNumber#015#012#015#0", new_field_value2);
                        let replacer_to2 = format!("{}{}", "cardNumber#015#012#015#0", "UNAVAILABLE_03");
                        msg = msg.replace(replacer_from2.as_str(), replacer_to2.as_str());
                    }
                }
            }

            if msg.contains(&"cardNumber=") {
                // println!("CONTAINTS CARDNUMBER");
                // println!("NAME: {:?}", index_name);
                // println!("ID: {:?}", index_id);
                // rcount += 1;

                if let Some(split_by_needle) = msg.split(&"cardNumber=".to_string()).last() {
                    // println!("{:?}", index);
                    let split_by_param3: Vec<&str> = split_by_needle.split('&').collect();
                    // index name
                    // param value
                    if let Some(split_by_param3_res) = split_by_param3.first() {
                        let split_by_ws_res3: Vec<&str> =
                            split_by_param3_res.split_whitespace().collect();
                        let mut new_field_value3 = String::new();
                        // println!("{}", new_field_value3);
                        if let Some(s) = split_by_ws_res3.first() {
                            new_field_value3 = s.to_string();
                        }
                        // if found string replace it in the original index
                        let replacer_from3 = format!("{}{}", "cardNumber=", new_field_value3);
                        let replacer_to3 = format!("{}{}", "cardNumber=", "UNAVAILABLE_04");
                        msg = msg.replace(replacer_from3.as_str(), replacer_to3.as_str());
                    }
                }
            }

            if msg.contains(&"password=") {
                // println!("CONTAINS PASSWORD");
                // println!("NAME: {:?}", index_name);
                // println!("ID: {:?}", index_id);
                // rcount += 1;

                if let Some(split_by_needle) = msg.split(&"password=".to_string()).last() {
                    // println!("{:?}", split_by_needle);
                    let split_by_param4: Vec<&str> = split_by_needle.split('&').collect();
                    // index name
                    // param value
                    if let Some(split_by_param4_res) = split_by_param4.first() {
                        let split_by_ws_res4: Vec<&str> =
                            split_by_param4_res.split_whitespace().collect();
                        let mut new_field_value4 = String::new();
                        // println!("{}", new_field_value4);
                        if let Some(s) = split_by_ws_res4.first() {
                            new_field_value4 = s.to_string();
                        }
                        // if found string replace it in the original index
                        let replacer_from = format!("{}{}", "password=", new_field_value4);
                        let replacer_to = format!("{}{}", "password=", "UNAVAILABLE_05");
                        msg = msg.replace(replacer_from.as_str(), replacer_to.as_str());
                    }
                }
            }

            if msg.contains(&"name='password'") {
                // println!("CONTAINS PASSWORD");
                // println!("NAME: {:?}", index_name);
                // println!("ID: {:?}", index_id);
                // rcount += 1;

                if let Some(split_by_needle) = msg.split(&"name='password'".to_string()).last() {
                    // println!("{:?}", split_by_needle);
                    let split_by_param5: Vec<&str> = split_by_needle.split("-----").collect();
                    // index name
                    // param value
                    if let Some(split_by_param5_res) = split_by_param5.first() {
                        let split_by_ws_res5: Vec<&str> =
                            split_by_param5_res.split_whitespace().collect();
                        let mut new_field_value5 = String::new();
                        // println!("{}", new_field_value5);
                        if let Some(s) = split_by_ws_res5.first() {
                            new_field_value5 = s.to_string();
                        }
                        // if found string replace it in the original index
                        let replacer_from5 = format!("{}{}", "name='password'", new_field_value5);
                        let replacer_to5 = format!("{}{}", "name='password'", "UNAVAILABLE_06");
                        msg = msg.replace(replacer_from5.as_str(), replacer_to5.as_str());
                    }
                }
            }



            // "cardNumber" AND "UNAVAILABLE" AND NOT "{UNAVAILABLE<!DOCTYPE html>" AND NOT "UNAVAILABLEMUNAVAILABLEaUNAVAILABLEyUNAVAILABLE" AND sanitized : **
            // println!("{msg}");

            // exit(0);

            // update to sanitize
            changes.push(HAProxyIndexUpdate {
                index_name: index_name.clone(),
                index_id: index_id.clone(),
                new_field_name: "message".to_string(),
                new_field_value: msg,
            });

            // update to create check field
            changes.push(HAProxyIndexUpdate {
                index_name: index_name.clone(),
                index_id: index_id.clone(),
                new_field_name: check_field.to_string(),
                new_field_value: "true".to_string(),
            });
        }
    }

    println!(
        "Index Updates: {} .. {} - {}",
        changes.len() / 2,
        p_index_prefix,
        p_destination_field
    );
    // println!("{:?}", changes);
    send_bulk_sanitize_haproxy(
        changes,
        elastic_url,
        elastic_user,
        elastic_pass,
        check_field,
    )
    .await;
}

pub async fn send_bulk_sanitize_haproxy(
    updates: Vec<HAProxyIndexUpdate>,
    elastic_url: String,
    elastic_user: String,
    elastic_pass: String,
    check_field: String,
) {
    // println!("{}", updates.len());

    let full_url = format!("{}{}{}", elastic_url.to_string(), "/", "_bulk/",);

    let json = bulk_haproxy_sanitize(updates, check_field);

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

fn bulk_haproxy_sanitize(changes: Vec<HAProxyIndexUpdate>, check_field: String) -> String {
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

        new_value = new_value.replace("#22", "'"); // clean output
        new_value = new_value.replace("#20", " "); // clean output
        new_value = new_value.replace("#7D", ""); // clean output
        new_value = new_value.replace("#7B", ""); // clean output
        new_value = new_value.replace("#012", " "); // clean output
        new_value = new_value.replace("#015", " "); // clean output
        new_value = new_value.replace("#0D", " "); // clean output
        new_value = new_value.replace("#0A", " "); // clean output
        new_value = new_value.replace(",", ", "); // clean output
        new_value = new_value.replace("  ", " "); // clean output
        new_value = new_value.replace("   ", " "); // clean output

        let doc = format!(
            "{{ \"doc\" : {{\"{}\" : \"{}\"}} }}\n",
            new_field, new_value
        );
        full_string.push_str(doc.as_str());

        // add overwrite //
        let update = format!(
            "{{ \"update\" : {{\"_id\" : \"{}\", \"_index\" : \"{}\"}} }}\n",
            ch.index_id, ch.index_name
        );
        full_string.push_str(update.as_str());
        let update = index_update_script_json("event.original".to_string(), "redacted".to_string());
        full_string.push_str(update.as_str());
        // //

        // add overwrite //
        let update = format!(
            "{{ \"update\" : {{\"_id\" : \"{}\", \"_index\" : \"{}\"}} }}\n",
            ch.index_id, ch.index_name
        );
        full_string.push_str(update.as_str());
        let update = index_update_script_json(
            "haproxy.http.request.captured_headers".to_string(),
            "redacted".to_string(),
        );
        full_string.push_str(update.as_str());
        // //

        // add overwrite //
        let update = format!(
            "{{ \"update\" : {{\"_id\" : \"{}\", \"_index\" : \"{}\"}} }}\n",
            ch.index_id, ch.index_name
        );
        full_string.push_str(update.as_str());
        let update = index_update_script_json(
            "haproxy.http.response.captured_headers".to_string(),
            "redacted".to_string(),
        );
        full_string.push_str(update.as_str());
        // //

        // add sanitized field //
        let update = format!(
            "{{ \"update\" : {{\"_id\" : \"{}\", \"_index\" : \"{}\"}} }}\n",
            ch.index_id, ch.index_name
        );
        full_string.push_str(update.as_str());
        let doc = format!("{{ \"doc\" : {{\"{}\" : \"{}\"}} }}\n", check_field, "true");
        full_string.push_str(doc.as_str());
        // //
    }
    full_string.push_str("\n");
    // println!("{full_string}");
    full_string
}
