/* 模块抽象定义 */
pub mod utils {
    pub mod identity;
    pub mod poi;
    pub mod posts; //帖子相关 //帖子兴趣点相关
}

struct Stroage {
    ticket_id: String,
    token: String,
    poi: Vec<u32>,
    processed_poi: Vec<u32>,
}

use crate::utils::{identity, poi};
use dotenvy::dotenv;
use log::info;
use reqwest::ClientBuilder;
use std::{env, sync::Arc, time::Duration, vec};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let mut stroage = Stroage {
        ticket_id: "".to_string(),
        token: "".to_string(),
        poi: vec![],
        processed_poi: vec![],
    };
    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(500))
        .build()
        .expect("Cannot create client");
    let redis_client = redis::Client::open(env::var("REDIS_URL").expect("请提供Redis的URL"))
        .expect("无法连接到Redis")
        .get_connection()
        .expect("无法获取连接到Redis");
    // 获取CaptchaTicket(阻塞)
    stroage.ticket_id = identity::get_captcha_id(&client).await;
    info!("获取到CAPTCHA ID{}", &stroage.ticket_id);
    stroage.token = identity::get_token(&client, &stroage.ticket_id).await;
    info!("成功获取token:{}", &stroage.token);
    //包进Arc,异步使用
    let stroage = Arc::new(Mutex::new(stroage));
    let client = Arc::new(client);
    let redis_client = Arc::new(Mutex::new(redis_client));
    loop {
        // 现在 `get_poi` 返回 `anyhow::Result<()>`，在此处 await 并记录错误
        if let Err(e) = poi::get_poi(client.clone(), stroage.clone(),redis_client.clone()).await {
            log::error!("get_poi failed: {:?}", e);
        }
        tokio::time::sleep(Duration::from_secs(60)).await; //每轮loop结束后等待一分钟后开下一轮
    }
}
