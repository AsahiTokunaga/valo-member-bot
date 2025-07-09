use serenity::all::CreateEmbed;
use serenity::builder::EditWebhookMessage;
use serenity::client::Context as SerenityContext;
use serenity::model::channel::Message;
use serenity::model::id::MessageId;
use serenity::model::webhook::Webhook;
use tracing::{Level, instrument};

use crate::dotenv_handler;
use crate::error::BotError;
use crate::valkey::commands;

#[instrument(name = "handler/webhook_edit/edit", skip_all, level = Level::INFO, err(level = Level::ERROR) fields(message_id = %message_id))]
pub async fn edit(
  ctx: &SerenityContext,
  message_id: MessageId,
  field_value: &str,
  title: (usize, u8),
) -> Result<(), BotError> {
  tracing::info!("処理開始");
  let redis_pass = dotenv_handler::get("REDIS_PASS")?;
  let channel_id = dotenv_handler::get("CHANNEL_ID")?;
  let webhook_url = commands::get(&redis_pass, &channel_id).await?.unwrap();
  let webhook = Webhook::from_url(&ctx.http, &webhook_url).await?;
  let prev = webhook.get_message(&ctx.http, None, message_id).await?;
  let wh_message =
    EditWebhookMessage::new().embed(get_embed(prev, field_value, title));
  webhook
    .edit_message(&ctx.http, message_id, wh_message)
    .await?;
  tracing::info!("処理終了");
  Ok(())
}

fn get_embed(
  message: Message,
  field_value: &str,
  title: (usize, u8),
) -> CreateEmbed {
  let embed = message.embeds.first().unwrap();
  let field = embed.fields.first().unwrap();
  CreateEmbed::new()
    .title(format!("({}/{})", title.0, title.1))
    .description(embed.description.as_deref().unwrap())
    .color(embed.colour.unwrap())
    .thumbnail(embed.thumbnail.as_ref().unwrap().url.as_str())
    .field(&field.name, field_value, false)
}
