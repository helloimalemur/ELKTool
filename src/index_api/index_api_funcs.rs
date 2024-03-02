use crate::alerts_api_funcs::discord::send_discord;
use crate::ilm_api::ilm_api_funcs::set_number_of_replicas_to_zero;
use crate::index_api::elasticindex::ElasticIndex;
use crate::index_api::healthcheck::HealthCheck;
use chrono::{Local, NaiveDate};
use std::collections::HashMap;
use std::thread;
use std::time::{Duration as SysDuration, Duration};
use substring::Substring;

pub async fn close_indexes_over_age_threshold(
    settings_map: HashMap<String, String>,
    policies_map: HashMap<String, String>,
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

    let index_data = get_open_index_data(elastic_url, elastic_user, elastic_pass)
        .await
        .clone();

    for index in index_data.iter() {
        let length = index.to_owned().index.to_owned().unwrap().len();
        // substring to date on name
        let d_string = index.index.clone().unwrap();
        // if first char is not a "."
        if !d_string.substring(0, 1).contains('.') {
            // sub string and parse date
            if d_string.len() > 9 {
                let date_string = d_string.substring(length - 10, length);
                let naive_date = NaiveDate::parse_from_str(date_string, "%Y.%m.%d").unwrap_or_else(|_| NaiveDate::parse_from_str(
                    Local::now().format("%Y.%m.%d").to_string().as_str(),
                    "%Y.%m.%d",
                )
                    .unwrap());
                let days_since = NaiveDate::parse_from_str(
                    Local::now().format("%Y.%m.%d").to_string().as_str(),
                    "%Y.%m.%d",
                )
                .unwrap()
                .signed_duration_since(naive_date)
                .num_days();
                // // compare parsed date with current date to see if we're over lifetime policy threshold
                for policy in policies_map.iter() {
                    // if index name contains policy
                    let policy_ident = policy.0.split('_').enumerate().last().unwrap().1;
                    // if contains policy name
                    if index.index.clone().unwrap().contains(policy_ident) {
                        // if threshold exceeded
                        if policy.0.contains("close_")
                            && days_since as i32 > policy.1.parse::<i32>().unwrap()
                        {
                            println!(
                                "close - {} -- age {} is over thresh {}",
                                index.index.clone().unwrap(),
                                days_since,
                                policy.1
                            );
                            close_index(
                                elastic_url,
                                elastic_user,
                                elastic_pass,
                                index.index.clone().unwrap(),
                            )
                            .await;
                            thread::sleep(SysDuration::new(0, 600));
                        }
                    } else {
                        // println!("{}", index.index.clone().unwrap());
                    }
                }
            }
        }
    }
}

async fn close_index(
    elastic_url: &str,
    elastic_user: &str,
    elastic_pass: &str,
    index_name: String,
) {
    println!("Closing: {}", index_name);

    let full_url = format!("{}{}{}{}", elastic_url, "/", index_name, "/_close");
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

    let res = client.unwrap().text().await;
    println!("Success: {:?}", res);

    // get indicies
    // println!("{}", res);

    // let res = client.unwrap().text().await.unwrap();

    // deserialize from json to Vec of ElasticSearch Index obj
    // let res: Vec<ElasticIndex> = match serde_json::from_str(res.clone().as_str()) {
    //     Ok(r) => r,
    //     Err(e) => panic!("{}", e.to_string())
    // };
}

