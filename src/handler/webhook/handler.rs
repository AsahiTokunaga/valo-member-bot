use std::str::FromStr;

use anyhow::{Context, Result as AnyhowResult};
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::client::Context as SerenityContext;
use serenity::http::CacheHttp;
use serenity::http::Http;
use serenity::model::application::ComponentInteraction;
use serenity::model::id::{InteractionId, UserId};

use crate::dotenv_handler;
use crate::handler::webhook::edit::edit;
use crate::handler::webhook::{InteractionIdStore, WebhookDatas};
use crate::valkey::Valkey;

pub async fn join(ctx: SerenityContext, component: ComponentInteraction) -> AnyhowResult<()> {
    let enter_join_user = component.user.id;
    let message_id = component.message.id;
    let interaction_id = if let Ok(id) = InteractionIdStore::get(message_id).await {
        id
    } else {
        timeout(&ctx.http, &component).await?;
        return Ok(());
    };
    let data = WebhookDatas::get(&interaction_id).await;
    if let Some(webhook_data) = data {
        let webhook_data = webhook_data.read().await;
        if webhook_data.joined.contains(&enter_join_user) {
            response(&ctx.http, &component, "すでに募集に参加しています").await?;
        } else {
            drop(webhook_data);
            update_webhook_data(&interaction_id, enter_join_user, 'p').await?;
            let webhook_data = WebhookDatas::get(&interaction_id).await.unwrap();
            let webhook_data = webhook_data.read().await;
            let names = get_field_value(&webhook_data).await;
            let title: (&usize, &u8) = (&webhook_data.joined.len(), &webhook_data.max_member);
            let (e, r) = tokio::join!(
                edit(&ctx, message_id, &names, title),
                response(&ctx.http, &component, "募集に参加しました")
            );
            drop(webhook_data);
            match (e, r) {
                (Ok(()), Ok(())) => {}
                (Err(e), _) => {
                    eprintln!("[ FAILED ] 募集の編集に失敗しました: {}", e);
                }
                (_, Err(e)) => {
                    eprintln!("[ FAILED ] 参加の応答に失敗しました: {}", e);
                }
            }
        }
    }
    Ok(())
}

pub async fn leave(ctx: SerenityContext, component: ComponentInteraction) -> AnyhowResult<()> {
    let enter_leave_user = component.user.id;
    let message_id = component.message.id;
    let redis_pass =
        dotenv_handler::get("REDIS_PASS").context("[ FAILED ] REDIS_PASSが設定されていません")?;
    let linked_message_user = Valkey::get(&redis_pass, message_id.to_string().as_str()).await?;
    if let Some(user) = linked_message_user {
        let linked_message_user_id = UserId::from_str(&user)?;
        let interaction_id = if let Ok(id) = InteractionIdStore::get(message_id).await {
            id
        } else {
            timeout(&ctx.http, &component).await?;
            return Ok(());
        };
        let data = WebhookDatas::get(&interaction_id).await;
        if let Some(webhook_data) = data {
            let webhook_data = webhook_data.read().await;
            if linked_message_user_id == enter_leave_user {
                response(&ctx.http, &component, "募集の作成者は参加を取り消せません\n募集取り消しの場合は削除ボタンを押してください").await?;
            } else {
                if webhook_data.joined.contains(&enter_leave_user) {
                    drop(webhook_data);
                    update_webhook_data(&interaction_id, enter_leave_user, 'r').await?;
                    let webhook_data = WebhookDatas::get(&interaction_id).await.unwrap();
                    let webhook_data = webhook_data.read().await;
                    let names = get_field_value(&webhook_data).await;
                    let title: (&usize, &u8) =
                        (&webhook_data.joined.len(), &webhook_data.max_member);
                    let (e, r) = tokio::join!(
                        edit(&ctx, message_id, &names, title),
                        response(&ctx.http, &component, "参加を取り消しました")
                    );
                    drop(webhook_data);
                    match (e, r) {
                        (Ok(()), Ok(())) => {}
                        (Err(e), _) => {
                            eprintln!("[ FAILED ] 募集の編集に失敗しました: {}", e);
                        }
                        (_, Err(e)) => {
                            eprintln!("[ FAILED ] 参加の取り消しの応答に失敗しました: {}", e);
                        }
                    }
                } else {
                    response(&ctx.http, &component, "募集に参加していません").await?;
                }
            }
        }
    } else {
        timeout(&ctx.http, &component).await?;
    }
    Ok(())
}

pub async fn delete(ctx: SerenityContext, component: ComponentInteraction) -> AnyhowResult<()> {
    let enter_delete_user = component.user.id;
    let message_id = component.message.id.to_string();
    let redis_pass =
        dotenv_handler::get("REDIS_PASS").context("[ FAILED ] REDIS_PASSが設定されていません")?;
    let linked_message_user = Valkey::get(&redis_pass, message_id.as_str()).await?;
    if let Some(user) = linked_message_user {
        let linked_message_user_id = UserId::from_str(&user)?;
        if linked_message_user_id != enter_delete_user {
            response(
                &ctx.http,
                &component,
                "募集の取り消しは、募集作成者のみ行えます",
            )
            .await?;
        } else {
            let channel_id = component.message.channel_id;
            channel_id
                .delete_message(&ctx.http, component.message.id)
                .await
                .context("[ FAILED ] メッセージの削除に失敗しました")?;
            let interaction_id = InteractionIdStore::get(component.message.id).await?;
            let (w, i) = tokio::join!(
                WebhookDatas::del(&interaction_id),
                InteractionIdStore::del(component.message.id)
            );
            match (w, i) {
                (Ok(()), Ok(())) => {}
                (Err(e), _) => {
                    eprintln!("[ FAILED ] 募集の削除に失敗しました: {}", e);
                }
                (_, Err(e)) => {
                    eprintln!("[ FAILED ] インタラクションIDの削除に失敗しました: {}", e);
                }
            }
        }
    } else {
        timeout(&ctx.http, &component).await?;
    }
    Ok(())
}

async fn timeout<T: AsRef<Http> + CacheHttp + Copy>(
    http: T,
    comp: &ComponentInteraction,
) -> AnyhowResult<()> {
    let res = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .ephemeral(true)
            .content("有効期限切れのため募集を削除します"),
    );
    let message_id = comp.message.id;
    let channel_id = comp.message.channel_id;
    let create_response_task = comp.create_response(http, res);
    let delete_message_task = channel_id.delete_message(http, message_id);
    tokio::try_join!(create_response_task, delete_message_task)?;
    Ok(())
}

async fn response<T: CacheHttp>(
    http: T,
    comp: &ComponentInteraction,
    cont: &str,
) -> AnyhowResult<()> {
    let res = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .ephemeral(true)
            .content(cont),
    );
    comp.create_response(http, res).await?;
    Ok(())
}

async fn update_webhook_data(
    interaction_id: &InteractionId,
    user_id: UserId,
    p_r: char,
) -> AnyhowResult<()> {
    match p_r {
        'p' => {
            WebhookDatas::with_mute(interaction_id, |w| {
                w.joined.insert(user_id);
            })
            .await
        }
        'r' => {
            WebhookDatas::with_mute(interaction_id, |w| {
                w.joined.remove(&user_id);
            })
            .await
        }
        _ => {
            return Err(anyhow::anyhow!("I'm a teapot"));
        }
    }
}

async fn get_field_value(webhook_data: &WebhookDatas) -> String {
    webhook_data
        .joined
        .iter()
        .map(|user_id| format!("<@{}>", user_id))
        .collect::<Vec<String>>()
        .join("\n")
}
