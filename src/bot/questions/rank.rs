use serenity::all::{CacheHttp, CreateEmbed, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, EditInteractionResponse, Http, UserId};

use crate::{bot::{colors::BASE_COLOR, types::{Rank, WebhookDataExt}, Handler}, error::BotError};

impl Handler {
  pub async fn rank<T>(&self, http: T, user: UserId) -> Result<(), BotError> 
  where
    T: AsRef<Http> + CacheHttp + Copy,
  {
    let embed = CreateEmbed::new()
      .title("ランクを選択してください")
      .color(BASE_COLOR);
    let select_menu = CreateSelectMenu::new("ランク選択", CreateSelectMenuKind::String {
      options: vec![
        CreateSelectMenuOption::new(Rank::Unranked.as_str(), Rank::Unranked.as_str()),
        CreateSelectMenuOption::new(Rank::Iron.as_str(), Rank::Iron.as_str()),
        CreateSelectMenuOption::new(Rank::Bronze.as_str(), Rank::Bronze.as_str()),
        CreateSelectMenuOption::new(Rank::Silver.as_str(), Rank::Silver.as_str()),
        CreateSelectMenuOption::new(Rank::Gold.as_str(), Rank::Gold.as_str()),
        CreateSelectMenuOption::new(Rank::Platinum.as_str(), Rank::Platinum.as_str()),
        CreateSelectMenuOption::new(Rank::Diamond.as_str(), Rank::Diamond.as_str()),
        CreateSelectMenuOption::new(Rank::Ascendant.as_str(), Rank::Ascendant.as_str()),
        CreateSelectMenuOption::new(Rank::Immortal.as_str(), Rank::Immortal.as_str()),
        CreateSelectMenuOption::new(Rank::Radiant.as_str(), Rank::Radiant.as_str()),
      ]
    })
    .min_values(1)
    .max_values(1);
    let response = EditInteractionResponse::new()
      .embed(embed)
      .select_menu(select_menu);
    if let Some(comp) = &self.component_store.get(&user) {
      comp.edit_response(http, response).await?;
      return Ok(());
    } else {
      return Err(BotError::ComponentInteractionNotFound);
    }
  }
}