pub async fn delete_indexes_over_age_threshold(
    settings_map: HashMap<String, String>,
    policies_map: HashMap<String, String>,
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

    let index_data = get_open_index_data(elastic_url, elastic_user, elastic_pass)
        .await
        .clone();

    for index in index_data.iter() {
        let length = index.to_owned().index.to_owned().unwrap().len();
        // substring to date on name
        let d_string = index.index.clone().unwrap();
        // if first char is not a "."
        if !d_string.substring(0, 1).contains('.') && d_string.len() > 9 {
            // sub string and parse date
            let date_string = d_string.substring(length - 10, length);
            let naive_date = match NaiveDate::parse_from_str(date_string, "%Y.%m.%d") {
                Ok(r) => r,
                Err(_) => NaiveDate::parse_from_str(
                    Local::now().format("%Y.%m.%d").to_string().as_str(),
                    "%Y.%m.%d",
                )
                .unwrap(),
            };
            let days_since = NaiveDate::parse_from_str(
                Local::now().format("%Y.%m.%d").to_string().as_str(),
                "%Y.%m.%d",
            )
            .unwrap()
            .signed_duration_since(naive_date)
            .num_days();
            // // compare parsed date with current date to see if we're over lifetime policy threshold
            for policy in policies_map.iter() {
                // if index name contains policy
                let policy_ident = policy.0.split('_').enumerate().last().unwrap().1;
                // if contains policy name
                if index.index.clone().unwrap().contains(policy_ident)
                    && days_since as i32 > policy.1.parse::<i32>().unwrap()
                {
                    // if threshold exceeded
                    if policy.0.contains("delete_") {
                        println!(
                            "delete - {} -- age {} is over thresh {}",
                            index.index.clone().unwrap(),
                            days_since,
                            policy.1
                        );
                        delete_index(
                            elastic_url,
                            elastic_user,
                            elastic_pass,
                            index.index.clone().unwrap(),
                        )
                        .await;
                        thread::sleep(SysDuration::new(0, 600));
                    }
                } else {
                    // println!("{}", index.index.clone().unwrap());
                }
            }
        }
    }
}

async fn delete_index(
    elastic_url: &str,
    elastic_user: &str,
    elastic_pass: &str,
    index_name: String,
) {
    println!("Deleting: {}", index_name);

    let full_url = format!("{}{}{}", elastic_url, "/", index_name);

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

    let res = client.unwrap().text().await;
    println!("Success: {:?}", res);

    // get indicies
    // println!("{}", res);

    // let res = client.unwrap().text().await.unwrap();

    // deserialize from json to Vec of ElasticSearch Index obj
    // let res: Vec<ElasticIndex> = match serde_json::from_str(res.clone().as_str()) {
    //     Ok(r) => r,
    //     Err(e) => panic!("{}", e.to_string())
    // };
}

pub async fn get_open_index_data(
    elastic_url: &str,
    elastic_user: &str,
    elastic_pass: &str,
) -> Vec<ElasticIndex> {
    // curl -i -s -k -X $'GET' -H $'Host: stats.konnektive.com:9200' -H $'Cache-Control: max-age=0' -H $'Authorization: Basic ZWxhc3RpYzpLVUxnZEVEcmE9dlBRMUhlRW8reA==' -H $'Accept: text/html' -H $'Accept-Encoding: gzip, deflate' $'https://stats.konnektive.com:9200/_cat/indices?v'
    let full_url = format!("{}{}", elastic_url, "/_cat/indices?v");
    // let full_url = format!("{}{}", elastic_url, "?h=health,status,index,id,pri,rep,docs.count,docs.deleted,store.size,creation.date.string");

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
    let res = client.unwrap().text().await.unwrap();
    // deserialize from json to Vec of ElasticSearch Index obj
    let res: Vec<ElasticIndex> = match serde_json::from_str(res.clone().as_str()) {
        Ok(r) => r,
        Err(e) => panic!("{}", e.to_string()),
    };

    // println!("{:?}", res);

    // print indicies
    // for (x,i) in res.clone().iter().enumerate() {
    //     if i.index.clone().unwrap().contains("2023.03") {
    //         println!("{}", i.index.clone().unwrap().as_str());
    //     }
    // }

    res
}

