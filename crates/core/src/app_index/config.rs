use serde_derive::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct AppIndexConfigOuter {
    pub index: Vec<AppIndexConfig>,
}

#[derive(Debug, Deserialize)]
pub struct AppIndexConfig {
    pub enabled: String,
    pub index_prefix: String,
    pub needle: String,
    pub alert: String,
    pub total_to_process: u16,
}

#[allow(unused)]
pub fn get_app_index_config() -> Vec<AppIndexConfig> {
    let toml_str = fs::read_to_string("../../config/App_Index.toml").unwrap();
    if let Ok(entries) = toml::from_str::<AppIndexConfigOuter>(&toml_str) {
        entries.index
    } else {
        vec![]
    }
}
