use serenity::prelude::*;
use serenity::Client as SerenityClient;
use dotenv::dotenv;
use anyhow::{
    Result as AnyhowResult,
    Context
};

mod handler;
use handler::Handler;
mod valkey;
use valkey::ValkeyModules;

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    dotenv().ok();
    let redis_pass = dotenv::var("REDIS_PASS")
        .context("[ FAILED ] Redisのパスワードが設定されていません")?;
    let _connection = ValkeyModules::new(redis_pass).await
        .context("[ FAILED ] Redisの接続に失敗しました")?;
    let token = dotenv::var("TOKEN")
        .context("[ FAILED ] トークンが設定されていません")?;
    let intents = 
        GatewayIntents::GUILD_MESSAGES |
        GatewayIntents::MESSAGE_CONTENT;
    let mut client = SerenityClient::builder(&token, intents)
        .event_handler(Handler).await
        .context("[ FAILED ] botの起動に失敗しました")?;

    client.start().await.context("[ FAILED ] botの起動に失敗しました")?;
    Ok(())
}
