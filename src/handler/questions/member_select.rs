use std::vec;

use serenity::builder::{
  CreateEmbed, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
  EditInteractionResponse,
};

use crate::handler::{
  colors::BASE_COLOR,
  state::{
    MaxMember::{Duo, Eight, Five, Nine, Quad, Seven, Six, Ten, Trio},
    Mode::{self, Competitive, Custom, Unrated},
  },
};

pub fn member(mode: Mode) -> EditInteractionResponse {
  let embed = CreateEmbed::new()
    .colour(BASE_COLOR)
    .title("募集人数を選択してください");
  let select_menu_vec = match mode {
    Unrated => {
      vec![
        CreateSelectMenuOption::new(Duo.as_str(), Duo.as_str()),
        CreateSelectMenuOption::new(Trio.as_str(), Trio.as_str()),
        CreateSelectMenuOption::new(Quad.as_str(), Quad.as_str()),
        CreateSelectMenuOption::new(Five.as_str(), Five.as_str()),
      ]
    }
    Competitive => {
      vec![
        CreateSelectMenuOption::new(Duo.as_str(), Duo.as_str()),
        CreateSelectMenuOption::new(Trio.as_str(), Trio.as_str()),
        CreateSelectMenuOption::new(Five.as_str(), Five.as_str()),
      ]
    }
    Custom => {
      vec![
        CreateSelectMenuOption::new(Duo.as_str(), Duo.as_str()),
        CreateSelectMenuOption::new(Trio.as_str(), Trio.as_str()),
        CreateSelectMenuOption::new(Quad.as_str(), Quad.as_str()),
        CreateSelectMenuOption::new(Five.as_str(), Five.as_str()),
        CreateSelectMenuOption::new(Six.as_str(), Six.as_str()),
        CreateSelectMenuOption::new(Seven.as_str(), Seven.as_str()),
        CreateSelectMenuOption::new(Eight.as_str(), Eight.as_str()),
        CreateSelectMenuOption::new(Nine.as_str(), Nine.as_str()),
        CreateSelectMenuOption::new(Ten.as_str(), Ten.as_str()),
      ]
    }
  };

  let select_menu_kind = CreateSelectMenuKind::String {
    options: select_menu_vec,
  };
  let select_menu = CreateSelectMenu::new("募集人数を選択", select_menu_kind)
    .placeholder("募集人数を選択してください")
    .min_values(1)
    .max_values(1);
  EditInteractionResponse::new()
    .embed(embed)
    .select_menu(select_menu)
}
