use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::async_trait;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("[ OK ] {}が起動しました", ready.user.name)
    }
}