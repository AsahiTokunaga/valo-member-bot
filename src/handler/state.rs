use std::{
  collections::{HashMap, HashSet},
  fmt::Display,
  str::FromStr,
  sync::Arc,
};

use serenity::{
  all::{ComponentInteraction, InteractionId, MessageId, UserId},
  prelude::TypeMapKey,
};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy)]
pub enum APServer {
  Tokyo,
  HongKong,
  Singapore,
  Sydney,
  Mumbai,
}
impl APServer {
  pub fn variants() -> impl Iterator<Item = APServer> {
    [
      APServer::Tokyo,
      APServer::HongKong,
      APServer::Singapore,
      APServer::Sydney,
      APServer::Mumbai,
    ]
    .into_iter()
  }
}
impl Display for APServer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      APServer::Tokyo => write!(f, "Tokyo/東京🇯🇵"),
      APServer::HongKong => write!(f, "Hong Kong/香港 🇭🇰"),
      APServer::Singapore => write!(f, "Singapore/シンガポール 🇸🇬"),
      APServer::Sydney => write!(f, "Sydney/シドニー 🇦🇺"),
      APServer::Mumbai => write!(f, "Mumbai/ムンバイ 🇮🇳"),
    }
  }
}
impl FromStr for APServer {
  type Err = &'static str;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::variants()
      .find(|&server| server.to_string() == s)
      .ok_or("Invalid APServer string")
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
  Unrated,
  Competitive,
  Custom,
}
impl Mode {
  pub fn variants() -> impl Iterator<Item = Mode> {
    [Mode::Unrated, Mode::Competitive, Mode::Custom].into_iter()
  }
}
impl Display for Mode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Mode::Unrated => write!(f, "アンレート"),
      Mode::Competitive => write!(f, "コンペティティブ"),
      Mode::Custom => write!(f, "カスタム"),
    }
  }
}
impl FromStr for Mode {
  type Err = &'static str;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::variants()
      .find(|&mode| mode.to_string() == s)
      .ok_or("Invalid Mode string")
  }
}

#[derive(Debug, Clone, Copy)]
pub enum Rank {
  Radiant,
  Immortal,
  Ascendant,
  Diamond,
  Platinum,
  Gold,
  Silver,
  Bronze,
  Iron,
  Unranked,
}
impl Rank {
  pub fn variants() -> impl Iterator<Item = Rank> {
    [
      Rank::Radiant,
      Rank::Immortal,
      Rank::Ascendant,
      Rank::Diamond,
      Rank::Platinum,
      Rank::Gold,
      Rank::Silver,
      Rank::Bronze,
      Rank::Iron,
      Rank::Unranked,
    ]
    .into_iter()
  }
}
impl Display for Rank {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Rank::Radiant => write!(f, "レディアント"),
      Rank::Immortal => write!(f, "イモータル"),
      Rank::Ascendant => write!(f, "アセンダント"),
      Rank::Diamond => write!(f, "ダイヤモンド"),
      Rank::Platinum => write!(f, "プラチナ"),
      Rank::Gold => write!(f, "ゴールド"),
      Rank::Silver => write!(f, "シルバー"),
      Rank::Bronze => write!(f, "ブロンズ"),
      Rank::Iron => write!(f, "アイアン"),
      Rank::Unranked => write!(f, "どこでも"),
    }
  }
}
impl FromStr for Rank {
  type Err = &'static str;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::variants()
      .find(|&rank| rank.to_string() == s)
      .ok_or("Invalid Rank string")
  }
}

