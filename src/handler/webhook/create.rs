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

pub async fn create(ctx: &SerenityContext, modal: ModalInteraction) -> AnyhowResult<()> {
    let user_id = modal.user.id;
    let channel_id_string = dotenv_handler::get("CHANNEL_ID")?;
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
    let mode = match info.mode.as_str() {
        "コンペティティブ" => &info.rank,
        _ => &info.mode,
    };
    let colour = get_colour(&info.rank);
    let thumbnail = get_thumbnail(&mode);
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
    if let Some(colour) = colour {
        embed = embed.colour(colour);
    }
    embed
}

async fn get_button() -> CreateButton {
    let emoji =
        dotenv_handler::get("JOIN_EMOJI").expect("[ FAILED ] JOIN_EMOJIが設定されていません");
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

fn get_thumbnail(webhook: &str) -> Option<String> {
    let base_img_url = dotenv_handler::get("BASE_IMG_URL").unwrap_or_else(|e| {
        println!("{}", e);
        String::new()
    });
    match webhook {
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
        "アイアン" => Some(5197647),
        "ブロンズ" => Some(8804608),
        "シルバー" => Some(14278877),
        "ゴールド" => Some(15383876),
        "プラチナ" => Some(3776945),
        "ダイヤモンド" => Some(12945397),
        "アセンダント" => Some(2001489),
        "イモータル" => Some(12530508),
        "レディアント" => Some(16379810),
        _ => None,
    }
}
