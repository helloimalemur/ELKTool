use crate::common::haproxy_index::get_haproxy_index_missing_field_but_has;
use crate::common::haproxy_index_entities::HAProxyIndexUpdate;
use crate::common::today_index_name;
use crate::transform::haproxy_transforms::send_bulk_haproxy_update_index_add_field;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use substring::Substring;

pub async fn start_replicate_haproxy(settings_map: HashMap<String, String>) {
    let repl = get_haproxy_replicate_config();
    for entry in repl {
        for _i in 0..entry.multiplier {
            replicate_haproxy_index(
                entry.index_prefix.to_string(),
                entry.check_field_value.to_string(),
                entry.has_field_value.clone(),
                settings_map.clone(),
                entry.total_to_process,
                entry.dest_url.clone(),
                entry.dest_username.clone(),
                entry.dest_password.clone(),
            )
            .await
        }
    }
}

fn get_haproxy_replicate_config() -> Vec<Replicate> {
    let toml_str = fs::read_to_string("config/Replicate.toml").unwrap();
    // println!("{:?}", toml_str);
    match toml::from_str::<ReplicateOuter>(&toml_str) {
        Ok(a) => a.haproxy,
        Err(e) => {
            println!("{e}");
            vec![]
        }
    }
}

#[derive(Debug, Deserialize)]
struct ReplicateOuter {
    haproxy: Vec<Replicate>,
}

#[derive(Debug, Deserialize)]
struct Replicate {
    index_prefix: String,
    check_field_value: String,
    has_field_value: String,
    total_to_process: u16,
    dest_url: String,
    dest_username: String,
    dest_password: String,
    multiplier: u16,
}

pub async fn replicate_haproxy_index(
    index_prefix: String,
    check_field: String,
    has_field: String,
    settings_map: HashMap<String, String>,
    total: u16,
    dest_url: String,
    dest_username: String,
    dest_password: String,
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

    #[allow(unused)]
    let mut rcount = 0;
    let mut source_changes: Vec<HAProxyIndexUpdate> = vec![];
    let mut changes: Vec<HAProxyIndexCreate> = vec![];

    // let mut ind_name = index_prefix.to_string();

    let ind_name = today_index_name(index_prefix.to_string());

    // #[allow(unused)]
    //     let mut day_str = String::new();
    // #[allow(unused)]
    //     let mut month_str = String::new();
    // #[allow(unused)]
    //     let mut year_str = String::new();
    //
    // let tdy = chrono::Local::now();
    // let day = tdy.day();
    // if day < 10 {
    //     day_str = format!("{}{}", 0, day)
    // } else {
    //     day_str = day.to_string()
    // }
    // let month = tdy.month();
    // if month < 10 {
    //     month_str = format!("{}{}", 0, month)
    // } else {
    //     month_str = month.to_string()
    // }
    // let year = tdy.year();
    //
    // if index_prefix.contains("TODAY") {
    //     let date_str = format!("{}.{}.{}", year, month_str, day_str);
    //     ind_name = ind_name.replace("TODAY", date_str.as_str());
    // }

    // println!("{}", ind_name);

    // get list of indexes missing new field
    let index_data = get_haproxy_index_missing_field_but_has(
        ind_name,
        check_field.clone(),
        has_field.clone(),
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
            let index_name = index.clone().index.unwrap();
            let index_id = index.clone().id.unwrap();

            let source = index.clone().source.unwrap();

            // update to create check field
            changes.push(HAProxyIndexCreate {
                timestamp: source.timestamp,
                index_name: index_name.clone(),
                index_id: index_id.clone(),
                message: source.message.unwrap_or("".to_string()),
                accept_date: source.accept_date.unwrap_or("".to_string()),
                login_id: source.login_id.unwrap_or("".to_string()),
            });

            source_changes.push(HAProxyIndexUpdate {
                index_name,
                index_id,
                new_field_name: check_field.to_string(),
                new_field_value: "true".to_string(),
            })
        }
    }

    println!(
        "Index To Replicate: {} .. {} - {}",
        changes.len(),
        p_index_prefix,
        p_destination_field
    );

    // println!("{:?}", changes);

    send_bulk_replicate_create_haproxy(
        changes,
        dest_url,
        dest_username,
        dest_password,
        check_field,
    )
    .await;

    // println!("{:?}", changes);

    send_bulk_haproxy_update_index_add_field(
        source_changes,
        elastic_url,
        elastic_user,
        elastic_pass,
    )
    .await;
}

