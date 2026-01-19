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
/// CAPTCHA Ticket 的响应结构。
///
/// 包含用于后续登录请求的 `ticket` 字段。
///
/// 示例：
///
/// ```json
/// {
///   "rule": "DEFAULT",
///   "appid": "",
///   "ticket": "bcd4e343e9f24f7bbfdf9ee6c2dbac9f"
/// }
/// ```
#[derive(Deserialize, Serialize)]
pub struct CaptchaTicket {
    /// 验证规则标识
    rule: String,
    /// 应用 ID
    appid: String,
    /// 验证票据，用于登录时传递到 `X-Captcha-Ticket` 头
    ticket: String,
}

/// 获取 CAPTCHA ticket。
///
/// 使用 `client` 向验证码服务发送请求，返回响应中的 `ticket` 字符串。
///
/// 环境变量：
/// - `USERNAME`（可选）：用于请求体中的 `identity` 字段，若未设置则使用空字符串。
///
/// 返回：`String` —— captcha ticket。
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
    "avatar_url": "https://cdn-community.bcmcdn.com/47/community/d2ViXzEwMDFfM  jYxNTUwNV8yNjE1NTA1XzE3NDczOTU2NzM5MjlfNDY0NWQwZDM.png",
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
/// 登录并获取用户 Token。
///
/// 参数：
/// - `client`: 已配置的 `reqwest::Client`。
/// - `captcha_ticket`: 来自 `get_captcha_id` 的 captcha ticket，会被作为 `X-Captcha-Ticket` 请求头发送。
///
/// 环境变量：
/// - `USERNAME`：作为 `identity`；若缺失，默认使用 `"114514"`（仅作占位）。
/// - `PASSWORD`：作为 `password`；若缺失，默认使用 `"r"`（仅作占位）。
///
/// 返回：`String` —— 登录成功后的 `token` 字段。
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
