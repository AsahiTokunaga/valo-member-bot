use serenity::{
  builder::{CreateButton, CreateEmbed, CreateMessage},
  client::Context,
  model::{
    application::ButtonStyle,
    id::{ChannelId, EmojiId, MessageId},
  },
};
use std::str::FromStr;
use tracing::{Level, instrument};

use crate::dotenv_handler;
use crate::error::BotError;
use crate::handler::colors::PIN_MESSAGE_COLOR;
use crate::valkey::commands;

#[instrument(name = "handler/pin/pin", skip_all, level = Level::INFO, err(level = Level::WARN))]
pub async fn pin(ctx: &Context, channel_id: ChannelId) -> Result<(), BotError> {
  tracing::info!("処理開始");
  let delete_latest_task = delete_latest(ctx);
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

  let send_message_task = channel_id.send_message(&ctx.http, message);
  let (delete_latest, send_message) =
    tokio::join!(delete_latest_task, send_message_task);
  delete_latest?;
  commands::set(&redis_pass, "latest", &send_message?.id.to_string()).await?;
  tracing::info!("処理終了");
  Ok(())
}

#[instrument(name = "handler/pin/delete_latest", skip_all, level = Level::INFO, err(level = Level::WARN))]
async fn delete_latest(ctx: &Context) -> Result<(), BotError> {
  tracing::info!("処理開始");
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
  tracing::info!("処理終了");
  Ok(())
}
