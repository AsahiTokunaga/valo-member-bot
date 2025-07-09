use std::num::ParseIntError;

use redis::RedisError;
use thiserror::Error;
use tracing::subscriber::SetGlobalDefaultError;

#[derive(Debug, Error)]
pub enum BotError {
  #[error("[BotError::AppStateError] {0}")]
  AppStateError(#[from] AppStateError),
  #[error("[BotError::ValkeyError] {0}")]
  ValkeyError(#[from] RedisError),
  #[error("[BotError::DotEnvError] {0}")]
  DotEnvError(#[from] dotenv::Error),
  #[error("[BotError::SerenityError] {0}")]
  SerenityError(#[from] serenity::Error),
  #[error("[BotError::PinMessageError] {0}")]
  PinMessageError(#[from] ParseIntError),
  #[error("[BotError::TracingError] {0}")]
  TracingError(#[from] SetGlobalDefaultError),
  #[error("[BotError::InvalidInput] I'm a teapot")]
  InvalidInput,
}

#[derive(Debug, Error)]
pub enum AppStateError {
  #[error(
    "[AppStateError::WebhookMapWithMuteError] WebhookDataが見つかりません"
  )]
  WebhookDataNotFound,
  #[error(
    "[AppStateError::InteractionIdNotFound] InteractionIdが見つかりません"
  )]
  InteractionIdNotFound,
  #[error(
    "[AppStateError::ComponentInteractionNotFound] ComponentInteractionが見つかりません"
  )]
  ComponentInteractionNotFound,
}

#[macro_export]
macro_rules! log_error {
    ($error:expr, $msg:expr) => {
        tracing::error!(error = %$error, "{} ({}: Ln {})", $msg, file!(), line!());
    };
}