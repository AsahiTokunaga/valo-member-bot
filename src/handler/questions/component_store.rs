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
        let test = if let Some(value) = map.get(&user_id) {
            value
        } else {
            panic!(
                "[ FAILED ] ユーザーのコンポーネントが見つかりません: {:?}",
                user_id
            );
        };
        test.clone()
    }
}