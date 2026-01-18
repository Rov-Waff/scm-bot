/* 模块抽象定义 */
pub mod entity {
    pub mod identity;
}

struct Stroage {
    ticket_id: String,
}

use dotenvy::dotenv;
use log::info;
use reqwest::ClientBuilder;
use std::time::Duration;

use crate::entity::identity;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let mut stroage = Stroage {
        ticket_id: "".to_string(),
    };
    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(500))
        .build()
        .expect("Cannot create client");

    // 获取CaptchaTicket(阻塞)
    stroage.ticket_id = identity::get_captcha_id(&client).await;
    info!("获取到CAPTCHA ID{}", &stroage.ticket_id)
}
