use serenity::client::Context as SerenityContext;
use anyhow::Result as AnyhowResult;
use serenity::model::application::ComponentInteraction;
use serenity::builder::CreateEmbed;
use serenity::builder::CreateInteractionResponse;
use serenity::builder::CreateInteractionResponseMessage;
use serenity::builder::CreateSelectMenu;
use serenity::builder::CreateSelectMenuKind;
use serenity::builder::CreateSelectMenuOption;

pub async fn question(ctx: SerenityContext, component: ComponentInteraction) -> AnyhowResult<()> {
    match component.data.custom_id.as_str() {
        "募集を作成" => {
            let embed = CreateEmbed::new()
                .colour(16732498)
                .title("サーバーを選択してください");
            let select_menu_kind= CreateSelectMenuKind::String { options: vec![
                CreateSelectMenuOption::new("アジア", "アジア"),
                CreateSelectMenuOption::new("北アメリカ", "北アメリカ"),
                CreateSelectMenuOption::new("ラテンアメリカ", "ラテンアメリカ"),
                CreateSelectMenuOption::new("ブラジル", "ブラジル"),
                CreateSelectMenuOption::new("ヨーロッパ", "ヨーロッパ"),
                CreateSelectMenuOption::new("韓国", "韓国")
            ] };
            let select_menu = CreateSelectMenu::new("サーバー選択", select_menu_kind)
                .placeholder("サーバーを選択してください")
                .min_values(1)
                .max_values(1);
            let response_message = CreateInteractionResponseMessage::new()
                .ephemeral(true)
                .embed(embed)
                .select_menu(select_menu);
            let response = CreateInteractionResponse::Message(response_message);
            component.create_response(&ctx.http, response).await?;
        }
        _ => {}
    }
    Ok(())
}
