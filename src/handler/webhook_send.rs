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
use serenity::model::id::{ChannelId, MessageId, UserId};
use serenity::model::webhook::Webhook;
use std::str::FromStr;

use crate::dotenv_handler;
use crate::handler::colors::{
    ASCENDANT_COLOR, BASE_COLOR, BRONZE_COLOR, DIAMOND_COLOR, GOLD_COLOR, IMMORTAL_COLOR,
    IRON_COLOR, PLATINUM_COLOR, RADIANT_COLOR, SILVER_COLOR,
};
use crate::state_handler::methods::{component_store_map, interaction_id_map, webhook_map};
use crate::state_handler::{Mode, Rank, WebhookData};
use crate::valkey::commands;

pub async fn send(ctx: &SerenityContext, modal: ModalInteraction) -> AnyhowResult<()> {
    let user_id: UserId = modal.user.id;
    let channel_id = ChannelId::from_str(&dotenv_handler::get("CHANNEL_ID")?)?;
    let (user_name, user_avatar): (&str, &str) = (
        modal.user.global_name.as_ref().unwrap_or(&modal.user.name),
        &modal
            .user
            .avatar_url()
            .unwrap_or(modal.user.default_avatar_url()),
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
    let component = component_store_map::get(&ctx, &user_id).await;
    if let Some(component) = component {
        let component = component.read().await;
        let data = webhook_map::get(&ctx, &component.id).await;
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
                let (s, i, _) = tokio::join!(
                    store_user(message.id, user_id),
                    interaction_id_map::set(&ctx, message.id, component.id),
                    component_store_map::del(&ctx, &user_id)
                );
                drop(component);
                match (s, i) {
                    (Ok(()), Ok(())) => {}
                    (Err(e), _) => {
                        eprintln!(
                            "[ FAILED ] MessageIdに対するUserIdの保存に失敗しました: {}",
                            e
                        );
                    }
                    (_, Err(e)) => {
                        eprintln!("[ FAILED ] インタラクションIDの保存に失敗しました: {}", e);
                    }
                }
            }
        }
    }
    Ok(())
}

async fn get_embed(ctx: &SerenityContext, webhook_data: &WebhookData) -> CreateEmbed {
    let mut users = webhook_data
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
    let colour = get_color(webhook_data);
    let thumbnail = get_thumbnail(webhook_data);
    let mut embed = CreateEmbed::new()
        .color(BASE_COLOR)
        .title(format!(
            "({}/{})",
            webhook_data.joined.len(),
            webhook_data.max_member
        ))
        .description(format!(
            "サーバー：{}\nモード　：{}{}",
            webhook_data.ap_server,
            webhook_data.mode,
            if webhook_data.rank.is_none() {
                String::new()
            } else {
                format!("\nランク　：{}", webhook_data.rank.unwrap().to_string())
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
    if let Ok(Some(webhook_url)) = commands::get(&redis_pass, &channel_id.to_string()).await {
        if let Ok(webhook) = Webhook::from_url(&ctx.http, &webhook_url).await {
            return Ok(webhook);
        }
    }
    let builder = CreateWebhook::new("valo募集パネルwebhook")
        .execute(&ctx.http, channel_id)
        .await?;
    if let Ok(webhook_url) = builder.url() {
        commands::set(&redis_pass, &channel_id.to_string(), &webhook_url).await?;
    }
    Ok(builder)
}

fn get_thumbnail(webhook_data: &WebhookData) -> Option<String> {
    let base_img_url = dotenv_handler::get("BASE_IMG_URL").unwrap_or(String::new());
    if let Some(rank) = &webhook_data.rank {
        match rank {
            Rank::Radiant => Some(format!("{}radiant.png", base_img_url)),
            Rank::Immortal => Some(format!("{}immortal.png", base_img_url)),
            Rank::Ascendant => Some(format!("{}ascendant.png", base_img_url)),
            Rank::Diamond => Some(format!("{}diamond.png", base_img_url)),
            Rank::Platinum => Some(format!("{}platinum.png", base_img_url)),
            Rank::Gold => Some(format!("{}gold.png", base_img_url)),
            Rank::Silver => Some(format!("{}silver.png", base_img_url)),
            Rank::Bronze => Some(format!("{}bronze.png", base_img_url)),
            Rank::Iron => Some(format!("{}iron.png", base_img_url)),
            Rank::Unranked => Some(format!("{}unranked.png", base_img_url)),
        }
    } else if webhook_data.mode == Mode::Unrated {
        Some(format!("{}unrated.png", base_img_url))
    } else {
        None
    }
}

fn get_color(webhook_data: &WebhookData) -> Option<u32> {
    if let Some(rank) = &webhook_data.rank {
        match rank {
            Rank::Iron => Some(IRON_COLOR),
            Rank::Bronze => Some(BRONZE_COLOR),
            Rank::Silver => Some(SILVER_COLOR),
            Rank::Gold => Some(GOLD_COLOR),
            Rank::Platinum => Some(PLATINUM_COLOR),
            Rank::Diamond => Some(DIAMOND_COLOR),
            Rank::Ascendant => Some(ASCENDANT_COLOR),
            Rank::Immortal => Some(IMMORTAL_COLOR),
            Rank::Radiant => Some(RADIANT_COLOR),
            Rank::Unranked => None,
        }
    } else {
        None
    }
}

async fn store_user(message_id: MessageId, user_id: UserId) -> AnyhowResult<()> {
    let redis_pass = dotenv_handler::get("REDIS_PASS")?;
    commands::ttl_set(
        redis_pass.as_str(),
        message_id.to_string().as_str(),
        user_id.to_string().as_str(),
        259200,
    )
    .await
}
