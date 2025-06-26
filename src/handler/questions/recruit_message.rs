use serenity::builder::{CreateActionRow, CreateInputText, CreateModal, CreateInteractionResponse};
use serenity::model::application::InputTextStyle;

pub async fn recruit_message() -> CreateInteractionResponse {
    let action_row = vec![CreateActionRow::InputText(
        CreateInputText::new(
            InputTextStyle::Paragraph,
            "募集メッセージを入力しましょう",
            "募集メッセージ",
        )
        .required(false)
        .max_length(100)
        .placeholder("例: たくさん喋れる人募集！"),
    )];
    let modal = CreateModal::new("一言", "募集メッセージ").components(action_row);
    CreateInteractionResponse::Modal(modal)
}
