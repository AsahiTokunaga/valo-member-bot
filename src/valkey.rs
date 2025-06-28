use tokio::sync::{OnceCell, RwLock};

use anyhow::{Context, Result as AnyhowResult};
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client};

static VALKEY_CONNECTION: OnceCell<RwLock<ConnectionManager>> = OnceCell::const_new();

pub struct Valkey;

impl Valkey {
    async fn new(redis_pass: &str) -> AnyhowResult<&'static RwLock<ConnectionManager>> {
        let client = Client::open(format!("redis://:{}@127.0.0.1/", redis_pass))
            .context("[ FAILED ] Redisのクライアントの作成に失敗しました")?;
        let result = VALKEY_CONNECTION
            .get_or_init(|| async {
                let manager = ConnectionManager::new(client)
                    .await
                    .expect("[ FAILED ] Redisの接続に失敗しました");
                RwLock::new(manager)
            })
            .await;
        Ok(result)
    }

    pub async fn ping(redis_pass: &str) -> AnyhowResult<()> {
        let mut connection = Self::new(redis_pass).await?.try_write()?;
        let pong = connection
            .ping::<String>()
            .await
            .context("[ FAILED ] pingに失敗しました")?;
        println!("[ OK ] Redisに接続しました: {}", pong);
        Ok(())
    }

    pub async fn set(redis_pass: &str, key: &str, value: &str) -> AnyhowResult<()> {
        let mut connection = Self::new(redis_pass).await?.try_write()?;
        connection
            .set::<&str, &str, ()>(key, value)
            .await
            .context("[ FAILED ] Redisに値を設定できませんでした")?;
        Ok(())
    }

    pub async fn get(redis_pass: &str, key: &str) -> AnyhowResult<Option<String>> {
        let mut connection = Self::new(redis_pass).await?.try_write()?;
        let value: Option<String> = connection
            .get(key)
            .await
            .context("[ FAILED ] Redisから値を取得できませんでした")?;
        Ok(value)
    }
}
