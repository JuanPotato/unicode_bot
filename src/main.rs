/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use futures::StreamExt;
use tg_botapi::methods::{AnswerInlineQuery, SendMessage};
use tg_botapi::types::{
    ChatType, InlineQuery, InlineQueryResultArticle, InputTextMessageContent, Message, ParseMode,
    UpdateType,
};
use tg_botapi::Bot;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use std::collections::HashMap;

mod messages;

#[tokio::main]
async fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() == 2 {
        run_bot(args[1].clone()).await;
    } else {
        eprintln!("USAGE: {} TOKEN", args[0]);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum StatisticAction {
    Breakdown,
    StartAbout,
    Help,
    Raw,
    Filter,
    Codepoint,
    Unique,
    BadCmd,
    Inline,
}

#[derive(Debug, Copy, Clone)]
struct Statistic {
    user_id: i64,
    action: StatisticAction,
}

async fn run_bot(token: String) {
    let bot = Bot::new(token);

    let mut updates = bot.start_polling();

    let (stat_tx, stat_rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(stat_handler(stat_rx));

    while let Some(update) = updates.next().await {
        match update.update_type {
            UpdateType::Message(message) => {
                tokio::spawn(handle_message(bot.clone(), message, stat_tx.clone()));
            }

            UpdateType::InlineQuery(query) => {
                tokio::spawn(handle_inline_query(bot.clone(), query, stat_tx.clone()));
            }
            _ => {}
        }
    }
}

async fn stat_handler(mut rx: UnboundedReceiver<Statistic>) {
    let mut action_stats = HashMap::new();
    let mut user_stats = HashMap::new();
    let mut last_hour = 0;

    println!("Hour, Unique Users, Total Msgs, Breakdown, StartAbout, Help, Raw, Filter, Codepoint, Unique, BadCmd, Inline");

    while let Some(stat) = rx.recv().await {
        let stat_hour = (chrono::Utc::now().timestamp() / 3600) * 3600;

        if stat_hour != last_hour && last_hour != 0 {
            let total_unique = user_stats.len();
            let total_messages = user_stats.values().sum::<i32>();
            user_stats.clear();

            let breakdown = action_stats.remove(&StatisticAction::Breakdown).unwrap_or(0);
            let startabout = action_stats.remove(&StatisticAction::StartAbout).unwrap_or(0);
            let help = action_stats.remove(&StatisticAction::Help).unwrap_or(0);
            let raw = action_stats.remove(&StatisticAction::Raw).unwrap_or(0);
            let filter = action_stats.remove(&StatisticAction::Filter).unwrap_or(0);
            let codepoint = action_stats.remove(&StatisticAction::Codepoint).unwrap_or(0);
            let unique = action_stats.remove(&StatisticAction::Unique).unwrap_or(0);
            let badcmd = action_stats.remove(&StatisticAction::BadCmd).unwrap_or(0);
            let inline = action_stats.remove(&StatisticAction::Inline).unwrap_or(0);

            println!("{last_hour}, {total_unique}, {total_messages}, {breakdown}, {startabout}, {help}, {raw}, {filter}, {codepoint}, {unique}, {badcmd}, {inline}");
        }

        last_hour = stat_hour;

        *action_stats.entry(stat.action).or_insert(0) += 1;
        *user_stats.entry(stat.user_id).or_insert(0) += 1;
    }
}

struct Response(String, StatisticAction);

async fn handle_message(bot: Bot, msg: Message, stat_tx: UnboundedSender<Statistic>) {
    if msg.chat.chat_type == ChatType::Private {
        let msg_text = match msg.get_text() {
            Some(text) => text.as_str(),
            None => return,
        };

        let user_id = msg.from.as_ref().map(|u| u.id).unwrap_or(0);

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

        use StatisticAction::*;
        let Response(response, action) = match cmd {
            "/start" | "/about" => Response(messages::ABOUT_MESSAGE.into(), StartAbout),

            "/help" => Response(messages::HELP_MESSAGE.into(), Help),

            // Reply to a message that was interpreted as a command to get its breakdown
            "/raw" => {
                if let Some(reply_text) = reply_text_option {
                    Response(get_char_names(reply_text.chars()), Raw)
                } else {
                    Response(messages::NEED_REPLY_MESSAGE.into(), BadCmd)
                }
            }

            // Filter away certain characters
            "/filter" => {
                if let Some(reply_text) = reply_text_option {
                    if let Some(args) = args {
                        let filtered_chars = reply_text.chars().filter(|&c| !args.contains(c));
                        let reply = get_char_names(filtered_chars);

                        if !reply.is_empty() {
                            Response(reply, Filter)
                        } else {
                            Response(messages::FILTER_EXHAUSTIVE.into(), BadCmd)
                        }
                    } else {
                        Response(messages::NO_FILTER.into(), BadCmd)
                    }
                } else {
                    Response(messages::NEED_REPLY_MESSAGE.into(), BadCmd)
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
                        Response(c.to_string(), Codepoint)
                    } else {
                        Response(messages::INVALID_CODEPOINT.into(), BadCmd)
                    }
                } else {
                    Response(messages::NO_CODEPOINT.into(), BadCmd)
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

                    Response(get_char_names(unique.drain(0..)), Unique)
                } else {
                    Response(messages::NEED_REPLY_MESSAGE.into(), BadCmd)
                }
            }

            // Break down anything else
            _ => Response(get_char_names(msg_text.chars()), Breakdown),
        };

        let mut req = SendMessage::new(msg.chat.id, response);
        req.parse_mode = ParseMode::Markdown;
        req.disable_web_page_preview = Some(true);

        bot.send(&req).await.unwrap();

        stat_tx.send(Statistic { user_id, action }).unwrap();
    }
}

async fn handle_inline_query(bot: Bot, query: InlineQuery, stat_tx: UnboundedSender<Statistic>) {
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

    let user_id = query.from.id;

    stat_tx.send(Statistic {
        user_id,
        action: StatisticAction::Inline,
    }).unwrap();
}

fn get_char_names(chars: impl Iterator<Item=char>) -> String {
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
