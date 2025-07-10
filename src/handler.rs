use std::collections::HashMap;
use std::sync::Arc;

use serenity::async_trait;
use serenity::client::Context as SerenityContext;
use serenity::client::EventHandler;
use serenity::model::application::Interaction;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;

mod pin;
use pin::pin;
mod questions;
use questions::questions;
use tokio::sync::RwLock;
use tracing::Level;
use tracing::instrument;
use tracing::Span;
mod colors;
mod state;
mod webhook_buttons;
mod webhook_edit;
mod webhook_send;
use crate::dotenv_handler;
use state::{ComponentStoreMap, InteractionIdMap, WebhookMap};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
  #[instrument(name = "handler/ready", skip_all, level = Level::INFO)]
  async fn ready(&self, ctx: SerenityContext, ready: Ready) {
    let mut data = ctx.data.write().await;
    data.insert::<WebhookMap>(Arc::new(RwLock::new(HashMap::new())));
    data.insert::<InteractionIdMap>(Arc::new(RwLock::new(HashMap::new())));
    data.insert::<ComponentStoreMap>(Arc::new(RwLock::new(HashMap::new())));
    drop(data);
    tracing::info!("データストアの初期化が完了しました");
    tracing::info!("BOT {} が起動しました", ready.user.name);
  }

  #[instrument(name = "handler/message", skip_all, level = Level::INFO)]
  async fn message(&self, ctx: SerenityContext, msg: Message) {
    let bot_id = match dotenv_handler::get("BOT_ID") {
      Ok(id) => id,
      Err(e) => {
        tracing::warn!(error = %e, "環境変数 BOT_ID の取得に失敗しました");
        return;
      }
    };
    if msg.author.id.to_string() != bot_id {
      let result = pin(&ctx, msg.channel_id).await;
      match result {
        Ok(_) => {
          tracing::info!("ピンメッセージの更新に成功しました");
        }
        Err(e) => {
          tracing::warn!(error = %e, "ピンメッセージの更新に失敗しました");
        }
      }
    }
  }

  #[instrument(name = "handler/interaction_create", skip_all, level = Level::INFO, fields(user_id = tracing::field::Empty, custom_id = tracing::field::Empty))]
  async fn interaction_create(
    &self,
    ctx: SerenityContext,
    interaction: Interaction,
  ) {
    if let Interaction::Component(component) = interaction {
      Span::current().record("user_id", component.user.id.to_string());
      Span::current().record("custom_id", &component.data.custom_id);
      match component.data.custom_id.as_str() {
        "参加する" => match webhook_buttons::join(ctx, component).await {
          Ok(_) => {
            tracing::info!("募集参加処理が終了しました");
          }
          Err(e) => {
            tracing::warn!(error = %e, "募集参加処理に失敗しました")
          }
        },
        "参加をやめる" => {
          match webhook_buttons::leave(ctx, component).await {
            Ok(_) => {
              tracing::info!("募集参加取り消し処理が終了しました");
            }
            Err(e) => {
              tracing::warn!(error = %e, "募集参加取り消し処理に失敗しました")
            }
          }
        }
        "削除" => match webhook_buttons::delete(ctx, component).await {
          Ok(_) => {
            tracing::info!("募集削除処理が終了しました");
          }
          Err(e) => {
            tracing::warn!(error = %e, "募集削除処理に失敗しました");
          }
        },
        _ => match questions(ctx, component).await {
          Ok(_) => {
            tracing::info!("質問送信処理が終了しました");
          }
          Err(e) => {
            tracing::warn!(error = %e, "質問送信処理に失敗しました")
          }
        },
      }
    } else if let Interaction::Modal(modal) = interaction {
      Span::current().record("user_id", modal.user.id.to_string());
      Span::current().record("custom_id", &modal.data.custom_id);
      match modal.defer(&ctx.http).await {
        Ok(_) => {
          tracing::info!("モーダルの応答送信に成功しました");
        }
        Err(e) => {
          tracing::warn!(error = %e, "モーダルの応答送信に失敗しました");
          return;
        }
      }
      match webhook_send::send(&ctx, modal).await {
        Ok(_) => {
          tracing::info!("募集送信処理が終了しました");
        }
        Err(e) => {
          tracing::warn!(error = %e, "募集送信処理に失敗しました");
        }
      }
    }
  }
}
