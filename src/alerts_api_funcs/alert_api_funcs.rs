use std::collections::HashMap;
use crate::alerts_api_funcs::alert::{delete_alert_indicies, get_alert_indicies};
use crate::alerts_api_funcs::discord::send_discord;
use crate::alerts_api_funcs::email::send_email;


pub async fn alert_sequence(
    elastic_url: &str,
    elastic_user: &str,
    elastic_pass: &str,
    settings_map: HashMap<String, String>,
) {
    if let Ok(alert_indicies) = get_alert_indicies(elastic_url, elastic_user, elastic_pass, settings_map.clone()).await {
        #[allow(unused)]
            let mut message = String::new();

        for entry in alert_indicies.iter() {
            message = format!("{} :: {}", entry.0, entry.1);
            println!("{}", message.clone());
            send_alerts(message.clone(), settings_map.clone()).await;
        }
        let _ = delete_alert_indicies(elastic_url, elastic_user, elastic_pass).await;
    }
}

pub(crate) async fn send_alerts(message: String, settings_map: HashMap<String, String>) {
    send_discord(&settings_map, "CapnHook", message.as_str()).await;
    send_email(&settings_map, "ALERT", message.as_str()).await;
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use config::Config;
    use tokio::runtime;
    use crate::alerts_api_funcs::alert::get_alert_indicies;
    use crate::alerts_api_funcs::alert_api_funcs::send_alerts;
    
    fn setup_test() -> HashMap<String, String> {
        let settings = Config::builder()
            .add_source(config::File::with_name("config/Settings"))
            .build()
            .unwrap();
        settings
            .try_deserialize::<HashMap<String, String>>()
            .unwrap()
    }

    #[test]
    fn alerts() {
        let settings_map: HashMap<String, String> = setup_test();
        let rt = runtime::Runtime::new();
        rt.unwrap().block_on(send_alerts("test alert".to_string(), settings_map));
    }
    
    #[test]
    fn test_get_alert_ind() {
        let settings_map: HashMap<String, String> = setup_test();

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

        let rt = runtime::Runtime::new();
        rt.unwrap().block_on(get_alert_indicies(elastic_url, elastic_user, elastic_pass, ));
        
    }
    
}
