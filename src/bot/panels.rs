use serenity::all::{
  ButtonStyle, CacheHttp, ComponentInteraction, CreateActionRow, CreateButton,
  CreateInteractionResponse, CreateInteractionResponseMessage, Http, ReactionType
};

mod send;
mod edit;
mod entry;
mod delete;

pub use entry::entry;
pub use send::send;
pub use edit::edit;
pub use delete::delete;

use crate::{bot::types::{Rank, RedisClient}, config, error::BotError};

pub fn get_button(join_disable: bool) -> CreateActionRow {
  let buttons = vec![
    CreateButton::new("参加する")
      .label("参加する")
      .style(ButtonStyle::Secondary)
      .emoji(ReactionType::Unicode("✋".to_string()))
      .disabled(join_disable),
    CreateButton::new("参加をやめる")
      .label("参加をやめる")
      .style(ButtonStyle::Secondary)
      .emoji(ReactionType::Unicode("👋".to_string()))
      .disabled(join_disable),
    CreateButton::new("削除")
      .label("削除")
      .style(ButtonStyle::Secondary)
      .emoji(ReactionType::Unicode("🚫".to_string())),
  ];
  CreateActionRow::Buttons(buttons)
}

pub fn get_thumbnail(rank: Option<Rank>) -> Result<String, BotError> {
  let base_url = config::get("BASE_IMG_URL")?;
  match rank {
    Some(Rank::Radiant) => Ok(format!("{}radiant.png", base_url)),
    Some(Rank::Immortal) => Ok(format!("{}immortal.png", base_url)),
    Some(Rank::Ascendant) => Ok(format!("{}ascendant.png", base_url)),
    Some(Rank::Diamond) => Ok(format!("{}diamond.png", base_url)),
    Some(Rank::Platinum) => Ok(format!("{}platinum.png", base_url)),
    Some(Rank::Gold) => Ok(format!("{}gold.png", base_url)),
    Some(Rank::Silver) => Ok(format!("{}silver.png", base_url)),
    Some(Rank::Bronze) => Ok(format!("{}bronze.png", base_url)),
    Some(Rank::Iron) => Ok(format!("{}iron.png", base_url)),
    _ => Ok(format!("{}unrated.png", base_url)),
  }
}

pub async fn handle_expired<T>(http: T, component: &ComponentInteraction, redis_client: &RedisClient)
where
  T: AsRef<Http> + CacheHttp + Copy,
{
  component.create_response(http, CreateInteractionResponse::Message(
    CreateInteractionResponseMessage::new()
      .content("期限切れの募集のため削除します。")
      .ephemeral(true)
    ))
    .await
    .map_err(|e| tracing::warn!(error = %e, "Failed to create join response"))
    .ok();
  self::delete(http, redis_client, component.message.id).await
    .map_err(|e| tracing::warn!(error = %e, "Failed to delete expired panel"))
    .ok();
}
