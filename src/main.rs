use crate::alerts_api_funcs::alert_api_funcs::alert_sequence;
use crate::ilm_api::ilm_api_funcs::stop_ilm_service;
use chrono::Local;
use config::Config;
use std::collections::HashMap;
use std::process::Command;
use std::thread;
use std::time::Duration;

mod index_api;
use crate::index_api::index_api_funcs::{
    close_indexes_over_age_threshold, cluster_disk_alloc_check, cluster_health_check,
    delete_indexes_over_age_threshold,
};

use crate::search_api::search_api_funcs::max_async_search_response_size;
use crate::snapshot_api::snapshot_api_funcs::{
    check_disk_space, check_threshold_and_create_snapshot,
};
use crate::transform::{parse_to_new_haproxy_field, start_transforms};

mod alerts_api_funcs;
mod ilm_api;
mod repository_api;
mod search_api;
mod snapshot_api;
mod tests;
mod transform;

#[tokio::main]
async fn main() {
    // load config
    let settings = Config::builder()
        .add_source(config::File::with_name("config/Settings"))
        .build()
        .unwrap();
    let settings_map = settings
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    // load lifetime policies
    let policies = Config::builder()
        .add_source(config::File::with_name("config/Policy"))
        .build()
        .unwrap();
    let policies_map = policies
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    let bind_delay = settings_map.clone();
    let delay = bind_delay.get("delay").unwrap();

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
    let run_lm_on_start = settings_map
        .get("run_lm_on_start")
        .expect("COULD NOT GET run_lm_on_start")
        .as_str();
    let alerting_enabled = settings_map
        .get("alerting_enabled")
        .expect("COULD NOT GET alerting_enabled")
        .as_str();
    let parallelism = settings_map
        .get("parallelism")
        .expect("COULD NOT GET parallelism")
        .to_string();

    let _ = Command::new("ulimit").arg("-n").arg("999999").spawn();

    println!("Running.. ");

    let _startup_timestamp = Local::now();

    if run_lm_on_start.contains("true") {
        run_lm_and_backup_routine(settings_map.clone(), policies_map.clone()).await;
    }

    let mut default_parallel = 1;

    if let Ok(ap) = thread::available_parallelism() {
        default_parallel = ap.get();
    }

    match parallelism.as_str() {
        "full" => {
            if default_parallel >= 2 {
                default_parallel = default_parallel / 2;
            }
        }
        &_ => {
            if default_parallel >= 2 {
                default_parallel = default_parallel / 2;
            }
        }
    }

    println!("Parallelism: {} {}", parallelism, default_parallel);

    // main outer loop
    let mut timestamp = Local::now().timestamp();
    loop {
        // run alert sequence
        if alerting_enabled.contains("true") {
            let _ = alert_sequence(
                elastic_url,
                elastic_user,
                elastic_pass,
                settings_map.clone(),
            )
            .await;
        }

        // run index transforms
        let mut handles = vec![];
        for _i in 0..default_parallel {
            let sm = settings_map.clone();
            let tk = tokio::runtime::Runtime::new();
            let handle = thread::spawn(move || tk.unwrap().block_on(start_transforms(sm)));
            handles.push(handle);
            let _ = tokio::time::sleep(Duration::new(0, 200000000)).await;
        }
        for handle in handles {
            // handle.join().unwrap()
            if let Err(_) = handle.join() {
                println!("WARNING: could not join on handle")
            }
        }

        // inner loop based on delay/threshold variable
        if Local::now().timestamp() > timestamp + delay.parse::<i64>().unwrap() {
            timestamp = Local::now().timestamp();
            run_lm_and_backup_routine(settings_map.clone(), policies_map.clone()).await;
        }
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

        println!("Closing indexes over threshold");
        close_indexes_over_age_threshold(settings_map.clone(), policies_map.clone()).await; // policies beginning with "close_"

        println!("Deleting indexes over threshold");
        delete_indexes_over_age_threshold(settings_map.clone(), policies_map.clone()).await; // policies beginning with "delete_"

        println!("Checking cluster health");
        cluster_health_check(settings_map.clone()).await; // check Elastic API for status and report

        if cluster_disk_alloc_check(settings_map.clone()).await {
            // double check drive space
            check_threshold_and_create_snapshot(settings_map.clone(), policies_map.clone()).await;
            // check last_snapshot and compare with threshold
        }; // check remaining drive space and report status
    }
}
