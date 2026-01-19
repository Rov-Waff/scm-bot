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
use std::{sync::Arc, time::Duration, vec};
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

    // 获取CaptchaTicket(阻塞)
    stroage.ticket_id = identity::get_captcha_id(&client).await;
    info!("获取到CAPTCHA ID{}", &stroage.ticket_id);
    stroage.token = identity::get_token(&client, &stroage.ticket_id).await;
    info!("成功获取token:{}", &stroage.token);
    //包进Arc,异步使用
    let stroage = Arc::new(Mutex::new(stroage));
    let client = Arc::new(client);

    loop {
        let get_poi = poi::get_poi(client.clone(), stroage.clone());
        tokio::join!(get_poi);
        tokio::time::sleep(Duration::from_secs(60)).await; //每轮loop结束后等待一分钟后开下一轮
    }
}
