use std::str::FromStr;
use anyhow::{Context, Result as AnyhowResult};
use serenity::builder::{CreateButton, CreateEmbed, CreateMessage};
use serenity::client::Context as SerenityContext;
use serenity::model::application::ButtonStyle;
use serenity::model::channel::Message;
use serenity::model::id::ChannelId;
use serenity::model::id::EmojiId;
use serenity::model::id::MessageId;

use crate::dotenv_handler;
use crate::valkey::Valkey;

pub async fn pin(ctx: SerenityContext, msg: &Message) -> AnyhowResult<()> {
    delete_latest(&ctx).await?;
    let embed = CreateEmbed::new()
        .colour(16777215)
        .description("# ここから募集作成！\nサーバーのみんなとVALORANTをするために、下のボタンを押すとアンレートやコンペティティブ、カスタムの募集を作成することが出来ます！");
    let button = CreateButton::new("募集を作成")
        .label("募集を作成")
        .style(ButtonStyle::Secondary)
        .emoji(EmojiId::new(dotenv_handler::get("PLUS_EMOJI").await?.parse::<u64>()?));
    let message = CreateMessage::new().embed(embed).button(button);

    let redis_pass = dotenv_handler::get("REDIS_PASS").await?;
    let res = msg.channel_id.send_message(&ctx.http, message).await?;
    let res_id = res.id.to_string();

    Valkey::set(&redis_pass, "latest", &res_id).await?;
    Ok(())
}

async fn delete_latest(ctx: &SerenityContext) -> AnyhowResult<()> {
    let redis_pass =
        dotenv_handler::get("REDIS_PASS").await?;
    if let Some(latest_id) = Valkey::get(&redis_pass, "latest")
        .await
        .context("[ FAILED ] Redisから最新のメッセージIDを取得できませんでした")?
    {
        let channel_id = ChannelId::from_str(&dotenv_handler::get("CHANNEL_ID").await?)?;
        let message_id = MessageId::from_str(&latest_id)?;
        if let Ok(message) = channel_id.message(&ctx.http, message_id).await {
            message.delete(&ctx.http).await?;
        }
    }
    Ok(())
}
