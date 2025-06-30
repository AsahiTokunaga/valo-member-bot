use anyhow::{Context, Result as AnyhowResult};
use dotenv::dotenv;
use serenity::Client as SerenityClient;
use serenity::prelude::*;

mod handler;
use handler::Handler;
mod valkey;
use valkey::Valkey;
mod dotenv_handler;

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    dotenv().context("[ FAILED ] .envファイルの読み込みに失敗しました")?;
    let redis_pass = dotenv_handler::get("REDIS_PASS")?;
    Valkey::ping(&redis_pass).await?;
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
