use crate::alerts_api_funcs::discord::send_discord;
use crate::repository_api::repo_api_funcs::get_snapshot_repo;
use crate::snapshot_api::snapshot::{SnapshotCreationConfirmation, Snapshots};
use chrono::{Datelike, Days, Local};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::time::Duration;
use sysinfo::Disks;

// pub async fn create_snapshot(
//     elastic_url: &str,
//     elastic_user: &str,
//     elastic_pass: &str,
//     settings_map: HashMap<String, String>,
// ) {
//     let message = "Creating Elastic Snapshot ..";
//     send_discord(&settings_map, "CapnHook", message).await;
//
//     let now = Local::now();
//     let full_url = format!(
//         "{}{}{}{}{}{}{}{}",
//         elastic_url,
//         "/_snapshot/default_snapshot_repo/snapshot_",
//         now.day(),
//         "-",
//         now.month(),
//         "-",
//         now.year(),
//         "?wait_for_completion=false"
//     );
//
//     let data = SnapshotCreationConfirmation {
//         indices: "*".to_string(),
//         ignore_unavailable: true,
//         include_global_state: false,
//         expand_wildcards: "open".to_string(),
//         metadata: SnapShotMetadata {
//             taken_by: "elk-tool".to_string(),
//             taken_because: "backup".to_string(),
//         },
//     };
//
//     // deserialize from json to Vec of ElasticSearch Index obj
//     // let res: Vec<ElasticIndex> = match serde_json::from_str(res.clone().as_str()) {
//     //     Ok(r) => r,
//     //     Err(e) => panic!("{}", e.to_string())
//     // };
//
//     // {"indices":"*","ignore_unavailable":true,"include_global_state":false,"metadata":{"taken_by":"james","taken_because":"test_snapshot"}}
//     let data = serde_json::to_string(&data).unwrap();
//     let _client = reqwest::Client::builder()
//         .danger_accept_invalid_certs(true)
//         .build()
//         .unwrap()
//         .post(full_url)
//         .basic_auth(elastic_user, Some(elastic_pass))
//         .header("Cache-Control", "max-age=0")
//         .header("Accept", "application/json")
//         .header("Accept-Encoding", "gzip, deflate")
//         .json(&data)
//         .send()
//         .await;
//
//     // get indicies
//     // let res = client.unwrap().text().await.unwrap();
//
//     // deserialize from json to Vec of ElasticSearch Index obj
//     // let res: Snapshots = match serde_json::from_str(res.clone().as_str()) {
//     //     Ok(r) => r,
//     //     Err(e) => panic!("{}", e.to_string()),
//     // };
//
//     // println!("{:?}", res);
// }

