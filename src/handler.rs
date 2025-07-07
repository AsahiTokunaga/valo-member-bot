use std::collections::HashMap;
use std::sync::Arc;

use crate::dotenv_handler;
use crate::state_handler::ComponentStoreMap;
use crate::state_handler::InteractionIdMap;
use crate::state_handler::WebhookMap;
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
mod webhook_buttons_handler;
mod webhook_edit;
mod webhook_send;
mod colors;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: SerenityContext, ready: Ready) {
        println!("[ OK ] {}が起動しました", ready.user.name);

        // アプリ状態を管理する構造体を初期化
        let mut data = ctx.data.write().await;
        data.insert::<WebhookMap>(Arc::new(RwLock::new(HashMap::new())));
        data.insert::<InteractionIdMap>(Arc::new(RwLock::new(HashMap::new())));
        data.insert::<ComponentStoreMap>(Arc::new(RwLock::new(HashMap::new())));
    }

    async fn message(&self, ctx: SerenityContext, msg: Message) {
        println!("[ OK ] メッセージを受信しました");
        if msg.author.id.to_string()
            != dotenv_handler::get("BOT_ID").expect("[ FAILED ] BOT_IDが設定されていません")
        {
            pin(ctx, &msg)
                .await
                .expect("[ FAILED ] 募集の作成に失敗しました");
        }
    }

    async fn interaction_create(&self, ctx: SerenityContext, interaction: Interaction) {
        if let Interaction::Component(component) = interaction {
            println!(
                "[ OK ] インタラクションを受信しました: {}",
                component.data.custom_id
            );
            match component.data.custom_id.as_str() {
                "参加する" => webhook_buttons_handler::join(ctx, component).await
                    .expect("[ FAILED ] 参加に失敗しました"),
                "参加をやめる" => webhook_buttons_handler::leave(ctx, component).await
                    .expect("[ FAILED ] 参加の取り消しに失敗しました"),
                "削除" => webhook_buttons_handler::delete(ctx, component).await
                    .expect("[ FAILED ] Webhookの削除に失敗しました"),
                _ => questions(ctx, component)
                    .await
                    .expect("[ FAILED ] インタラクションの処理に失敗しました")
            }
        } else if let Interaction::Modal(modal) = interaction {
            modal
                .defer(&ctx.http)
                .await
                .expect("[ FAILED ] モーダルの応答に失敗しました");
            println!("[ OK ] モーダルを受信しました: {}", modal.data.custom_id);
            webhook_send::send(&ctx, modal)
                .await
                .expect("[ FAILED ] Webhookの作成に失敗しました");
        }
    }
}