#[derive(Debug, Clone, Copy)]
pub enum MaxMember {
  Duo,
  Trio,
  Quad,
  Five,
  Six,
  Seven,
  Eight,
  Nine,
  Ten,
}
impl MaxMember {
  pub fn variants() -> impl Iterator<Item = MaxMember> {
    [
      MaxMember::Duo,
      MaxMember::Trio,
      MaxMember::Quad,
      MaxMember::Five,
      MaxMember::Six,
      MaxMember::Seven,
      MaxMember::Eight,
      MaxMember::Nine,
      MaxMember::Ten,
    ]
    .into_iter()
  }
}
impl Display for MaxMember {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      MaxMember::Duo => write!(f, "デュオ"),
      MaxMember::Trio => write!(f, "トリオ"),
      MaxMember::Quad => write!(f, "クアッド"),
      MaxMember::Five => write!(f, "フルパ"),
      MaxMember::Six => write!(f, "6人"),
      MaxMember::Seven => write!(f, "7人"),
      MaxMember::Eight => write!(f, "8人"),
      MaxMember::Nine => write!(f, "9人"),
      MaxMember::Ten => write!(f, "10人"),
    }
  }
}
impl FromStr for MaxMember {
  type Err = &'static str;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::variants()
      .find(|&max_member| max_member.to_string() == s)
      .ok_or("Invalid MaxMember string")
  }
}
impl Into<u8> for MaxMember {
  fn into(self) -> u8 {
    match self {
      MaxMember::Duo => 2,
      MaxMember::Trio => 3,
      MaxMember::Quad => 4,
      MaxMember::Five => 5,
      MaxMember::Six => 6,
      MaxMember::Seven => 7,
      MaxMember::Eight => 8,
      MaxMember::Nine => 9,
      MaxMember::Ten => 10,
    }
  }
}

#[derive(Debug)]
pub struct WebhookData {
  pub ap_server: APServer,
  pub mode: Mode,
  pub rank: Option<Rank>,
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
  use crate::error::AppStateError;
  use serenity::all::Context;
  use tracing::Level;
  use tracing::instrument;

  pub mod webhook_map {
    use super::*;
    pub async fn new(
      ctx: &Context,
      interaction_id: InteractionId,
      user_id: UserId,
    ) {
      let map =
        get_typemap_arc::<WebhookMap, InteractionId, Arc<RwLock<WebhookData>>>(
          ctx,
        )
        .await;
      let mut map = map.write().await;
      map.insert(
        interaction_id,
        Arc::new(RwLock::new(WebhookData {
          ap_server: APServer::Tokyo,
          mode: Mode::Unrated,
          rank: None,
          max_member: 5,
          joined: HashSet::from([user_id]),
        })),
      );
    }
    #[instrument(name = "state/methods/webhook_map/with_mute", level = Level::WARN, skip_all, fields(interaction_id = %key))]
    pub async fn with_mute<F: FnOnce(&mut WebhookData)>(
      ctx: &Context,
      key: &InteractionId,
      f: F,
    ) -> Result<(), AppStateError> {
      let map =
        get_typemap_arc::<WebhookMap, InteractionId, Arc<RwLock<WebhookData>>>(
          ctx,
        )
        .await;
      let mut map = map.write().await;
      let mut webhook = map
        .get_mut(key)
        .ok_or(AppStateError::WebhookDataNotFound)?
        .write()
        .await;
      f(&mut webhook);
      Ok(())
    }
    #[instrument(name = "state/methods/webhook_map/get", level = Level::WARN, skip_all, fields(interaction_id = %key))]
    pub async fn get(
      ctx: &Context,
      key: &InteractionId,
    ) -> Result<Arc<RwLock<WebhookData>>, AppStateError> {
      let map =
        get_typemap_arc::<WebhookMap, InteractionId, Arc<RwLock<WebhookData>>>(
          ctx,
        )
        .await;
      let map = map.read().await;
      map
        .get(key)
        .ok_or(AppStateError::WebhookDataNotFound)
        .cloned()
    }
    #[instrument(name = "state/methods/webhook_map/del", level = Level::WARN, skip_all, fields(interaction_id = %key))]
    pub async fn del(
      ctx: &Context,
      key: &InteractionId,
    ) -> Result<Arc<RwLock<WebhookData>>, AppStateError> {
      let map =
        get_typemap_arc::<WebhookMap, InteractionId, Arc<RwLock<WebhookData>>>(
          ctx,
        )
        .await;
      let mut map = map.write().await;
      map.remove(key).ok_or(AppStateError::WebhookDataNotFound)
    }
  }

