extern crate rand;
use std::env;

use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};
use serenity::model::prelude::{ChannelId, GuildId};
use serenity::prelude::Context;

pub async fn run(options: &[CommandDataOption], ctx: &Context) -> String {
    let operation = match options.iter().find(|option| option.name == "operation") {
        Some(option) => match &option.resolved {
            Some(value) => match value {
                CommandDataOptionValue::String(text) => text,
                _ => return "オペレーションが不正です".to_string(),
            },
            None => return "オペレーションが不正です".to_string(),
        },
        None => return "オペレーションが不正です".to_string(),
    };

    let todo_message = match options.iter().find(|option| option.name == "message") {
        Some(option) => match &option.resolved {
            Some(value) => match value {
                CommandDataOptionValue::String(text) => text,
                _ => "",
            },
            None => "",
        },
        None => "",
    };

    let todo_id = match options.iter().find(|option| option.name == "id") {
        Some(option) => match &option.resolved {
            Some(value) => match value {
                CommandDataOptionValue::String(text) => text,
                _ => "",
            },
            None => "",
        },
        None => "",
    };

    let db_channel_id = match env::var("DISCORD_DB_CHANNEL_ID_RINTON_BOT")
        .expect("search engine id is not defined")
        .parse::<u64>()
    {
        Ok(db_channel_id) => db_channel_id,
        Err(_) => return "DBチャンネルのIDが見つかりません。".to_string(),
    };
    let guild_id = GuildId(889012300705591307);

    let channels = match guild_id.channels(&ctx.http).await {
        Ok(channel) => channel,
        Err(_) => return "チャンネル一覧の取得に失敗しました".to_string(),
    };
    let db_channel = match channels.get(&ChannelId(db_channel_id)) {
        Some(channel) => channel,
        None => return "DBチャンネルが見つかりません。".to_string(),
    };

    let messages = match db_channel
        .messages(&ctx.http, |retriever| retriever.limit(100))
        .await
    {
        Ok(messages) => messages
            .into_iter()
            .filter(|message| message.content.starts_with("todo_message"))
            .collect::<Vec<_>>(),
        Err(_) => return "メッセージの取得に失敗しました".to_string(),
    };

    match operation.as_str() {
        "add" => {
            let id = match messages.len() {
                0 => 1,
                _ => {
                    let mut ids = messages
                        .iter()
                        .map(|message| {
                            message
                                .content
                                .split_whitespace()
                                .nth(1)
                                .unwrap()
                                .parse::<u64>()
                                .unwrap()
                        })
                        .collect::<Vec<_>>();
                    ids.sort();
                    ids.last().unwrap() + 1
                }
            };

            let _ = match ChannelId(db_channel_id)
                .say(&ctx.http, format!("todo_message {} {}", id, todo_message))
                .await
            {
                Ok(_) => "",
                Err(_) => return "メッセージの送信に失敗しました".to_string(),
            };
            return format!("{}: {} を追加しました。", id, todo_message);
        }
        "rm" => {
            match messages.iter().position(|x| {
                x.content.split_whitespace().nth(1).unwrap() == todo_message
                    || x.content.split_whitespace().nth(2).unwrap() == todo_message
                    || x.content.split_whitespace().nth(1).unwrap() == todo_id
                    || x.content.split_whitespace().nth(2).unwrap() == todo_id
            }) {
                Some(index) => {
                    let todo_message_id = messages[index].id;
                    match ChannelId(db_channel_id)
                        .delete_message(&ctx.http, todo_message_id)
                        .await
                    {
                        Ok(_) => (),
                        Err(_) => return "メッセージの削除に失敗しました".to_string(),
                    }
                    return format!("{} を削除しました。", todo_message_id);
                }
                None => {
                    return format!("{} は見つかりませんでした。", todo_message);
                }
            }
        }
        "ls" => {
            if messages.is_empty() {
                return "TODOリストには何もありません。".to_string();
            } else {
                return format!(
                    "TODOリスト:
・{}",
                    messages
                        .iter()
                        .map(|x| format!(
                            "{} {}",
                            x.content.split_whitespace().nth(1).unwrap(),
                            x.content.split_whitespace().nth(2).unwrap()
                        ))
                        .collect::<Vec<_>>()
                        .join("\n・")
                );
            }
        }

        "edit" => {
            match messages.iter().position(|x| {
                x.content.split_whitespace().nth(1).unwrap() == todo_message
                    || x.content.split_whitespace().nth(2).unwrap() == todo_message
                    || x.content.split_whitespace().nth(1).unwrap() == todo_id
                    || x.content.split_whitespace().nth(2).unwrap() == todo_id
            }) {
                Some(index) => {
                    let todo_message_id = messages[index].id;
                    let _ = match ChannelId(db_channel_id)
                        .edit_message(&ctx.http, todo_message_id, |m| {
                            m.content(format!("todo_message {} {}", todo_id, todo_message))
                        })
                        .await
                    {
                        Ok(_) => (),
                        Err(_) => return "メッセージの編集に失敗しました".to_string(),
                    };

                    return format!("{}: {} を編集しました。", todo_id, todo_message);
                }
                None => return format!("{} は存在しません。", todo_id),
            }
        }
        _ => {}
    }

    "".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("todo")
        .description("todo list を管理します")
        .create_option(|option| {
            option
                .name("operation")
                .kind(CommandOptionType::String)
                .description("オペレーション")
                .add_string_choice("add", "add")
                .add_string_choice("rm", "rm")
                .add_string_choice("ls", "ls")
                .add_string_choice("edit", "edit")
                .required(true)
        })
        .create_option(|option| {
            option
                .name("message")
                .kind(CommandOptionType::String)
                .description("メッセージ")
                .required(false)
        })
        .create_option(|option| {
            option
                .name("id")
                .kind(CommandOptionType::String)
                .description("ID")
                .required(false)
        })
}
