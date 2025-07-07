use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use serenity::{
    all::{ComponentInteraction, InteractionId, MessageId, UserId},
    prelude::TypeMapKey,
};
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct WebhookData {
    pub ap_server: String,
    pub mode: String,
    pub rank: String,
    pub max_member: u8,
    pub joined: HashSet<UserId>,
}

pub struct WebhookMap;
impl TypeMapKey for WebhookMap {
    type Value = Arc<RwLock<HashMap<InteractionId, Arc<RwLock<WebhookData>>>>>;
}

pub struct InteractionIdMap;
impl TypeMapKey for InteractionIdMap {
    type Value = Arc<RwLock<HashMap<MessageId, InteractionId>>>;
}

pub struct ComponentStoreMap;
impl TypeMapKey for ComponentStoreMap {
    type Value = Arc<RwLock<HashMap<UserId, Arc<RwLock<ComponentInteraction>>>>>;
}

pub mod methods {
    use super::*;
    use anyhow::Result as AnyhowResult;
    use serenity::all::Context;
    pub mod webhook_map {
        use super::*;
        pub async fn new(
            ctx: &Context,
            interaction_id: InteractionId,
            user_id: UserId,
        ) -> AnyhowResult<()> {
            let map =
                get_typemap_arc::<WebhookMap, InteractionId, Arc<RwLock<WebhookData>>>(ctx).await;
            let mut map = map.write().await;
            map.insert(
                interaction_id,
                Arc::new(RwLock::new(WebhookData {
                    ap_server: String::new(),
                    mode: String::new(),
                    rank: String::new(),
                    max_member: 1,
                    joined: HashSet::from([user_id]),
                })),
            );
            Ok(())
        }
        pub async fn with_mute<F>(ctx: &Context, key: &InteractionId, f: F) -> AnyhowResult<()>
        where
            F: FnOnce(&mut WebhookData),
        {
            let map =
                get_typemap_arc::<WebhookMap, InteractionId, Arc<RwLock<WebhookData>>>(ctx).await;
            let mut map = map.write().await;
            let mut webhook = if let Some(w) = map.get_mut(key) {
                w.write().await
            } else {
                return Err(anyhow::anyhow!(
                    "[ FAILED ] InteractionIdに対するWebhookDataが見つかりません: with_mute"
                ));
            };
            f(&mut webhook);
            Ok(())
        }
        pub async fn get(ctx: &Context, key: &InteractionId) -> Option<Arc<RwLock<WebhookData>>> {
            let map =
                get_typemap_arc::<WebhookMap, InteractionId, Arc<RwLock<WebhookData>>>(ctx).await;
            map.read().await.get(key).cloned()
        }
        pub async fn del(ctx: &Context, key: &InteractionId) -> AnyhowResult<()> {
            let map =
                get_typemap_arc::<WebhookMap, InteractionId, Arc<RwLock<WebhookData>>>(ctx).await;
            let mut map = map.write().await;
            if map.remove(key).is_none() {
                return Err(anyhow::anyhow!(
                    "
                    [ FAILED ] InteractionIdに対するWebhookDataが見つかりません: del"
                ));
            }
            Ok(())
        }
    }
    pub mod interaction_id_map {
        use super::*;
        pub async fn set(
            ctx: &Context,
            message_id: MessageId,
            interaction_id: InteractionId,
        ) -> AnyhowResult<()> {
            let map = get_typemap_arc::<InteractionIdMap, MessageId, InteractionId>(ctx).await;
            let mut map = map.write().await;
            map.insert(message_id, interaction_id);
            Ok(())
        }
        pub async fn get(ctx: &Context, key: &MessageId) -> AnyhowResult<InteractionId> {
            let map = get_typemap_arc::<InteractionIdMap, MessageId, InteractionId>(ctx).await;
            let map = map.read().await;
            if let Some(interaction_id) = map.get(key) {
                Ok(*interaction_id)
            } else {
                Err(anyhow::anyhow!(
                    "[ FAILED ] MessageIdに対するInteractionIdが見つかりません: get"
                ))
            }
        }
        pub async fn del(ctx: &Context, key: &MessageId) -> AnyhowResult<()> {
            let map = get_typemap_arc::<InteractionIdMap, MessageId, InteractionId>(ctx).await;
            let mut map = map.write().await;
            if map.remove(key).is_none() {
                return Err(anyhow::anyhow!(
                    "[ FAILED ] MessageIdに対するInteractionIdが見つかりません: del"
                ));
            }
            Ok(())
        }
    }
    pub mod component_store_map {
        use super::*;
        pub async fn set(
            ctx: &Context,
            user_id: UserId,
            component: &ComponentInteraction,
        ) -> AnyhowResult<()> {
            let map =
                get_typemap_arc::<ComponentStoreMap, UserId, Arc<RwLock<ComponentInteraction>>>(
                    ctx,
                )
                .await;
            let mut map = map.write().await;
            map.insert(user_id, Arc::new(RwLock::new(component.clone())));
            Ok(())
        }
        pub async fn get(ctx: &Context, key: &UserId) -> Option<Arc<RwLock<ComponentInteraction>>> {
            let map =
                get_typemap_arc::<ComponentStoreMap, UserId, Arc<RwLock<ComponentInteraction>>>(
                    ctx,
                )
                .await;
            let map = map.read().await;
            map.get(key).cloned()
        }
        pub async fn del(ctx: &Context, key: &UserId) -> AnyhowResult<()> {
            let map =
                get_typemap_arc::<ComponentStoreMap, UserId, Arc<RwLock<ComponentInteraction>>>(
                    ctx,
                )
                .await;
            let mut map = map.write().await;
            if map.remove(key).is_none() {
                return Err(anyhow::anyhow!(
                    "[ FAILED ] UserIdに対するComponentInteractionが見つかりません: del"
                ));
            }
            Ok(())
        }
    }
    async fn get_typemap_arc<K, K2, V>(ctx: &Context) -> Arc<RwLock<HashMap<K2, V>>>
    where
        K: TypeMapKey<Value = Arc<RwLock<HashMap<K2, V>>>> + 'static,
        K2: std::cmp::Eq + std::hash::Hash + Sync + Send + 'static,
        V: Clone + Sync + Send + 'static,
    {
        let data = ctx.data.read().await;
        data.get::<K>().unwrap().clone()
    }
}
