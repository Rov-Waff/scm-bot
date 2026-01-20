use std::{sync::Arc, time::UNIX_EPOCH};

use log::info;
use redis::{Commands, Connection};
use tokio::sync::Mutex;

pub async fn remove_expr_element(
    redis_cli: Arc<Mutex<Connection>>,
) -> Result<(), redis::RedisError> {
    let mut redis_cli = redis_cli.lock().await;
    let count:i32 = redis_cli.zrembyscore(
        "poi",
        0,
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    )?;
    info!("完成清理poi,数量:{}",count);
    let count:i32 = redis_cli.zrembyscore(
        "processed_poi",
        0,
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    )?;
    info!("完成清理processed_poi,数量:{}",count);

    Ok(())
}
