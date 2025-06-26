use serenity::builder::{
    CreateEmbed, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
    EditInteractionResponse,
};

pub async fn member(mode: String) -> EditInteractionResponse {
    let embed = CreateEmbed::new()
        .colour(16732498)
        .title("募集人数を選択してください");
    let select_menu_vec = match mode.as_str() {
        "アンレート" => vec![
            CreateSelectMenuOption::new("デュオ", "デュオ"),
            CreateSelectMenuOption::new("トリオ", "トリオ"),
            CreateSelectMenuOption::new("クアッド", "クアッド"),
            CreateSelectMenuOption::new("フルパ", "フルパ"),
        ],
        "コンペティティブ" => vec![
            CreateSelectMenuOption::new("デュオ", "デュオ"),
            CreateSelectMenuOption::new("トリオ", "トリオ"),
            CreateSelectMenuOption::new("フルパ", "フルパ"),
        ],
        "カスタム" => vec![
            CreateSelectMenuOption::new("2人", "2人"),
            CreateSelectMenuOption::new("3人", "3人"),
            CreateSelectMenuOption::new("4人", "4人"),
            CreateSelectMenuOption::new("5人", "5人"),
            CreateSelectMenuOption::new("6人", "6人"),
            CreateSelectMenuOption::new("7人", "7人"),
            CreateSelectMenuOption::new("8人", "8人"),
            CreateSelectMenuOption::new("9人", "9人"),
            CreateSelectMenuOption::new("10人", "10人"),
        ],
        _ => vec![
            CreateSelectMenuOption::new("デュオ", "デュオ"),
            CreateSelectMenuOption::new("トリオ", "トリオ"),
            CreateSelectMenuOption::new("フルパ", "フルパ"),
        ],
    };
    let select_menu_kind = CreateSelectMenuKind::String {
        options: select_menu_vec,
    };
    let select_menu = CreateSelectMenu::new("募集人数を選択", select_menu_kind)
        .placeholder("募集人数を選択してください")
        .min_values(1)
        .max_values(1);
    EditInteractionResponse::new()
        .embed(embed)
        .select_menu(select_menu)
}
