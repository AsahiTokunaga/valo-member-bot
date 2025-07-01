use std::vec;

use serenity::builder::{
    CreateEmbed, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
    EditInteractionResponse,
};

use crate::handler::BASE_COLOR;

pub async fn member(mode: String) -> EditInteractionResponse {
    let create_embed_task = tokio::spawn(async {
        CreateEmbed::new()
            .colour(BASE_COLOR)
            .title("募集人数を選択してください")
    });
    let create_select_menu_task = tokio::spawn(async move {
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
            "どこでも" => vec![CreateSelectMenuOption::new("フルパ", "フルパ")],
            _ => vec![
                CreateSelectMenuOption::new("デュオ", "デュオ"),
                CreateSelectMenuOption::new("トリオ", "トリオ"),
                CreateSelectMenuOption::new("フルパ", "フルパ"),
            ],
        };
        let select_menu_kind = CreateSelectMenuKind::String {
            options: select_menu_vec,
        };
        CreateSelectMenu::new("募集人数を選択", select_menu_kind)
            .placeholder("募集人数を選択してください")
            .min_values(1)
            .max_values(1)
    });

    let (embed, select_menu) = tokio::join!(create_embed_task, create_select_menu_task);
    match (embed, select_menu) {
        (Ok(embed), Ok(select_menu)) => EditInteractionResponse::new()
            .embed(embed)
            .select_menu(select_menu),
        (Err(e), _) => {
            println!("[ FAILED ] Embedの生成に失敗しました: {}", e);
            EditInteractionResponse::new()
                .content("Embedの生成に失敗しました。もう一度お試しください。")
                .components(vec![])
                .embeds(vec![])
        }
        (_, Err(e)) => {
            println!("[ FAILED ] SelectMenuの生成に失敗しました: {}", e);
            EditInteractionResponse::new()
                .content("SelectMenuの生成に失敗しました。もう一度お試しください。")
                .components(vec![])
                .embeds(vec![])
        }
    }
}
