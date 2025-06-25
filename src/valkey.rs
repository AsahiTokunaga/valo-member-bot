use tokio::sync::{Mutex, OnceCell};

use anyhow::{Context, Result as AnyhowResult};
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client};

static VALKEY_CONNECTION: OnceCell<Mutex<ConnectionManager>> = OnceCell::const_new();

pub struct ValkeyModules;

impl ValkeyModules {
    async fn new(redis_pass: String) -> AnyhowResult<&'static Mutex<ConnectionManager>> {
        let client = Client::open(format!("redis://:{}@127.0.0.1/", redis_pass))
            .context("[ FAILED ] Redisのクライアントの作成に失敗しました")?;
        let result = VALKEY_CONNECTION
            .get_or_init(|| async {
                let manager = ConnectionManager::new(client)
                    .await
                    .expect("[ FAILED ] Redisの接続に失敗しました");
                Mutex::new(manager)
            })
            .await;
        Ok(result)
    }

    pub async fn ping(redis_pass: String) -> AnyhowResult<()> {
        let mut connection = Self::new(redis_pass).await?.lock().await;
        let pong = connection
            .ping::<String>()
            .await
            .context("[ FAILED ] pingに失敗しました")?;
        println!("[ OK ] Redisに接続しました: {}", pong);
        Ok(())
    }

    pub async fn set(redis_pass: String, key: &str, value: &str) -> AnyhowResult<()> {
        let mut connection = Self::new(redis_pass).await?.lock().await;
        connection
            .set::<&str, &str, ()>(key, value)
            .await
            .context("[ FAILED ] Redisに値を設定できませんでした")?;
        Ok(())
    }

    pub async fn _get(redis_pass: String, key: &str) -> AnyhowResult<Option<String>> {
        let mut connection = Self::new(redis_pass).await?.lock().await;
        let value: Option<String> = connection
            .get(key)
            .await
            .context("[ FAILED ] Redisから値を取得できませんでした")?;
        Ok(value)
    }
}
