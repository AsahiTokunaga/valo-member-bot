use anyhow::Result as AnyhowResult;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use serenity::all::{Builder, ReactionType};
use serenity::builder::{
    CreateActionRow, CreateButton, CreateEmbed, CreateWebhook, ExecuteWebhook,
};
use serenity::client::Context as SerenityContext;
use serenity::model::application::ActionRowComponent;
use serenity::model::application::ButtonStyle;
use serenity::model::application::ModalInteraction;
use serenity::model::id::ChannelId;
use serenity::model::id::UserId;
use serenity::model::webhook::Webhook;
use std::str::FromStr;

use crate::dotenv_handler;
use crate::handler::questions::component_store::ComponentStore;
use crate::handler::webhook::{InteractionIdStore, WebhookDatas};
use crate::handler::{
    ASCENDANT_COLOR, BASE_COLOR, BRONZE_COLOR, DIAMOND_COLOR, GOLD_COLOR, IMMORTAL_COLOR,
    IRON_COLOR, PLATINUM_COLOR, RADIANT_COLOR, SILVER_COLOR,
};
use crate::valkey::Valkey;

pub async fn create(ctx: &SerenityContext, modal: ModalInteraction) -> AnyhowResult<()> {
    let user_id: UserId = modal.user.id;
    let channel_id = ChannelId::from_str(&dotenv_handler::get("CHANNEL_ID")?)?;
    let (user_name, user_avatar): (&str, &str) = (
        modal
            .user
            .global_name
            .as_ref()
            .unwrap_or_else(|| &modal.user.name),
        &modal
            .user
            .avatar_url()
            .unwrap_or_else(|| modal.user.default_avatar_url()),
    );
    let content = match modal
        .data
        .components
        .get(0)
        .and_then(|row| row.components.get(0))
    {
        Some(ActionRowComponent::InputText(input)) => Some(input.value.clone()),
        _ => None,
    };
    let button = get_button();
    let action_row = CreateActionRow::Buttons(button);
    let webhook = get_webhook(ctx, channel_id);
    let component = ComponentStore::get(user_id).await;
    let data = WebhookDatas::get(&component.id).await;
    if let Some(webhook_data) = data {
        let webhook_data = webhook_data.read().await;
        let embed = get_embed(ctx, &webhook_data);
        let mut builder = ExecuteWebhook::new()
            .avatar_url(user_avatar)
            .username(user_name)
            .embed(embed.await)
            .components(vec![action_row]);
        drop(webhook_data);
        if let Some(content) = content {
            if let Some(content) = content {
                builder = builder.content(content);
            }
        }

        let webhook = webhook.await?;
        let execute_webhook_handle = webhook.execute(&ctx.http, true, builder);
        let delete_response_handle = component.delete_response(&ctx.http);
        let (execute_webhook_result, _) =
            tokio::try_join!(execute_webhook_handle, delete_response_handle)?;
        if let Some(message) = execute_webhook_result {
            let message_id = message.id.to_string();
            store_user(message_id.as_str(), user_id.to_string().as_str()).await?;
            InteractionIdStore::set(message.id, component.id).await?;
            ComponentStore::del(user_id).await;
        }
    }
    Ok(())
}

async fn get_embed(ctx: &SerenityContext, info: &WebhookDatas) -> CreateEmbed {
    let mut users = info
        .joined
        .iter()
        .map(|user_id| user_id.to_user(&ctx.http))
        .collect::<FuturesUnordered<_>>();
    let mut names_vec: Vec<String> = Vec::new();
    while let Some(user) = users.next().await {
        if let Ok(user) = user {
            names_vec.push(user.to_string());
        }
    }

    let names = names_vec.join("\n");
    let mode = match info.mode.as_str() {
        "コンペティティブ" => &info.rank,
        _ => &info.mode,
    };
    let colour = get_colour(&info.rank);
    let thumbnail = get_thumbnail(&mode);
    let mut embed = CreateEmbed::new()
        .color(BASE_COLOR)
        .title(format!("({}/{})", info.joined.len(), info.max_member))
        .description(format!(
            "サーバー：{}\nモード　：{}{}",
            info.ap_server,
            info.mode,
            if info.rank.is_empty() {
                String::new()
            } else {
                format!("\nランク　：{}", info.rank)
            }
        ))
        .field("参加者", names, false);
    if let Some(url) = thumbnail {
        embed = embed.thumbnail(url);
    }
    if let Some(colour) = colour {
        embed = embed.colour(colour);
    }

    embed
}

