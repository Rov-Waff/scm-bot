use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::Stroage;
use anyhow::Context;
use anyhow::Result;
use log::info;
use redis::TypedCommands as _;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

pub enum Mark {
    UnProcessed(u32),
    Processed,
}

/// 响应结构：包含一个帖子 ID 的列表（字符串形式）
///
/// 示例 JSON: {"items": ["123", "456"]}
#[derive(Serialize, Deserialize, Debug)]
struct PostListResponse {
    items: Vec<String>,
}

// 请求信息：
// URL: https://api.codemao.cn/web/forums/posts/hots/all
// 方法: GET

/// 从 CodeMao 热门接口拉取帖子 ID 列表，并将未处理的 ID 推入共享的 `Stroage.poi`。
pub(crate) async fn get_poi(
    client: Arc<Client>,
    stroage: Arc<Mutex<Stroage>>,
    redis_client: Arc<Mutex<redis::Connection>>,
) -> Result<()> {
    info!("get_poi: 开始");

    // 在发送请求前短暂获取锁以克隆需要的 token，避免在 await 期间持有锁
    let token = {
        let stro = stroage.lock().await;
        info!("get_poi: 克隆 token (长度={})", stro.token.len());
        stro.token.clone()
    };
    let mut redis_cli = redis_client.lock().await;
    info!("get_poi: 向 codemao 热门接口发送请求");
    let r = client
        .get("https://api.codemao.cn/web/forums/posts/hots/all")
        .header("Cookie", format!("authorization={}", token))
        .send()
        .await
        .context("get_poi: 请求失败")?;
    let r = r.json::<PostListResponse>().await?.items;
    for i in r {
        // 判断是否存在于 `poi` 或 `processed_poi` 集合
        let in_poi = redis_cli.zrank("poi", &i).ok().flatten().is_some();
        let in_processed = redis_cli
            .zrank("processed_poi", &i)
            .ok()
            .flatten()
            .is_some();
        if in_poi || in_processed {
            info!("id:{:?} 已存在，不推入Redis", &i);
        } else {
            // 不存在则送进 poi 集合，member 为帖子ID，score 为当前时间戳+86400
            let expr_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            redis_cli.zadd("poi", &i, expr_time + 86400)?;
            info!("ID:{:?} 已推入Redis", &i);
        }
    }
    Ok(())
}
