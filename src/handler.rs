use crate::dotenv_handler;
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
mod webhook;
use crate::handler::webhook::create::create as webhook_create;

const PIN_MESSAGE_COLOR: u32 = 0xffffff;
const BASE_COLOR: u32 = 0xff5152;
const RADIANT_COLOR: u32 = 0xf9efa2;
const IMMORTAL_COLOR: u32 = 0xbf334c;
const ASCENDANT_COLOR: u32 = 0x1e8a51;
const DIAMOND_COLOR: u32 = 0xc587f5;
const PLATINUM_COLOR: u32 = 0x39a1b1;
const GOLD_COLOR: u32 = 0xeabd44;
const SILVER_COLOR: u32 = 0xd9e0dd;
const BRONZE_COLOR: u32 = 0x865900;
const IRON_COLOR: u32 = 0x5197647;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: SerenityContext, ready: Ready) {
        println!("[ OK ] {}が起動しました", ready.user.name);
    }

    async fn message(&self, ctx: SerenityContext, msg: Message) {
        println!("[ OK ] メッセージを受信しました");
        if msg.author.id.to_string()
            != dotenv_handler::get("BOT_ID")
                .expect("[ FAILED ] BOT_IDが設定されていません")
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
            questions(ctx, component)
                .await
                .expect("[ FAILED ] インタラクションの処理に失敗しました");
        } else if let Interaction::Modal(modal) = interaction {
            modal
                .defer(&ctx.http)
                .await
                .expect("[ FAILED ] モーダルの応答に失敗しました");
            println!("[ OK ] モーダルを受信しました: {}", modal.data.custom_id);
            webhook_create(&ctx, modal)
                .await
                .expect("[ FAILED ] Webhookの作成に失敗しました");
        }
    }
}
