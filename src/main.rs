mod commands;

use std::env;

use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::prelude::*;

use crate::commands::chatgpt::CHATGPT_COMMAND;
use crate::commands::friday::FRIDAY_COMMAND;
use crate::commands::github_trend::GITHUB_TREND_COMMAND;

use serenity::async_trait;
use serenity::model::application::interaction::Interaction;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::{ChannelId, GuildId};
use serenity::utils::colours;

#[group]
#[commands(cat, friday, image, random, chatgpt, eval, todo, github_trend, wiki)]
struct General;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let content = &msg.content;
        // guild_id はどこから参照しても同じ値なので最初に取得しておく
        let guild_id = match &msg.guild_id {
            Some(id) => id.0,
            None => return,
        };

        // health check
        let mentions = msg.mentions;
        if mentions.len() > 0 {
            let bot_id = "1097033145674649675";

            for mention in mentions {
                if mention.id.0.to_string() == bot_id {
                    if let Err(why) = msg.channel_id.say(&ctx.http, "なんや").await {
                        println!("Error sending message: {:?}", why);
                    }
                }
            }
        }

        // discord message url
        let re = match regex::Regex::new(r"https://discord.com/channels/\d+/\d+/\d+") {
            Ok(re) => re,
            Err(_) => return,
        };
        // match discord message urls
        let matches = re
            .find_iter(content)
            .map(|m| content[m.0..m.1].to_string())
            .collect::<Vec<String>>();
        // if message is discord message url, send opened message
        if matches.len() > 0 {
            // ids from url: e.g. https://discord.com/channels/{guild_id}/{channel_id}/{message_id}
            for content in matches {
                let channel_id = match content.split("/").nth(5) {
                    Some(id) => match id.parse::<u64>() {
                        Ok(id) => id,
                        Err(_) => return,
                    },
                    None => return,
                };
                let message_id = match content.split("/").nth(6) {
                    Some(id) => match id.parse::<u64>() {
                        Ok(id) => id,
                        Err(_) => return,
                    },
                    None => return,
                };

                if let Some(message) = ChannelId(channel_id)
                    .message(&ctx.http, message_id) //
                    .await
                    .ok()
                {
                    // guild info
                    let guild = GuildId(guild_id);

                    // user display name
                    let display_name = match message.author.nick_in(&ctx.http, guild).await {
                        Some(nick) => nick,
                        None => message.author.name.clone(),
                    };

                    let user_icon = match message.author.avatar_url() {
                        Some(url) => url,
                        None => message.author.default_avatar_url(),
                    };

                    let channel_name = match guild.channels(&ctx.http).await {
                        Ok(channels) => match channels.get(&ChannelId(channel_id)) {
                            Some(channel) => channel.name.clone(),
                            None => return,
                        },
                        Err(_) => return,
                    };

                    if let Err(why) = msg
                        .channel_id
                        .send_message(&ctx.http, |m| {
                            m.embed(|e| {
                                e.author(|a| {
                                    a.name(display_name);
                                    a.icon_url(user_icon);
                                    a
                                });
                                e.description(message.content);
                                e.timestamp(message.timestamp);
                                message.attachments.iter().for_each(|a| {
                                    e.image(a.url.clone());
                                });
                                e.footer(|f| {
                                    f.text(format!("#{}", channel_name));
                                    f
                                });
                                e.color(colours::branding::YELLOW);
                                e
                            });
                            m
                        })
                        .await
                    {
                        println!("Error sending message: {:?}", why);
                    }
                }
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "wiki" => commands::random::run(&command.data.options),
                "cat" => commands::cat::run(&command.data.options),
                "wiki" => commands::wiki::run(&command.data.options).await,
                "eval" => commands::eval::run(&command.data.options).await,
                "todo" => commands::todo::run(&command.data.options, &ctx).await,
                "image" => commands::image::run(&command.data.options).await,
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, _: Ready) {
        let guild_id = GuildId(889012300705591307);

        let _ = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands.create_application_command(|command| commands::random::register(command));
            commands.create_application_command(|command| commands::cat::register(command));
            commands.create_application_command(|command| commands::wiki::register(command));
            commands.create_application_command(|command| commands::eval::register(command));
            commands.create_application_command(|command| commands::todo::register(command));
            commands.create_application_command(|command| commands::image::register(command));
            commands.create_application_command(
                |command: &mut serenity::builder::CreateApplicationCommand| {
                    commands::todo::register(command)
                },
            )
        })
        .await;
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("RINTON_DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("/"))
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
