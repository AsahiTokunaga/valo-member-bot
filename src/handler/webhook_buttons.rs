use std::str::FromStr;

use serenity::builder::{
  CreateActionRow, CreateButton, CreateInteractionResponse,
  CreateInteractionResponseMessage, EditWebhookMessage,
};
use serenity::client::Context as SerenityContext;
use serenity::http::CacheHttp;
use serenity::http::Http;
use serenity::model::application::{ButtonStyle, ComponentInteraction};
use serenity::model::channel::ReactionType;
use serenity::model::id::{InteractionId, MessageId, UserId};
use serenity::model::webhook::Webhook;
use tracing::{Level, instrument};

use crate::error::BotError;
use crate::handler::state::WebhookData;
use crate::handler::state::methods::{interaction_id_map, webhook_map};
use crate::handler::webhook_edit::edit;
use crate::valkey::commands;
use crate::dotenv_handler;

#[instrument(name = "handler/webhook_buttons/join", skip_all, level = Level::INFO, fields(custom_id = %component.data.custom_id, user_id = %component.user.id, message_id = %component.message.id))]
pub async fn join(
  ctx: SerenityContext,
  component: ComponentInteraction,
) -> Result<(), BotError> {
  tracing::info!("処理開始");
  let enter_join_user = component.user.id;
  let message_id = component.message.id;
  let interaction_id =
    if let Ok(id) = interaction_id_map::get(&ctx, &message_id).await {
      id
    } else {
      timeout(&ctx.http, &component).await?;
      return Ok(());
    };
  let webhook_data = webhook_map::get(&ctx, &interaction_id).await?;
  let webhook_data = webhook_data.read().await;
  if webhook_data.joined.contains(&enter_join_user) {
    tracing::info!("すでに参加しているユーザー: {}", enter_join_user);
    response(&ctx.http, &component, "すでに募集に参加しています", true).await?;
  } else {
    drop(webhook_data);
    update_webhook_data(&ctx, &interaction_id, enter_join_user, 'p').await?;
    let webhook_data = webhook_map::get(&ctx, &interaction_id).await.unwrap();
    let webhook_data = webhook_data.read().await;
    let names = get_field_value(&webhook_data).await;
    let title: (usize, u8) =
      (webhook_data.joined.len(), webhook_data.max_member.into());
    edit(&ctx, message_id, &names, title).await?;
    if is_fill(webhook_data.joined.len(), webhook_data.max_member.into()) {
      recruitment_filled(&ctx.http, message_id).await?;
      let names: String = webhook_data
        .joined
        .iter()
        .map(|u| format!("<@{}>", u))
        .collect::<Vec<String>>()
        .join(" ");
      response(
        &ctx.http,
        &component,
        &format!("{} 募集が埋まりました！", names),
        false,
      )
      .await?;
    } else {
      response(&ctx.http, &component, "募集に参加しました", true).await?;
    }
    tracing::info!("処理終了");
    drop(webhook_data);
  }
  Ok(())
}

#[instrument(name = "handler/webhook_buttons/leave", skip_all, level = Level::INFO, fields(custom_id = %component.data.custom_id, user_id = %component.user.id, message_id = %component.message.id))]
pub async fn leave(
  ctx: SerenityContext,
  component: ComponentInteraction,
) -> Result<(), BotError> {
  tracing::info!("処理開始");
  let enter_leave_user = component.user.id;
  let message_id = component.message.id;
  let redis_pass = dotenv_handler::get("REDIS_PASS")?;
  let linked_message_user =
    commands::get(&redis_pass, message_id.to_string().as_str()).await?;
  if let Some(user) = linked_message_user {
    let linked_message_user_id = UserId::from_str(&user)?;
    let interaction_id =
      if let Ok(id) = interaction_id_map::get(&ctx, &message_id).await {
        id
      } else {
        timeout(&ctx.http, &component).await?;
        return Ok(());
      };
    let webhook_data = webhook_map::get(&ctx, &interaction_id).await?;
    let webhook_data = webhook_data.read().await;
    if linked_message_user_id == enter_leave_user {
      tracing::info!(
        "募集の作成者が参加を取り消そうとしました: {}",
        enter_leave_user
      );
      response(&ctx.http, &component, "募集の作成者は参加を取り消せません\n募集取り消しの場合は削除ボタンを押してください", true).await?;
    } else {
      if webhook_data.joined.contains(&enter_leave_user) {
        drop(webhook_data);
        update_webhook_data(&ctx, &interaction_id, enter_leave_user, 'r')
          .await?;
        let webhook_data =
          webhook_map::get(&ctx, &interaction_id).await.unwrap();
        let webhook_data = webhook_data.read().await;
        let names = get_field_value(&webhook_data).await;
        let title: (usize, u8) =
          (webhook_data.joined.len(), webhook_data.max_member.into());
        tokio::try_join!(
          edit(&ctx, message_id, &names, title),
          response(&ctx.http, &component, "参加を取り消しました", true)
        )?;
        drop(webhook_data);
      } else {
        tracing::info!(
          "参加していないユーザーが参加を取り消そうとしました: {}",
          enter_leave_user
        );
        response(&ctx.http, &component, "募集に参加していません", true).await?;
      }
    }
  } else {
    timeout(&ctx.http, &component).await?;
  }
  tracing::info!("処理終了");
  Ok(())
}

