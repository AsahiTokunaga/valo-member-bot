use once_cell::sync::Lazy;
use serenity::model::prelude::*;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub static COMPONENTS: Lazy<RwLock<HashMap<UserId, ComponentInteraction>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub struct ComponentHandler;

impl ComponentHandler {
    pub async fn set(component: &ComponentInteraction) {
        let user_id = component.user.id;
        let mut map = COMPONENTS.write().await;
        map.insert(user_id, component.clone());
        Self::ttl(&component, 180).await;
    }
    pub async fn get(user_id: UserId) -> Option<ComponentInteraction> {
        let map = COMPONENTS.read().await;
        map.get(&user_id).cloned()
    }
    async fn ttl(component: &ComponentInteraction, ttl_secs: u64) {
        let user_id = component.user.id;
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(ttl_secs)).await;
            Self::del(user_id).await;
        });
    }
    async fn del(user_id: UserId) {
        let mut map = COMPONENTS.write().await;
        map.remove(&user_id);
    }
}