use chrono::Local;
use config::Config;
use elktool_lib::ilm_api::ilm_api_funcs::stop_ilm_service;
use elktool_lib::lifetime_api::lifetime_api_funcs::{
    close_indexes_over_age_threshold, cluster_disk_alloc_check, cluster_health_check,
    delete_indexes_over_age_threshold,
};
use elktool_lib::search_settings::search_settings::max_async_search_response_size;
use elktool_lib::snapshot_api::snapshot_api_funcs::{
    check_disk_space, check_threshold_and_create_snapshot, delete_snapshots_over_age_threshold,
};
use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // load config
    let settings = Config::builder()
        .add_source(config::File::with_name("config/Settings"))
        .build()
        .expect("COULD NOT LOAD SETTINGS");
    let settings_map = settings
        .try_deserialize::<HashMap<String, String>>()
        .expect("COULD NOT LOAD SETTINGS");

    // load lifetime policies
    let policies = Config::builder()
        .add_source(config::File::with_name("config/Policy"))
        .build()
        .expect("COULD NOT LOAD SETTINGS");
    let policies_map = policies
        .try_deserialize::<HashMap<String, String>>()
        .expect("COULD NOT LOAD SETTINGS");

    let _ = Command::new("ulimit").arg("-n").arg("999999").spawn();

    println!("Running.. ");

    loop {
        run_lm_and_backup_routine(settings_map.clone(), policies_map.clone()).await;
        tokio::time::sleep(Duration::new(3600, 0)).await
    }
}

async fn run_lm_and_backup_routine(
    settings_map: HashMap<String, String>,
    policies_map: HashMap<String, String>,
) {
    println!(
        "Starting lm and backup routine .. {}",
        Local::now().format("%d-%m-%y - %H:%M")
    );

    // if we're not currently creating a snapshot, and there is enough space on the drive.
    if !check_disk_space(settings_map.clone(), policies_map.clone()).await {
        println!("Stopping Elastic built in index lifetime management service");
        stop_ilm_service(settings_map.clone(), policies_map.clone()).await; // stop built-in ILM services

        max_async_search_response_size(settings_map.clone(), policies_map.clone()).await; // resolve async search size kibana error

        println!("Deleting snapshots over threshold");
        delete_snapshots_over_age_threshold(settings_map.clone(), policies_map.clone()).await; // policies beginning with "delete_"

        println!("Deleting indexes over threshold");
        delete_indexes_over_age_threshold(settings_map.clone(), policies_map.clone()).await; // policies beginning with "delete_"

        println!("Closing indexes over threshold");
        close_indexes_over_age_threshold(settings_map.clone(), policies_map.clone()).await; // policies beginning with "close_"

        println!("Checking cluster health");
        cluster_health_check(settings_map.clone()).await; // check Elastic API for status and report

        if cluster_disk_alloc_check(settings_map.clone()).await {
            // double check drive space
            check_threshold_and_create_snapshot(settings_map.clone(), policies_map.clone()).await;
            // check last_snapshot and compare with threshold
        }; // check remaining drive space and report status
    }
}
