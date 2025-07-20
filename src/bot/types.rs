use deadpool_redis::{Config, Pool, Runtime};
use redis::AsyncTypedCommands;
use serenity::all::{Builder, CacheHttp, ChannelId, CreateWebhook, Http, MessageId, UserId, Webhook};
use std::{collections::HashMap, str::FromStr};
use crate::{bot::colors::*, config, error::{BotError, DbError}};

const THREE_DAYS_SECONDS: i64 = 3 * 24 * 60 * 60;

#[derive(Clone)]
pub struct RedisClient {
  pub connection: Pool,
}

pub trait WebhookDataExt: Sized {
  fn variants() -> impl Iterator<Item = Self>;
  fn as_str(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub struct WebhookData {
  pub creator: UserId,
  pub server: ApServer,
  pub mode: Mode,
  pub rank: Option<Rank>,
  pub member: Member,
  pub joined: Vec<UserId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApServer {
  Tokyo,
  HongKong,
  Singapore,
  Sydney,
  Mumbai,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
  Unrated,
  Competitive,
  Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rank {
  Unranked,
  Iron,
  Bronze,
  Silver,
  Gold,
  Platinum,
  Diamond,
  Ascendant,
  Immortal,
  Radiant,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Member {
  Duo = 2,
  Trio = 3,
  Quad = 4,
  FullParty = 5,
  Six = 6,
  Seven = 7,
  Eight = 8,
  Nine = 9,
  Ten = 10,
}

impl WebhookData {
  pub fn new(id: UserId) -> Self {
    Self {
      creator: id,
      server: ApServer::Tokyo,
      mode: Mode::Unrated,
      rank: None,
      member: Member::Duo,
      joined: vec![id],
    }
  } 
}

impl RedisClient {
  pub async fn new(redis_pass: &str) -> Result<Self, BotError> {
    let cfg = Config::from_url(format!("redis://:{}@127.0.0.1/", redis_pass));
    let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
    Ok(Self {
      connection: pool,
    })
  }
  pub async fn store_webhook_data(&self, id: MessageId, data: &WebhookData) -> Result<(), BotError> {
    let creator = data.creator.get().to_string();
    let joined_user: String = data.joined.iter()
      .map(|u| format!("{}", u.get()))
      .collect::<Vec<String>>()
      .join(",");
    let fields_value = [
      ("creator", creator.as_str()),
      ("server", data.server.as_str()),
      ("mode", data.mode.as_str()),
      ("rank", data.rank.map_or("None", |r| r.as_str())),
      ("member", data.member.as_str()),
      ("joined", joined_user.as_str()),
    ];
    let mut conn = self.connection.get().await.map_err(DbError::from)?;
    conn.hset_multiple(id.get(), &fields_value).await.map_err(DbError::from)?;
    conn.expire(id.get(), THREE_DAYS_SECONDS).await.map_err(DbError::from)?;
    Ok(())
  }
  pub async fn get_webhook_data(&self, id: MessageId) -> Result<WebhookData, BotError> {
    let mut conn = self.connection.get().await.map_err(DbError::from)?;
    let hash_set: HashMap<String, String> = conn.hgetall(id.get()).await.map_err(DbError::from)?;
    drop(conn);
    let creator = UserId::from_str(hash_set.get("creator").ok_or(BotError::WebhookDataNotFound)?)
      .map_err(|_| BotError::WebhookDataNotFound)?;
    let server = ApServer::from_str(hash_set.get("server").ok_or(BotError::WebhookDataNotFound)?)
      .map_err(|_| BotError::WebhookDataNotFound)?;
    let mode = Mode::from_str(hash_set.get("mode").ok_or(BotError::WebhookDataNotFound)?)
     .map_err(|_| BotError::WebhookDataNotFound)?;
    let rank = hash_set.get("rank")
     .filter(|&r| r != "None")
     .and_then(|r| Rank::from_str(r).ok());
    let member = Member::from_str(hash_set.get("member").ok_or(BotError::WebhookDataNotFound)?)
      .map_err(|_| BotError::WebhookDataNotFound)?;
    let joined: Vec<UserId> = hash_set
      .get("joined")
      .ok_or(BotError::WebhookDataNotFound)?
      .split(',')
      .filter_map(|u| UserId::from_str(u).ok())
      .collect();
    let webhook_data = WebhookData {
      creator,
      server,
      mode,
      rank,
      member,
      joined,
    };
    Ok(webhook_data)
  }
  pub async fn get_webhook<T: AsRef<Http> + CacheHttp + Copy>(&self, http: T) -> Result<Webhook, BotError> {
    let channel = ChannelId::from_str(&config::get("CHANNEL_ID")?)?;
    let mut conn = self.connection.get().await.map_err(DbError::from)?;
    let webhook_url: Option<String> = conn.get("webhook_url").await.map_err(DbError::from)?;
    match webhook_url {
      Some(url) => {
        let webhook = Webhook::from_url(http, &url).await?;
        Ok(webhook)
      }
      None => {
        let webhook = CreateWebhook::new("Valo Member Bot Webhook")
          .execute(http, channel).await?;
        conn.set("webhook_url", webhook.url()?).await.map_err(DbError::from)?;
        drop(conn);
        Ok(webhook)
      }
    }
  }
}

impl WebhookDataExt for ApServer {
  fn variants() -> impl Iterator<Item = Self> {
    [
      ApServer::Tokyo,
      ApServer::HongKong,
      ApServer::Singapore,
      ApServer::Sydney,
      ApServer::Mumbai,
    ]
    .into_iter()
  }
  fn as_str(&self) -> &'static str {
    match self {
      ApServer::Tokyo => "Tokyo/東京 🇯🇵",
      ApServer::HongKong => "Hong Kong/香港 🇭🇰",
      ApServer::Singapore => "Singapore/シンガポール 🇸🇬",
      ApServer::Sydney => "Sydney/シドニー 🇦🇺",
      ApServer::Mumbai => "Mumbai/ムンバイ 🇮🇳",
    }
  }
}
impl FromStr for ApServer {
  type Err = &'static str;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::variants()
      .find(|&server| server.as_str() == s)
      .ok_or("Invalid AP server")
  }
}

impl WebhookDataExt for Mode {
  fn variants() -> impl Iterator<Item = Self> {
    [Mode::Unrated, Mode::Competitive, Mode::Custom].into_iter()
  }
  fn as_str(&self) -> &'static str {
    match self {
      Self::Unrated => "アンレート",
      Self::Competitive => "コンペティティブ",
      Self::Custom => "カスタム",
    }
  }
}
impl FromStr for Mode {
  type Err = &'static str;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::variants()
      .find(|&mode| mode.as_str() == s)
      .ok_or("Invalid mode")
  }
}
impl Mode {
  pub fn to_mention_str(&self) -> String {
    match self {
      Mode::Unrated => format!("<@&{}>", config::get("UNRATED_MENTION_ID").unwrap_or_default()),
      Mode::Competitive => format!("<@&{}>", config::get("COMPETITIVE_MENTION_ID").unwrap_or_default()),
      Mode::Custom => format!("<@&{}>", config::get("CUSTOM_MENTION_ID").unwrap_or_default()),
    }
  }
}

impl WebhookDataExt for Rank {
  fn variants() -> impl Iterator<Item = Self> {
    [
      Rank::Unranked,
      Rank::Iron,
      Rank::Bronze,
      Rank::Silver,
      Rank::Gold,
      Rank::Platinum,
      Rank::Diamond,
      Rank::Ascendant,
      Rank::Immortal,
      Rank::Radiant,
    ]
    .into_iter()
  }
  fn as_str(&self) -> &'static str {
    match self {
      Self::Unranked => "どこでも",
      Self::Iron => "アイアン",
      Self::Bronze => "ブロンズ",
      Self::Silver => "シルバー",
      Self::Gold => "ゴールド",
      Self::Platinum => "プラチナ",
      Self::Diamond => "ダイヤモンド",
      Self::Ascendant => "アセンダント",
      Self::Immortal => "イモータル",
      Self::Radiant => "レディアント",
    }
  }
}
impl FromStr for Rank {
  type Err = &'static str;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::variants()
      .find(|&rank| rank.as_str() == s)
      .ok_or("Invalid rank")
  }
}
impl Rank {
  pub fn to_color(&self) -> u32 {
    match self {
      Rank::Radiant => RADIANT_COLOR,
      Rank::Immortal => IMMORTAL_COLOR,
      Rank::Ascendant => ASCENDANT_COLOR,
      Rank::Diamond => DIAMOND_COLOR,
      Rank::Platinum => PLATINUM_COLOR,
      Rank::Gold => GOLD_COLOR,
      Rank::Silver => SILVER_COLOR,
      Rank::Bronze => BRONZE_COLOR,
      Rank::Iron => IRON_COLOR,
      _ => BASE_COLOR
    }
  }
}

impl WebhookDataExt for Member {
  fn variants() -> impl Iterator<Item = Self> {
    [
      Member::Duo,
      Member::Trio,
      Member::Quad,
      Member::FullParty,
      Member::Six,
      Member::Seven,
      Member::Eight,
      Member::Nine,
      Member::Ten,
    ]
    .into_iter()
  }
  fn as_str(&self) -> &'static str {
    match self {
      Self::Duo => "デュオ",
      Self::Trio => "トリオ",
      Self::Quad => "クアッド",
      Self::FullParty => "フルパ",
      Self::Six => "6人",
      Self::Seven => "7人",
      Self::Eight => "8人",
      Self::Nine => "9人",
      Self::Ten => "10人",
    }
  }
}
impl FromStr for Member {
  type Err = &'static str;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::variants()
      .find(|&member| member.as_str() == s)
      .ok_or("Invalid member size")
  }
}
impl From<Member> for u8 {
  fn from(value: Member) -> Self {
    value as u8
  }
}
