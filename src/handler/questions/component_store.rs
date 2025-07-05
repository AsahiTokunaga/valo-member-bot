use once_cell::sync::Lazy;
use serenity::model::prelude::*;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub static COMPONENTS: Lazy<RwLock<HashMap<UserId, ComponentInteraction>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub struct ComponentStore;

impl ComponentStore {
    pub async fn set(component: &ComponentInteraction) {
        let user_id = component.user.id;
        let mut map = COMPONENTS.write().await;
        map.insert(user_id, component.clone());
    }
    pub async fn get(user_id: UserId) -> ComponentInteraction {
        let map = COMPONENTS.read().await;
        if let Some(value) = map.get(&user_id) {
            value.clone()
        } else {
            panic!("[ FAILED ] UserIdに対するComponentInteractionが見つかりません: get");
        }
    }
    pub async fn del(user_id: UserId) {
        let mut map = COMPONENTS.write().await;
        if map.remove(&user_id).is_none() {
            panic!("[ FAILED ] UserIdに対するComponentInteractionが見つかりません: del");
        }
    }
}