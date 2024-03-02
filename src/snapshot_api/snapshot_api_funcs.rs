use crate::alerts_api_funcs::discord::send_discord;
use crate::snapshot_api::snapshot::{SnapShotMetadata, SnapshotCreationConfirmation, Snapshots};
use chrono::{Datelike, Local};
use std::collections::HashMap;
use sysinfo::{DiskExt, System, SystemExt};

pub async fn create_snapshot(elastic_url: &str, elastic_user: &str, elastic_pass: &str) {
    // curl -X PUT "https://stats.konnektive.com:9200/_snapshot/default_snapshot_repo/snapshot_2?wait_for_completion=true&pretty" -H $'Authorization: Basic ZWxhc3RpYzpPVkIzN3dzUD1EUXlDU3J4U1hGVA==' -H 'Content-Type: application/json' -d'{"indices":"*","ignore_unavailable":true,"include_global_state":false,"metadata":{"taken_by":"james","taken_because":"test_snapshot"}}'

    let now = Local::now();
    let full_url = format!(
        "{}{}{}{}{}{}{}{}",
        elastic_url,
        "/_snapshot/default_snapshot_repo/snapshot_",
        now.day(),
        "-",
        now.month(),
        "-",
        now.year(),
        "?wait_for_completion=false"
    );

    let data = SnapshotCreationConfirmation {
        indices: "*".to_string(),
        ignore_unavailable: true,
        include_global_state: false,
        metadata: SnapShotMetadata {
            taken_by: "elk-tool".to_string(),
            taken_because: "backup".to_string(),
        },
    };

    // deserialize from json to Vec of ElasticSearch Index obj
    // let res: Vec<ElasticIndex> = match serde_json::from_str(res.clone().as_str()) {
    //     Ok(r) => r,
    //     Err(e) => panic!("{}", e.to_string())
    // };

    // {"indices":"*","ignore_unavailable":true,"include_global_state":false,"metadata":{"taken_by":"james","taken_because":"test_snapshot"}}
    let data = serde_json::to_string(&data).unwrap();
    let _client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
        .post(full_url)
        .basic_auth(elastic_user, Some(elastic_pass))
        .header("Cache-Control", "max-age=0")
        .header("Accept", "application/json")
        .header("Accept-Encoding", "gzip, deflate")
        .json(&data)
        .send()
        .await;

    // get indicies
    // let res = client.unwrap().text().await.unwrap();

    // deserialize from json to Vec of ElasticSearch Index obj
    // let res: Snapshots = match serde_json::from_str(res.clone().as_str()) {
    //     Ok(r) => r,
    //     Err(e) => panic!("{}", e.to_string()),
    // };

    // println!("{:?}", res);
}

pub async fn get_snapshots(elastic_url: &str, elastic_user: &str, elastic_pass: &str) -> Snapshots {
    // curl -X GET "https://stats.konnektive.com:9200/_snapshot/default_snapshot_repo/*?pretty" -H $'Authorization: Basic ZWxhc3RpYzpPVkIzN3dzUD1EUXlDU3J4U1hGVA=='

    let full_url = format!("{}{}", elastic_url, "/_snapshot/default_snapshot_repo/*");

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

    let mut resa = Snapshots {
        snapshots: None,
        total: None,
        remaining: None,
    };

    if client.is_ok() {
        let res = client.unwrap().text().await.unwrap();

        // deserialize from json to Vec of ElasticSearch Index obj
        resa = match serde_json::from_str(res.clone().as_str()) {
            Ok(r) => r,
            Err(e) => panic!("{}", e.to_string()),
        };
    }
    // get indicies

    // println!("{:?}", res);

    resa.clone()
}

