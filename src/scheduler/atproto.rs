use std::error::Error;

use serenity::{async_trait, client::Context, model::id::ChannelId};
use tracing::{error, info};

use crate::utils::{fetch_atproto::Feed, get_db_channel::get_db_channel};

use super::processer::Processer;

use crate::utils::fetch_atproto::fetch_atproto;

pub(crate) struct ProcesserStruct;

#[async_trait]
impl Processer<Feed> for ProcesserStruct {
    async fn fetch(&self, ctx: &Context) -> Result<Vec<Feed>, Box<dyn Error>> {
        let res = fetch_atproto(ctx).await;
        match res {
            Ok(res) => Ok(res),
            Err(why) => {
                error!("Error fetching atproto: {:?}", why);
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "API が取得できません。",
                )));
            }
        }
    }

    async fn post_to_channel(&self, ctx: &Context, items: Vec<Feed>) -> Result<(), Box<dyn Error>> {
        let channel = ChannelId(1191588266105917441);
        // card にして投稿する
        for item in items {
            let avatar = item.post.author.avatar;
            let auther = item.post.author.display_name;
            let text = item.post.record.text;
            let created_at = match chrono::NaiveDateTime::parse_from_str(
                &item.post.record.createdAt,
                "%Y-%m-%dT%H:%M:%S%.3fZ",
            ) {
                Ok(created_at) => {
                    let created_at = created_at + chrono::Duration::hours(9);
                    created_at.format("%Y-%m-%d %H:%M:%S").to_string()
                }
                Err(why) => {
                    error!("Error parsing created_at: {:?}", why);
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "created_at のパースに失敗しました。",
                    )));
                }
            };
            let _ = channel
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.description(text)
                            .author(|a| a.name(auther).icon_url(avatar).url("https://bsky.social/"))
                            .footer(|f| f.text(created_at))
                    })
                })
                .await;
        }
        Ok(())
    }

    async fn update_db_channel(&self, ctx: &Context) -> Result<(), Box<dyn Error>> {
        let db_channel = match get_db_channel(&ctx).await {
            Ok(db_channel) => db_channel,
            Err(why) => {
                error!("Error getting db channel: {:?}", why);
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "DBチャンネルが見つかりません。",
                )));
            }
        };

        let messages = db_channel
            .messages(&ctx.http, |retriever| retriever.limit(100))
            .await
            .unwrap()
            .into_iter()
            .filter(|message| message.content.starts_with("atproto_last_date"))
            .collect::<Vec<_>>();

        for message in messages {
            let _ = message.delete(&ctx.http).await;
        }

        let now = chrono::Utc::now();
        let now = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let _ = db_channel
            .send_message(&ctx.http, |m| {
                m.content(format!("atproto_last_date {}", now))
            })
            .await;
        Ok(())
    }

    async fn run(&self, ctx: &Context) -> Result<(), Box<dyn Error>> {
        info!("atproto retrieval is started.");

        let items = self.fetch(ctx).await?;
        self.post_to_channel(ctx, items).await?;
        self.update_db_channel(ctx).await?;

        info!("atproto retrieval is done.");

        Ok(())
    }
}
