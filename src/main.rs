use serenity::prelude::*;
use dotenv::dotenv;

mod handler;
use handler::Handler;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = dotenv::var("TOKEN").expect("[ FAILED ] トークンが設定されていません");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents).event_handler(Handler).await.expect("[ FAILED ] botの起動に失敗しました");

    if let Err(e) = client.start().await {
        println!("[ FAILED ] botの起動に失敗しました: {}", e);
    }
}
