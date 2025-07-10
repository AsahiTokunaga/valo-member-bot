use serenity::builder::{
  CreateEmbed, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
  EditInteractionResponse,
};

use crate::handler::{colors::BASE_COLOR, state::Mode};

pub fn q_match() -> EditInteractionResponse {
  let embed = CreateEmbed::new()
    .colour(BASE_COLOR)
    .title("募集形式を選択してください");
  let select_menu_kind = CreateSelectMenuKind::String {
    options: vec![
      CreateSelectMenuOption::new(
        Mode::Unrated.as_str(),
        Mode::Unrated.as_str(),
      ),
      CreateSelectMenuOption::new(
        Mode::Competitive.as_str(),
        Mode::Competitive.as_str(),
      ),
      CreateSelectMenuOption::new(
        Mode::Custom.as_str(),
        Mode::Custom.as_str(),
      ),
    ],
  };
  let select_menu = CreateSelectMenu::new("募集形式を選択", select_menu_kind)
    .placeholder("募集形式を選択してください")
    .min_values(1)
    .max_values(1);
  EditInteractionResponse::new()
    .embed(embed)
    .select_menu(select_menu)
}