pub async fn send_bulk_replicate_create_haproxy(
    updates: Vec<HAProxyIndexCreate>,
    dest_elastic_url: String,
    dest_elastic_user: String,
    dest_elastic_pass: String,
    check_field: String,
) {
    // println!("{}", updates.len());

    let full_url = format!("{}{}{}", dest_elastic_url.to_string(), "/", "_bulk/",);

    let json = bulk_haproxy_replicate(updates, check_field);

    // println!("{:?}", json.clone());
    //
    // exit(0);

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
        .post(full_url)
        .basic_auth(
            dest_elastic_user.to_string(),
            Some(dest_elastic_pass.to_string()),
        )
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
        // println!("success");
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

fn bulk_haproxy_replicate(changes: Vec<HAProxyIndexCreate>, _check_field: String) -> String {
    let mut full_string = String::new();
    // { "update" : {"_id" : "1", "_index" : "test"} }
    // { "doc" : {"field2" : "value2"} }
    // let bkslsh = r#"\u0027"#;
    // format!(
    //     "{{\"script\" : \"ctx._source.{} = {}{}{}\"}}",
    //     new_field, bkslsh, value, bkslsh
    // )

    // println!("CHANGES LEN: {}", changes.len());

    // { "create" : { "_index" : "test", "_id" : "3" } }
    // { "field1" : "value3" }

    for ch in changes {
        // //
        let update = format!(
            "{{ \"create\" : {{\"_id\" : \"{}\", \"_index\" : \"{}\"}} }}\n",
            ch.index_id, ch.index_name
        );
        full_string.push_str(update.as_str());
        // //

        // let doc = format!(
        //     "{{ \"doc\" : {{\"{}\" : \"{}\"}} }}\n",
        //     new_field, new_value
        // );

        let doc = serde_json::to_string(&ch).unwrap();
        full_string.push_str(doc.as_str());
        full_string.push_str("\n");

        // // add overwrite //
        // let update = format!(
        //     "{{ \"update\" : {{\"_id\" : \"{}\", \"_index\" : \"{}\"}} }}\n",
        //     ch.index_id, ch.index_name
        // );
        // full_string.push_str(update.as_str());
        // let update = index_update_script_json("event.original".to_string(), "redacted".to_string());
        // full_string.push_str(update.as_str());
        // // //
        //
        // // add overwrite //
        // let update = format!(
        //     "{{ \"update\" : {{\"_id\" : \"{}\", \"_index\" : \"{}\"}} }}\n",
        //     ch.index_id, ch.index_name
        // );
        // full_string.push_str(update.as_str());
        // let update = index_update_script_json(
        //     "haproxy.http.request.captured_headers".to_string(),
        //     "redacted".to_string(),
        // );
        // full_string.push_str(update.as_str());
        // // //

        // // add overwrite //
        // let update = format!(
        //     "{{ \"update\" : {{\"_id\" : \"{}\", \"_index\" : \"{}\"}} }}\n",
        //     ch.index_id, ch.index_name
        // );
        // full_string.push_str(update.as_str());
        // let update = index_update_script_json(
        //     "haproxy.http.response.captured_headers".to_string(),
        //     "redacted".to_string(),
        // );
        // full_string.push_str(update.as_str());
        // // //

        // // add replicated field //
        // let update = format!(
        //     "{{ \"update\" : {{\"_id\" : \"{}\", \"_index\" : \"{}\"}} }}\n",
        //     ch.index_id, ch.index_name
        // );
        // full_string.push_str(update.as_str());
        // let doc = format!("{{ \"doc\" : {{\"{}\" : \"{}\"}} }}\n", check_field, "true");
        // full_string.push_str(doc.as_str());
        // // //
    }
    full_string.push_str("\n");
    // println!("{full_string}");
    full_string
}

// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct HAProxyIndexUpdateSource {
//     index_name: String,
//     index_id: String,
//     new_field_name: String,
//     new_field_value: String,
// }

// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct HAProxyIndexCreate {
//     index_name: String,
//     index_id: String,
//     haproxy_source: HAProxySource,
// }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HAProxyIndexCreate {
    #[serde(rename = "@timestamp")]
    timestamp: Option<String>,
    index_name: String,
    index_id: String,
    message: String,
    accept_date: String,
    #[serde(rename = "loginId")]
    login_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct HAProxyIndexCreateScript {
    script: String,
}
