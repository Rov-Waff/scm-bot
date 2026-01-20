use log::error;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::Stroage;

/*
响应体示例
{
    "id": "1644992", #REQUIRE
    "user": {
        "id": "438337735",
        "nickname": "若言rJ0e",
        "avatar_url": "https://static.codemao.cn/nemo/BJFeaH4aR.jpeg",
        "subject_id": 30375,
        "work_shop_name": "Python,cPlusplus",
        "work_shop_level": 0,
        "wuhan_medal": false,
        "has_signed": false
    },
    "title": "给我出一道方程题在自己解答",  #REQUIRE
    "content": "<p><p>如果正确赏300000000000积分</p> \n<p>例题:</p> \n<p>2x^2+50=0</p> \n<p>不可借用AI</p></p>",  #REQUIRE
    "board_id": "5",
    "board_name": "你问我答",
    "updated_at": 1768797520,
    "created_at": 1768797520,
    "n_views": 10,
    "n_replies": 2,
    "n_comments": 0,
    "is_authorized": false,
    "is_featured": false,
    "is_hotted": false,
    "is_pinned": false,
    "tutorial_flag": 0,
    "ask_help_flag": 1
}

*/
#[derive(Serialize, Deserialize, Debug)]
struct GetPostResponse {
    id: String,
    title: String,
    content: String,
}

async fn get_post(
    client: Arc<Client>,
    id: u32,
    stroage: Arc<Mutex<Stroage>>,
) -> Option<GetPostResponse> {
    let stro = stroage.lock().await;
    match client
        .get(format!(
            "https://api.codemao.cn/web/forums/posts/{}/details",
            id
        ))
        .header("Cookie", format!("authorization={}", stro.token))
        .send()
        .await
    {
        Ok(r) => match r.json::<GetPostResponse>().await {
            Ok(r) => Some(r),
            Err(_) => {
                error!("无法序列化,id:{:?}", id);
                None
            }
        },
        Err(_) => {
            error!("无法发送请求,id:{:?}", id);
            None
        }
    }
}

pub async fn consume_poi(client: Arc<Client>, stroage: Arc<Mutex<Stroage>>,redis_client:Arc<Mutex<redis::Connection>>) {
    
}
