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
        CreateSelectMenuOption::new(Duo.to_string(), Duo.to_string()),
        CreateSelectMenuOption::new(Trio.to_string(), Trio.to_string()),
        CreateSelectMenuOption::new(Quad.to_string(), Quad.to_string()),
        CreateSelectMenuOption::new(Five.to_string(), Five.to_string()),
      ]
    }
    Competitive => {
      vec![
        CreateSelectMenuOption::new(Duo.to_string(), Duo.to_string()),
        CreateSelectMenuOption::new(Trio.to_string(), Trio.to_string()),
        CreateSelectMenuOption::new(Five.to_string(), Five.to_string()),
      ]
    }
    Custom => {
      vec![
        CreateSelectMenuOption::new(Duo.to_string(), Duo.to_string()),
        CreateSelectMenuOption::new(Trio.to_string(), Trio.to_string()),
        CreateSelectMenuOption::new(Quad.to_string(), Quad.to_string()),
        CreateSelectMenuOption::new(Five.to_string(), Five.to_string()),
        CreateSelectMenuOption::new(Six.to_string(), Six.to_string()),
        CreateSelectMenuOption::new(Seven.to_string(), Seven.to_string()),
        CreateSelectMenuOption::new(Eight.to_string(), Eight.to_string()),
        CreateSelectMenuOption::new(Nine.to_string(), Nine.to_string()),
        CreateSelectMenuOption::new(Ten.to_string(), Ten.to_string()),
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
