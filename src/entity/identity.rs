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

/*
Auth Example:
```json
{
  "auth": {
    "token": "",
    "phone_number": "",
    "email": "",
    "has_password": true,
    "is_weak_password": false
  },
  "user_info": {
    "id": 2615505,
    "nickname": "xiaole233",
    "avatar_url": "https://cdn-community.bcmcdn.com/47/community/d2ViXzEwMDFfMjYxNTUwNV8yNjE1NTA1XzE3NDczOTU2NzM5MjlfNDY0NWQwZDM.png",
    "fullname": "",
    "birthday": ,
    "sex": 1,
    "qq": "",
    "description": "",
    "grade": 10,
    "grade_desc": ""
  }
}
```
*/
#[derive(Deserialize, Serialize, Debug)]
pub struct Auth {
    token: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct LoginResponse {
    auth: Auth,
}

pub async fn get_token(client: &Client, captcha_ticket: &String) -> String {
    let mut req_body = HashMap::new();
    /*
    Request Body:
    ```
    {
        "identity": "手机号",
        "password": "密码",
        "pid": "65edCTyg"
    }
    ```
    */
    req_body.insert("identity", {
        match std::env::var("USERNAME") {
            Ok(r) => r,
            Err(_) => "114514".to_string(),
        }
    });
    req_body.insert("password", {
        match std::env::var("PASSWORD") {
            Ok(r) => r,
            Err(_) => "r".to_string(),
        }
    });
    req_body.insert("pid", "65edCTyg".to_string());

    let res = client
        .post("https://api.codemao.cn/tiger/v3/web/accounts/login/security")
        .json(&req_body)
        .header("X-Captcha-Ticket", captcha_ticket.to_string())
        .send()
        .await
        .expect("无法获取Token")
        .json::<LoginResponse>()
        .await
        .unwrap()
        .auth
        .token;
    res
}
