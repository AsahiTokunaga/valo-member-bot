use std::str::FromStr;

use crate::state_handler::methods::{component_store_map, webhook_map};
use crate::state_handler::{APServer, Mode, Rank};
use anyhow::Result as AnyhowResult;
use serenity::client::Context as SerenityContext;
use serenity::model::application::ComponentInteraction;
use serenity::model::application::ComponentInteractionDataKind;
mod server_select;
use server_select::server;
mod mode_select;
use mode_select::q_match;
mod member_select;
use member_select::member;
mod recruit_message;
use recruit_message::recruit_message;
mod rank_select;
use rank_select::rank;

pub async fn questions(ctx: SerenityContext, component: ComponentInteraction) -> AnyhowResult<()> {
    match component.data.custom_id.as_str() {
        "募集を作成" => {
            let question = server();
            component_store_map::set(&ctx, component.user.id, &component).await?;
            webhook_map::new(&ctx, component.id, component.user.id).await?;
            component.create_response(&ctx.http, question).await?;
            println!("[ OK ] インタラクションを正常に終了しました: 募集を作成");
        }
        "サーバーを選択" => {
            component.defer(&ctx.http).await?;
            let user_id = component.user.id;
            let ap_server = if let ComponentInteractionDataKind::StringSelect { values } =
                &component.data.kind
            {
                APServer::from_str(&values.get(0).unwrap()).ok()
            } else {
                None
            };
            let question = q_match();
            let component = component_store_map::get(&ctx, &user_id).await;
            if let Some(component) = component {
                let component = component.read().await;
                let interaction_id = &component.id;
                component.edit_response(&ctx.http, question).await?;
                if ap_server.is_some() {
                    webhook_map::with_mute(&ctx, interaction_id, |w| {
                        w.ap_server = ap_server.unwrap()
                    })
                    .await?;
                }
                drop(component);
            }
        }
        "募集形式を選択" => {
            component.defer(&ctx.http).await?;
            let user_id = component.user.id;
            let mode = if let ComponentInteractionDataKind::StringSelect { values } =
                &component.data.kind
            {
                Mode::from_str(&values.get(0).unwrap()).ok()
            } else {
                None
            };
            let component = component_store_map::get(&ctx, &user_id).await;
            if let Some(component) = component {
                let component = component.read().await;
                if mode.is_some() {
                    webhook_map::with_mute(&ctx, &component.id, |w| w.mode = mode.unwrap()).await?;
                    if mode.unwrap() == Mode::Unrated
                        || mode.unwrap() == Mode::Custom
                        || mode.is_none()
                    {
                        let question = member(mode.unwrap());
                        component.edit_response(&ctx.http, question).await?;
                    } else {
                        let question = rank();
                        component.edit_response(&ctx.http, question).await?;
                    };
                }
                drop(component);
            }
        }
        "募集人数を選択" => {
            let user_id = component.user.id;
            let question = recruit_message();
            let party = if let ComponentInteractionDataKind::StringSelect { values } =
                &component.data.kind
            {
                &values.get(0).unwrap()
            } else {
                "フルパ"
            };
            let max_member: u8 = match party {
                "デュオ" | "2人" => 2,
                "トリオ" | "3人" => 3,
                "クアッド" | "4人" => 4,
                "フルパ" | "5人" => 5,
                "6人" => 6,
                "7人" => 7,
                "8人" => 8,
                "9人" => 9,
                "10人" => 10,
                _ => 5,
            };
            component.create_response(&ctx.http, question).await?;
            let component = component_store_map::get(&ctx, &user_id).await;
            if let Some(component) = component {
                let component = component.read().await;
                webhook_map::with_mute(&ctx, &component.id, |w| w.max_member = max_member).await?;
                drop(component);
            }
        }
        "ランクを選択" => {
            component.defer(&ctx.http).await?;
            let user_id = component.user.id;
            let rank = if let ComponentInteractionDataKind::StringSelect { values } =
                &component.data.kind
            {
                Rank::from_str(&values.get(0).unwrap()).ok()
            } else {
                None
            };
            let question = member(Mode::Competitive);
            let component = component_store_map::get(&ctx, &user_id).await;
            if let Some(component) = component {
                let component = component.read().await;
                component.edit_response(&ctx.http, question).await?;
                if rank.is_some() {
                    webhook_map::with_mute(&ctx, &component.id, |w| w.rank = rank).await?;
                }
                drop(component);
            }
        }
        _ => {}
    }
    Ok(())
}
