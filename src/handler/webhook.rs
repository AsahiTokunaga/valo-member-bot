use anyhow::Result as AnyhowResult;
use once_cell::sync::Lazy;
use serenity::model::application::ComponentInteraction;
use serenity::model::id::{InteractionId, UserId, MessageId};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod create;
pub mod handler;
pub mod edit;

type Webhooks = Lazy<RwLock<HashMap<InteractionId, Arc<RwLock<WebhookDatas>>>>>;

pub static WEBHOOKS: Webhooks = Lazy::new(|| RwLock::new(HashMap::new()));
#[derive(Debug)]
pub struct WebhookDatas {
    pub ap_server: String,
    pub mode: String,
    pub rank: String,
    pub max_member: u8,
    pub joined: HashSet<UserId>,
}
impl WebhookDatas {
    pub async fn new(component: &ComponentInteraction) -> AnyhowResult<()> {
        let mut map = WEBHOOKS.write().await;
        map.insert(
            component.id,
            Arc::new(RwLock::new(WebhookDatas {
                ap_server: String::new(),
                mode: String::new(),
                rank: String::new(),
                max_member: 0,
                joined: HashSet::from([component.user.id]),
            })),
        );
        Ok(())
    }

    pub async fn with_mute<F>(id: &InteractionId, f: F) -> AnyhowResult<()>
    where 
        F: FnOnce(&mut WebhookDatas),
    {
        let mut map = WEBHOOKS.write().await;
        let mut webhook = if let Some(w) = map.get_mut(id) {
            w.write().await
        } else {
            return Err(anyhow::anyhow!(
                "[ FAILED ] InteractionIdに対するWebhookDatasが見つかりません: with_mute"
            ));
        };
        f(&mut webhook);
        Ok(())
    }

    pub async fn get(id: &InteractionId) -> Option<Arc<RwLock<WebhookDatas>>> {
        let map = WEBHOOKS.read().await;
        map.get(id).cloned()
    }
    pub async fn del(id: &InteractionId) -> AnyhowResult<()> {
        let mut map = WEBHOOKS.try_write()?;
        if map.remove(id).is_none() {
            return Err(anyhow::anyhow!(
                "[ FAILED ] InteractionIdに対するWebhookDatasが見つかりません: del"
            ));
        }
        Ok(())
    }
}

pub static INTERACTION_IDS: Lazy<RwLock<HashMap<MessageId, InteractionId>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

struct InteractionIdStore;

impl InteractionIdStore {
    pub async fn set(message_id: MessageId, interaction_id: InteractionId) -> AnyhowResult<()> {
        let mut map = INTERACTION_IDS.write().await;
        map.insert(message_id, interaction_id);
        Ok(())
    }
    pub async fn get(message_id: MessageId) -> AnyhowResult<InteractionId> {
        let map = INTERACTION_IDS.read().await;
        if let Some(value) = map.get(&message_id) {
            Ok(*value)
        } else {
            Err(anyhow::anyhow!(
                "[ FAILED ] MessageIdに対するInteractionIdが見つかりません: get"
            ))
        }
    }
    pub async fn del(message_id: MessageId) -> AnyhowResult<()> {
        let mut map = INTERACTION_IDS.write().await;
        if map.remove(&message_id).is_none() {
            return Err(anyhow::anyhow!(
                "[ FAILED ] MessageIdに対するInteractionIdが見つかりません: del"
            ));
        }
        Ok(())
    }
}