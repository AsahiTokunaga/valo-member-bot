use std::collections::HashMap;

use dotenv::dotenv;
use once_cell::sync::Lazy;
use tokio::sync::RwLock;
use anyhow::Result as AnyhowResult;

static ENV_VERS: Lazy<RwLock<HashMap<String, String>>> = Lazy::new(|| {
    let mut vars = HashMap::new();
    if dotenv().is_ok() {
        vars.extend(std::env::vars());
    }
    RwLock::new(vars)
});

pub async fn get(key: &str) -> AnyhowResult<String> {
    let vars = ENV_VERS.try_read()?;
    vars.get(key).cloned().ok_or_else(|| {
        anyhow::anyhow!(
            "[ FAILED ] 環境変数 '{}' が設定されていません",
            key
        )
    })
}