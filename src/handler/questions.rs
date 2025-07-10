use std::str::FromStr;

use crate::error::BotError;
use crate::handler::state::methods::{component_store_map, webhook_map};
use crate::handler::state::{APServer, Mode, Rank};
use serenity::all::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::client::Context as SerenityContext;
use serenity::model::application::ComponentInteraction;
use serenity::model::application::ComponentInteractionDataKind;
mod server_select;
use server_select::server;
mod mode_select;
use mode_select::q_match;
mod member_select;
use member_select::member;
mod recruit_message;
use recruit_message::recruit_message;
mod rank_select;
use rank_select::rank;
use tracing::{Level, instrument};

#[instrument(name = "handler/questions/questions", level = Level::INFO, err(level = Level::WARN), skip_all, fields(custom_id = %component.data.custom_id, user_id = %component.user.id))]
pub async fn questions(
  ctx: SerenityContext,
  component: ComponentInteraction,
) -> Result<(), BotError> {
  if component.data.custom_id != "募集を作成"
    && component_store_map::get(&ctx, &component.user.id)
      .await
      .is_err()
  {
    tracing::warn!("期限切れの質問です");
    component
      .create_response(&ctx.http, CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
          .content("期限切れの質問です。再度募集を作成してください。")
          .ephemeral(true)
      ))
      .await?;
    return Ok(());
  }
  match component.data.custom_id.as_str() {
    "募集を作成" => {
      let question = server();
      component_store_map::set(&ctx, component.user.id, &component).await;
      webhook_map::new(&ctx, component.id, component.user.id).await;
      component.create_response(&ctx.http, question).await?;
    }
    "サーバーを選択" => {
      component.defer(&ctx.http).await?;
      let user_id = component.user.id;
      let ap_server =
        if let ComponentInteractionDataKind::StringSelect { values } =
          &component.data.kind
        {
          APServer::from_str(&values.get(0).unwrap()).ok()
        } else {
          None
        };
      let question = q_match();
      let component = component_store_map::get(&ctx, &user_id).await?;
      let component = component.read().await;
      let interaction_id = &component.id;
      component.edit_response(&ctx.http, question).await?;
      if ap_server.is_some() {
        webhook_map::with_mute(&ctx, interaction_id, |w| {
          w.ap_server = ap_server.unwrap()
        })
        .await?;
      }
      drop(component);
    }
    "募集形式を選択" => {
      component.defer(&ctx.http).await?;
      let user_id = component.user.id;
      let mode = if let ComponentInteractionDataKind::StringSelect { values } =
        &component.data.kind
      {
        Mode::from_str(&values.get(0).unwrap()).ok()
      } else {
        None
      };
      let component = component_store_map::get(&ctx, &user_id).await?;
      let component = component.read().await;
      if mode.is_some() {
        webhook_map::with_mute(&ctx, &component.id, |w| w.mode = mode.unwrap())
          .await?;
        if mode.unwrap() == Mode::Unrated
          || mode.unwrap() == Mode::Custom
          || mode.is_none()
        {
          let question = member(mode.unwrap());
          component.edit_response(&ctx.http, question).await?;
        } else {
          let question = rank();
          component.edit_response(&ctx.http, question).await?;
        };
      }
      drop(component);
    }
    "募集人数を選択" => {
      let user_id = component.user.id;
      let question = recruit_message();
      let max_member =
        if let ComponentInteractionDataKind::StringSelect { values } =
          &component.data.kind
        {
          crate::handler::state::MaxMember::from_str(&values.get(0).unwrap())
            .ok()
        } else {
          None
        };
      component.create_response(&ctx.http, question).await?;
      let component = component_store_map::get(&ctx, &user_id).await?;
      let component = component.read().await;
      webhook_map::with_mute(&ctx, &component.id, |w| {
        w.max_member = max_member.unwrap() // enumでの管理なのでunwrapを使用
      })
      .await?;
      drop(component);
    }
    "ランクを選択" => {
      component.defer(&ctx.http).await?;
      let user_id = component.user.id;
      let rank = if let ComponentInteractionDataKind::StringSelect { values } =
        &component.data.kind
      {
        Rank::from_str(&values.get(0).unwrap()).ok()
      } else {
        None
      };
      let question = member(Mode::Competitive);
      let component = component_store_map::get(&ctx, &user_id).await?;
      let component = component.read().await;
      component.edit_response(&ctx.http, question).await?;
      if rank.is_some() {
        webhook_map::with_mute(&ctx, &component.id, |w| w.rank = rank).await?;
      }
      drop(component);
    }
    _ => {}
  }
  Ok(())
}
