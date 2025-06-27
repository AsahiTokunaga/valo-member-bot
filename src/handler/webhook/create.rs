use serenity::client::Context as SerenityContext;
use anyhow::Result as AnyhowResult;
use serenity::model::application::ModalInteraction;

use crate::handler::questions::component_handler::ComponentHandler;
use crate::handler::webhook::WebhookHandler;

pub async fn create(_ctx: &SerenityContext, modal: ModalInteraction) -> AnyhowResult<()> {
    let user_id = modal.user.id;
    if let Some(component) = ComponentHandler::get(user_id).await {
        let webhook = WebhookHandler::get(&component).await?;
        println!("Webhook: {:#?}", webhook);
    }
    Ok(())
}
