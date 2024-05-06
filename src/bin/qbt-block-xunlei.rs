use std::env;
use std::ops::{Deref};
use std::sync::{Arc};
use std::time::Duration;

use lazy_static::lazy_static;
use regex::Regex;
use tokio::sync::RwLock;
use tokio::time;

use qBittorrent_block_xunlei::handler::res::flat_client_from_peers;
use qBittorrent_block_xunlei::req::qb::{block_peers, get_torrent_peers};
use qBittorrent_block_xunlei::{
    handler::res::{get_cookie, get_torrents},
    model::config::Config,
    req::qb::get_all_mission_info,
    req::qb::post_qb_login,
};

lazy_static! {
    static ref BAN_REGEX: Regex = Regex::new(r"(-xl0012)|(xunlei)|(^7\.)|(qqduwnload)").unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = ftlog::builder()
        .try_init()
        .expect("Failed to initialize logger");
    dotenv::dotenv().ok();
    log::info!("test");

    let ban_count: RwLock<u32> = RwLock::new(0);
    let http_client = reqwest::Client::new();

    let conf = Arc::new(RwLock::new(Config::new(
        env::var("QB_URL")?,
        env::var("USERNAME")?,
        env::var("PASSWORD")?,
    )));

    //首先登陆，拿到cookie

    let login_res = post_qb_login(conf.read().await.deref(), &http_client)
        .await
        .inspect_err(|e| log::error!("获取Cookie失败 原因是：{}", e))
        .unwrap();
    conf.write()
        .await
        .set_cookie(get_cookie(&login_res).expect("获取Cookie失败"));

    log::info!("登陆成功，开始任务");
    poll(
        Arc::clone(&conf),
        &http_client,
        &ban_count,
    )
    .await;
    // .await;

    Ok(())
}

/// 判断客户端是否需要被ban
fn need_ban(client_name: impl Into<String>) -> bool {
    let client_name = client_name.into();
    if client_name.is_empty() {
        return false;
    }
    let client_name = client_name.to_lowercase();
    return BAN_REGEX.is_match(&client_name);
}
// #[tokio::main]
async fn poll(
    conf: Arc<RwLock<Config>>,
    http_client: &reqwest::Client,
    ban_count: &RwLock<u32>,
) {
    let interval = Duration::from_secs(10);
    // let runtime = Runtime::new().unwrap();
    // runtime.block_on(async move {
    let mut intv = time::interval(interval);
    loop {
        //在这里对列表进行检查
        let all_mission = get_all_mission_info(conf.read().await.deref(), &http_client)
            .await
            .inspect_err(|e| log::error!("获取任务列表失败 原因是：{}", e))
            .unwrap();
        //获取到每个torrent的详细信息 hash和返回对应
        let torrent_peers = get_torrent_peers(
            conf.read().await.deref(),
            &http_client,
            get_torrents(all_mission).await,
        )
        .await;

        match torrent_peers {
            Ok(list) => {
                let peers = flat_client_from_peers(list).await;
                let mut ban_list = vec![];
                // log::info!("peers: {:?}", peers);
                for (hash, peer) in peers {
                    for (ip_port, info) in peer {
                        let info = info.as_object().unwrap();
                        let client = info["client"].as_str().unwrap();
                        if need_ban(client) {
                            log::info!("{} 发现需要ban的客户端: {}", hash.as_str(), client);
                            ban_list.push((ip_port, client.to_string()));
                        }
                    }
                }
                //开始ban客户端
                block_peers(conf.read().await.deref(), &http_client, ban_list,ban_count).await.ok();
            }
            //错误了什么都不干，下次继续轮询
            Err(_) => {}
        }

        intv.tick().await;
    }
}
