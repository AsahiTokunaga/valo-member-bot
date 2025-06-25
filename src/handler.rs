use serenity::async_trait;
use serenity::client::Context as SerenityContext;
use serenity::client::EventHandler;
use serenity::model::application::Interaction;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;

mod pin;
use pin::pin;
mod question;
use question::question;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: SerenityContext, ready: Ready) {
        println!("[ OK ] {}が起動しました", ready.user.name);
    }

    async fn message(&self, ctx: SerenityContext, msg: Message) {
        println!("[ OK ] メッセージを受信しました");
        if msg.author.id.to_string() != dotenv::var("BOT_ID").unwrap() {
            pin(ctx, &msg)
                .await
                .expect("[ FAILED ] 募集の作成に失敗しました");
        }
    }

    async fn interaction_create(&self, ctx: SerenityContext, interaction: Interaction) {
        if let Interaction::Component(component) = interaction {
            println!("[ OK ] インタラクションを受信しました");
            question(ctx, component)
                .await
                .expect("[ FAILED ] インタラクションの処理に失敗しました");
        }
    }
}
