use serenity::all::{CacheHttp, CreateEmbed, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, EditInteractionResponse, Http, UserId};

use crate::{bot::{colors::BASE_COLOR, types::{Mode, WebhookDataExt}, Handler}, error::BotError};

impl Handler {
  pub async fn mode<T>(&self, http: T, user: UserId) -> Result<(), BotError>
  where
    T: AsRef<Http> + CacheHttp + Copy,
  {
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
    if let Some(comp) = &self.component_store.get(&user) {
      comp.edit_response(http, response).await?;
      Ok(())
    } else {
      Err(BotError::ComponentInteractionNotFound)
    }
  }
}