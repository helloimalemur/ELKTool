use crate::alerts_api_funcs::alert::{delete_alert_indicies_by_query, get_alert_indicies};
use crate::notifications::discord::send_discord;
use crate::notifications::email::send_email;
use std::collections::HashMap;

pub async fn alert_sequence(
    elastic_url: &str,
    elastic_user: &str,
    elastic_pass: &str,
    settings_map: HashMap<String, String>,
) {
    if let Ok(alert_indicies) = get_alert_indicies(
        elastic_url,
        elastic_user,
        elastic_pass,
        settings_map.clone(),
    )
    .await
    {
        #[allow(unused)]
        let mut message = String::new();
        for entry in alert_indicies.iter() {
            message = format!("{} :: {}", entry.0, entry.1);
            println!("Deleting and sending alert: {}", entry.0);
            println!("{}", message.clone());
            send_alerts(message.clone(), settings_map.clone()).await;
            let _success = delete_alert_indicies_by_query(
                elastic_url,
                elastic_user,
                elastic_pass,
                entry.0.to_string(),
            )
            .await;
        }
        // let _ = delete_alert_indicies(elastic_url, elastic_user, elastic_pass).await;
    }
}

pub async fn send_alerts(message: String, settings_map: HashMap<String, String>) {
    send_discord(&settings_map, "CapnHook", message.as_str()).await;
    let subject = format!("ALERT:: {}", message);
    send_email(&settings_map, subject.as_str(), message.as_str()).await;
}

#[cfg(test)]
mod tests {
    use crate::alerts_api_funcs::alert::get_alert_indicies;
    use crate::alerts_api_funcs::alert_api_funcs::send_alerts;
    use config::Config;
    use std::collections::HashMap;
    use tokio::runtime;

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
    #[ignore]
    fn alerts() {
        let settings_map: HashMap<String, String> = setup_test();
        let rt = runtime::Runtime::new();
        rt.unwrap()
            .block_on(send_alerts("test alert".to_string(), settings_map));
    }

    #[test]
    #[ignore]
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

        let rt = runtime::Runtime::new();
        let _ = rt.unwrap().block_on(get_alert_indicies(
            elastic_url,
            elastic_user,
            elastic_pass,
            Default::default(),
        ));
    }
}
