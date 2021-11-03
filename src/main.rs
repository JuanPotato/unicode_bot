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
            Some(text) => text.as_str(),
            None => return,
        };

        let reply_text_option = if let Some(ref reply) = msg.reply_to_message {
            reply.get_text()
        } else {
            None
        };

        let (cmd, args) = if let Some(ix) = msg_text.find(|c: char| c.is_ascii_whitespace()) {
            (&msg_text[..ix], Some(&msg_text[ix + 1..]))
        } else {
            (msg_text, None)
        };

        let response = match cmd {
            "/start" | "/about" => Some(messages::ABOUT_MESSAGE.into()),

            "/help" => Some(messages::HELP_MESSAGE.into()),

            // Reply to a message that was interpreted as a command to get its breakdown
            "/raw" => {
                if let Some(reply_text) = reply_text_option {
                    Some(get_char_names(reply_text.chars()))
                } else {
                    Some(messages::NEED_REPLY_MESSAGE.into())
                }
            }

            // Filter away certain characters
            "/filter" => {
                if let Some(reply_text) = reply_text_option {
                    if let Some(args) = args {
                        let filtered_chars = reply_text.chars().filter(|&c| !args.contains(c));
                        let reply = get_char_names(filtered_chars);

                        if !reply.is_empty() {
                            Some(reply)
                        } else {
                            Some(messages::FILTER_EXHAUSTIVE.into())
                        }
                    } else {
                        Some(messages::NO_FILTER.into())
                    }
                } else {
                    Some(messages::NEED_REPLY_MESSAGE.into())
                }
            }

            // Turn U+1F954 -> :potato:
            "/codepoint" => {
                if let Some(codepoint) = args {
                    let codepoint = codepoint.to_lowercase();
                    let codepoint = match codepoint.strip_prefix('u') {
                        Some(c) => c,
                        None => &codepoint,
                    };

                    let unicode = u32::from_str_radix(codepoint, 16)
                        .ok()
                        .and_then(char::from_u32);
                    if let Some(c) = unicode {
                        Some(c.to_string())
                    } else {
                        Some(messages::INVALID_CODEPOINT.into())
                    }
                } else {
                    Some(messages::NO_CODEPOINT.into())
                }
            }

            // Break down characters only once
            "/unique" => {
                if let Some(reply_text) = reply_text_option {
                    let mut unique = Vec::with_capacity(64);

                    for c in reply_text.chars() {
                        // For bounded values of n, the O(n) search of a list is O(1)
                        if !unique.contains(&c) {
                            unique.push(c);

                            if unique.len() > 50 {
                                break;
                            }
                        }
                    }

                    Some(get_char_names(unique.drain(0..)))
                } else {
                    Some(messages::NEED_REPLY_MESSAGE.into())
                }
            }

            // Break down anything else
            _ => Some(get_char_names(msg_text.chars())),
        };

        if let Some(s) = response {
            let mut req = SendMessage::new(msg.chat.id, s);
            req.parse_mode = ParseMode::Markdown;
            req.disable_web_page_preview = Some(true);

            bot.send(&req).await.unwrap();
        }
    }
}

async fn handle_inline_query(bot: Bot, query: InlineQuery) {
    if query.query.is_empty() {
        return;
    }

    let mut response = AnswerInlineQuery::new(query.id, Vec::new());

    let char_names = get_char_names(query.query.chars());
    let mut content: InputTextMessageContent = char_names.into();
    content.parse_mode = ParseMode::Markdown;

    response.add(InlineQueryResultArticle::new("ID", query.query, content));

    response.cache_time = Some(0);

    response.is_personal = Some(false);

    bot.send(&response).await.unwrap();
}

fn get_char_names(chars: impl Iterator<Item = char>) -> String {
    let mut text = String::with_capacity(4096); // max telegram message length
    let mut entities = 0;

    for c in chars {
        let name = charname::get_name(c as u32);

        let new_part = format!(
            "`U+{val:04X}` [{}](https://fileformat.info/info/unicode/char/{val:X})\n",
            name,
            val = c as u32
        );
        entities += 1;

        // Don't want to exceed message limit or message entity limit
        if text.len() + new_part.len() >= 4000 || entities > 50 {
            text.push_str("\nYour message has been truncated because it was too big. Try some of the special commands in /help");
            break;
        } else {
            text.push_str(&new_part);
        }
    }

    text
}
