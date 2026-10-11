use crate::messages;
use crate::telegram::{
    InlineQueryResultArticle, InputTextMessageContent, Message, TelegramMethod, Update,
};

/// Processes an incoming Telegram Update and returns an optional Bot API method
/// to be executed as the webhook response.
pub fn process_update(update: &Update) -> Option<TelegramMethod> {
    if let Some(ref msg) = update.message {
        handle_message(msg)
    } else if let Some(ref query) = update.inline_query {
        handle_inline_query(query)
    } else {
        None
    }
}

/// Handles incoming Telegram messages in private chats.
pub fn handle_message(msg: &Message) -> Option<TelegramMethod> {
    // Only handle private chats
    if msg.chat.r#type != "private" {
        return None;
    }

    let msg_text = msg.text.as_deref()?;

    let reply_text_option = msg
        .reply_to_message
        .as_ref()
        .and_then(|r| r.text.as_deref());

    // Split command and arguments
    let (raw_cmd, args) = if let Some(ix) = msg_text.find(|c: char| c.is_ascii_whitespace()) {
        let (cmd_part, arg_part) = msg_text.split_at(ix);
        if arg_part.trim().is_empty() {
            (cmd_part, None)
        } else {
            (cmd_part, Some(arg_part.trim()))
        }
    } else {
        (msg_text, None)
    };

    // Strip bot mention if present (e.g. /start@UnicodeInfoBot -> /start)
    let cmd = if let Some((base, _bot)) = raw_cmd.split_once('@') {
        base
    } else {
        raw_cmd
    };

    let response: String = match cmd {
        "/start" | "/about" => messages::ABOUT_MESSAGE.to_string(),

        "/help" => messages::HELP_MESSAGE.to_string(),

        // Reply to a message that was interpreted as a command to get its breakdown
        "/raw" => {
            if reply_text_option.is_none() {
                messages::NEED_REPLY_MESSAGE.to_string()
            } else {
                get_char_names(reply_text_option.unwrap().chars())
            }
        }

        // Filter away certain characters
        "/filter" => {
            if reply_text_option.is_none() {
                messages::NEED_REPLY_MESSAGE.to_string()
            } else if args.is_none() {
                messages::NO_FILTER.to_string()
            } else {
                let reply_text = reply_text_option.unwrap();
                let filter_chars = args.unwrap();

                let filtered_chars = reply_text.chars().filter(|&c| !filter_chars.contains(c));
                let reply = get_char_names(filtered_chars);

                if reply.is_empty() {
                    messages::FILTER_EXHAUSTIVE.to_string()
                } else {
                    reply
                }
            }
        }

        // Turn U+1F954 -> :potato:
        "/codepoint" => {
            if args.is_none() {
                messages::NO_CODEPOINT.to_string()
            } else {
                let codepoint = args.unwrap().to_lowercase();
                let codepoint = codepoint.strip_prefix('u').unwrap_or(&codepoint);

                let unicode = u32::from_str_radix(codepoint, 16)
                    .ok()
                    .and_then(char::from_u32);

                match unicode {
                    None => messages::INVALID_CODEPOINT.to_string(),
                    Some(c) => c.to_string(),
                }
            }
        }

        // Break down characters only once
        "/unique" => {
            if reply_text_option.is_none() {
                messages::NEED_REPLY_MESSAGE.to_string()
            } else {
                let mut unique = Vec::with_capacity(64);

                for c in reply_text_option.unwrap().chars() {
                    // For bounded values of n, the O(n) search of a list is O(1)
                    if !unique.contains(&c) {
                        unique.push(c);

                        if unique.len() > 50 {
                            break;
                        }
                    }
                }

                get_char_names(unique.into_iter())
            }
        }

        // Break down anything else
        _ => get_char_names(msg_text.chars()),
    };

    Some(TelegramMethod::reply_markdown(
        msg.chat.id,
        msg.message_id,
        response,
    ))
}

pub fn handle_inline_query(query: &crate::telegram::InlineQuery) -> Option<TelegramMethod> {
    if query.query.is_empty() {
        return None;
    }

    let char_names = get_char_names(query.query.chars());
    let content = InputTextMessageContent::markdown(char_names);
    let article = InlineQueryResultArticle::new("ID", &query.query, content);

    Some(TelegramMethod::answer_inline_query(
        &query.id,
        vec![article],
    ))
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
