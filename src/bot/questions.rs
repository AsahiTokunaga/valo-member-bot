mod member;
mod message;
mod mode;
mod rank;
mod server;

use crate::{
  bot::{types::{RedisClient, WebhookData}, Handler},
  error::BotError,
};
use serenity::all::{ComponentInteraction, UserId};

// 質問フロー内でデータ作成、編集等に使用するメソッドを実装
impl Handler {
  pub async fn create(&self, id: UserId) {
    let mut lock = self.question_state.lock().await;
    let data = WebhookData::new(id);
    lock.insert(id, data);
  }
  pub async fn set<F: FnMut(&mut WebhookData)>(&self, id: UserId, mut f: F) {
    let mut lock = self.question_state.lock().await;
    if let Some(data) = lock.get_mut(&id) {
      f(data);
    } else {
      tracing::warn!("No data found for user: {}", id);
    }
  }
  pub async fn get_question_state(&self, id: UserId) -> Result<WebhookData, BotError> {
    let lock = self.question_state.lock().await;
    if let Some(data) = lock.get(&id) {
      Ok(data.clone())
    } else {
      Err(BotError::WebhookDataNotFound)
    }
  }
  pub async fn get_component(&self, id: UserId) -> Option<ComponentInteraction> {
    let lock = self.component_store.lock().await;
    lock.get(&id).cloned()
  }
  pub async fn get_redis_client(&self) -> RedisClient {
    let lock = self.redis_client.lock().await;
    lock.clone()
  }
  pub async fn remove_temp_data(&self, id: UserId) -> Result<(), BotError> {
    let mut lock = self.question_state.lock().await;
    if lock.remove(&id).is_none() {
      drop(lock);
      return Err(BotError::WebhookDataNotFound);
    }
    let mut lock = self.component_store.lock().await;
    if lock.remove(&id).is_none() {
      drop(lock);
      return Err(BotError::ComponentInteractionNotFound);
    }
    Ok(())
  }
}