#[instrument(name = "handler/webhook_buttons/delete", skip_all, level = Level::INFO, fields(custom_id = %component.data.custom_id, user_id = %component.user.id, message_id = %component.message.id))]
pub async fn delete(
  ctx: SerenityContext,
  component: ComponentInteraction,
) -> Result<(), BotError> {
  tracing::info!("処理開始");
  let enter_delete_user = component.user.id;
  let message_id = component.message.id.to_string();
  let redis_pass = dotenv_handler::get("REDIS_PASS")?;
  let linked_message_user =
    commands::get(&redis_pass, message_id.as_str()).await?;
  if let Some(user) = linked_message_user {
    let linked_message_user_id = UserId::from_str(&user)?;
    if linked_message_user_id != enter_delete_user {
      tracing::info!(
        "募集の作成者以外が削除しようとしました: {}",
        enter_delete_user
      );
      response(
        &ctx.http,
        &component,
        "募集の取り消しは、募集作成者のみ行えます",
        true,
      )
      .await?;
    } else {
      let channel_id = component.message.channel_id;
      channel_id
        .delete_message(&ctx.http, component.message.id)
        .await?;
      let interaction_id =
        interaction_id_map::get(&ctx, &component.message.id).await?;
      let (webhook_map_del, interaction_id_map_del, commands_del) = tokio::join!(
        webhook_map::del(&ctx, &interaction_id),
        interaction_id_map::del(&ctx, &component.message.id),
        commands::del(&redis_pass, &message_id)
      );
      match (webhook_map_del, interaction_id_map_del, commands_del) {
        (Ok(_), Ok(_), Ok(_)) => {}
        (_, _, Err(e)) => {
          tracing::error!(error = %e, "message_id : user_idの削除に失敗");
          return Err(e);
        }
        (_, Err(e), _) => {
          tracing::error!(error = %e, "message_id : interaction_idの削除に失敗");
          return Err(BotError::AppStateError(e));
        }
        (Err(e), _, _) => {
          tracing::error!(error = %e, "interaction_id : webhook_dataの削除に失敗");
          return Err(BotError::AppStateError(e));
        }
      }
    }
  } else {
    timeout(&ctx.http, &component).await?;
  }
  tracing::info!("処理終了");
  Ok(())
}

#[instrument(name = "handler/webhook_buttons/timeout", skip_all, level = Level::INFO, err(level = Level::WARN), fields(custom_id = %comp.data.custom_id, user_id = %comp.user.id, message_id = %comp.message.id))]
async fn timeout<T: AsRef<Http> + CacheHttp + Copy>(
  http: T,
  comp: &ComponentInteraction,
) -> Result<(), BotError> {
  tracing::info!("処理開始");
  let res = CreateInteractionResponse::Message(
    CreateInteractionResponseMessage::new()
      .ephemeral(true)
      .content("有効期限切れのため募集を削除します"),
  );
  let message_id = comp.message.id;
  let channel_id = comp.message.channel_id;
  let create_response_task = comp.create_response(http, res);
  let delete_message_task = channel_id.delete_message(http, message_id);
  let (create_response, delete_message) =
    tokio::join!(create_response_task, delete_message_task);
  match (create_response, delete_message) {
    (Ok(_), Ok(_)) => tracing::info!("期限切れの募集を削除しました"),
    (_, Err(e)) => {
      tracing::warn!(error = %e, "期限切れ募集の削除に失敗");
      return Err(BotError::SerenityError(e));
    }
    (Err(e), _) => {
      tracing::warn!(error = %e, "期限切れ募集の応答送信に失敗");
      return Err(BotError::SerenityError(e));
    }
  }
  Ok(())
}

