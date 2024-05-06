use reqwest::{Error as ReqError, Error, Response, Url};
use std::ops::{Add, Deref};
use tokio::sync::RwLock;

use crate::model::config::Config;
use crate::model::user::IntoUser;

/// let allMission = await axios.request({
/// url: "http://" + configObj.root + "/api/v2/sync/maindata",
/// method: "get",
/// headers: {
/// Accept: "application/json",
/// ...commonHeader,
/// cookie,
/// },
/// })

pub async fn get_all_mission_info(
    config: &Config,
    http_client: &reqwest::Client,
) -> Result<Response, ReqError> {
    http_client
        .get(format!("http://{}/api/v2/sync/maindata", config.url()).as_str())
        .header(reqwest::header::ACCEPT, "application/json")
        .header(reqwest::header::COOKIE, config.cookie())
        .send()
        .await
}

pub async fn post_qb_login(
    config: &Config,
    http_client: &reqwest::Client,
) -> Result<Response, ReqError> {
    let header = get_header(config);

    let url = Url::parse(format!("http://{}/api/v2/auth/login", config.url()).as_str()).unwrap();

    http_client
        .post(url)
        .headers(header)
        .form(&config.to_user())
        .send()
        .await
}

fn get_header(config: &Config) -> reqwest::header::HeaderMap {
    let mut header = reqwest::header::HeaderMap::new();
    header.insert(reqwest::header::HOST, config.url().parse().unwrap());
    header.insert(
        reqwest::header::ORIGIN,
        format!("http://{}", config.url()).parse().unwrap(),
    );
    header.insert(
        reqwest::header::REFERER,
        format!("http://{}/", config.url()).parse().unwrap(),
    );
    header.insert(
        reqwest::header::ACCEPT_ENCODING,
        "gzip, deflate, br".parse().unwrap(),
    );
    header.insert(
        reqwest::header::CONTENT_TYPE,
        "application/x-www-from-urlencoded; charset=UTF-8"
            .parse()
            .unwrap(),
    );
    header.insert(reqwest::header::ACCEPT, "*/*".parse().unwrap());
    header
}

pub async fn get_torrent_peers(
    config: &Config,
    http_client: &reqwest::Client,
    torrent_list: Vec<String>,
) -> Result<Vec<(String, Response)>, Error> {
    async fn query_once(
        config: &Config,
        http_client: &reqwest::Client,
        torrent: &str,
    ) -> Result<(String, Response), Error> {
        http_client
            .get(format!("http://{}/api/v2/sync/torrentPeers", config.url()).as_str())
            .header(reqwest::header::ACCEPT, "application/json")
            .header(reqwest::header::COOKIE, config.cookie())
            .headers(get_header(config))
            .query(&[("hash", torrent)])
            .send()
            .await
            .map(|r| (torrent.to_string(), r))
    }
    let mut response_list = Vec::with_capacity(torrent_list.len());

    for hash in torrent_list.into_iter() {
        let a = query_once(config, http_client, hash.as_str()).await?;
        response_list.push(a);
    }

    Ok(response_list)
}
/// peers: hash:xxx,peers:xxx
pub async fn block_peers(
    config: &Config,
    http_client: &reqwest::Client,
    //hash peer
    peers: Vec<(String, String)>,
    ban_count: &RwLock<u32>,
) -> Result<(), usize> {
    let mut count = peers.len();
    let ban_count = ban_count.write().await;
    for (hash, peer) in peers {
        http_client
            .post(format!("http://{}/api/v2/transfer/banPeers", config.url()).as_str())
            .header(reqwest::header::ACCEPT, "application/json")
            .header(reqwest::header::COOKIE, config.cookie())
            .form(&[("hash", hash.as_str()), ("peers", peer.as_str())])
            .send()
            .await
            .ok();
        log::info!("{}已阻止 {} 的主机 {}", ban_count.deref()+1, hash, peer);
        count -= 1;
        let _ = ban_count.add(1);
    }

    if count == 0 {
        Ok(())
    } else {
        Err(count)
    }
}