#[allow(unused)]
pub async fn get_available_space_on_drive(elastic_url: &str, elastic_user: &str, elastic_pass: &str) -> Snapshots {
    // curl -X GET "https://stats.konnektive.com:9200/_snapshot/default_snapshot_repo/*?pretty" -H $'Authorization: Basic ZWxhc3RpYzpPVkIzN3dzUD1EUXlDU3J4U1hGVA=='

    let full_url = format!("{}{}", elastic_url, "/_snapshot/default_snapshot_repo/*");

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

    let mut resa = Snapshots {
        snapshots: None,
        total: None,
        remaining: None,
    };

    if client.is_ok() {
        let res = client.unwrap().text().await.unwrap();

        // deserialize from json to Vec of ElasticSearch Index obj
        resa = match serde_json::from_str(res.clone().as_str()) {
            Ok(r) => r,
            Err(e) => panic!("{}", e.to_string()),
        };
    }
    // get indicies

    // println!("{:?}", res);

    resa.clone()
}


pub async fn check_for_running_backup_check_space_cancel_backup_if_low(
    settings_map: HashMap<String, String>,
    _policies_map: HashMap<String, String>,
) -> bool {
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

    let res = get_snapshots(elastic_url, elastic_user, elastic_pass).await;

    // testing
    // stop_pending_snapshots(elastic_url, elastic_user, elastic_pass).await;

    let mut backup_running_or_space_low = false;

    for snapshot in res.snapshots.expect("CHECK ELASTIC CREDENTIALS").iter() {
        let _now = Local::now();
        // let date_string = format!("{}{}{}{}{}", now.day(), "-", now.month(), "-", now.year());
        let check_in_prog = "IN_PROGRESS".to_string();

        // if snapshot.snapshot.clone().unwrap().contains(&date_string) {
        if snapshot.state.clone().unwrap().contains(&check_in_prog) {
            println!("Backup is already in progress!!");
            backup_running_or_space_low = true;
        }
    }

    // check space - do not backup of less than 100GB free
    let mut sys = System::new_all();
    sys.refresh_all();
    println!("{:#?}", sys.disks());
    let pat = settings_map.get("snapshot_repo_backup_drive").unwrap();
    for disk in sys.disks() {
        if disk.name().to_string_lossy().contains(pat.as_str()) && disk.available_space() > 0 {
            println!(
                "{} - FREE: {}MB",
                disk.name().to_string_lossy(),
                disk.available_space() / 1000000
            );
            if disk.available_space() / 1000000
                < settings_map
                    .get("snapshot_min_free_space")
                    .unwrap()
                    .parse()
                    .unwrap()
            {
                println!("Backup called off. Drive too low on space.");
                let message = format!(
                    "Elastic Snapshot backed off, disk low on space: {}",
                    disk.name().to_string_lossy()
                );
                send_discord(&settings_map,
                    
                    "CapnHook",
                    message.as_str(),
                )
                .await;
                backup_running_or_space_low = true;
                stop_pending_snapshots(elastic_url, elastic_user, elastic_pass).await;
            }
        }
    }
    // if !backup_running_or_space_low {
    // }

    backup_running_or_space_low
}

pub async fn stop_pending_snapshots(elastic_url: &str, elastic_user: &str, elastic_pass: &str) {
    let snapshots = get_snapshots(elastic_url, elastic_user, elastic_pass).await;

    for snapshots in snapshots.snapshots.unwrap().iter() {
        if snapshots.state.clone().unwrap().contains("IN_PROGRESS") {
            delete_snapshot(
                elastic_url,
                elastic_user,
                elastic_pass,
                snapshots.snapshot.clone().unwrap(),
            )
            .await;
        }
    }
}

async fn delete_snapshot(
    elastic_url: &str,
    elastic_user: &str,
    elastic_pass: &str,
    snapshot: String,
) {
    let full_url = format!(
        "{}{}{}",
        elastic_url, "/_snapshot/default_snapshot_repo/", snapshot
    );

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

    let res = client.unwrap().text().await.unwrap();

    println!("{:?}", res);
}

pub async fn delete_all_snapshots(elastic_url: &str, elastic_user: &str, elastic_pass: &str) {
    let full_url = format!("{}{}", elastic_url, "/_snapshot/default_snapshot_repo/*");

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

    let res = client.unwrap().text().await.unwrap();

    println!("{:?}", res);
}
