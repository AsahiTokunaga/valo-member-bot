mod bot;
mod config;
mod error;

use std::{collections::HashMap,sync::Arc};

use error::BotError;
use bot::Handler;
use serenity::all::GatewayIntents;
use tokio::sync::Mutex;
use tracing::{Level, instrument};
use tracing_subscriber::fmt::time::FormatTime;

use crate::bot::types::RedisClient;

#[tokio::main(flavor = "multi_thread")]
#[instrument(name = "main", err)]
async fn main() -> Result<(), BotError> {
  let logger = tracing_subscriber::fmt::Subscriber::builder()
    .with_max_level(Level::INFO)
    .with_timer(JapanStandardTime)
    .finish();
  tracing::subscriber::set_global_default(logger)?;
  config::load()?;
  let token = config::get("TOKEN")?;
  let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
  let handler = Handler {
    question_state: Arc::new(Mutex::new(HashMap::new())),
    component_store: Arc::new(Mutex::new(HashMap::new())),
    redis_client: Arc::new(Mutex::new(RedisClient::new(&config::get("REDIS_PASS")?).await?)),
  };
  let mut client = serenity::Client::builder(token, intents)
    .event_handler_arc(Arc::new(handler))
    .await?;
  client.start().await?;
  Ok(())
}

struct JapanStandardTime;

impl FormatTime for JapanStandardTime {
  fn format_time(
    &self,
    w: &mut tracing_subscriber::fmt::format::Writer<'_>,
  ) -> std::fmt::Result {
    let utc_now = chrono::Utc::now();
    let jst_now = utc_now + chrono::Duration::hours(9);
    write!(w, "{}", jst_now.format("%Y-%m-%d %H:%M:%S"))
  }
}
