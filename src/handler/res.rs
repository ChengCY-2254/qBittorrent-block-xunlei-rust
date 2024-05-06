use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Response;
use serde_json::Value;

lazy_static! {
    static ref REG: Regex = Regex::new("(.*?);").unwrap();
}

pub fn get_cookie(response: &Response) -> Option<String> {
    log::info!("{:?}", response.headers());
    if let Some(cookie_str) = response.headers().get("set-cookie") {
        let reg_match = REG.find(cookie_str.to_str().unwrap());
        if let Some(m) = reg_match {
            return Some(m.as_str().to_string());
        }
    }
    None
}

pub async fn get_torrents(response: Response) -> Vec<String> {
    let result: Value = serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();

    result["torrents"]
        .as_object()
        .unwrap()
        .iter()
        .map(|v| v.0.to_owned())
        .collect()
}

pub async fn flat_client_from_peers(
    response_vec: Vec<(String,Response)>,
) -> Vec<(String, serde_json::Map<String, Value>)> {
    async fn get_client(response: Response) -> serde_json::Map<String, Value> {
        let result: Value = serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();
        let peers = result["peers"].as_object().unwrap();
        peers.clone()
    }
    let mut ret = Vec::with_capacity(response_vec.len());
    for (hash,v) in response_vec.into_iter() {
        let peers = get_client(v).await;
        ret.push((hash,peers));
    }
    ret
}