fn get_button() -> Vec<CreateButton> {
    let join_button = CreateButton::new("参加する")
        .label("参加する")
        .style(ButtonStyle::Secondary)
        .emoji(ReactionType::Unicode("✋".to_string()));
    let leave_button = CreateButton::new("参加をやめる")
        .label("参加をやめる")
        .style(ButtonStyle::Secondary)
        .emoji(ReactionType::Unicode("👋".to_string()));
    let delete_button = CreateButton::new("削除")
        .label("削除")
        .style(ButtonStyle::Secondary)
        .emoji(ReactionType::Unicode("🚫".to_string()));
    vec![join_button, leave_button, delete_button]
}

async fn get_webhook(ctx: &SerenityContext, channel_id: ChannelId) -> AnyhowResult<Webhook> {
    let redis_pass = dotenv_handler::get("REDIS_PASS")?;
    if let Ok(Some(webhook_url)) = Valkey::get(&redis_pass, &channel_id.to_string()).await {
        if let Ok(webhook) = Webhook::from_url(&ctx.http, &webhook_url).await {
            return Ok(webhook);
        }
    }
    let builder = CreateWebhook::new("valo募集パネルwebhook")
        .execute(&ctx.http, channel_id)
        .await?;
    if let Ok(webhook_url) = builder.url() {
        Valkey::set(&redis_pass, &channel_id.to_string(), &webhook_url).await?;
    }
    Ok(builder)
}

fn get_thumbnail(rank: &str) -> Option<String> {
    let base_img_url = dotenv_handler::get("BASE_IMG_URL").unwrap_or_else(|e| {
        println!("{}", e);
        String::new()
    });
    match rank {
        "レディアント" => Some(format!("{}radiant.png", base_img_url)),
        "イモータル" => Some(format!("{}immortal.png", base_img_url)),
        "アセンダント" => Some(format!("{}ascendant.png", base_img_url)),
        "ダイヤモンド" => Some(format!("{}diamond.png", base_img_url)),
        "プラチナ" => Some(format!("{}platinum.png", base_img_url)),
        "ゴールド" => Some(format!("{}gold.png", base_img_url)),
        "シルバー" => Some(format!("{}silver.png", base_img_url)),
        "ブロンズ" => Some(format!("{}bronze.png", base_img_url)),
        "アイアン" => Some(format!("{}iron.png", base_img_url)),
        "どこでも" => Some(format!("{}unranked.png", base_img_url)),
        "アンレート" => Some(format!("{}unrated.png", base_img_url)),
        _ => None,
    }
}

fn get_colour(rank: &str) -> Option<u32> {
    match rank {
        "アイアン" => Some(IRON_COLOR),
        "ブロンズ" => Some(BRONZE_COLOR),
        "シルバー" => Some(SILVER_COLOR),
        "ゴールド" => Some(GOLD_COLOR),
        "プラチナ" => Some(PLATINUM_COLOR),
        "ダイヤモンド" => Some(DIAMOND_COLOR),
        "アセンダント" => Some(ASCENDANT_COLOR),
        "イモータル" => Some(IMMORTAL_COLOR),
        "レディアント" => Some(RADIANT_COLOR),
        _ => None,
    }
}

async fn store_user(message_id: &str, channel_id: &str) -> AnyhowResult<()> {
    let redis_pass = dotenv_handler::get("REDIS_PASS")?;
    Valkey::ttl_set(redis_pass.as_str(), message_id, channel_id, 259200).await
}
