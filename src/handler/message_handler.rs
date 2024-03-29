use serenity::model::channel::Message;
use serenity::model::prelude::{ChannelId, GuildId};
use serenity::prelude::*;
use serenity::utils::colours;
use tracing::error;

use crate::utils::fetch_chatgpt::fetch_chatgpt;

pub async fn message(ctx: Context, msg: Message) {
    let content = &msg.content;
    // guild_id はどこから参照しても同じ値なので最初に取得しておく
    let guild_id = match &msg.guild_id {
        Some(id) => id.0,
        None => return,
    };

    let mentions = msg.mentions;
    if mentions.len() > 0 {
        let bot_id = "1097033145674649675";

        for mention in mentions {
            if mention.id.0.to_string() == bot_id {
                let text = match regex::Regex::new(r"<@!\d+>").unwrap() {
                    re => re.replace_all(content, ""),
                }
                .replace("\n", " ");

                let typing = msg.channel_id.start_typing(&ctx.http).unwrap();

                let response = fetch_chatgpt(text, vec![]).await;

                let _ = typing.stop();

                if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                    error!("Error sending message: {:?}", why);
                }
            }
        }
    }

    // discord message url
    let re =
        match regex::Regex::new(r"https://(?:discord\.com|discordapp\.com)/channels/\d+/\d+/\d+") {
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
                    error!("Error sending message: {:?}", why);
                }
            }
        }
    }
}
