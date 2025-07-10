use serenity::builder::{
  CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
  CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
};
use serenity::model::channel::ReactionType;

use crate::handler::colors::BASE_COLOR;

pub fn server() -> CreateInteractionResponse {
  let embed = CreateEmbed::new()
    .colour(BASE_COLOR)
    .title("サーバーを選択してください");
  let select_menu_kind = CreateSelectMenuKind::String {
    options: vec![
      CreateSelectMenuOption::new("Tokyo/東京", "Tokyo/東京🇯🇵")
        .emoji(ReactionType::Unicode("🇯🇵".to_string())),
      CreateSelectMenuOption::new("Hong Kong/香港", "Hong Kong/香港 🇭🇰")
        .emoji(ReactionType::Unicode("🇭🇰".to_string())),
      CreateSelectMenuOption::new(
        "Singapore/シンガポール",
        "Singapore/シンガポール 🇸🇬",
      )
      .emoji(ReactionType::Unicode("🇸🇬".to_string())),
      CreateSelectMenuOption::new("Sydney/シドニー", "Sydney/シドニー 🇦🇺")
        .emoji(ReactionType::Unicode("🇦🇺".to_string())),
      CreateSelectMenuOption::new("Mumbai/ムンバイ", "Mumbai/ムンバイ 🇮🇳")
        .emoji(ReactionType::Unicode("🇮🇳".to_string())),
    ],
  };
  let select_menu = CreateSelectMenu::new("サーバーを選択", select_menu_kind)
    .placeholder("サーバーを選択してください")
    .min_values(1)
    .max_values(1);
  let response_message = CreateInteractionResponseMessage::new()
    .ephemeral(true)
    .embed(embed)
    .select_menu(select_menu);
  CreateInteractionResponse::Message(response_message)
}
