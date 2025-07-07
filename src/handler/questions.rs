use anyhow::Result as AnyhowResult;
use serenity::client::Context as SerenityContext;
use serenity::model::application::ComponentInteraction;
use serenity::model::application::ComponentInteractionDataKind;
mod server_select;
use server_select::server;
mod match_select;
use match_select::q_match;
pub mod component_store;
use component_store::ComponentStore;
mod member_select;
use member_select::member;
mod recruit_message;
use crate::handler::webhook::WebhookData;
use recruit_message::recruit_message;
mod rank_select;
use rank_select::rank;

pub async fn questions(ctx: SerenityContext, component: ComponentInteraction) -> AnyhowResult<()> {
    match component.data.custom_id.as_str() {
        "募集を作成" => {
            let question = server();
            let case = &component.data.custom_id;
            let component_set_task = ComponentStore::set(&component);
            let webhook_data_init_task = WebhookData::new(&component);
            let create_response_task = component.create_response(&ctx.http, question);
            let (_, webhook_data_init, create_response) = tokio::join!(
                component_set_task,
                webhook_data_init_task,
                create_response_task
            );
            match (webhook_data_init, create_response) {
                (Ok(_), Ok(_)) => {
                    println!("[ OK ] インタラクションを正常に終了しました: {}", case);
                }
                (_, Err(e)) => {
                    eprintln!(
                        "[ FAILED ] インタラクションに対するレスポンスの送信に失敗しました: {}",
                        e
                    );
                }
                (Err(e), _) => {
                    eprintln!("[ FAILED ] WebhookDataの初期化に失敗しました: {}", e);
                }
            }
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
            let question = q_match();
            let component = ComponentStore::get(user_id).await;
            let edit_response_task = component.edit_response(&ctx.http, question);
            let webhook_data_set_task =
                WebhookData::with_mute(&component.id, |w| w.ap_server = ap_server);
            let (webhook_data_set, edit_response) =
                tokio::join!(webhook_data_set_task, edit_response_task);
            match (webhook_data_set, edit_response) {
                (Ok(_), Ok(_)) => {
                    println!("[ OK ] インタラクションを正常に終了しました: サーバーを選択");
                }
                (_, Err(e)) => {
                    eprintln!(
                        "[ FAILED ] インタラクションに対するレスポンスの送信に失敗しました: {}",
                        e
                    );
                }
                (Err(e), _) => {
                    eprintln!("[ FAILED ] WebhookDataの更新に失敗しました: {}", e);
                }
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
            let component = ComponentStore::get(user_id).await;
            let webhook_data_set_task =
                WebhookData::with_mute(&component.id, |w| w.mode = mode.to_string());
            let edit_resuponse_task = if mode == "アンレート" || mode == "カスタム" {
                let question = member(mode.to_string());
                component.edit_response(&ctx.http, question)
            } else {
                let question = rank();
                component.edit_response(&ctx.http, question)
            };
            let (webhook_data_set, edit_response) =
                tokio::join!(webhook_data_set_task, edit_resuponse_task);
            match (webhook_data_set, edit_response) {
                (Ok(_), Ok(_)) => {
                    println!("[ OK ] インタラクションを正常に終了しました: 募集形式を選択");
                }
                (_, Err(e)) => {
                    eprintln!(
                        "[ FAILED ] インタラクションに対するレスポンスの送信に失敗しました: {}",
                        e
                    );
                }
                (Err(e), _) => {
                    eprintln!("[ FAILED ] WebhookDataの更新に失敗しました: {}", e);
                }
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
            let create_resuponse_task = component.create_response(&ctx.http, question);
            let component = ComponentStore::get(user_id).await;
            let webhook_data_set_task =
                WebhookData::with_mute(&component.id, |w| w.max_member = max_member);
            let (webhook_data_set, create_response) =
                tokio::join!(webhook_data_set_task, create_resuponse_task);
            match (webhook_data_set, create_response) {
                (Ok(_), Ok(_)) => {
                    println!("[ OK ] インタラクションを正常に終了しました: 募集人数を選択");
                }
                (_, Err(e)) => {
                    eprintln!(
                        "[ FAILED ] インタラクションに対するレスポンスの送信に失敗しました: {}",
                        e
                    );
                }
                (Err(e), _) => {
                    eprintln!("[ FAILED ] WebhookDataの更新に失敗しました: {}", e);
                }
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
            let question = member(rank.to_string());
            let component = ComponentStore::get(user_id).await;
            let edit_response_task = component.edit_response(&ctx.http, question);
            let webhook_data_set_task =
                WebhookData::with_mute(&component.id, |w| w.rank = rank.to_string());
            let (webhook_data_set, edit_response) =
                tokio::join!(webhook_data_set_task, edit_response_task);
            match (webhook_data_set, edit_response) {
                (Ok(_), Ok(_)) => {
                    println!("[ OK ] インタラクションを正常に終了しました: ランクを選択");
                }
                (_, Err(e)) => {
                    eprintln!(
                        "[ FAILED ] インタラクションに対するレスポンスの送信に失敗しました: {}",
                        e
                    );
                }
                (Err(e), _) => {
                    eprintln!("[ FAILED ] WebhookDataの更新に失敗しました: {}", e);
                }
            }
        }
        _ => {}
    }
    Ok(())
}