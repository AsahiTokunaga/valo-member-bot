mod join;
mod leave;
mod delete;

pub use join::join;
pub use leave::leave;
pub use delete::delete;
pub use join::JoinResponse;
pub use leave::LeaveResponse;
pub use delete::DeleteResponse;

use serenity::all::{CacheHttp, ComponentInteraction, CreateInteractionResponse, CreateInteractionResponseMessage, Http};
use crate::bot::{panels, types::RedisClient};

pub async fn expired<T>(http: T, component: &ComponentInteraction, redis_client: &mut RedisClient)
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
  panels::delete(http, redis_client, component.message.id).await
    .map_err(|e| tracing::warn!(error = %e, "Failed to delete expired panel"))
    .ok();
}
