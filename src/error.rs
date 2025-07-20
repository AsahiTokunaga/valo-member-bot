use std::num::ParseIntError;

use deadpool_redis::PoolError;
use redis::RedisError;
use thiserror::Error;
use tracing::subscriber::SetGlobalDefaultError;

#[derive(Debug, Error)]
pub enum BotError {
  #[error("[BotError::DbError] {0}")]
  DbError(#[from] DbError),
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

#[derive(Debug, Error)]
pub enum DbError {
  #[error("[DbError::PoolError] {0}")]
  PoolError(#[from] PoolError),
  #[error("[DbError::RedisError] {0}")]
  RedisError(#[from] RedisError),
}