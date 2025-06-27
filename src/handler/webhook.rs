use anyhow::Result as AnyhowResult;
use once_cell::sync::Lazy;
use serenity::model::application::ComponentInteraction;
use serenity::model::id::InteractionId;
use serenity::model::id::UserId;
use tokio::sync::RwLock;
use std::collections::HashMap;
use serenity::model::user::User;
use std::sync::Arc;

pub mod create;

type Webhook = Lazy<RwLock<HashMap<InteractionId, Arc<RwLock<WebhookHandler>>>>>;

pub static WEBHOOKS: Webhook = Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Debug)]
pub struct WebhookHandler {
    pub ap_server: String,
    pub mode: String,
    pub max_member: u8,
    pub joined: Vec<UserId>,
}

impl WebhookHandler {
    pub async fn new(component: &ComponentInteraction) -> AnyhowResult<()> {
        let mut map = WEBHOOKS.try_write()?;
        map.insert(
            component.id,
            Arc::new(RwLock::new(WebhookHandler {
                ap_server: String::new(),
                mode: String::new(),
                max_member: 0,
                joined: vec![component.user.id],
            })),
        );
        Ok(())
    }
    pub async fn set_ap_server(
        component: &ComponentInteraction,
        ap_server: String,
    ) -> AnyhowResult<()> {
        let id = component.id;
        let mut map = WEBHOOKS.try_write()?;
        let mut webhok = if let Some(webhook) = map.get_mut(&id) {
            webhook.write().await
        } else {
            Err(anyhow::anyhow!(
                "[ FAILED ] InteractionIdに対するWebhookHandlerが見つかりません: set_ap_server"
            ))?
        };
        webhok.ap_server = ap_server;
        Ok(())
    }
    pub async fn get(component: &ComponentInteraction) -> AnyhowResult<WebhookHandler> {
        let id = component.id;
        let map = WEBHOOKS.try_read()?;
        if let Some(webhook) = map.get(&id) {
            let webhook = webhook.read().await;
            let response = WebhookHandler {
                ap_server: webhook.ap_server.clone(),
                mode: webhook.mode.clone(),
                max_member: webhook.max_member,
                joined: webhook.joined.clone(),
            };
            Ok(response)
        } else {
            Err(anyhow::anyhow!(
                "[ FAILED ] InteractionIdに対するWebhookHandlerが見つかりません: get"
            ))?
        }
    }
    pub async fn set_mode(component: &ComponentInteraction, mode: &str) -> AnyhowResult<()> {
        let id = component.id;
        let mut map = WEBHOOKS.try_write()?;
        let mut webhook = if let Some(webhook) = map.get_mut(&id) {
            webhook.write().await
        } else {
            Err(anyhow::anyhow!(
                "[ FAILED ] InteractionIdに対するWebhookHandlerが見つかりません: set_mode"
            ))?
        };
        webhook.mode = mode.to_string();
        Ok(())
    }

    pub async fn set_max_member(component: &ComponentInteraction, max: u8) -> AnyhowResult<()> {
        let id = component.id;
        let mut map = WEBHOOKS.try_write()?;
        let mut webhook = if let Some(webhook) = map.get_mut(&id) {
            webhook.try_write()?
        } else {
            Err(anyhow::anyhow!(
                "[ FAILED ] InteractionIdに対するWebhookHandlerが見つかりません: add_member"
            ))?
        };
        webhook.max_member = max;
        Ok(())
    }
    
    // TODO: webhookメッセージのボタンを押すと下記の関数で値が増加する
    pub async fn _add_joined(component: &ComponentInteraction, user: &User) -> AnyhowResult<()> {
        let id = component.id;
        let mut map = WEBHOOKS.try_write()?;
        let mut webhook = if let Some(webhook) = map.get_mut(&id) {
            webhook.try_write()?
        } else {
            Err(anyhow::anyhow!(
                "[ FAILED ] InteractionIdに対するWebhookHandlerが見つかりません: add_joined"
            ))?
        };
        webhook.joined.push(user.id);
        Ok(())
    }
}
