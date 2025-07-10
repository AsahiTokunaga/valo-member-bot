use serenity::Client as SerenityClient;
use serenity::prelude::*;
use tracing_subscriber::fmt::time::FormatTime;

mod handler;
use handler::Handler;
mod valkey;
use tracing::Level;
use tracing::instrument;
use tracing_subscriber::FmtSubscriber;
use valkey::commands;

use crate::error::BotError;
mod dotenv_handler;
mod error;

#[tokio::main(flavor = "multi_thread")]
#[instrument(name = "main", err(level = Level::ERROR))]
async fn main() -> Result<(), BotError> {
  let subscriber = FmtSubscriber::builder()
    .with_max_level(Level::INFO)
    .with_timer(JST)
    .finish();
  tracing::subscriber::set_global_default(subscriber)?;
  dotenv_handler::load()?;
  let redis_pass = dotenv_handler::get("REDIS_PASS")?;
  commands::ping(&redis_pass).await?;
  let token = dotenv_handler::get("TOKEN")?;
  let intents =
    GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
  let mut client = SerenityClient::builder(&token, intents)
    .event_handler(Handler)
    .await?;
  client.start().await?;
  Ok(())
}

struct JST;

impl FormatTime for JST {
  fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
      let utc_now = chrono::Utc::now();
      let jst_now = utc_now + chrono::Duration::hours(9);
      write!(w, "{}", jst_now.format("%Y-%m-%d %H:%M:%S"))
  }
}