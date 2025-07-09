use dotenv::dotenv;
use tracing::instrument;

use crate::error::BotError;

#[instrument(name = "dotenv_handler/load", skip_all)]
pub fn load() -> Result<(), BotError> {
  dotenv()?;
  Ok(())
}

#[instrument(name = "dotenv_handler/get", skip_all, fields(key = %key))]
pub fn get(key: &str) -> Result<String, BotError> {
  let value = dotenv::var(key)?;
  Ok(value)
}
