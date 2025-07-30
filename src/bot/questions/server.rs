use std::sync::Arc;

use serenity::all::{
  ComponentInteraction, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
  CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, Http
};

use crate::{bot::{colors::BASE_COLOR, types::{ApServer, WebhookDataExt}, Handler}, error::BotError};

impl Handler {
  pub async fn server(&self, http: Arc<Http>, comp: Arc<ComponentInteraction>) -> Result<(), BotError> {
    let embed = CreateEmbed::new()
      .title("サーバーを選択してください")
      .color(BASE_COLOR);
    let select_menu = CreateSelectMenu::new("サーバー選択", CreateSelectMenuKind::String {
      options: vec![
        CreateSelectMenuOption::new(ApServer::Tokyo.as_str(), ApServer::Tokyo.as_str()),
        CreateSelectMenuOption::new(ApServer::HongKong.as_str(), ApServer::HongKong.as_str()),
        CreateSelectMenuOption::new(ApServer::Singapore.as_str(), ApServer::Singapore.as_str()),
        CreateSelectMenuOption::new(ApServer::Sydney.as_str(), ApServer::Sydney.as_str()),
        CreateSelectMenuOption::new(ApServer::Mumbai.as_str(), ApServer::Mumbai.as_str()),
      ]
    })
    .min_values(1)
    .max_values(1);
    let response = CreateInteractionResponse::Message(
      CreateInteractionResponseMessage::new()
        .embed(embed)
        .select_menu(select_menu)
        .ephemeral(true)
    );
    tokio::spawn({
      async move {
        if let Err(e) = comp.create_response(http, response).await {
          tracing::error!("Failed to create server selection response: {}", e);
        }
      }
    });
    Ok(())
  }
}