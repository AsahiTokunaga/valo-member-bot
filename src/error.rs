use std::num::ParseIntError;

use redis::RedisError;
use thiserror::Error;
use tracing::subscriber::SetGlobalDefaultError;

#[derive(Debug, Error)]
pub enum BotError {
  #[error("[BotError::DbError] {0}")]
  DbError(#[from] RedisError),
  #[error("[BotError::ConfigError] {0}")]
  ConfigError(#[from] dotenv::Error),
  #[error("[BotError::SerenityError] {0}")]
  SerenityError(#[from] serenity::Error),
  #[error("[BotError::PinMessageError] {0}")]
  PinMessageError(#[from] ParseIntError),
  #[error("[BotError::TracingError] {0}")]
  TracingError(#[from] SetGlobalDefaultError),
  #[error("[BotError::WebhookDataNotFound] WebhookDataが見つかりません")]
  WebhookDataNotFound,
  #[error("[BotError::ComponentInteractionNotFound] コンポーネントが見つかりません")]
  ComponentInteractionNotFound,
  #[error("[BotError::EmbedBroken] Embedが壊れています {0}")]
  EmbedBroken(&'static str),
}
