use serenity::builder::{
    CreateEmbed, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
    EditInteractionResponse,
};

pub fn q_match() -> EditInteractionResponse {
    let embed = CreateEmbed::new()
        .colour(16732498)
        .title("募集形式を選択してください");
    let select_menu_kind = CreateSelectMenuKind::String {
        options: vec![
            CreateSelectMenuOption::new("アンレート", "アンレート"),
            CreateSelectMenuOption::new("コンペティティブ", "コンペティティブ"),
            CreateSelectMenuOption::new("カスタム", "カスタム"),
        ],
    };
    let select_menu = CreateSelectMenu::new("募集形式を選択", select_menu_kind)
        .placeholder("募集形式を選択してください")
        .min_values(1)
        .max_values(1);
    EditInteractionResponse::new()
        .embed(embed)
        .select_menu(select_menu)
}