#[instrument(name = "handler/webhook_buttons/response", skip_all, level = Level::INFO, err(level = Level::WARN), fields(custom_id = %comp.data.custom_id, user_id = %comp.user.id, message_id = %comp.message.id))]
async fn response<T: CacheHttp>(
  http: T,
  comp: &ComponentInteraction,
  cont: &str,
  ephemeral: bool,
) -> Result<(), BotError> {
  tracing::info!("処理開始");
  let res = CreateInteractionResponse::Message(
    CreateInteractionResponseMessage::new()
      .ephemeral(ephemeral)
      .content(cont),
  );
  comp.create_response(http, res).await?;
  tracing::info!("処理終了");
  Ok(())
}

#[instrument(name = "handler/webhook_buttons/update_webhook_data", skip_all, level = Level::INFO, err(level = Level::WARN), fields(interaction_id = %interaction_id, user_id = %user_id, update_type = %p_r))]
async fn update_webhook_data(
  ctx: &SerenityContext,
  interaction_id: &InteractionId,
  user_id: UserId,
  p_r: char,
) -> Result<(), BotError> {
  match p_r {
    'p' => {
      webhook_map::with_mute(ctx, interaction_id, |w| {
        w.joined.push(user_id);
      })
      .await?;
      tracing::info!("参加者を追加しました");
      Ok(())
    }
    'r' => {
      webhook_map::with_mute(ctx, interaction_id, |w| {
        if let Some(idx) = w.joined.iter().position(|u| u == &user_id) {
          w.joined.remove(idx);
        }
      })
      .await?;
      tracing::info!("参加者を削除しました");
      Ok(())
    }
    _ => {
      tracing::warn!("不正な操作");
      return Err(BotError::InvalidInput);
    }
  }
}

async fn get_field_value(webhook_data: &WebhookData) -> String {
  webhook_data
    .joined
    .iter()
    .map(|user_id| format!("<@{}>", user_id))
    .collect::<Vec<String>>()
    .join("\n")
}

fn is_fill(joined_users: usize, max_member: u8) -> bool {
  joined_users as u8 == max_member
}

#[instrument(name = "handler/webhook_buttons/recruitment_filled", skip_all, level = Level::INFO, err(level = Level::WARN), fields(message_id = %message_id))]
async fn recruitment_filled<T: AsRef<Http> + CacheHttp + Copy>(
  http: T,
  message_id: MessageId,
) -> Result<(), BotError> {
  tracing::info!("処理開始");
  let redis_pass = dotenv_handler::get("REDIS_PASS")?;
  let channel_id = dotenv_handler::get("CHANNEL_ID")?;
  let webhook_url = commands::get(&redis_pass, &channel_id).await?.unwrap();
  let webhook = Webhook::from_url(http, &webhook_url).await?;
  let component = CreateActionRow::Buttons(get_button());
  let wh_message = EditWebhookMessage::new().components(vec![component]);
  webhook.edit_message(http, message_id, wh_message).await?;
  tracing::info!("処理終了");
  Ok(())
}

fn get_button() -> Vec<CreateButton> {
  let join_button = CreateButton::new("参加する")
    .label("参加する")
    .style(ButtonStyle::Secondary)
    .emoji(ReactionType::Unicode("✋".to_string()))
    .disabled(true);
  let leave_button = CreateButton::new("参加をやめる")
    .label("参加をやめる")
    .style(ButtonStyle::Secondary)
    .emoji(ReactionType::Unicode("👋".to_string()))
    .disabled(true);
  let delete_button = CreateButton::new("削除")
    .label("削除")
    .style(ButtonStyle::Secondary)
    .emoji(ReactionType::Unicode("🚫".to_string()));
  vec![join_button, leave_button, delete_button]
}
