use std::sync::Arc;

use serenity::all::{
  CreateEmbed, CreateSelectMenu, CreateSelectMenuKind, UserId,
  CreateSelectMenuOption, EditInteractionResponse, Http
};

use crate::{
  bot::{
    colors::BASE_COLOR,
    types::{Member, Mode, WebhookDataExt},
    Handler
  },
  error::BotError
};

impl Handler {
  pub async fn member(&self, http: Arc<Http>, user: UserId, mode: &str) -> Result<(), BotError> {
    let embed = CreateEmbed::new()
      .title("人数を選択してください")
      .color(BASE_COLOR);
    let mut select_menu_options = vec![
      CreateSelectMenuOption::new(Member::Duo.as_str(), Member::Duo.as_str()),
      CreateSelectMenuOption::new(Member::Trio.as_str(), Member::Trio.as_str()),
      CreateSelectMenuOption::new(Member::FullParty.as_str(), Member::FullParty.as_str()),
    ];
    if mode != Mode::Competitive.as_str() {
      select_menu_options.insert(2, CreateSelectMenuOption::new(Member::Quad.as_str(), Member::Quad.as_str()));
    }
    if mode == Mode::Custom.as_str() {
      select_menu_options.push(CreateSelectMenuOption::new(Member::Six.as_str(), Member::Six.as_str()));
      select_menu_options.push(CreateSelectMenuOption::new(Member::Seven.as_str(), Member::Seven.as_str()));
      select_menu_options.push(CreateSelectMenuOption::new(Member::Eight.as_str(), Member::Eight.as_str()));
      select_menu_options.push(CreateSelectMenuOption::new(Member::Nine.as_str(), Member::Nine.as_str()));
      select_menu_options.push(CreateSelectMenuOption::new(Member::Ten.as_str(), Member::Ten.as_str()));
    }
    let select_menu = CreateSelectMenu::new("人数選択", CreateSelectMenuKind::String {
      options: select_menu_options
    })
    .min_values(1)
    .max_values(1);
    let response = EditInteractionResponse::new()
      .embed(embed)
      .select_menu(select_menu);
    if let Some(comp) = self.component_store.get(&user) {
      comp.edit_response(http, response).await?;
      return Ok(());
    } else {
      return Err(BotError::ComponentInteractionNotFound);
    }
  }
}