  pub mod interaction_id_map {
    use super::*;
    pub async fn set(
      ctx: &Context,
      message_id: MessageId,
      interaction_id: InteractionId,
    ) {
      let map =
        get_typemap_arc::<InteractionIdMap, MessageId, InteractionId>(ctx)
          .await;
      let mut map = map.write().await;
      map.insert(message_id, interaction_id);
    }
    #[instrument(name = "state/methods/interaction_id_map/get", level = Level::WARN, skip_all, fields(interaction_id = %key))]
    pub async fn get(
      ctx: &Context,
      key: &MessageId,
    ) -> Result<InteractionId, AppStateError> {
      let map =
        get_typemap_arc::<InteractionIdMap, MessageId, InteractionId>(ctx)
          .await;
      let map = map.read().await;
      map
        .get(key)
        .ok_or(AppStateError::InteractionIdNotFound)
        .cloned()
    }
    #[instrument(name = "state/methods/interaction_id_map/del", level = Level::WARN, skip_all, fields(interaction_id = %key))]
    pub async fn del(
      ctx: &Context,
      key: &MessageId,
    ) -> Result<InteractionId, AppStateError> {
      let map =
        get_typemap_arc::<InteractionIdMap, MessageId, InteractionId>(ctx)
          .await;
      let mut map = map.write().await;
      map.remove(key).ok_or(AppStateError::InteractionIdNotFound)
    }
  }

  pub mod component_store_map {
    use super::*;
    pub async fn set(
      ctx: &Context,
      user_id: UserId,
      component: &ComponentInteraction,
    ) {
      let map = get_typemap_arc::<
        ComponentStoreMap,
        UserId,
        Arc<RwLock<ComponentInteraction>>,
      >(ctx)
      .await;
      let mut map = map.write().await;
      map.insert(user_id, Arc::new(RwLock::new(component.clone())));
    }
    #[instrument(name = "state/methods/component_store_map/get", level = Level::WARN, skip_all, fields(interaction_id = %key))]
    pub async fn get(
      ctx: &Context,
      key: &UserId,
    ) -> Result<Arc<RwLock<ComponentInteraction>>, AppStateError> {
      let map = get_typemap_arc::<
        ComponentStoreMap,
        UserId,
        Arc<RwLock<ComponentInteraction>>,
      >(ctx)
      .await;
      let map = map.read().await;
      map
        .get(key)
        .ok_or(AppStateError::ComponentInteractionNotFound)
        .cloned()
    }
    #[instrument(name = "state/methods/component_store_map/get", level = Level::WARN, skip_all, fields(interaction_id = %key))]
    pub async fn del(
      ctx: &Context,
      key: &UserId,
    ) -> Result<Arc<RwLock<ComponentInteraction>>, AppStateError> {
      let map = get_typemap_arc::<
        ComponentStoreMap,
        UserId,
        Arc<RwLock<ComponentInteraction>>,
      >(ctx)
      .await;
      let mut map = map.write().await;
      map
        .remove(key)
        .ok_or(AppStateError::ComponentInteractionNotFound)
    }
  }

  async fn get_typemap_arc<K, K2, V>(
    ctx: &Context,
  ) -> Arc<RwLock<HashMap<K2, V>>>
  where
    K: TypeMapKey<Value = Arc<RwLock<HashMap<K2, V>>>> + 'static,
    K2: std::cmp::Eq + std::hash::Hash + Sync + Send + 'static,
    V: Clone + Sync + Send + 'static,
  {
    let data = ctx.data.read().await;
    data.get::<K>().unwrap().clone()
  }
}
