use anyhow::Result as AnyhowResult;
use once_cell::sync::Lazy;
use serenity::model::application::ComponentInteraction;
use serenity::model::id::InteractionId;
use serenity::model::id::UserId;
use serenity::model::user::User;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod create;

type Webhooks = Lazy<RwLock<HashMap<InteractionId, Arc<RwLock<WebhookDatas>>>>>;

pub static WEBHOOKS: Webhooks = Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Debug)]
pub struct WebhookDatas {
    pub ap_server: String,
    pub mode: String,
    pub rank: String,
    pub max_member: u8,
    pub joined: Vec<UserId>,
}

impl WebhookDatas {
    pub async fn new(component: &ComponentInteraction) -> AnyhowResult<()> {
        let mut map = WEBHOOKS.try_write()?;
        map.insert(
            component.id,
            Arc::new(RwLock::new(WebhookDatas {
                ap_server: String::new(),
                mode: String::new(),
                rank: String::new(),
                max_member: 0,
                joined: vec![component.user.id],
            })),
        );
        Ok(())
    }

    pub async fn with_mute<F>(component: &ComponentInteraction, f: F) -> AnyhowResult<()>
    where 
        F: FnOnce(&mut WebhookDatas),
    {
        let id = component.id;
        let mut map = WEBHOOKS.write().await;
        let mut webhook = if let Some(w) = map.get_mut(&id) {
            w.write().await
        } else {
            return Err(anyhow::anyhow!(
                "[ FAILED ] InteractionIdに対するWebhookDatasが見つかりません: with_mute"
            ));
        };
        f(&mut webhook);
        Ok(())
    }

    pub async fn get(component: &ComponentInteraction) -> AnyhowResult<WebhookDatas> {
        let id = component.id;
        let map = WEBHOOKS.try_read()?;
        if let Some(webhook) = map.get(&id) {
            let webhook = webhook.read().await;
            let response = WebhookDatas {
                ap_server: webhook.ap_server.clone(),
                mode: webhook.mode.clone(),
                rank: webhook.rank.clone(),
                max_member: webhook.max_member,
                joined: webhook.joined.clone(),
            };
            Ok(response)
        } else {
            Err(anyhow::anyhow!(
                "[ FAILED ] InteractionIdに対するWebhookDatasが見つかりません: get"
            ))?
        }
    }
    // TODO: webhookメッセージのボタンを押すと下記の関数で値が増加する
    pub async fn _add_joined(component: &ComponentInteraction, user: &User) -> AnyhowResult<()> {
        let id = component.id;
        let mut map = WEBHOOKS.try_write()?;
        let mut webhook = if let Some(webhook) = map.get_mut(&id) {
            webhook.try_write()?
        } else {
            Err(anyhow::anyhow!(
                "[ FAILED ] InteractionIdに対するWebhookDatasが見つかりません: add_joined"
            ))?
        };
        webhook.joined.push(user.id);
        Ok(())
    }
}
