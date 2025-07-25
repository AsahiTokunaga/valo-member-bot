use serenity::all::{CacheHttp, CreateEmbed, ExecuteWebhook, Http};

use crate::{
  bot::{
    colors::BASE_COLOR,
    panels::{get_button, get_thumbnail},
    types::{RedisClient, WebhookData, WebhookDataExt}
  },
  error::BotError
};

pub async fn send<T: AsRef<Http> + CacheHttp + Copy>(http: T, redis_client: &RedisClient, webhook_data: &WebhookData, cont: Option<&str>) -> Result<(), BotError> {
  let webhook = redis_client.get_webhook(http);
  let joined_users: String = webhook_data.joined
    .iter()
    .map(|&u| format!("<@{}>", u.get()))
    .collect::<Vec<String>>()
    .join("\n");
  let thumbail = get_thumbnail(webhook_data.rank)?;
  let embed = CreateEmbed::new()
    .title(format!("({}/{})", webhook_data.joined.len(), u8::from(webhook_data.member)))
    .color(webhook_data.rank.map_or(BASE_COLOR, |r| r.to_color()))
    .description(format!(
      "サーバー：{}\nモード　：{}{}",
      webhook_data.server.as_str(),
      webhook_data.mode.as_str(),
      webhook_data.rank.map_or(String::new(), |r| format!("\nランク　：{}", r.as_str()))
    ))
    .thumbnail(thumbail)
    .field("参加者", joined_users, false);
  let buttons = get_button(false);
  let creator = webhook_data.creator.to_user(http).await?;
  let webhook_message = ExecuteWebhook::new()
    .username(creator.display_name())
    .avatar_url(creator.face())
    .embed(embed)
    .content(cont.map_or(format!("{}", webhook_data.mode.to_mention_str()), |f| format!("{} {}", webhook_data.mode.to_mention_str(), f)))
    .components(vec![buttons]);

  // 第2引数がtrueのため必ずSomeを返す
  // 詳細: https://docs.rs/serenity/latest/serenity/http/struct.Http.html#method.execute_webhook
  // Webhook::execute() -> ExecuteWebhook::execute() -> Http::execute_webhook()のラッパー
  let message = webhook.await?.execute(http, true, webhook_message).await?.unwrap();
  redis_client.store_webhook_data(message.id, webhook_data).await?;
  Ok(())
}
