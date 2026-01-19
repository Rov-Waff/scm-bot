use std::sync::Arc;

use crate::Stroage;
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
///
/// 参数：
/// - `client`: 已配置的 `reqwest::Client`（被 `Arc` 包装以便共享）
/// - `stroage`: 受 `tokio::sync::Mutex` 保护的 `Stroage`（被 `Arc` 包装以便跨任务共享）
///
/// 行为说明：
/// 1. 在发送请求前短暂获取锁并克隆 `stroage.token`，以避免在网络请求的 `await` 期间持有锁。
/// 2. 向远端接口发送 GET 请求，读取完整响应文本，再使用 `serde_json::from_str` 解析为 `PostListResponse`。
/// 3. 解析后重新获取锁，将解析出的 ID（字符串解析为 `u32`）按顺序 push 到 `stroage.poi`。
/// 4. 整个过程中会记录信息性/调试/错误日志，便于排查网络/解析问题。
pub(crate) async fn get_poi(client: Arc<Client>, stroage: Arc<Mutex<Stroage>>) {
    info!("get_poi: 开始");

    // 在发送请求前短暂获取锁以克隆需要的 token，避免在 await 期间持有锁
    let token = {
        let stro = stroage.lock().await;
        info!("get_poi: 克隆 token (长度={})", stro.token.len());
        stro.token.clone()
    };

    info!("get_poi: 向 codemao 热门接口发送请求");
    let resp = client
        .get("https://api.codemao.cn/web/forums/posts/hots/all")
        .header("Cookie", format!("authorization={}", token))
        .send()
        .await;

    match resp {
        Ok(r) => {
            info!("get_poi: 已收到响应 (状态={})", r.status());
            // 先读取完整响应文本，再尝试反序列化为 JSON，避免 Response 被移动后无法再次使用的问题
            match r.text().await {
                Ok(text) => {
                    debug!("get_poi: 响应文本长度={}", text.len());
                    match serde_json::from_str::<PostListResponse>(&text) {
                        Ok(list) => {
                            info!("get_poi: 解析到的项数量={}", list.items.len());
                            let mut stro = stroage.lock().await;
                            let before = stro.poi.len();
                            for (idx, i) in list.items.into_iter().enumerate() {
                                // 将字符串 ID 解析为 u32；解析失败时记录错误并跳过
                                match i.trim().parse::<u32>() {
                                    Ok(id) => {
                                        // 如果该 id 未被 processed_poi/poi 标记，则推入 poi 列表
                                        match stro.processed_poi.contains(&id)
                                            || stro.poi.contains(&id)
                                        {
                                            false => {
                                                stro.poi.push(id);
                                                debug!("get_poi: 推入项 #{} id={}", idx, id);
                                            }
                                            true => {
                                                debug!("get_poi: {:?} 已存在，不在推入", id)
                                            }
                                        };
                                    }
                                    Err(e) => {
                                        error!("get_poi: 在索引 {} 解析 id 失败: {:?}", idx, e);
                                    }
                                }
                            }
                            debug!("get_poi: poi 大小 原={} 现={}", before, stro.poi.len());
                            info!("get_poi: 完成，poi 总数={}", stro.poi.len());
                        }
                        Err(e) => {
                            error!("get_poi: JSON 反序列化失败: {:?}", e);
                            debug!("get_poi: 响应文本={}", text);
                        }
                    }
                }
                Err(e) => {
                    error!("get_poi: 读取响应文本失败: {:?}", e)
                }
            }
        }
        Err(e) => {
            error!("get_poi: 请求失败: {:?}", e)
        }
    };
}
