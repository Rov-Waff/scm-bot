/*
Captcha Ticket Example
```json
{
  "rule": "DEFAULT",
  "appid": "",
  "ticket": "bcd4e343e9f24f7bbfdf9ee6c2dbac9f"
}
```
*/

use std::collections::HashMap;

use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CaptchaTicket {
    rule: String,
    appid: String,
    ticket: String,
}

//获取CAPTCHA Ticket
pub async fn get_captcha_id(client: &Client) -> String {
    let mut req_body = HashMap::new();
    let username = std::env::var("USERNAME").unwrap_or_else(|_| String::new());
    req_body.insert("identity".to_string(), username);
    let res = client
        .post("https://open-service.codemao.cn/captcha/rule/v3")
        .json(&req_body)
        .send()
        .await
        .expect("无法获取Captcha Ticket")
        .json::<CaptchaTicket>()
        .await
        .unwrap()
        .ticket;
    res
}
