#[cfg(test)]
mod tests {
    use crate::lifetime_api::lifetime_api_funcs::{cluster_disk_alloc_check, cluster_health_check};
    use config::Config;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_health_check() {
        let settings_map = test_prereqs();
        let result = cluster_health_check(settings_map.0).await;
        println!("RESULT: {}", result);
        assert_eq!(!result.is_empty(), true);
    }

    #[tokio::test]
    async fn test_alloc_check() {
        let settings_map = test_prereqs();
        let result = cluster_disk_alloc_check(settings_map.0).await;
        println!("RESULT: {}", result);
        assert_eq!(!result.to_string().is_empty(), true);
    }

    fn test_prereqs() -> (HashMap<String, String>, String, String, String) {
        let settings = Config::builder()
            .add_source(config::File::with_name("../../config/Settings"))
            .build()
            .expect("COULD NOT LOAD SETTINGS");
        let settings_map = settings
            .try_deserialize::<HashMap<String, String>>()
            .expect("COULD NOT LOAD SETTINGS");
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

        (
            settings_map.clone(),
            String::from(elastic_url),
            String::from(elastic_user),
            String::from(elastic_pass),
        )
    }
}
