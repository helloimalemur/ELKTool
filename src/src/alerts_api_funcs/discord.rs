use reqwest::header::CONTENT_TYPE;
use reqwest::ClientBuilder;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process;

pub async fn send_discord(settings_map: &HashMap<String, String>, username: &str, message: &str) {
    let discord = settings_map
        .get("discord_webhook_url")
        .expect("COULD NOT GET discord_webhook_url")
        .as_str();
    let message = message.to_string();
    if discord.contains("https://discord.com/api/webhooks/")
        || discord.contains("https://discordapp.com/api/webhooks/")
    {
        let json_message = match jsonify(username, message) {
            Ok(j) => j,
            Err(_e) => process::exit(3),
        };
        push_message(discord, json_message).await;
    } else {
        println!("invalid discord url");
    }
}

// pub async fn send(api_url: &str, username: &str, message: String) {
//     let json_message = match jsonify(username, message) {
//         Ok(j) => j,
//         Err(_e) => process::exit(3),
//     };
//     push_message(api_url, json_message).await;
// }

async fn push_message(api_url: &str, json_message: Value) {
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .no_gzip()
        .build();

    let response = match client {
        Ok(r) => {
            r.post(api_url)
                .header(CONTENT_TYPE, "application/json")
                .json(&json_message)
                .send()
                .await
        }
        Err(_e) => process::exit(3),
    };
    let _result_text = match response {
        Ok(r) => r.text().await,
        Err(_e) => process::exit(3),
    };
    // println!("Discord Sent: {:?}", result_text)
}

pub fn jsonify(username: &str, message: String) -> serde_json::Result<Value> {
    let data = json!({
    "username": username,
    "content": message,
    });

    // println!("{}", data.to_string());

    Ok(data)
}
