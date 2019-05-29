/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(async_await)]

use futures::{FutureExt, StreamExt, TryFutureExt};

use tg_botapi::methods::{AnswerInlineQuery, SendMessage};
use tg_botapi::types::{
    ChatType, InlineQuery, InlineQueryResultArticle, InputTextMessageContent, Message, ParseMode,
    UpdateType,
};
use tg_botapi::Bot;

use serde_derive::{Serialize, Deserialize};
use std::fmt::Write;
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize)]
struct Config {
    token: String
}

fn main() {
    let mut config_file = File::open("bot.toml")
        .expect("Could not find config file bot.toml.");

    let mut config_contents = String::new();
    config_file.read_to_string(&mut config_contents).expect("Could not read config file");

    let config: Config = toml::from_str(&config_contents).expect("Could not parse config file");

    tokio::run(run_bot(config.token).boxed().unit_error().compat());
}

async fn run_bot(token: impl Into<String>) {
    let bot = Bot::new(token);

    let mut updates = bot.start_polling();

    while let Some(update) = updates.next().await {
        match update.update_type {
            UpdateType::Message(message) => {
                tokio::spawn(
                    handle_message(bot.clone(), message)
                        .boxed()
                        .unit_error()
                        .compat(),
                );
            }

            UpdateType::InlineQuery(query) => {
                tokio::spawn(
                    handle_inline_query(bot.clone(), query)
                        .boxed()
                        .unit_error()
                        .compat(),
                );
            }
            _ => {}
        }
    }
}

async fn handle_message(bot: Bot, msg: Message) {
    if msg.chat.chat_type == ChatType::Private {
        let msg_text = msg.get_text();

        if msg_text.is_none() {
            return;
        }

        let mut req = SendMessage::new(msg.chat.id, get_char_names(msg_text.unwrap()).0);
        req.parse_mode = ParseMode::Markdown;
        req.disable_web_page_preview = Some(true);

        bot.send(&req).await.unwrap();
    }
}

async fn handle_inline_query(bot: Bot, query: InlineQuery) {
    if query.query.is_empty() {
        return;
    }

    let mut response = AnswerInlineQuery::new(query.id, Vec::new());

    let (char_names, cache) = get_char_names(&query.query);
    let mut content: InputTextMessageContent = char_names.into();
    content.parse_mode = ParseMode::Markdown;

    response.add(InlineQueryResultArticle::new("ID", query.query, content));

    response.cache_time = Some(if cache { 24 * 60 * 60 } else { 60 });

    response.is_personal = Some(false);

    bot.send(&response).await.unwrap();
}

fn get_char_names(string: &str) -> (String, bool) {
    let mut text = String::with_capacity(4096); // max telegram message length
    let mut cache = true;

    for (i, c) in string.chars().enumerate() {
        let name = charname::get_name_checked(c as u32).unwrap_or_else(|| {
            cache = false;
            "UNKNOWN CHARACTER"
        });

        let new_part = format!(
            "`U+{val:04X}` [{}](http://www.fileformat.info/info/unicode/char/{val:X})\n",
            name,
            val = c as u32
        );

        // Don't want to exceed message limit or message entity limit
        if text.len() + new_part.len() >= 3900 || i >= 50 {
            write!(
                text,
                "\nYour mesage has been truncated because it was too big"
            )
            .unwrap();
            break;
        } else {
            write!(text, "{}", new_part).unwrap();
        }
    }

    (text, cache)
}
