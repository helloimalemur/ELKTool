use crate::alerts_api_funcs::discord::send_discord;
use crate::repository_api::repository::Repository;
use crate::snapshot_api::snapshot::Snapshots;
use crate::snapshot_api::snapshot_api_funcs::{
    create_snapshot, delete_all_snapshots, get_snapshots,
};
use chrono::{Datelike, Local};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::process::{Command, Stdio};

pub struct SSHAuthConfig {
    ssh_user: String,
    ssh_port: String,
    ssh_host: String,
    ssh_key: String,
    ssh_src_dir: String,
    ssh_dest_dir: String,
    backup_server_archive_dest_dir: String,
    settings_map: HashMap<String, String>,
}

pub async fn check_threshold_passed_create_snapshot(
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
    let threshold_in_seconds = settings_map
        .get("snapshot_inverval_days")
        .unwrap()
        .parse::<i128>()
        .unwrap()
        * 86400;
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
        if diff > threshold_in_seconds {
            println!("THRESHOLD PASSED");
            println!("CURRENT: {}", now);
            println!("LAST: {}", last_snapshot);
            println!("THRESHOLD: {}", threshold_in_seconds);
            println!("DIFF: {}", diff);
            // create / update last_snapshot timestamp file
            let _ = fs::remove_file(settings_map.get("snapshot_last_timestamp").unwrap());
            let _ = File::create(settings_map.get("snapshot_last_timestamp").unwrap());
            fs::write(
                settings_map.get("snapshot_last_timestamp").unwrap(),
                Local::now().timestamp().to_string().as_bytes(),
            )
            .unwrap();

            let now = Local::now();
            let date_string = format!("{}{}{}{}{}", now.day(), "-", now.month(), "-", now.year());

            // get snapshots
            let snapshots: Snapshots = get_snapshots(elastic_url, elastic_user, elastic_pass).await;

            // list snapshots by name
            if snapshots.snapshots.clone().is_some()
                && !snapshots.snapshots.clone().unwrap().is_empty()
            {
                println!("Existing snapshots:");
                for snapshot in snapshots.snapshots.unwrap().iter() {
                    println!("{}", snapshot.snapshot.clone().unwrap());
                    if !snapshot
                        .snapshot
                        .clone()
                        .unwrap()
                        .contains(date_string.as_str())
                    {
                        // if snapshot does not exist already
                        println!("Creating Snapshot");
                        // create snapshot for the last threshold period
                        if settings_map
                            .get("snapshot_backup_enabled")
                            .unwrap()
                            .contains("true")
                        {
                            let message = "Creating Elastic Snapshot ..";
                            send_discord(&settings_map,
                                "CapnHook",
                                message,
                            )
                            .await;
                            create_snapshot(elastic_url, elastic_user, elastic_pass).await;
                        }
                    } else if snapshot
                        .clone()
                        .state
                        .unwrap()
                        .to_lowercase()
                        .contains("partial")
                    {
                        let message = "PARTIAL BACKUP EXISTS .. DELETING AND TRYING AGAIN ..";
                        send_discord(&settings_map,
                            
                            "CapnHook",
                            message,
                        )
                        .await;
                        delete_all_snapshots(elastic_url, elastic_user, elastic_pass).await;
                        create_snapshot(elastic_url, elastic_user, elastic_pass).await;
                    } else {
                        let message =
                            "Snapshot for today exists.. skipping Elastic snapshot creation";
                        println!("{message}");
                        send_discord(&settings_map,
                            
                            "CapnHook",
                            message,
                        )
                        .await;
                    }
                }
            } else {
                println!("Creating Snapshot");
                // create snapshot for the last threshold period
                if settings_map
                    .get("snapshot_backup_enabled")
                    .unwrap()
                    .contains("true")
                {
                    let message = "Creating Elastic Snapshot backup ..".to_string();
                    send_discord(&settings_map,
                        
                        "CapnHook",
                        message.as_str(),
                    )
                    .await;
                    create_snapshot(elastic_url, elastic_user, elastic_pass).await;
                }
            }
        }
    } else {
        println!("NO REPO FOUND -- SKIPPING SNAPSHOT CREATION");
    }
}

pub async fn get_snapshot_repo(
    elastic_url: &str,
    elastic_user: &str,
    elastic_pass: &str,
) -> Repository {
    // curl -X GET "https://stats.konnektive.com:9200/_snapshot/default_snapshot_repo/" -H $'Authorization: Basic ZWxhc3RpYzpPVkIzN3dzUD1EUXlDU3J4U1hGVA=='

    let full_url = format!("{}{}", elastic_url, "/_snapshot/default_snapshot_repo/");

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
    let res: Repository = match serde_json::from_str(res.clone().as_str()) {
        Ok(r) => r,
        Err(e) => panic!("{}", e.to_string()),
    };

    // println!("{:?}", res);

    res
}

