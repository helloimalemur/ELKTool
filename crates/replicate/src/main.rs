use chrono::Local;
use config::Config;
use elktool_lib::replicate::haproxy_replicate::start_replicate_haproxy;
use elktool_lib::replicate::jdbc_replicate::start_replicate_jdbc;
use std::collections::HashMap;
use std::process::Command;
use std::thread;
use std::time::Duration;

macro_rules! thread_func {
    ($a:expr, $sm:expr, $def_par:expr) => {
        let mut handles = vec![];
        for _i in 0..$def_par {
            let sm = $sm.clone();
            let tk = tokio::runtime::Runtime::new();
            let handle = thread::spawn(move || tk.unwrap().block_on($a(sm)));
            handles.push(handle);
            // let _ = tokio::time::sleep(Duration::new(0, 200000000)).await;
            // let _ = tokio::time::sleep(Duration::new(1, 0)).await;
            let _ = tokio::time::sleep(Duration::new(2, 0)).await;
        }

        let _ = tokio::time::sleep(Duration::new(2, 0)).await;

        for handle in handles {
            // handle.join().unwrap()
            if let Err(_) = handle.join() {
                println!("WARNING: could not join on handle")
            }
        }
    };
}

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

    let parallelism = settings_map
        .get("parallelism")
        .expect("COULD NOT GET parallelism")
        .to_string();

    // let replicate_settings = Config::builder()
    //     .add_source(config::File::with_name("config/Replicate"))
    //     .build()
    //     .unwrap();
    // let _replicate_settings_map = replicate_settings
    //     .try_deserialize::<HashMap<String, String>>()
    //     .unwrap();

    let _ = Command::new("ulimit").arg("-n").arg("999999").spawn();

    println!("Running.. ");

    let _startup_timestamp = Local::now();

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
    loop {
        // run alert sequence
        // if alerting_enabled.contains("true") {
        //     let _ = alert_sequence(
        //         elastic_url,
        //         elastic_user,
        //         elastic_pass,
        //         settings_map.clone(),
        //     )
        //         .await;
        // }
        // sanitize
        thread_func!(
            start_replicate_haproxy,
            settings_map.clone(),
            default_parallel
        );
        thread_func!(start_replicate_jdbc, settings_map.clone(), default_parallel);
    }
}
