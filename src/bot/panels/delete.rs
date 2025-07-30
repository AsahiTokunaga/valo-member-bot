use std::sync::Arc;

use serenity::all::{Http, MessageId};

use crate::{bot::types::RedisClient, error::BotError};

pub async fn delete(http: Arc<Http>, redis_client: Arc<RedisClient>, message: MessageId) -> Result<(), BotError> {
  let webhook = Arc::new(redis_client.get_webhook(http.clone()).await?);
  tokio::spawn(async move {
    if let Err(e) = webhook.delete_message(http, None, message).await {
      tracing::warn!("Failed to delete message: {}", e);
    }
  });
  Ok(())
}
