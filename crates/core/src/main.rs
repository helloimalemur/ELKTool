use chrono::Local;
use config::Config;
use elktool_lib::alerts_api_funcs::alert_api_funcs::alert_sequence;
use elktool_lib::transform::haproxy_transforms::start_haproxy_transforms;
use elktool_lib::transform::jdbc_transforms::start_jdbc_transforms;
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

    let then = Local::now();

    // main outer loop
    loop {
        let now = Local::now();
        if now.signed_duration_since(then).num_seconds() > 60 {
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
        }


        // run index transforms
        thread_func!(
            start_haproxy_transforms,
            settings_map.clone(),
            default_parallel
        );
        // jdbc transforms
        thread_func!(
            start_jdbc_transforms,
            settings_map.clone(),
            default_parallel
        );
    }
}
