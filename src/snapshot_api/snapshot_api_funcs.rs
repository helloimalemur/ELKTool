use crate::alerts_api_funcs::discord::send_discord;
use crate::repository_api::repo_api_funcs::get_snapshot_repo;
use crate::snapshot_api::snapshot::{SnapShotMetadata, SnapshotCreationConfirmation, Snapshots};
use chrono::{Datelike, Days, Local, NaiveDate};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use sysinfo::{DiskExt, System, SystemExt};

pub async fn create_snapshot(
    elastic_url: &str,
    elastic_user: &str,
    elastic_pass: &str,
    settings_map: HashMap<String, String>,
) {
    let message = "Creating Elastic Snapshot ..";
    send_discord(&settings_map, "CapnHook", message).await;

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
pub async fn get_available_space_on_drive(
    elastic_url: &str,
    elastic_user: &str,
    elastic_pass: &str,
) -> Snapshots {
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

pub async fn check_for_running_snapshot_check_space(
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

    let mut snapshot_running_or_space_low = false;

    for snapshot in res.snapshots.expect("CHECK ELASTIC CREDENTIALS").iter() {
        let _now = Local::now();
        // let date_string = format!("{}{}{}{}{}", now.day(), "-", now.month(), "-", now.year());
        let check_in_prog = "IN_PROGRESS".to_string();

        // if snapshot.snapshot.clone().unwrap().contains(&date_string) {
        if snapshot.state.clone().unwrap().contains(&check_in_prog) {
            println!("Snapshot creation is already in progress!!");
            snapshot_running_or_space_low = true;
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
                send_discord(&settings_map, "CapnHook", message.as_str()).await;
                snapshot_running_or_space_low = true;
                stop_pending_snapshots(elastic_url, elastic_user, elastic_pass).await;
            }
        }
    }

    snapshot_running_or_space_low
}

pub async fn check_threshold_and_create_snapshot(
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

    // verify a threshold has passed
    // let threshold_in_seconds = settings_map
    //     .get("snapshot_inverval_days")
    //     .unwrap()
    //     .parse::<i128>()
    //     .unwrap()
    //     * 86400;
    // let threshold_in_seconds = 3; // testing

    let now = Local::now().timestamp() as i128;
    let mut last_snapshot: i128 = 0;
    let read_last_snapshot =
        fs::read_to_string(settings_map.get("snapshot_last_timestamp").unwrap());
    if read_last_snapshot.is_ok() {
        last_snapshot = read_last_snapshot.unwrap().trim().parse::<i128>().unwrap();
    }
    let diff = now - last_snapshot;

    let repo = get_snapshot_repo(elastic_url, elastic_user, elastic_pass).await;
    if repo.default_snapshot_repo.is_some() {
        if diff > 86400 {
            // one day epoch
            prepare_snapshot(settings_map.clone()).await;
        }
    } else {
        println!("NO REPO FOUND -- SKIPPING SNAPSHOT CREATION");
    }
}

async fn prepare_snapshot(settings_map: HashMap<String, String>) {
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

    // verify a threshold has passed
    // let threshold_in_seconds = settings_map
    //     .get("snapshot_inverval_days")
    //     .unwrap()
    //     .parse::<i128>()
    //     .unwrap()
    //     * 86400;
    // let threshold_in_seconds = 3; // testing
    // let now = Local::now().timestamp() as i128;
    // let mut last_snapshot: i128 = 0;
    // let read_last_snapshot =
    //     fs::read_to_string(settings_map.get("snapshot_last_timestamp").unwrap());
    // if read_last_snapshot.is_ok() {
    //     last_snapshot = read_last_snapshot.unwrap().trim().parse::<i128>().unwrap();
    // }
    // let diff = now - last_snapshot;

    // println!("THRESHOLD PASSED");
    // println!("CURRENT: {}", now);
    // println!("LAST: {}", last_snapshot);
    // println!("THRESHOLD: {}", threshold_in_seconds);
    // println!("DIFF: {}", diff);
    // create / update last_snapshot timestamp file
    let snapshot_timestamp_path = settings_map.get("snapshot_last_timestamp").unwrap();
    let mut snapshot_timestamp_filepath = snapshot_timestamp_path.clone();
    snapshot_timestamp_filepath.push_str("last_snapshot");
    let _ = fs::remove_file(&snapshot_timestamp_filepath);
    if let Err(_) = File::create(&snapshot_timestamp_filepath) {
        let _ = fs::create_dir_all(&snapshot_timestamp_path);
        let _ = File::create(&snapshot_timestamp_filepath);
    }

    fs::write(
        snapshot_timestamp_filepath,
        Local::now().timestamp().to_string().as_bytes(),
    )
    .expect("snapshot_last_timestamp path does not exist!!");

    let now = Local::now();
    let _yesterday =
        NaiveDate::checked_sub_days(NaiveDate::from(now.naive_local()), Days::new(1)).unwrap();
    let date_string = format!("{}{}{}{}{}", now.day(), "-", now.month(), "-", now.year());
    // let yesterday_date_string = format!("{}{}{}{}{}", yesterday.day(), "-", yesterday.month(), "-", yesterday.year());
    // get snapshots
    let snapshots: Snapshots = get_snapshots(elastic_url, elastic_user, elastic_pass).await;
    let mut init_create_snapshot = false;
    if snapshots.snapshots.clone().is_some() // if snapshots exist
        && !snapshots.snapshots.clone().unwrap().is_empty()
    {
        // let mut init_create_snapshot = false;
        println!("Existing snapshots:");
        for snapshot in snapshots.snapshots.unwrap().iter() {
            println!("{}", snapshot.snapshot.clone().unwrap());
            if !snapshot
                .snapshot
                .clone()
                .unwrap()
                .contains(date_string.as_str())
            // if snapshot is not from today
            {
                // if snapshot does not exist already

                // create snapshot for the last threshold period
                if settings_map
                    .get("snapshot_backup_enabled")
                    .unwrap()
                    .contains("true")
                {
                    init_create_snapshot = true;
                }
            } else if snapshot
                .clone()
                .state
                .unwrap()
                .to_lowercase()
                .contains("partial")
            {
                init_create_snapshot = false;
                let message = "PARTIAL BACKUP EXISTS ...";
                send_discord(&settings_map, "CapnHook", message).await;
            } else {
                init_create_snapshot = false;
                let message = "Snapshot for today exists.. skipping Elastic snapshot creation";
                println!("{message}");
                send_discord(&settings_map, "CapnHook", message).await;
            }

            if init_create_snapshot {
                create_snapshot(
                    elastic_url,
                    elastic_user,
                    elastic_pass,
                    settings_map.clone(),
                )
                .await;
            }
        }
    }
    // else {
    //     // create snapshot for the last threshold period
    //     if settings_map
    //         .get("snapshot_backup_enabled")
    //         .unwrap()
    //         .contains("true")
    //     {
    //         println!("Creating Snapshot 1");
    //         create_snapshot(
    //             elastic_url,
    //             elastic_user,
    //             elastic_pass,
    //             settings_map.clone(),
    //         )
    //         .await;
    //     }
    // }
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

// pub async fn remove_partial_snapshots(elastic_url: &str, elastic_user: &str, elastic_pass: &str) {
//     let snapshots = get_snapshots(elastic_url, elastic_user, elastic_pass).await;
//
//     for snapshots in snapshots.snapshots.unwrap().iter() {
//         if snapshots.state.clone().unwrap().contains("PARTIAL") {
//             delete_snapshot(
//                 elastic_url,
//                 elastic_user,
//                 elastic_pass,
//                 snapshots.snapshot.clone().unwrap(),
//             )
//                 .await;
//         }
//     }
// }

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

// pub async fn delete_all_snapshots(elastic_url: &str, elastic_user: &str, elastic_pass: &str) {
//     let full_url = format!("{}{}", elastic_url, "/_snapshot/default_snapshot_repo/*");
//
//     let client = reqwest::Client::builder()
//         .danger_accept_invalid_certs(true)
//         .build()
//         .unwrap()
//         .delete(full_url)
//         .basic_auth(elastic_user, Some(elastic_pass))
//         .header("Cache-Control", "max-age=0")
//         .header("Accept", "application/json")
//         .header("Accept-Encoding", "gzip, deflate")
//         .send()
//         .await;
//
//     let res = client.unwrap().text().await.unwrap();
//
//     println!("{:?}", res);
// }