pub async fn copy_backup_if_not_running_snapshot(
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
    let ssh_host = settings_map
        .get("backup_server_host")
        .expect("COULD NOT GET backup_server_host")
        .as_str();
    let ssh_key_path = settings_map
        .get("backup_server_ssh_key")
        .expect("COULD NOT GET backup_server_ssh_key")
        .as_str();
    let ssh_src_dir = settings_map
        .get("backup_server_src_dir")
        .expect("COULD NOT GET backup_server_dir")
        .as_str();
    let ssh_dest_dir = settings_map
        .get("backup_server_dest_dir")
        .expect("COULD NOT GET backup_server_dir")
        .as_str();
    let _backup_server_archive_dest_dir = settings_map
        .get("backup_server_archive_dest_dir")
        .expect("COULD NOT GET backup_server_archive_dest_dir")
        .as_str();
    let ssh_port = settings_map
        .get("backup_server_ssh_port")
        .expect("COULD NOT GET backup_server_dir")
        .as_str();
    let ssh_user = settings_map
        .get("backup_server_ssh_user")
        .expect("COULD NOT GET backup_server_dir")
        .as_str();

    // check if snapshot is running
    let mut snapshot_in_prog = false;
    let snapshots = get_snapshots(elastic_url, elastic_user, elastic_pass).await;
    for snapshots in snapshots.snapshots.unwrap().iter() {
        if snapshots.state.clone().unwrap().contains("IN_PROGRESS") {
            snapshot_in_prog = true;
        }
    }

    // verify backup threshold has passed
    let threshold_in_seconds = settings_map
        .get("backup_inverval_days")
        .unwrap()
        .parse::<i128>()
        .unwrap()
        * 86400;
    // let threshold_in_seconds = 3; // testing

    let now = Local::now().timestamp() as i128;
    let mut last_backup: i128 = 0;
    let read_last_snapshot = fs::read_to_string(settings_map.get("backup_last_timestamp").unwrap());
    if read_last_snapshot.is_ok() {
        last_backup = read_last_snapshot.unwrap().trim().parse::<i128>().unwrap();
    }
    let diff = now - last_backup;

    let enabled = settings_map.get("backups_enabled").unwrap();

    let mut success = false;
    let _is_success = success;

    if enabled.contains("true") && diff > threshold_in_seconds && !snapshot_in_prog {
        // ?
        // ?
        // ?
        // ?
        // VERIFY SUCCESSFUL SNAPSHOT PRIOR TO SCP
        let snapshots = get_snapshots(elastic_url, elastic_user, elastic_pass).await;
        for snapshots in snapshots.snapshots.unwrap().iter() {
            if snapshots.state.clone().unwrap().contains("SUCCESS") {
                println!("copying snapshot to backup server..");
                println!("thresh {}", threshold_in_seconds);
                println!("diff {}", diff);

                let ssh_config = SSHAuthConfig {
                    ssh_user: String::from(ssh_user),
                    ssh_port: String::from(ssh_port),
                    ssh_host: String::from(ssh_host),
                    ssh_key: "".to_string(),
                    ssh_src_dir: String::from(ssh_src_dir),
                    ssh_dest_dir: String::from(ssh_dest_dir),
                    backup_server_archive_dest_dir: "".to_string(),
                    settings_map: settings_map.clone(),
                };
                success = copy_backup(ssh_config).await;
                if success {
                    println!("Copy backup successful ..");

                    // create / update last_snapshot timestamp file
                    let _ = fs::remove_file(settings_map.get("backup_last_timestamp").unwrap());
                    let _ = File::create(settings_map.get("backup_last_timestamp").unwrap());
                    fs::write(
                        settings_map.get("backup_last_timestamp").unwrap(),
                        Local::now().timestamp().to_string().as_bytes(),
                    )
                    .unwrap();

                    println!("Cleaning up backed up snapshots and pre-compression dir");
                    let _ = delete_all_snapshots(elastic_url, elastic_user, elastic_pass).await;
                    delete_src_dir_contents(
                        ssh_user,
                        ssh_port,
                        ssh_host,
                        ssh_key_path,
                        ssh_src_dir,
                        ssh_dest_dir,
                        settings_map.clone(),
                    )
                    .await;
                }
            } else {
                println!("No snapshots to copy..");
            }
        }
        // ?
        // ?
        // ?
        // ?

        // OR

        // ?
        // ?
        // ?
        // ?
        // copy_backup(ssh_user, ssh_port, ssh_host, ssh_key_path, ssh_src_dir, ssh_dest_dir, settings_map.clone()).await;
        // // create / update last_snapshot timestamp file
        // let _ = fs::remove_file(settings_map.get("backup_last_timestamp").unwrap());
        // let _ = File::create(settings_map.get("backup_last_timestamp").unwrap());
        // let _ = fs::write(
        //     settings_map.get("backup_last_timestamp").unwrap(),
        //     Local::now().timestamp().to_string().as_bytes(),
        // )
        //     .unwrap();
        // if success {
        //     // create / update last_snapshot timestamp file
        //     let _ =
        //         fs::remove_file(settings_map.get("backup_last_timestamp").unwrap());
        //     let _ =
        //         File::create(settings_map.get("backup_last_timestamp").unwrap());
        //     let _ = fs::write(
        //         settings_map.get("backup_last_timestamp").unwrap(),
        //         Local::now().timestamp().to_string().as_bytes(),
        //     )
        //         .unwrap();
        //
        //     // Delete snapshots post backup?
        //     delete_all_snapshots(elastic_url, elastic_user, elastic_pass).await;
        // }
        // ?
        // ?
        // ?
        // ?
    }
}

