use tokio::sync::{OnceCell, RwLock};

use anyhow::Result as AnyhowResult;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client};

static VALKEY_CONNECTION: OnceCell<RwLock<ConnectionManager>> = OnceCell::const_new();

pub struct Valkey;

impl Valkey {
    fn new(
        redis_pass: &str,
    ) -> impl Future<Output = AnyhowResult<&'static RwLock<ConnectionManager>>> {
        VALKEY_CONNECTION.get_or_try_init(move || async move {
            let client = Client::open(format!("redis://:{}@127.0.0.1/", redis_pass))?;
            let manager = ConnectionManager::new(client).await?;
            Ok(RwLock::new(manager))
        })
    }

    pub async fn ping(redis_pass: &str) -> AnyhowResult<()> {
        let lock = Self::new(redis_pass).await?;
        let mut connection = lock.write().await;
        let pong: String = connection.ping().await?;
        println!("[ OK ] Redisに接続しました: {}", pong);
        Ok(())
    }

    pub fn set<'a>(
        redis_pass: &'a str,
        key: &'a str,
        value: &'a str,
    ) -> impl Future<Output = AnyhowResult<()>> + 'a {
        async move {
            let lock = Self::new(redis_pass).await?;
            let mut connection = lock.write().await;
            connection.set::<&str, &str, ()>(key, value).await?;
            Ok(())
        }
    }

    pub fn get<'a>(
        redis_pass: &'a str,
        key: &'a str,
    ) -> impl Future<Output = AnyhowResult<Option<String>>> + 'a {
        async move {
            let lock = Self::new(redis_pass).await?;
            let mut connection = lock.write().await;
            let value: Option<String> = connection.get(key).await?;
            Ok(value)
        }
    }

    pub fn ttl_set<'a>(
        redis_pass: &'a str,
        key: &'a str,
        val: &'a str,
        ttl: u64,
    ) -> impl Future<Output = AnyhowResult<()>> + 'a {
        async move {
            let lock = Self::new(redis_pass).await?;
            let mut connection = lock.write().await;
            connection.set_ex::<&str, &str, ()>(key, val, ttl).await?;
           Ok(())
        }
    }
}
