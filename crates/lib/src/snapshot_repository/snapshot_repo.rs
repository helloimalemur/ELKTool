use crate::snapshot_repository::repository::Repository;
use std::time::Duration;

pub async fn get_snapshot_repo(
    elastic_url: &str,
    elastic_user: &str,
    elastic_pass: &str,
) -> Repository {
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
        .timeout(Duration::new(6, 0))
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