pub async fn copy_backup(ssh_config: SSHAuthConfig) -> bool {
    let ssh_user: &str = ssh_config.ssh_user.as_str();
    let ssh_port: &str = ssh_config.ssh_port.as_str();
    let ssh_host: &str = ssh_config.ssh_host.as_str();
    let _ssh_key: &str = ssh_config.ssh_key.as_str();
    let ssh_src_dir: &str = ssh_config.ssh_src_dir.as_str();
    let ssh_dest_dir: &str = ssh_config.ssh_dest_dir.as_str();
    let backup_server_archive_dest_dir: &str = ssh_config.backup_server_archive_dest_dir.as_str();
    let settings_map: HashMap<String, String> = ssh_config.settings_map.clone();

    let ssh_from_host = settings_map
        .get("ssh_from_host")
        .expect("COULD NOT GET ssh_from_host")
        .as_str();

    let copy_from = format!("{}@{}:{}", ssh_user, ssh_from_host, ssh_src_dir);
    let copy_to = format!("{}@{}:{}", ssh_user, ssh_host, ssh_dest_dir);

    println!("{} {}", copy_from, copy_to);

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

    let archive_dir = settings_map
        .get("backup_server_archive_dest_dir")
        .expect("COULD NOT GET backup_server_archive_dest_dir")
        .as_str();

    let remote_copy = settings_map
        .get("remote_copy_enabled")
        .expect("COULD NOT GET remote_copy_enabled")
        .as_str();

    let mut copied: bool = false;
    println!("Successfully copied: {}", copied);
    let remote: bool = remote_copy.trim().contains("true");

    if remote {
        let message = format!("SCPing snapshot to backup server {}", ssh_dest_dir);
        send_discord(&settings_map,
            
            "CapnHook",
            message.as_str(),
        )
        .await;

        let copy_status = Command::new("scp")
            .arg("-r")
            .arg("-P")
            .arg(ssh_port)
            .arg(copy_from.clone())
            .arg(copy_to.clone())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .expect("failed to run scp");

        send_discord(&settings_map,
            
            "CapnHook",
            copy_status.to_string().as_str(),
        )
        .await;
        copied = copy_status.success();
    } else {
        let message = format!("Local rsync snapshot to backup dir {}", ssh_dest_dir);
        send_discord(&settings_map,
            
            "CapnHook",
            message.as_str(),
        )
        .await;

        let copy_status = Command::new("rsync")
            .arg("-r")
            .arg(ssh_src_dir)
            .arg(ssh_dest_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .expect("failed to run rsync");

        send_discord(&settings_map,
            
            "CapnHook",
            copy_status.to_string().as_str(),
        )
        .await;
        copied = copy_status.success();
    }

    let message = format!("backup copy success: {}", copied);
    send_discord(&settings_map,
        
        "CapnHook",
        message.as_str(),
    )
    .await;

    // Delete snapshots post backup?
    if copied {
        delete_all_snapshots(elastic_url, elastic_user, elastic_pass).await;
        println!("Snapshots removed..");
    }

    let mut compressed = false;
    if copied {
        if remote {
            println!("Compressing backup..");
            let message = "compressing the backup..".to_string();
            send_discord(&settings_map,
                
                "CapnHook",
                message.as_str(),
            )
            .await;

            let now = Local::now();
            let date_string = format!("{}{}{}{}{}", now.day(), "-", now.month(), "-", now.year());
            let addr = format!("{}@{}", ssh_user, ssh_host);
            let addr_port = format!("-p{}", ssh_port);
            let outfile = format!(
                "{}backup_snapshot_repo-{}.tar.gz",
                backup_server_archive_dest_dir.to_owned(),
                date_string
            );
            let in_dir = format!("{}backup_snapshot_repo/", ssh_dest_dir.to_owned());

            let compress_status = Command::new("ssh")
                .arg(addr_port)
                .arg(addr)
                .arg("tar")
                .arg("cvzf")
                .arg(outfile)
                .arg(in_dir)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .expect("failed to compress");
            compressed = compress_status.success();

            println!("backup compression success: {}", compressed.clone());
            let message = format!("backup compression success: {}", compressed);
            send_discord(&settings_map,
                
                "CapnHook",
                message.as_str(),
            )
            .await;
        } else if !remote {
            let now = Local::now();
            let date_string = format!("{}{}{}{}{}", now.day(), "-", now.month(), "-", now.year());
            let outfile = format!(
                "{}backup_snapshot_repo-{}.tar.gz",
                archive_dir.to_owned(),
                date_string
            );
            let in_dir = ssh_dest_dir.to_owned().to_string();

            println!("Compressing backup..");
            let message = "compressing the backup..".to_string();
            send_discord(&settings_map,
                
                "CapnHook",
                message.as_str(),
            )
            .await;

            let compress_status = Command::new("tar")
                .arg("cvzf")
                .arg(outfile)
                .arg(in_dir)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .expect("failed to compress");
            compressed = compress_status.success();

            let message = format!("backup compression success: {}", compressed);
            send_discord(&settings_map,
                
                "CapnHook",
                message.as_str(),
            )
            .await;
        }
    }

    copied && compressed
}

pub async fn delete_src_dir_contents(
    ssh_user: &str,
    ssh_port: &str,
    ssh_host: &str,
    _ssh_key: &str,
    ssh_src_dir: &str,
    ssh_dest_dir: &str,
    settings_map: HashMap<String, String>,
) {
    let _copy_from = ssh_src_dir.to_string();

    let _copy_to = format!("{}@{}:{}", ssh_user, ssh_host, ssh_dest_dir);

    println!("Deleting source dir/raw backup..");
    let message = format!("Deleting source dir/raw backup {}", ssh_dest_dir);
    send_discord(&settings_map,
        
        "CapnHook",
        message.as_str(),
    )
    .await;

    let remote_copy = settings_map
        .get("remote_copy_enabled")
        .expect("COULD NOT GET remote_copy_enabled")
        .as_str();

    let remote: bool = remote_copy.trim().contains("true");

    let mut del = false;
    println!("Delete: {}", del);
    if remote {
        // if compression is successful
        // delete uncompressed data and snapshots
        let addr = format!("{}@{}", ssh_user, ssh_host);
        let addr_port = format!("-p{}", ssh_port);
        let del_dir = format!("{}*", ssh_dest_dir.to_owned());
        let del_status = Command::new("ssh")
            .arg(addr_port)
            .arg(addr)
            .arg("rm")
            .arg("-rf")
            .arg(del_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .expect("cannot empty dir");
        del = del_status.success();

        let message = format!("clear temp data post compression: {}", del);
        send_discord(&settings_map,
            
            "CapnHook",
            message.as_str(),
        )
        .await;
    } else {
        // if compression is successful
        // delete uncompressed data and snapshots

        let del_dir = ssh_dest_dir.to_owned().to_string();
        println!("Deleting and recreating {}", del_dir.clone());

        let del_status = Command::new("rm")
            .arg("-rf")
            .arg(del_dir.clone())
            .output()
            .expect("cannot empty dir");
        let output =
            String::from_utf8(del_status.stdout).expect("unable to convert stdout to string");
        println!("{}", output);
        del = true;

        let recreate = fs::create_dir(del_dir.clone());
        if recreate.is_ok() {
            println!("Successfully recreated {}", del_dir.clone());
        }

        let message = format!("clear temp data post compression: {}", del);
        println!("{}", message.clone());
        send_discord(&settings_map,
            
            "CapnHook",
            message.as_str(),
        )
        .await;
    }
}
