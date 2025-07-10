pub mod commands {

  use redis::{AsyncCommands, Client, aio::ConnectionManager};
  use tracing::{Level, instrument};

  use crate::error::BotError;

  #[instrument(name = "valkey/commands/new", err, level = Level::WARN, skip_all)]
  async fn new(redis_pass: &str) -> Result<ConnectionManager, BotError> {
    let client = Client::open(format!("redis://:{}@127.0.0.1/", redis_pass))?;
    let manager = ConnectionManager::new(client).await?;
    Ok(manager)
  }

  #[instrument(name = "valkey/commands/ping", level = Level::INFO, err(level = Level::WARN), skip_all)]
  pub async fn ping(redis_pass: &str) -> Result<(), BotError> {
    let mut connection = new(redis_pass).await?;
    let pong: String = connection.ping().await?;
    tracing::info!("Redisに接続しました: {}", pong);
    Ok(())
  }

  #[instrument(name = "valkey/commands/set", level = Level::INFO, err(level = Level::WARN), skip_all, fields(key = %key, value = %value))]
  pub async fn set(
    redis_pass: &str,
    key: &str,
    value: &str,
  ) -> Result<(), BotError> {
    let mut connection = new(redis_pass).await?;
    connection.set::<&str, &str, ()>(key, value).await?;
    tracing::info!("Redisに値をセットしました");
    Ok(())
  }

  #[instrument(name = "valkey/commands/get", level = Level::INFO, err(level = Level::WARN), skip_all, fields(key = %key))]
  pub async fn get(
    redis_pass: &str,
    key: &str,
  ) -> Result<Option<String>, BotError> {
    let mut connection = new(redis_pass).await?;
    let value: Option<String> = connection.get(key).await?;
    Ok(value)
  }

  #[instrument(name = "valkey/commands/ttl_set", level = Level::INFO, err(level = Level::WARN), skip_all, fields(key = %key, value = %value, ttl))]
  pub async fn ttl_set(
    redis_pass: &str,
    key: &str,
    value: &str,
    ttl: u64,
  ) -> Result<(), BotError> {
    let mut connection = new(redis_pass).await?;
    connection.set_ex::<&str, &str, ()>(key, value, ttl).await?;
    Ok(())
  }

  pub async fn del(
    redis_pass: &str,
    key: &str,
  ) -> Result<(), BotError> {
    let mut connection = new(redis_pass).await?;
    connection.del::<&str, ()>(key).await?;
    Ok(())
  }
}
