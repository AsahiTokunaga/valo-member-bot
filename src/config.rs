use dotenv::dotenv;

use crate::BotError;

pub fn load() -> Result<(), BotError> {
    dotenv().map_err(|e| BotError::ConfigError(e))?;
    Ok(())
}

pub fn get(key: &str) -> Result<String, BotError> {
    let value = dotenv::var(key)?;
    Ok(value)
}
