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

use std::fmt::Write;

fn main() {
    let token = "210382785:AAEk4K8dZcGmCU-REWTR0sGEFuaNltu4CWk";

    tokio::run(run_bot(token).boxed().unit_error().compat());
}

async fn run_bot(token: &str) {
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

        let mut req = SendMessage::new(msg.chat.id, get_char_names(msg_text.unwrap()));
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

    let mut content: InputTextMessageContent = get_char_names(&query.query).into();
    content.parse_mode = ParseMode::Markdown;

    response.add(InlineQueryResultArticle::new("ID", query.query, content));
    response.cache_time = Some(0);
    response.is_personal = Some(false);

    bot.send(&response).await.unwrap();
}

fn get_char_names(string: &str) -> String {
    let mut text = String::with_capacity(4096); // max telegram message length

    for (i, c) in string.chars().enumerate() {
        let name = charname::get_name(c as u32);

        let new_part = format!(
            "`U+{val:04X}` [{}](http://www.fileformat.info/info/unicode/char/{val:X})\n",
            name,
            val = c as u32
        );

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

    text
}
