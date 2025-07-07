use dotenv::dotenv;
use anyhow::Result as AnyhowResult;

pub fn load() -> AnyhowResult<()> {
    dotenv().map_err(|e| anyhow::anyhow!("[ FAILED ] 環境変数の読み込みに失敗しました: {}", e))?;
    Ok(())
}

pub fn get(key: &str) -> AnyhowResult<String> {
    std::env::var(key)
        .map_err(|_| anyhow::anyhow!("[ FAILED ] 環境変数 '{}' が設定されていません", key))
}