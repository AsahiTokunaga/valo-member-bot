pub mod commands {
    use anyhow::Result as AnyhowResult;
    use redis::{aio::ConnectionManager, AsyncCommands, Client};
    async fn new(redis_pass: &str) -> AnyhowResult<ConnectionManager> {
        let client = Client::open(format!("redis://:{}@127.0.0.1/", redis_pass))?;
        let manager = ConnectionManager::new(client).await?;
        Ok(manager)
    }
    pub async fn ping(redis_pass: &str) -> AnyhowResult<()> {
        let mut connection = new(redis_pass).await?;
        let pong: String = connection.ping().await?;
        println!("[ OK ] Redisに接続しました: {}", pong);
        Ok(())
    }
    pub async fn set(redis_pass: &str, key: &str, value: &str) -> AnyhowResult<()> {
        let mut connection = new(redis_pass).await?;
        connection.set::<&str, &str, ()>(key, value).await?;
        Ok(())
    }
    pub async fn get(redis_pass: &str, key: &str) -> AnyhowResult<Option<String>> {
        let mut connection = new(redis_pass).await?;
        let value: Option<String> = connection.get(key).await?;
        Ok(value)
    }
    pub async fn ttl_set(redis_pass: &str, key: &str, value: &str, ttl: u64) -> AnyhowResult<()> {
        let mut connection = new(redis_pass).await?;
        connection.set_ex::<&str, &str, ()>(key, value, ttl).await?;
        Ok(())
    }
}
