use serenity::builder::{
    CreateEmbed, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
    EditInteractionResponse,
};

pub async fn rank() -> EditInteractionResponse {
    let embed = CreateEmbed::new()
        .colour(16732498)
        .title("ランクを選択してください");
    let select_menu_kind = CreateSelectMenuKind::String {
        options: vec![
            CreateSelectMenuOption::new("どこでも", "どこでも"),
            CreateSelectMenuOption::new("アイアン", "アイアン"),
            CreateSelectMenuOption::new("ブロンズ", "ブロンズ"),
            CreateSelectMenuOption::new("シルバー", "シルバー"),
            CreateSelectMenuOption::new("ゴールド", "ゴールド"),
            CreateSelectMenuOption::new("プラチナ", "プラチナ"),
            CreateSelectMenuOption::new("ダイヤモンド", "ダイヤモンド"),
            CreateSelectMenuOption::new("アセンダント", "アセンダント"),
            CreateSelectMenuOption::new("イモータル", "イモータル"),
            CreateSelectMenuOption::new("レディアント", "レディアント"),
        ],
    };
    let select_menu = CreateSelectMenu::new("ランクを選択", select_menu_kind)
        .placeholder("ランクを選択してください")
        .min_values(1)
        .max_values(1);
    EditInteractionResponse::new()
        .embed(embed)
        .select_menu(select_menu)
}