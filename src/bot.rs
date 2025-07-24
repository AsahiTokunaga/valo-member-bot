pub mod buttons;
pub mod questions;
pub mod types;
pub mod colors;
pub mod panels;

use dashmap::DashMap;
use serenity::{
  all::{ActionRowComponent, ChannelId, ComponentInteraction, ComponentInteractionDataKind, Context, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage, EventHandler, ExecuteWebhook, Interaction, Message, Ready, UserId, Webhook},
  async_trait,
};
use std::{str::FromStr, sync::Arc};
use types::WebhookData;

use crate::{bot::{buttons::{DeleteResponse, JoinResponse, LeaveResponse}, types::{ApServer, Member, Mode, Rank, RedisClient, WebhookDataExt}}, config, worker::Worker};

#[derive(Clone)]
pub struct Handler {
  pub question_state: DashMap<UserId, WebhookData>,
  pub component_store: DashMap<UserId, ComponentInteraction>,
  pub redis_client: Arc<RedisClient>,
  pub worker: Arc<Worker>,
}

#[async_trait]
impl EventHandler for Handler {
  async fn ready(&self, _: Context, ready: Ready) {
    tracing::info!("{} is ready", ready.user.name);
  }
  async fn message(&self, ctx: Context, msg: Message) {
    if msg.author.id == UserId::new(302050872383242240) && msg.embeds.get(0).map_or(false, |e| e.description.as_ref().map_or(false, |d| d.contains("表示順をアップしたよ"))) {
      let webhook = Webhook::from_url(&ctx.http, "https://discord.com/api/webhooks/1396712367752216646/6W2ICA0CeM2bHn8xotvXZIYEP9Y6c8h1Oss5b80NIEi6avbpepxW6ZYLGoo5jPRCobB3");
      let based_webhook = ExecuteWebhook::new()
        .avatar_url(format!(
          "{}bell-logo.jpg",
          config::get("BASE_IMG_URL")
            .unwrap_or("https://raw.githubusercontent.com/AsahiTokunaga/valo-member-bot-images/main/".to_string())
        ))
        .username("BUMP Reminder");
      let detection_bump_webhook = based_webhook
        .clone()
        .embed(CreateEmbed::new()
          .title("BUMP通知を検知しました")
          .description("BUMPありがとう！2時間後にまたBUMPしてね！")
        );
      let reminder_bump_webhook = based_webhook
        .clone()
        .content("<@&1396745451189047296>")
        .embed(CreateEmbed::new()
          .title("BUMPの時間です")
          .description("BUMPしてこのサーバーをより盛り上げることにご協力ください！")
        );

      if let Ok(webhook) = webhook.await {
        let webhook = Arc::new(webhook);
        let detection_bump_webhook = Arc::new(detection_bump_webhook);
        let reminder_bump_webhook = Arc::new(reminder_bump_webhook);
        let worker = self.worker.clone();
        let http = ctx.http.clone();
        worker.spawn(move || {
          async move {
            tokio::spawn({
              let webhook = webhook.clone();
              let http = http.clone();
              let detection_bump_webhook = detection_bump_webhook.as_ref().clone().clone();
              async move {
                if let Err(e) = webhook.execute(&http, false, detection_bump_webhook).await {
                  tracing::warn!(error = %e, "Failed to send BUMP notification");
                }
              }
            });
            tokio::spawn({
              let webhook = webhook.clone();
              let http = http.clone();
              let reminder_bump_webhook = reminder_bump_webhook.as_ref().clone().clone();
              async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(2 * 60 * 60)).await;
                if let Err(e) = webhook.execute(&http, false, reminder_bump_webhook).await {
                  tracing::warn!(error = %e, "Failed to send BUMP reminder");
                }
              }
            });
          }
        }).await;
      }
    }

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
    if msg.channel_id == channel {
      let bot = match config::get("BOT_ID") {
        Ok(id) => id,
        Err(e) => {
          tracing::warn!(error = %e, "Failed to get BOT_ID from environment variables");
          return;
        }
      };
      if msg.author.id.to_string() != bot {
        let worker = self.worker.clone();
        let http = ctx.http.clone();
        let redis_client = Arc::new(self.redis_client.clone());
        worker.spawn(move || {
          let http = http.clone();
          let redis_client = redis_client.clone();
          async move {
            if let Err(e) = panels::entry(&http, &redis_client).await {
              tracing::warn!(error = %e, "Failed to create entry panel");
            }
          }
        }).await;
      }
    }
  }
  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    match interaction {
      Interaction::Component(component) => {
        match component.data.custom_id.as_str() {
          "募集を作成" => {
            self.create(component.user.id);
            self.component_store.insert(component.user.id, component.clone());
            tokio::spawn({
              let handler = self.clone();
              async move {
                if let Err(e) = handler.server(&ctx.http, &component).await {
                  tracing::warn!(error = %e, "Failed to create server selection interaction");
                }
              }
            });
          }
          "サーバー選択" => {
            let _ = component.defer(&ctx.http).await;
            if let ComponentInteractionDataKind::StringSelect { values } = &component.data.kind {
              self.set(component.user.id, |data| {
                data.server = ApServer::from_str(&values[0]).unwrap_or(ApServer::Tokyo);
              });
            }
            tokio::spawn({
              let handler = self.clone();
              async move {
                if let Err(e) = handler.mode(&ctx.http, component.user.id).await {
                  tracing::warn!(error = %e, "Failed to create mode selection interaction");
                }
              }
            });
          }
          "モード選択" => {
            let _ = component.defer(&ctx.http).await;
            if let ComponentInteractionDataKind::StringSelect { values } = &component.data.kind {
              self.set(component.user.id, |data| {
                data.mode = Mode::from_str(&values[0]).unwrap_or(Mode::Unrated);
              });
              if Mode::from_str(&values[0]).unwrap_or(Mode::Unrated) == Mode::Competitive {
                tokio::spawn({
                  let handler = self.clone();
                  async move {
                    if let Err(e) = handler.rank(&ctx.http, component.user.id).await {
                      tracing::warn!(error = %e, "Failed to create rank selection interaction");
                    }
                  }
                });
              } else {
                tokio::spawn({
                  let handler = self.clone();
                  let values = values.clone();
                  async move {
                    if let Err(e) = handler.member(&ctx.http, component.user.id, &values[0]).await {
                      tracing::warn!(error = %e, "Failed to create member selection interaction");
                    }
                  }
                });
              }
            }
          }
          "人数選択" => {
            if let ComponentInteractionDataKind::StringSelect { values } = &component.data.kind {
              self.set(component.user.id, |data| {
                data.member = Member::from_str(&values[0]).unwrap_or(Member::FullParty);
              });
              tokio::spawn({
                let handler = self.clone();
                async move {
                  if let Err(e) = handler.message(&ctx.http, &component).await {
                    tracing::warn!(error = %e, "Failed to create message interaction");
                  }
                }
              });
            }
          }
          "ランク選択" => {
            let _ = component.defer(&ctx.http).await;
            if let ComponentInteractionDataKind::StringSelect { values } = &component.data.kind {
              self.set(component.user.id, |data| {
                data.rank = Some(Rank::from_str(&values[0]).unwrap_or(Rank::Unranked));
              });
              tokio::spawn({
                let handler = self.clone();
                async move {
                  if let Err(e) = handler.member(&ctx.http, component.user.id, Mode::Competitive.as_str()).await {
                    tracing::warn!(error = %e, "Failed to create member selection interaction");
                  }
                }
              });
            }
          }
          "参加する" => {
            match buttons::join(&self.redis_client, component.user.id, component.message.id).await {
              Ok(JoinResponse::Joined) => {
                tokio::spawn({
                  let http = ctx.http.clone();
                  let component = component.clone();
                  async move {
                    if let Err(e) = component.create_response(&http, CreateInteractionResponse::Message(
                      CreateInteractionResponseMessage::new()
                        .content("募集に参加しました。")
                        .ephemeral(true)
                    )).await {
                      tracing::warn!(error = %e, "Failed to create join response");
                    }
                  }
                });
                let is_fill = if let Ok(is_fill) = panels::edit(&ctx.http, &self.redis_client, component.message.id).await {
                  is_fill
                } else {
                  tracing::warn!("Failed to edit panel after join");
                  return;
                };
                if is_fill {
                  match self.redis_client.get_webhook_data(component.message.id).await {
                    Err(e) => tracing::warn!(error = %e, "Failed to get webhook data after join"),
                    Ok(webhook_data) => {
                      let joined_users = webhook_data.joined.iter()
                        .map(|&u| format!("<@{}>", u.get()))
                        .collect::<Vec<String>>()
                        .join(" ");
                      tokio::spawn({
                        async move {
                          if let Err(e) = component.message.reply(&ctx.http, format!("{} 募集が埋まりました！", joined_users)).await {
                            tracing::warn!(error = %e, "Failed to reply after join");
                          }
                        }
                      });
                    }
                  }
                }
              }
              Ok(JoinResponse::AlreadyJoined) => {
                tokio::spawn(async move {
                  if let Err(e) = component.create_response(&ctx.http, CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                      .content("すでに参加しています。")
                      .ephemeral(true)
                  )).await {
                    tracing::warn!(error = %e, "Failed to create join response");
                  }
                });
              }
              Ok(JoinResponse::Expired) => {
                tokio::spawn({
                  let redis_client = self.redis_client.clone();
                  async move {
                    panels::handle_expired(&ctx.http, &component, &redis_client).await
                  }
                });
              }
              Err(e) => tracing::warn!(error = %e, "Failed to join"),
            }
          }
          "参加をやめる" => {
            match buttons::leave(&self.redis_client, component.user.id, component.message.id).await {
              Ok(LeaveResponse::Left) => {
                tokio::spawn({
                  let redis_client = self.redis_client.clone();
                  async move {
                    if let Err(e) = component.create_response(&ctx.http, CreateInteractionResponse::Message(
                      CreateInteractionResponseMessage::new()
                        .content("募集参加を取り消しました。")
                        .ephemeral(true)
                    )).await {
                      tracing::warn!(error = %e, "Failed to create leave response");
                    }
                    if let Err(e) = panels::edit(&ctx.http, &redis_client, component.message.id).await {
                      tracing::warn!(error = %e, "Failed to edit panel after leave");
                    }
                  }
                });
              }
              Ok(LeaveResponse::CreatorLeave) => {
                tokio::spawn(async move {
                  if let Err(e) = component.create_response(&ctx.http, CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                      .content("募集作成者は募集参加を取り消せません。\n募集を削除したい場合は「削除」ボタンを押してください。")
                      .ephemeral(true)
                  )).await {
                    tracing::warn!(error = %e, "Failed to create leave response");
                  }
                });
              }
              Ok(LeaveResponse::NotJoined) => {
                tokio::spawn(async move {
                  if let Err(e) = component.create_response(&ctx.http, CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                      .content("募集に参加していません。")
                      .ephemeral(true)
                  )).await {
                    tracing::warn!(error = %e, "Failed to create leave response");
                  }
                });
              }
              Ok(LeaveResponse::Expired) => {
                tokio::spawn({
                  let redis_client = self.redis_client.clone();
                  async move {
                    panels::handle_expired(&ctx.http, &component, &redis_client).await
                  }
                });
              }
              Err(e) => tracing::warn!(error = %e, "Failed to leave"),
            }
          }
          "削除" => {
            match buttons::delete(&self.redis_client, component.user.id, component.message.id).await {
              Ok(DeleteResponse::Deleted) => {
                tokio::spawn({
                  let http = ctx.http.clone();
                  let redis_client = self.redis_client.clone();
                  async move {
                    if let Err(e) = component.create_response(&http, CreateInteractionResponse::Message(
                      CreateInteractionResponseMessage::new()
                        .content("募集を削除しました。")
                        .ephemeral(true)
                    )).await {
                      tracing::warn!(error = %e, "Failed to create delete response");
                    }
                    if let Err(e) = panels::delete(&ctx.http, &redis_client, component.message.id).await {
                      tracing::warn!(error = %e, "Failed to delete panel after deletion");
                    }
                  }
                });
              }
              Ok(DeleteResponse::NotCreator) => {
                tokio::spawn({
                  async move {
                    if let Err(e) = component.create_response(&ctx.http, CreateInteractionResponse::Message(
                      CreateInteractionResponseMessage::new()
                        .content("募集作成者のみが削除できます。")
                        .ephemeral(true)
                    )).await {
                      tracing::warn!(error = %e, "Failed to create delete response");
                    }
                  }
                });
              }
              Ok(DeleteResponse::NotJoined) => {
                tokio::spawn({
                  async move {
                    if let Err(e) = component.create_response(&ctx.http, CreateInteractionResponse::Message(
                      CreateInteractionResponseMessage::new()
                        .content("募集に参加していません。")
                        .ephemeral(true)
                    )).await {
                      tracing::warn!(error = %e, "Failed to create delete response");
                    }
                  }
                });
              }
              Ok(DeleteResponse::Expired) => {
                tokio::spawn({
                  let redis_client = self.redis_client.clone();
                  async move {
                    panels::handle_expired(&ctx.http, &component, &redis_client).await
                  }
                });
              }
              Err(e) => tracing::warn!(error = %e, "Failed to delete"),
            }
          }
          _ => {}
        }
      }
      Interaction::Modal(component) => {
        if let ActionRowComponent::InputText(input) = &component.data.components.first().unwrap().components.first().unwrap() {
          let webhook_data = match self.get_question_state(component.user.id) {
            Ok(data) => data,
            Err(e) => {
              tracing::warn!(error = %e, "Failed to get question state");
              return;
            }
          };
          if let Some(comp) = self.get_component(component.user.id) {
            tokio::spawn({
              let http = ctx.http.clone();
              async move {
                if let Err(e) = comp.delete_response(&http).await {
                  tracing::warn!(error = %e, "Failed to delete response");
                }
              }
            });
          }
          if let Err(e) = self.remove_temp_data(component.user.id) {
            tracing::warn!(error = %e, "Failed to remove question state");
          }
          let _ = component.defer(&ctx.http).await;
          tokio::spawn({
            let redis_client = self.redis_client.clone();
            let http = ctx.http;
            let input = input.clone();
            async move {
              if let Err(e) = panels::send(&http, &redis_client, &webhook_data, input.value.as_deref()).await {
                tracing::warn!(error = %e, "Failed to send webhook message")
              }
            }
          });
        }
      }
      _ => {}
    }
  }
}