pub async fn create_yesterday_snapshot(
    elastic_url: &str,
    elastic_user: &str,
    elastic_pass: &str,
    settings_map: HashMap<String, String>,
) {
    let message = "Creating Elastic Snapshot ..";
    send_discord(&settings_map, "CapnHook", message).await;

    let yesterday = Local::now().checked_sub_days(Days::new(1)).unwrap();

    let yesterday_year = yesterday.year();

    #[allow(unused)]
    let mut yesterday_month = String::new();
    if yesterday.month().to_string().len() == 1 {
        yesterday_month = format!("0{}", yesterday.month());
    } else {
        yesterday_month = format!("{}", yesterday.month());
    }

    #[allow(unused)]
    let mut yesterday_day = String::new();
    if yesterday.day().to_string().len() == 1 {
        yesterday_day = format!("0{}", yesterday.day());
    } else {
        yesterday_day = format!("{}", yesterday.day());
    }

    let yesterday_date_string = format!(
        "{}{}{}{}{}",
        yesterday_year, ".", yesterday_month, ".", yesterday_day
    );

    let full_url = format!(
        "{}{}{}{}",
        elastic_url,
        "/_snapshot/default_snapshot_repo/snapshot_",
        yesterday_date_string,
        "?wait_for_completion=false"
    );

    let indices = format!("*-{}", yesterday_date_string);

    let data = SnapshotCreationConfirmation {
        indices,
        ignore_unavailable: true,
        include_global_state: false,
        expand_wildcards: "open".to_string(),
    };

    // deserialize from json to Vec of ElasticSearch Index obj
    // let res: Vec<ElasticIndex> = match serde_json::from_str(res.clone().as_str()) {
    //     Ok(r) => r,
    //     Err(e) => panic!("{}", e.to_string())
    // };

    // {"indices":"*","ignore_unavailable":true,"include_global_state":false,"metadata":{"taken_by":"james","taken_because":"test_snapshot"}}
    let data = serde_json::to_string(&data).unwrap();
    println!("Creating Elastic Snapshot ..");

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
        .post(full_url)
        .basic_auth(elastic_user, Some(elastic_pass))
        .header("Content-Type", "application/json")
        .body(data)
        .timeout(Duration::new(2, 0))
        .send()
        .await;
    if client.is_err() {
        println!("{:?}", client.err())
    }
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
        .timeout(Duration::new(2, 0))
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
        .timeout(Duration::new(2, 0))
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

pub async fn check_disk_space(
    settings_map: HashMap<String, String>,
    _policies_map: HashMap<String, String>,
) -> bool {
    let mut space_low = false;

    // check space - do not backup of less than 100GB free
    let disk_interface = Disks::new_with_refreshed_list();
    println!("{:#?}", disk_interface.list());
    let pat = settings_map.get("snapshot_repo_backup_drive").unwrap();
    for disk in disk_interface.list() {
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
                let message = format!(
                    "Elastic disk low on space: {}",
                    disk.name().to_string_lossy()
                );
                send_discord(&settings_map, "CapnHook", message.as_str()).await;
                space_low = true;
            }
        }
    }

    space_low
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

    if let Some(read_last_snapshot) = settings_map.get("snapshot_last_timestamp") {
        if let Ok(r) = read_last_snapshot.trim().parse::<i128>() {
            last_snapshot = r
        }
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

    let snapshot_timestamp_path = settings_map.get("snapshot_last_timestamp").unwrap();
    let mut snapshot_timestamp_filepath = snapshot_timestamp_path.clone();
    snapshot_timestamp_filepath.push_str("last_snapshot");
    let _ = fs::remove_file(&snapshot_timestamp_filepath);
    if File::create(&snapshot_timestamp_filepath).is_err() {
        let _ = fs::create_dir_all(snapshot_timestamp_path);
        let _ = File::create(&snapshot_timestamp_filepath);
    }

    fs::write(
        snapshot_timestamp_filepath,
        Local::now().timestamp().to_string().as_bytes(),
    )
    .expect("snapshot_last_timestamp path does not exist!!");

    let yesterday = Local::now().checked_sub_days(Days::new(1)).unwrap();

    let yesterday_year = yesterday.year();

    #[allow(unused)]
    let mut yesterday_month = String::new();
    if yesterday.month().to_string().len() == 1 {
        yesterday_month = format!("0{}", yesterday.month());
    } else {
        yesterday_month = format!("{}", yesterday.month());
    }

    #[allow(unused)]
    let mut yesterday_day = String::new();
    if yesterday.day().to_string().len() == 1 {
        yesterday_day = format!("0{}", yesterday.day());
    } else {
        yesterday_day = format!("{}", yesterday.day());
    }

    let date_string = format!(
        "{}{}{}{}{}",
        yesterday_year, ".", yesterday_month, ".", yesterday_day
    );

    // get snapshots
    let snapshots: Snapshots = get_snapshots(elastic_url, elastic_user, elastic_pass).await;
    let mut init_create_snapshot = false;
    if snapshots.snapshots.clone().is_some() // if snapshots exist
        && !snapshots.snapshots.clone().unwrap().is_empty()
    {
        // let mut init_create_snapshot = false;
        // println!("Existing snapshots:");
        for snapshot in snapshots.snapshots.unwrap().iter() {
            // println!("{}", snapshot.snapshot.clone().unwrap());
            if !snapshot
                .snapshot
                .clone()
                .unwrap()
                .contains(date_string.as_str())
            // if snapshot is not from today
            {
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
            } else if snapshot
                .snapshot
                .clone()
                .unwrap()
                .contains(date_string.as_str())
            {
                init_create_snapshot = false;
                let message = "Snapshot for yesterday exists.. skipping Elastic snapshot creation";
                println!("{message}");
                send_discord(&settings_map, "CapnHook", message).await;
            }
        }
    } else {
        // create snapshot for the last threshold period
        if settings_map
            .get("snapshot_backup_enabled")
            .unwrap()
            .contains("true")
        {
            println!("Creating Snapshot 1");
            create_yesterday_snapshot(
                elastic_url,
                elastic_user,
                elastic_pass,
                settings_map.clone(),
            )
            .await;
        }
    }
    if init_create_snapshot {
        create_yesterday_snapshot(
            elastic_url,
            elastic_user,
            elastic_pass,
            settings_map.clone(),
        )
        .await;
    }
}

// pub async fn stop_pending_snapshots(elastic_url: &str, elastic_user: &str, elastic_pass: &str) {
//     let snapshots = get_snapshots(elastic_url, elastic_user, elastic_pass).await;
//
//     for snapshots in snapshots.snapshots.unwrap().iter() {
//         if snapshots.state.clone().unwrap().contains("IN_PROGRESS") {
//             delete_snapshot(
//                 elastic_url,
//                 elastic_user,
//                 elastic_pass,
//                 snapshots.snapshot.clone().unwrap(),
//             )
//             .await;
//         }
//     }
// }

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

// async fn delete_snapshot(
//     elastic_url: &str,
//     elastic_user: &str,
//     elastic_pass: &str,
//     snapshot: String,
// ) {
//     let full_url = format!(
//         "{}{}{}",
//         elastic_url, "/_snapshot/default_snapshot_repo/", snapshot
//     );
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

#[cfg(test)]
mod tests {
    use crate::snapshot_api::snapshot_api_funcs::create_yesterday_snapshot;
    use config::Config;
    use std::collections::HashMap;

    #[test]
    #[ignore]
    fn test_create_snapshot() {
        let settings = Config::builder()
            .add_source(config::File::with_name("config/Settings"))
            .build()
            .unwrap();
        let settings_map = settings
            .try_deserialize::<HashMap<String, String>>()
            .unwrap();

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

        let rt = tokio::runtime::Runtime::new();
        rt.unwrap().block_on(create_yesterday_snapshot(
            elastic_url,
            elastic_user,
            elastic_pass,
            settings_map.clone(),
        ));
    }
}
