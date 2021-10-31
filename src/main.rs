/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fs::File;
use std::io::Read;

use futures::StreamExt;
use serde_derive::{Deserialize, Serialize};
use structopt::StructOpt;
use tg_botapi::methods::{AnswerInlineQuery, SendMessage};
use tg_botapi::types::{
    ChatType, InlineQuery, InlineQueryResultArticle, InputTextMessageContent, Message, ParseMode,
    UpdateType,
};
use tg_botapi::Bot;

mod messages;

#[derive(Serialize, Deserialize)]
struct Config {
    token: String,
}

#[derive(Debug, StructOpt)]
struct CliArgs {
    /// Telegram bot api token to use
    #[structopt(long = "token", short = "t", required_unless = "config_file")]
    token: Option<String>,

    /// Config file that holds the token in toml format
    #[structopt(long = "config", short = "c", conflicts_with = "token")]
    config_file: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = CliArgs::from_args();

    let token = if args.token.is_some() {
        args.token.unwrap()
    } else {
        let mut config_file =
            File::open(args.config_file.unwrap()).expect("Could not open config file");

        let mut config_contents = String::new();
        config_file
            .read_to_string(&mut config_contents)
            .expect("Could not read config file");

        let config: Config = toml::from_str(&config_contents).expect("Could not parse config file");

        config.token
    };

    run_bot(token).await;
}

async fn run_bot(token: String) {
    let bot = Bot::new(token);

    let mut updates = bot.start_polling();

    while let Some(update) = updates.next().await {
        match update.update_type {
            UpdateType::Message(message) => {
                tokio::spawn(handle_message(bot.clone(), message));
            }

            UpdateType::InlineQuery(query) => {
                tokio::spawn(handle_inline_query(bot.clone(), query));
            }
            _ => {}
        }
    }
}

async fn handle_message(bot: Bot, msg: Message) {
    if msg.chat.chat_type == ChatType::Private {
        let msg_text = match msg.get_text() {
            // Cloned because otherwise the entire message stays borrowed.
            Some(text) => text.clone(),
            None => return,
        };

        let (cmd, args): (&str, Option<&str>) = {
            match msg_text.find(|c: char| c.is_ascii_whitespace()) {
                Some(ix) => {
                    let (cmd, args) = msg_text.split_at(ix);
                    (cmd, Some(&args[1..]))
                }

                None => (msg_text.as_str(), None),
            }
        };

        match cmd {
            "/start" | "/about" => {
                let mut req = SendMessage::new(msg.chat.id, messages::ABOUT_MESSAGE);
                req.parse_mode = ParseMode::Markdown;
                req.disable_web_page_preview = Some(true);

                bot.send(&req).await.unwrap();
            }

            "/help" => {
                let mut req = SendMessage::new(msg.chat.id, messages::HELP_MESSAGE);
                req.parse_mode = ParseMode::Markdown;
                req.disable_web_page_preview = Some(true);

                bot.send(&req).await.unwrap();
            }

            // Reply to a message that was interpreted as a command to get its breakdown
            "/raw" => {
                if let Some(reply) = msg.reply_to_message {
                    if let Some(reply_text) = reply.get_text() {
                        let mut req = SendMessage::new(msg.chat.id, get_char_names(reply_text));
                        req.parse_mode = ParseMode::Markdown;
                        req.disable_web_page_preview = Some(true);

                        bot.send(&req).await.unwrap();
                    } else {
                        let req =
                            SendMessage::new(msg.chat.id, messages::NEED_REPLY_TEXT_MESSAGE);
                        bot.send(&req).await.unwrap();
                    }
                } else {
                    let req = SendMessage::new(msg.chat.id, messages::NEED_REPLY_MESSAGE);
                    bot.send(&req).await.unwrap();
                }
            }

            // Filter away certain characters
            "/filter" => {
                if let Some(reply) = msg.reply_to_message {
                    if let Some(reply_text) = reply.get_text() {
                        if let Some(args) = args {
                            let reply = get_char_names_filtered(reply_text, args);

                            if !reply.is_empty() {
                                let mut req = SendMessage::new(msg.chat.id, reply);
                                req.parse_mode = ParseMode::Markdown;
                                req.disable_web_page_preview = Some(true);

                                bot.send(&req).await.unwrap();
                            }
                        }
                    }
                }
            }

            _ => {
                let mut req = SendMessage::new(msg.chat.id, get_char_names(&msg_text));
                req.parse_mode = ParseMode::Markdown;
                req.disable_web_page_preview = Some(true);

                bot.send(&req).await.unwrap();
            }
        }
    }
}

async fn handle_inline_query(bot: Bot, query: InlineQuery) {
    if query.query.is_empty() {
        return;
    }

    let mut response = AnswerInlineQuery::new(query.id, Vec::new());

    let char_names = get_char_names(&query.query);
    let mut content: InputTextMessageContent = char_names.into();
    content.parse_mode = ParseMode::Markdown;

    response.add(InlineQueryResultArticle::new("ID", query.query, content));

    response.cache_time = Some(0);

    response.is_personal = Some(false);

    bot.send(&response).await.unwrap();
}

#[inline(always)]
fn get_char_names(string: &str) -> String {
    _get_char_names_filtered(string, None)
}

#[inline(always)]
fn get_char_names_filtered(string: &str, filter: &str) -> String {
    _get_char_names_filtered(string, Some(filter))
}

fn _get_char_names_filtered(string: &str, filter: Option<&str>) -> String {
    let mut text = String::with_capacity(4096); // max telegram message length
    let mut entities = 0;

    for c in string.chars() {
        if let Some(f) = filter {
            if f.contains(c) {
                continue;
            }
        }

        let name = charname::get_name(c as u32);

        let new_part = format!(
            "`U+{val:04X}` [{}](https://fileformat.info/info/unicode/char/{val:X})\n",
            name,
            val = c as u32
        );
        entities += 1;

        // Don't want to exceed message limit or message entity limit
        if text.len() + new_part.len() >= 4000 || entities > 50 {
            text.push_str("\nYour mesage has been truncated because it was too big");
            break;
        } else {
            text.push_str(&new_part);
        }
    }

    text
}
