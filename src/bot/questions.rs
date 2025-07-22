mod member;
mod message;
mod mode;
mod rank;
mod server;

use crate::{
  bot::{types::WebhookData, Handler},
  error::BotError,
};
use serenity::all::{ComponentInteraction, UserId};

// 質問フロー内でデータ作成、編集等に使用するメソッドを実装
impl Handler {
  pub fn create(&self, id: UserId) {
    let data = WebhookData::new(id);
    self.question_state.insert(id, data);
  }
  pub fn set<F: FnMut(&mut WebhookData)>(&self, id: UserId, mut f: F) {
    if let Some(mut data) = self.question_state.get_mut(&id) {
      f(&mut data);
    } else {
      tracing::warn!("No data found for user: {}", id);
    }
  }
  pub fn get_question_state(&self, id: UserId) -> Result<WebhookData, BotError> {
    if let Some(data) = self.question_state.get(&id) {
      Ok(data.clone())
    } else {
      Err(BotError::WebhookDataNotFound)
    }
  }
  pub fn get_component(&self, id: UserId) -> Option<ComponentInteraction> {
    self.component_store.get(&id)
      .map(|comp| comp.clone())
  }
  pub fn remove_temp_data(&self, id: UserId) -> Result<(), BotError> {
    if self.question_state.remove(&id).is_none() {
      return Err(BotError::WebhookDataNotFound);
    }
    if self.component_store.remove(&id).is_none() {
      return Err(BotError::ComponentInteractionNotFound);
    }
    Ok(())
  }
}
