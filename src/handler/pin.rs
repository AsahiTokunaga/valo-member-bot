use anyhow::Result as AnyhowResult;
use serenity::builder::{CreateButton, CreateEmbed, CreateMessage};
use serenity::client::Context as SerenityContext;
use serenity::model::application::ButtonStyle;
use serenity::model::channel::Message;
use serenity::model::id::ChannelId;
use serenity::model::id::EmojiId;
use serenity::model::id::MessageId;
use std::str::FromStr;

use crate::dotenv_handler;
use crate::handler::colors::PIN_MESSAGE_COLOR;
use crate::valkey::commands;

pub async fn pin(ctx: SerenityContext, msg: &Message) -> AnyhowResult<()> {
    let delete_latest_task = delete_latest(&ctx);
    let embed = CreateEmbed::new()
        .color(PIN_MESSAGE_COLOR)
        .description("# ここから募集作成！\nサーバーのみんなとVALORANTをするために、下のボタンを押すとアンレートやコンペティティブ、カスタムの募集を作成することが出来ます！");
    let button = CreateButton::new("募集を作成")
        .label("募集を作成")
        .style(ButtonStyle::Secondary)
        .emoji(EmojiId::new(
            dotenv_handler::get("PLUS_EMOJI")?.parse::<u64>()?,
        ));
    let message = CreateMessage::new().embed(embed).button(button);
    let redis_pass = dotenv_handler::get("REDIS_PASS")?;

    let send_message_task = msg.channel_id.send_message(&ctx.http, message);
    let (delete_latest, send_message) = tokio::join!(delete_latest_task, send_message_task);
    delete_latest?;
    commands::set(&redis_pass, "latest", &send_message?.id.to_string()).await?;
    Ok(())
}

async fn delete_latest(ctx: &SerenityContext) -> AnyhowResult<()> {
    let redis_pass = dotenv_handler::get("REDIS_PASS")?;
    let channel_id = ChannelId::from_str(&dotenv_handler::get("CHANNEL_ID")?)?;
    let get_latest_id = commands::get(&redis_pass, "latest").await;
    if let Ok(latest_id) = get_latest_id {
        if let Some(latest_id) = latest_id {
            let message_id = MessageId::from_str(latest_id.as_str());
            if let Ok(message_id) = message_id {
                let get_message_task = channel_id.message(&ctx.http, message_id);
                get_message_task.await?.delete(&ctx.http).await?;
            }
        }
    }
    Ok(())
}
