use anyhow::Result as AnyhowResult;
use serenity::builder::CreateInteractionResponseFollowup;
use serenity::client::Context as SerenityContext;
use serenity::model::application::ComponentInteraction;
use serenity::model::application::ComponentInteractionDataKind;
mod server_select;
use server_select::server;
mod match_select;
use match_select::q_match;
mod component_handler;
use component_handler::ComponentHandler;
mod member_select;
use member_select::member;
mod recruit_message;
use recruit_message::recruit_message;

pub async fn questions(ctx: SerenityContext, component: ComponentInteraction) -> AnyhowResult<()> {
    match component.data.custom_id.as_str() {
        "募集を作成" => {
            ComponentHandler::set(&component).await;
            let question = server().await;
            component.create_response(&ctx.http, question).await?;
        }
        "サーバーを選択" => {
            component.defer(&ctx.http).await?;
            let user_id = component.user.id;
            if let Some(component) = ComponentHandler::get(user_id).await {
                let question = q_match().await;
                component.edit_response(&ctx.http, question).await?;
            } else {
                component
                    .create_followup(
                        &ctx.http,
                        CreateInteractionResponseFollowup::new()
                            .content("応答時間を超過しました。やり直してください。")
                            .ephemeral(true),
                    )
                    .await?;
            }
        }
        "募集形式を選択" => {
            component.defer(&ctx.http).await?;
            let user_id = component.user.id;
            let mode = if let ComponentInteractionDataKind::StringSelect { values } =
                &component.data.kind
            {
                values.get(0).cloned().unwrap()
            } else {
                "Unknow".to_string()
            };
            if let Some(component) = ComponentHandler::get(user_id).await {
                let question = member(mode).await;
                component.edit_response(&ctx.http, question).await?;
            } else {
                component
                    .create_followup(
                        &ctx.http,
                        CreateInteractionResponseFollowup::new()
                            .content("応答時間を超過しました。やり直してください。")
                            .ephemeral(true),
                    )
                    .await?;
            }
        }
        "募集人数を選択" => {
            let _user_id = component.user.id;
            let response = recruit_message().await;
            component.create_response(&ctx.http, response).await?;
        }
        _ => {}
    }
    Ok(())
}
