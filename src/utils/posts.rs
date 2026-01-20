use log::{debug, error, info};
use redis::TypedCommands;
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
/// API 返回的帖子简要结构（只包含当前需要的字段）
///
/// 字段:
/// - `id`: 帖子 ID（字符串）
/// - `title`: 帖子标题
/// - `content`: 帖子内容（HTML）
struct GetPostResponse {
    id: String,
    title: String,
    content: String,
}

/// 异步从 CodeMao API 获取帖子详情并尝试反序列化为 `GetPostResponse`。
///
/// - `client`: 已配置的 `reqwest::Client`。
/// - `id`: 帖子 ID。
/// - `stroage`: 包含授权 `token` 的存储引用。
///
/// 返回 `Some(GetPostResponse)` 表示成功，否则返回 `None`（请求或反序列化失败）。
async fn get_post(
    client: Arc<Client>,
    id: u32,
    stroage: Arc<Mutex<Stroage>>,
) -> Option<GetPostResponse> {
    let stro = stroage.lock().await;
    let req = client
        .get(format!(
            "https://api.codemao.cn/web/forums/posts/{}/details",
            id
        ))
        .header("Cookie", format!("authorization={}", stro.token))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    debug!("Response Text: {}", req);
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
            Err(e) => {
                error!("无法序列化,id:{:?},Error:{:?}", id, e);
                None
            }
        },
        Err(_) => {
            error!("无法发送请求,id:{:?}", id);
            None
        }
    }
}

/// 从 Redis 的有序集合 `poi` 中弹出最小分数的一个帖子 ID，
/// 获取帖子详情并执行消费逻辑；随后将该 ID 加入 `processed_poi`（带过期时间的分数）。
///
/// 注意：`ZPOPMIN` 在 redis-rs 中会返回 `(member, score)` 的元组，这里会解析 `member` 为 `u32`。
pub(crate) async fn consume_poi(
    client: Arc<Client>,
    stroage: Arc<Mutex<Stroage>>,
    redis_client: Arc<Mutex<redis::Connection>>,
) {
    let mut redis_client = redis_client.lock().await;
    //从poi这个ZSET中弹出一个元素，这个元素为帖子ID
    let id: Option<u32> = match redis_client.zpopmin("poi", 1) {
        Ok(r) => r
            .get(0)
            .map(|entry| entry.parse::<u32>().unwrap()),
        Err(e) => {
            error!("无法从poi集合中弹出元素，Error:{:?}", e);
            return;
        }
    };
    match id {
        Some(id) => {
            match get_post(client, id, stroage).await {
                Some(post) => {
                    info!("获取到帖子:id:{},标题:{}", post.id, post.title);
                    //消费逻辑
                }
                None => {
                    error!("无法获取帖子详情,id:{}", id);
                }
            }
        }
        None => {
            info!("当前没有可处理的帖子ID");
        }
    }
    //完成消费，将ID加入processed_poi集合，分数为当前时间戳+86400
    if let Some(id) = id {
        let score = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as f64
            + 86400.0;
        let _ = redis_client
            .zadd("processed_poi", id.to_string(), score)
            .unwrap();
        info!("已将帖子ID:{}加入processed_poi集合", id);
    }
}
