use serenity::all::{CacheHttp, ComponentInteraction, CreateActionRow, CreateInputText, CreateInteractionResponse, CreateModal, Http, InputTextStyle};

use crate::{bot::Handler, error::BotError};

impl Handler {
  pub async fn message<T>(&self, http: T, comp: &ComponentInteraction) -> Result<(), BotError> 
  where 
    T: AsRef<Http> + CacheHttp + Copy,
  {
    let action_row = vec![
      CreateActionRow::InputText(
        CreateInputText::new(
          InputTextStyle::Short,
          "募集メッセージを入力しましょう",
          "募集メッセージ"
        )
        .required(false)
        .max_length(100)
        .placeholder("例: たくさん喋れる人募集！")
      )
    ];
    let modal = CreateModal::new("募集メッセージ", "募集メッセージ").components(action_row);
    let response = CreateInteractionResponse::Modal(modal);
    comp.create_response(http, response).await?;
    Ok(())
  }
}
