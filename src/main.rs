use anyhow::{Context, Result as AnyhowResult};
use serenity::Client as SerenityClient;
use serenity::prelude::*;

mod handler;
use handler::Handler;
mod valkey;
use valkey::commands;
mod dotenv_handler;
mod state_handler;

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    dotenv_handler::load()?;
    let redis_pass = dotenv_handler::get("REDIS_PASS")?;
    commands::ping(&redis_pass).await?;
    let token = dotenv_handler::get("TOKEN")?;
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = SerenityClient::builder(&token, intents)
        .event_handler(Handler)
        .await
        .context("[ FAILED ] botの起動に失敗しました")?;

    client
        .start()
        .await
        .context("[ FAILED ] botの起動に失敗しました")?;
    Ok(())
}
