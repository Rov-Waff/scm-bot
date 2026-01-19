/* 模块抽象定义 */
pub mod utils {
    pub mod identity;
    pub mod posts;//帖子相关
    pub mod poi;//帖子兴趣点相关
}

struct Stroage {
    ticket_id: String,
    token:String,

}

use dotenvy::dotenv;
use log::info;
use reqwest::ClientBuilder;
use std::{sync::Arc, time::Duration};

use crate::utils::identity;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let mut stroage = Stroage {ticket_id:"".to_string(), token: "".to_string() };
    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(500))
        .build()
        .expect("Cannot create client");

    // 获取CaptchaTicket(阻塞)
    stroage.ticket_id = identity::get_captcha_id(&client).await;
    info!("获取到CAPTCHA ID{}", &stroage.ticket_id);
    stroage.token = identity::get_token(&client, &stroage.ticket_id).await;
    info!("成功获取token:{}",&stroage.token);
    //包进Arc,异步使用
    let stroage = Arc::new(stroage);
    
}
