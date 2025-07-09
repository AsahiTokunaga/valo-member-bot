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
        Mode::Unrated.to_string(),
        Mode::Unrated.to_string(),
      ),
      CreateSelectMenuOption::new(
        Mode::Competitive.to_string(),
        Mode::Competitive.to_string(),
      ),
      CreateSelectMenuOption::new(
        Mode::Custom.to_string(),
        Mode::Custom.to_string(),
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
