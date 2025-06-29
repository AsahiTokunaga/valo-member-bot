use std::str::FromStr;

use anyhow::Result as AnyhowResult;
use serenity::all::Builder;
use serenity::builder::{
    CreateActionRow, CreateButton, CreateEmbed, CreateWebhook, ExecuteWebhook,
};
use serenity::client::Context as SerenityContext;
use serenity::model::application::ActionRowComponent;
use serenity::model::application::ButtonStyle;
use serenity::model::application::ModalInteraction;
use serenity::model::id::{ChannelId, EmojiId};
use serenity::model::webhook::Webhook;

use crate::dotenv_handler;
use crate::handler::questions::component_handler::ComponentHandler;
use crate::handler::webhook::WebhookHandler;
use crate::valkey::Valkey;

const BASE_IMG_URL: &str =
    "https://raw.githubusercontent.com/AsahiTokunaga/valo-member-bot/feature/get-image/imgs/";

pub async fn create(ctx: &SerenityContext, modal: ModalInteraction) -> AnyhowResult<()> {
    let user_id = modal.user.id;
    let channel_id_string = dotenv_handler::get("CHANNEL_ID").await?;
    let channel_id = ChannelId::from_str(&channel_id_string)?;
    let webhook = get_webhook(&ctx, channel_id).await?;

    let user_name = modal
        .user
        .global_name
        .as_ref()
        .unwrap_or_else(|| &modal.user.name);
    let user_avatar = modal
        .user
        .avatar_url()
        .unwrap_or_else(|| modal.user.default_avatar_url());
    let content = if let Some(components) = modal.data.components.get(0) {
        if let Some(components) = components.components.get(0) {
            if let ActionRowComponent::InputText(input_text) = components {
                input_text.value.clone()
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let component = if let Some(component) = ComponentHandler::get(user_id).await {
        component
    } else {
        return Err(anyhow::anyhow!(
            "[ FAILED ] 募集の作成に失敗しました: ユーザーのコンポーネントが見つかりません"
        ));
    };
    let info = WebhookHandler::get(&component).await?;
    let embed = get_embed(&ctx, &info).await;
    let button = get_button().await;
    let action_row = CreateActionRow::Buttons(vec![button]);
    let mut builder = ExecuteWebhook::new()
        .avatar_url(user_avatar)
        .username(user_name)
        .embed(embed)
        .components(vec![action_row]);
    builder = if let Some(content) = content {
        builder.content(content)
    } else {
        builder
    };
    let _message = webhook.execute(&ctx.http, true, builder).await?;
    component.delete_response(&ctx.http).await?;
    Ok(())
}

async fn get_embed(ctx: &SerenityContext, info: &WebhookHandler) -> CreateEmbed {
    let mut names: Vec<String> = Vec::new();
    for user_id in &info.joined {
        if let Ok(user) = user_id.to_user(&ctx.http).await {
            let name = user.global_name.unwrap_or_else(|| user.name.clone());
            names.push(name);
        }
    }
    let mode = if info.mode == "アンレート" {
        "アンレート"
    } else if info.mode == "コンペティティブ" {
        info.rank.as_str()
    } else {
        "カスタム"
    };
    let thumbnail = get_thumbnail(&mode).await;
    let mut embed = CreateEmbed::new()
        .color(16732498)
        .description(format!(
            "## ({}/{}) {}\nサーバー：{}",
            info.joined.len(),
            info.max_member,
            info.mode,
            info.ap_server,
        ))
        .field("参加者", names.join("\n"), false);
    if let Some(url) = thumbnail {
        embed = embed.thumbnail(url);
    }
    embed
}

async fn get_button() -> CreateButton {
    let emoji = dotenv_handler::get("JOIN_EMOJI")
        .await
        .expect("[ FAILED ] JOIN_EMOJIが設定されていません");
    let button = CreateButton::new("参加する")
        .label("参加する")
        .style(ButtonStyle::Secondary)
        .emoji(EmojiId::new(
            emoji
                .parse::<u64>()
                .expect("[ FAILED ] JOIN_EMOJIのパースに失敗しました"),
        ));
    button
}

async fn get_webhook(ctx: &SerenityContext, channel_id: ChannelId) -> AnyhowResult<Webhook> {
    let redis_pass = dotenv_handler::get("REDIS_PASS").await?;
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

async fn get_thumbnail(webhook: &str) -> Option<String> {
    match webhook {
        "レディアント" => Some(format!("{}radiant.png", BASE_IMG_URL)),
        "イモータル" => Some(format!("{}immortal.png", BASE_IMG_URL)),
        "アセンダント" => Some(format!("{}ascendant.png", BASE_IMG_URL)),
        "ダイヤモンド" => Some(format!("{}diamond.png", BASE_IMG_URL)),
        "プラチナ" => Some(format!("{}platinum.png", BASE_IMG_URL)),
        "ゴールド" => Some(format!("{}gold.png", BASE_IMG_URL)),
        "シルバー" => Some(format!("{}silver.png", BASE_IMG_URL)),
        "ブロンズ" => Some(format!("{}bronze.png", BASE_IMG_URL)),
        "アイアン" => Some(format!("{}iron.png", BASE_IMG_URL)),
        "どこでも" => Some(format!("{}unranked.png", BASE_IMG_URL)),
        "アンレート" => Some(format!("{}unrated.png", BASE_IMG_URL)),
        _ => None,
    }
}
