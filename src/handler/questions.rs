use anyhow::Result as AnyhowResult;
use serenity::client::Context as SerenityContext;
use serenity::model::application::ComponentInteraction;
use serenity::model::application::ComponentInteractionDataKind;
mod server_select;
use server_select::server;
mod match_select;
use match_select::q_match;
pub mod component_handler;
use component_handler::ComponentHandler;
mod member_select;
use member_select::member;
mod recruit_message;
use crate::handler::webhook::WebhookHandler;
use recruit_message::recruit_message;
mod rank_select;
use rank_select::rank;

pub async fn questions(ctx: SerenityContext, component: ComponentInteraction) -> AnyhowResult<()> {
    match component.data.custom_id.as_str() {
        "募集を作成" => {
            ComponentHandler::set(&component).await;
            WebhookHandler::new(&component).await?;
            let question = server().await;
            component.create_response(&ctx.http, question).await?;
        }
        "サーバーを選択" => {
            component.defer(&ctx.http).await?;
            let user_id = component.user.id;
            let ap_server = if let ComponentInteractionDataKind::StringSelect { values } =
                &component.data.kind
            {
                values.get(0).cloned().unwrap()
            } else {
                "Tokyo/東京".to_string()
            };
            if let Some(component) = ComponentHandler::get(user_id).await {
                let question = q_match().await;
                component.edit_response(&ctx.http, question).await?;
                WebhookHandler::set_ap_server(&component, ap_server).await?;
            }
        }
        "募集形式を選択" => {
            component.defer(&ctx.http).await?;
            let user_id = component.user.id;
            let mode = if let ComponentInteractionDataKind::StringSelect { values } =
                &component.data.kind
            {
                &values.get(0).unwrap()
            } else {
                "アンレート"
            };
            let component = if let Some(component) = ComponentHandler::get(user_id).await {
                component
            } else {
                return Err(anyhow::anyhow!(
                    "[ FAILED ] 募集の作成に失敗しました: ユーザーのコンポーネントが見つかりません"
                ));
            };
            if mode == "アンレート" || mode == "カスタム" {
                let question = member(&mode).await;
                component.edit_response(&ctx.http, question).await?;
                WebhookHandler::set_mode(&component, &mode).await?;
            } else {
                let question = rank().await;
                component.edit_response(&ctx.http, question).await?;
                WebhookHandler::set_mode(&component, &mode).await?;
            }
        }
        "募集人数を選択" => {
            let user_id = component.user.id;
            let question = recruit_message().await;
            let party = if let ComponentInteractionDataKind::StringSelect { values } =
                &component.data.kind
            {
                &values.get(0).unwrap()
            } else {
                "フルパ"
            };
            let max_member: u8 = match party {
                "デュオ" => 2,
                "トリオ" => 3,
                "クアッド" => 4,
                "フルパ" => 5,
                "2人" => 2,
                "3人" => 3,
                "4人" => 4,
                "5人" => 5,
                "6人" => 6,
                "7人" => 7,
                "8人" => 8,
                "9人" => 9,
                "10人" => 10,
                _ => 5,
            };
            component.create_response(&ctx.http, question).await?;
            if let Some(component) = ComponentHandler::get(user_id).await {
                WebhookHandler::set_max_member(&component, max_member).await?;
            }
        }
        "ランクを選択" => {
            component.defer(&ctx.http).await?;
            let user_id = component.user.id;
            let rank = if let ComponentInteractionDataKind::StringSelect { values } =
                &component.data.kind
            {
                &values.get(0).unwrap()
            } else {
                "どこでも"
            };
            let component = if let Some(component) = ComponentHandler::get(user_id).await {
                component
            } else {
                return Err(anyhow::anyhow!(
                    "[ FAILED ] 募集の作成に失敗しました: ユーザーのコンポーネントが見つかりません"
                ));
            };
            WebhookHandler::set_rank(&component, rank).await?;
            let question = member(&rank).await;
            component.edit_response(&ctx.http, question).await?;
        }
        _ => {}
    }
    Ok(())
}
