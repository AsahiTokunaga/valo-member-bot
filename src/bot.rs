pub mod buttons;
pub mod questions;
pub mod types;
pub mod colors;
pub mod panels;

use serenity::{
  all::{ActionRowComponent, ChannelId, ComponentInteraction, ComponentInteractionDataKind, Context, CreateInteractionResponse, CreateInteractionResponseMessage, EventHandler, Interaction, Message, Ready, UserId},
  async_trait,
};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::sync::Mutex;
use types::WebhookData;

use crate::{bot::{buttons::{DeleteResponse, LeaveResponse}, types::{ApServer, Member, Mode, Rank, RedisClient, WebhookDataExt}}, config};

#[derive(Clone)]
pub struct Handler {
  pub question_state: Arc<Mutex<HashMap<UserId, WebhookData>>>,
  pub component_store: Arc<Mutex<HashMap<UserId, ComponentInteraction>>>,
  pub redis_client: Arc<Mutex<RedisClient>>,
}

#[async_trait]
impl EventHandler for Handler {
  async fn ready(&self, _: Context, ready: Ready) {
    tracing::info!("{} is ready", ready.user.name);
  }
  async fn message(&self, ctx: Context, msg: Message) {
    let channel = match config::get("CHANNEL_ID") {
      Ok(id) => match ChannelId::from_str(&id) {
        Ok(id) => id,
        Err(e) => {
          tracing::warn!(error = %e, "Failed to parse CHANNEL_ID");
          return;
        }
      }
      Err(e) => {
        tracing::warn!(error = %e, "Failed to get CHANNEL_ID from environment variables");
        return;
      }
    };
    if msg.channel_id != channel {
      return;
    }

    let bot = match config::get("BOT_ID") {
      Ok(id) => id,
      Err(e) => {
        tracing::warn!(error = %e, "Failed to get BOT_ID from environment variables");
        return;
      }
    };
    if msg.author.id.to_string() != bot {
      let mut redis_client = self.redis_client.lock().await;
      match panels::entry(&ctx.http, &mut redis_client).await {
        Ok(_) => {
          tracing::info!("Entry panel update successfully");
        }
        Err(e) => {
          tracing::warn!(error = %e, "Failed to update entry panel");
        }
      }
      drop(redis_client);
    }
  }
  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    match interaction {
      Interaction::Component(component) => {
        match component.data.custom_id.as_str() {
          "募集を作成" => {
            self.create(component.user.id).await;
            match self.server(component.user.id, &ctx.http, &component).await {
              Err(e) => tracing::warn!(error = %e, "Failed to create server selection interaction"),
              _ => {}
            }
          }
          "サーバー選択" => {
            let _ = component.defer(&ctx.http).await;
            if let ComponentInteractionDataKind::StringSelect { values } = &component.data.kind {
              self.set(component.user.id, |data| {
                data.server = ApServer::from_str(&values[0]).unwrap_or(ApServer::Tokyo);
              }).await;
            }
            match self.mode(&ctx.http, component.user.id).await {
              Err(e) => tracing::warn!(error = %e, "Failed to create mode selection interaction"),
              _ => {}
            }
          }
          "モード選択" => {
            let _ = component.defer(&ctx.http).await;
            if let ComponentInteractionDataKind::StringSelect { values } = &component.data.kind {
              self.set(component.user.id, |data| {
                data.mode = Mode::from_str(&values[0]).unwrap_or(Mode::Unrated);
              }).await;
              if Mode::from_str(&values[0]).unwrap_or(Mode::Unrated) == Mode::Competitive {
                match self.rank(&ctx.http, component.user.id).await {
                  Err(e) => tracing::warn!(error = %e, "Failed to create rank selection interaction"),
                  _ => {}
                }
              } else {
                match self.member(&ctx.http, component.user.id, &values[0]).await {
                  Err(e) => tracing::warn!(error = %e, "Failed to create member selection interaction"),
                  _ => {}
                }
              }
            }
          }
          "人数選択" => {
            if let ComponentInteractionDataKind::StringSelect { values } = &component.data.kind {
              self.set(component.user.id, |data| {
                data.member = Member::from_str(&values[0]).unwrap_or(Member::FullParty);
              }).await;
              match self.message(&ctx.http, &component).await {
                Err(e) => tracing::warn!(error = %e, "Failed to create message interaction"),
                _ => {}
              }
            }
          }
          "ランク選択" => {
            let _ = component.defer(&ctx.http).await;
            if let ComponentInteractionDataKind::StringSelect { values } = &component.data.kind {
              self.set(component.user.id, |data| {
                data.rank = Some(Rank::from_str(&values[0]).unwrap_or(Rank::Unranked));
              }).await;
              match self.member(&ctx.http, component.user.id, Mode::Competitive.as_str()).await {
                Err(e) => tracing::warn!(error = %e, "Failed to create member selection interaction"),
                _ => {}
              }
            }
          }
          "参加する" => {
            let mut redis_client = self.get_redis_client().await;
            match buttons::join(&mut redis_client, component.user.id, component.message.id).await {
              Ok(buttons::JoinResponse::Joined) => {
                component.create_response(&ctx.http, CreateInteractionResponse::Message(
                  CreateInteractionResponseMessage::new()
                    .content("募集に参加しました。")
                    .ephemeral(true)
                )).await.map_err(|e| {
                  tracing::warn!(error = %e, "Failed to create join response");
                }).ok();
                let is_fill = panels::edit(&ctx.http, &mut redis_client, component.message.id).await.map_err(|e| {
                  tracing::warn!(error = %e, "Failed to edit panel after join");
                });
                if let Ok(if_fill) = is_fill {
                  if !if_fill { return; }
                  match redis_client.get_webhook_data(component.message.id).await {
                    Err(e) => tracing::warn!(error = %e, "Failed to get webhook data after join"),
                    Ok(webhook_data) => {
                      let joined_users = webhook_data.joined.iter()
                        .map(|&u| format!("<@{}>", u.get()))
                        .collect::<Vec<String>>()
                        .join(" ");
                      component.message.reply(&ctx.http, format!("{} 募集が埋まりました！", joined_users))
                        .await
                        .map_err(|e| tracing::warn!(error = %e, "Failed to reply after join"))
                        .ok();
                    }
                  }
                }
              }
              Ok(buttons::JoinResponse::AlreadyJoined) => {
                component.create_response(&ctx.http, CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                      .content("すでに参加しています。")
                      .ephemeral(true)
                  ))
                  .await
                  .map_err(|e| tracing::warn!(error = %e, "Failed to create join response"))
                  .ok();
              }
              Ok(buttons::JoinResponse::Expired) => {
                panels::handle_expired(&ctx.http, &component, &mut self.get_redis_client().await).await;
              }
              Err(e) => tracing::warn!(error = %e, "Failed to join"),
            }
          }
          "参加をやめる" => {
            let mut redis_client = self.get_redis_client().await;
            match buttons::leave(&mut redis_client, component.user.id, component.message.id).await {
              Ok(LeaveResponse::Left) => {
                component.create_response(&ctx.http, CreateInteractionResponse::Message(
                  CreateInteractionResponseMessage::new()
                    .content("募集参加を取り消しました。") 
                    .ephemeral(true)
                )).await
                  .map_err(|e| tracing::warn!(error = %e, "Failed to create leave response"))
                  .ok();
                panels::edit(&ctx.http, &mut redis_client, component.message.id)
                  .await
                  .map_err(|e| tracing::warn!(error = %e, "Failed to edit panel after leave"))
                  .ok();
              }
              Ok(LeaveResponse::CreatorLeave) => {
                component.create_response(&ctx.http, CreateInteractionResponse::Message(
                  CreateInteractionResponseMessage::new()
                    .content("募集作成者は募集参加を取り消せません。\n募集を削除したい場合は「削除」ボタンを押してください。")
                    .ephemeral(true)
                )).await
                  .map_err(|e| tracing::warn!(error = %e, "Failed to create leave response"))
                  .ok();
              }
              Ok(LeaveResponse::NotJoined) => {
                component.create_response(&ctx.http, CreateInteractionResponse::Message(
                  CreateInteractionResponseMessage::new()
                    .content("募集に参加していません。")
                    .ephemeral(true)
                )).await
                  .map_err(|e| tracing::warn!(error = %e, "Failed to create leave response"))
                  .ok();
              }
              Ok(LeaveResponse::Expired) => {
                panels::handle_expired(&ctx.http, &component, &mut redis_client).await;
              }
              Err(e) => tracing::warn!(error = %e, "Failed to leave"),
            }
          }
          "削除" => {
            let mut redis_client = self.get_redis_client().await;
            match buttons::delete(&mut redis_client, component.user.id, component.message.id).await {
              Ok(DeleteResponse::Deleted) => {
                component.create_response(&ctx.http, CreateInteractionResponse::Message(
                  CreateInteractionResponseMessage::new()
                    .content("募集を削除しました。")
                    .ephemeral(true)
                )).await
                  .map_err(|e| tracing::warn!(error = %e, "Failed to create delete response"))
                  .ok();
                panels::delete(&ctx.http, &mut redis_client, component.message.id)
                  .await
                  .map_err(|e| tracing::warn!(error = %e, "Failed to delete panel after deletion"))
                  .ok();
              }
              Ok(DeleteResponse::NotCreator) => {
                component.create_response(&ctx.http, CreateInteractionResponse::Message(
                  CreateInteractionResponseMessage::new()
                    .content("募集作成者のみが削除できます。")
                    .ephemeral(true)
                )).await
                  .map_err(|e| tracing::warn!(error = %e, "Failed to create delete response"))
                  .ok();
              }
              Ok(DeleteResponse::NotJoined) => {
                component.create_response(&ctx.http, CreateInteractionResponse::Message(
                  CreateInteractionResponseMessage::new()
                    .content("募集に参加していません。")
                    .ephemeral(true)
                )).await
                  .map_err(|e| tracing::warn!(error = %e, "Failed to create delete response"))
                  .ok();
              }
              Ok(DeleteResponse::Expired) => {
                panels::handle_expired(&ctx.http, &component, &mut redis_client).await;
              }
              Err(e) => tracing::warn!(error = %e, "Failed to delete"),
            }
          }
          _ => {}
        }
      }
      Interaction::Modal(component) => {
        if let ActionRowComponent::InputText(input) = &component.data.components.first().unwrap().components.first().unwrap() {
          let webhook_data = match self.get_question_state(component.user.id).await {
            Ok(data) => data,
            Err(e) => {
              tracing::warn!(error = %e, "Failed to get question state");
              return;
            }
          };
          let _ = component.defer(&ctx.http).await;
          let mut redis_client = self.get_redis_client().await;
          match panels::send(&ctx.http, &mut redis_client, &webhook_data, input.value.as_deref()).await {
            Err(e) => tracing::warn!(error = %e, "Failed to send webhook message"),
            _ => {}
          }
          if let Some(comp) = self.get_component(component.user.id).await {
            match comp.delete_response(&ctx.http).await {
              Err(e) => tracing::warn!(error = %e, "Failed to delete response"),
              _ => {}
            }
          }
          match self.remove_temp_data(component.user.id).await {
            Err(e) => tracing::warn!(error = %e, "Failed to remove question state"),
            _ => {}
          }
        }
      }
      _ => {}
    }
  }
}