pub async fn cluster_health_check(settings_map: HashMap<String, String>) -> String {
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
    let discord_webhook_url = settings_map
        .get("discord_webhook_url")
        .expect("COULD NOT GET discord_webhook_url")
        .as_str();

    set_number_of_replicas_to_zero(elastic_url, elastic_user, elastic_pass).await;

    let full_url = format!("{}{}", elastic_url, "/_cluster/health");
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
        .get(full_url.clone())
        .basic_auth(elastic_user, Some(elastic_pass))
        .header("Cache-Control", "max-age=0")
        .header("Accept", "text/html")
        .header("Accept-Encoding", "gzip, deflate")
        .send()
        .await;

    let res = client.unwrap().text().await.unwrap();

    // deserialize from json to Vec of ElasticSearch Index obj
    let res: HealthCheck = match serde_json::from_str(res.clone().as_str()) {
        Ok(r) => r,
        Err(e) => panic!("{}", e.to_string()),
    };
    let cluster_status = res.clone().status.unwrap();
    println!("Cluster Status is: [{}]", cluster_status);

    let message = format!("Status is {}", res.clone().status.unwrap());
    send_discord(&settings_map, "CapnHook", message.as_str()).await;

    // if cluster status is not green, determine issue and remediate
    if cluster_status != "green" {
        println!("Remediating health issues..");

        // check for and close unassigned shards
        let unassigned_shards = res.unassigned_shards.unwrap();
        if unassigned_shards > 0 {
            println!("{} Unassigned shards found", unassigned_shards);
            set_number_of_replicas_to_zero(elastic_url, elastic_user, elastic_pass).await;
        }

        // check again
        thread::sleep(Duration::new(7, 0));
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
            .get(full_url)
            .basic_auth(elastic_user, Some(elastic_pass))
            .header("Cache-Control", "max-age=0")
            .header("Accept", "text/html")
            .header("Accept-Encoding", "gzip, deflate")
            .send()
            .await;

        let res = client.unwrap().text().await.unwrap();

        // deserialize from json to Vec of ElasticSearch Index obj
        let res: HealthCheck = match serde_json::from_str(res.clone().as_str()) {
            Ok(r) => r,
            Err(e) => panic!("{}", e.to_string()),
        };
        let cluster_status = res.clone().status.unwrap();
        println!("Cluster Status is: [{}]", cluster_status);
    } else {
        println!("Health check OK .. skipping issue remediation steps");
    }
    println!("Cluster Status is: [{}]", cluster_status);
    cluster_status
}

pub async fn cluster_disk_alloc_check(settings_map: HashMap<String, String>) -> bool {
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
    let discord_webhook_url = settings_map
        .get("discord_webhook_url")
        .expect("COULD NOT GET discord_webhook_url")
        .as_str();

    // curl -i -s -k -X $'GET' -H $'Host: stats.konnektive.com:9200' -H $'Cache-Control: max-age=0' -H $'Authorization: Basic ZWxhc3RpYzpPVkIzN3dzUD1EUXlDU3J4U1hGVA==' -H $'Accept: text/html' -H $'Accept-Encoding: gzip, deflate' $'https://stats.konnektive.com:9200/_cat/allocation?v&pretty'

    let full_url = format!("{}{}", elastic_url, "/_cat/allocation?v&pretty");
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
        .get(full_url.clone())
        .basic_auth(elastic_user, Some(elastic_pass))
        .header("Cache-Control", "max-age=0")
        .header("Accept", "text/html")
        .header("Accept-Encoding", "gzip, deflate")
        .send()
        .await;

    let res = client.unwrap().text().await.unwrap();

    let space_used = res.lines().nth(1).unwrap().split_whitespace().nth(2).unwrap();
    let space_available = res.lines().nth(1).unwrap().split_whitespace().nth(3).unwrap();
    let total_drive_size = res.lines().nth(1).unwrap().split_whitespace().nth(4).unwrap();
    let percentage_used = res.lines().nth(1).unwrap().split_whitespace().nth(5).unwrap();

    let message = format!("Disk Percentage Used:\n[{}]\nDisk space available:\n[{}]\nDisk space used:\n[{}]\nDisk total space:\n[{}]\n", percentage_used, space_available, space_used, total_drive_size);
    println!("{}", message.clone());
    send_discord(&settings_map, "CapnHook", message.as_str()).await;

    if percentage_used.parse::<i32>().unwrap() > 70 {
        let message = "Disk Percentage Used too high!!!";
        println!("{}", message);
        send_discord(&settings_map, "CapnHook", message).await;
        false
    } else {
        true
    }
}
