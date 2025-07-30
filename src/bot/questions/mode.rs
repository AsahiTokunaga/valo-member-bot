use std::sync::Arc;

use serenity::all::{
  CreateEmbed, CreateSelectMenu, CreateSelectMenuKind, UserId,
  CreateSelectMenuOption, EditInteractionResponse, Http
};

use crate::{
  bot::{
    colors::BASE_COLOR,
    types::{Mode, WebhookDataExt},
    Handler
  },
  error::BotError
};

impl Handler {
  pub async fn mode(&self, http: Arc<Http>, user: UserId) -> Result<(), BotError> {
    let embed = CreateEmbed::new()
      .title("モードを選択してください")
      .color(BASE_COLOR);
    let select_menu = CreateSelectMenu::new("モード選択", CreateSelectMenuKind::String {
      options: vec![
        CreateSelectMenuOption::new(Mode::Unrated.as_str(), Mode::Unrated.as_str()),
        CreateSelectMenuOption::new(Mode::Competitive.as_str(), Mode::Competitive.as_str()),
        CreateSelectMenuOption::new(Mode::Custom.as_str(), Mode::Custom.as_str()),
      ]
    })
    .min_values(1)
    .max_values(1);
    let response = EditInteractionResponse::new()
      .embed(embed)
      .select_menu(select_menu);
    if let Some(comp) = self.component_store.get(&user) {
      comp.edit_response(http, response).await?;
      Ok(())
    } else {
      Err(BotError::ComponentInteractionNotFound)
    }
  }
}