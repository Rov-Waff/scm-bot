use std::sync::Arc;

use crate::Stroage;
use anyhow::Context;
use anyhow::Result;
use log::{debug, error, info};
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
pub(crate) async fn get_poi(client: Arc<Client>, stroage: Arc<Mutex<Stroage>>) -> Result<()> {
    info!("get_poi: 开始");

    // 在发送请求前短暂获取锁以克隆需要的 token，避免在 await 期间持有锁
    let token = {
        let stro = stroage.lock().await;
        info!("get_poi: 克隆 token (长度={})", stro.token.len());
        stro.token.clone()
    };

    info!("get_poi: 向 codemao 热门接口发送请求");
    let r = client
        .get("https://api.codemao.cn/web/forums/posts/hots/all")
        .header("Cookie", format!("authorization={}", token))
        .send()
        .await
        .context("get_poi: 请求失败")?;

    info!("get_poi: 已收到响应 (状态={})", r.status());

    let text = r.text().await.context("get_poi: 读取响应文本失败")?;
    debug!("get_poi: 响应文本长度={}", text.len());

    let list: PostListResponse = serde_json::from_str(&text)
        .context("get_poi: JSON 反序列化失败")?;

    info!("get_poi: 解析到的项数量={}", list.items.len());
    let mut stro = stroage.lock().await;
    let before = stro.poi.len();
    for (idx, i) in list.items.into_iter().enumerate() {
        match i.trim().parse::<u32>() {
            Ok(id) => {
                if !(stro.processed_poi.contains(&id) || stro.poi.contains(&id)) {
                    stro.poi.push(id);
                    debug!("get_poi: 推入项 #{} id={}", idx, id);
                } else {
                    debug!("get_poi: {} 已存在，不在推入", id)
                }
            }
            Err(e) => {
                error!("get_poi: 在索引 {} 解析 id 失败: {:?}", idx, e);
            }
        }
    }
    debug!("get_poi: poi 大小 原={} 现={}", before, stro.poi.len());
    info!("get_poi: 完成，poi 总数={}", stro.poi.len());

    Ok(())
}
