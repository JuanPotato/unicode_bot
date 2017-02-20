#![feature(plugin)]
#![plugin(phf_macros)]

extern crate phf;

extern crate tg_botapi;

use tg_botapi::args;
use tg_botapi::BotApi;

use std::fmt::Write;
use std::sync::Arc;
use std::thread;
use std::env;

mod unicode;
use unicode::UNICODE;



fn main() {
    let token = &env::var("TOKEN")
        .expect("No bot token provided, please set the environment variable TOKEN");
    let bot_arc = Arc::new(BotApi::new(token));

    let mut update_args = args::GetUpdates::new().timeout(600).offset(0);

    loop {
        let res_updates = bot_arc.get_updates(&update_args);

        match res_updates {
            Ok(updates) => {
                for update in updates {
                    update_args.offset = Some(update.update_id + 1);

                    if let Some(message) = update.message {
                        let bot = bot_arc.clone();

                        thread::spawn(move || {
                            let chat_id = message.chat.id;
                            let msg_id = message.message_id;

                            if let Some(ref text) = message.text {
                                let _ = bot.send_message(&args::SendMessage
                                    ::new(get_char_names(text).as_str())
                                    .chat_id(chat_id)
                                    .reply_to_message_id(msg_id)
                                    .parse_mode("Markdown"));
                                }
                        });
                    }
                }
            },
            Err(e) => {
                // What am I supposed to do here, something intelligent I hope
            }
        }
    }
}

fn get_char_names(string: &str) -> String {
    let unknown = "UNKNOWN CHARACTER";
    let mut chars = string.chars();
        
    let (lower, _) = chars.size_hint();
    let mut text = String::with_capacity(23 * lower); // 24 is a magic number

    // [{ }](http://unic.gq/{:X})
    //   ^  ^______________^ = 16 + ) and at least 1 hex = 18
    //   |                             19 + 5 = 23
    //   at least 3 + [] = 5

    // unic.gq/hex_code_here is just a site I made for shorthanding
    // http://www.fileformat.info/info/unicode/char/hex_code_here

    let first_c = chars.next().unwrap() as u32;
    let _ = write!(text, "[{}](http://unic.gq/{:X})", 
        UNICODE.get(&first_c).unwrap_or(&unknown),
        first_c);

    for c in chars {
        if text.len() >= 3800 {
            let _ = write!(text, "\n\nYour mesage has been truncated because it was too big");
            break;
        } else {
            let _ = write!(text, "\n[{}](http://unic.gq/{:X})", 
                UNICODE.get(&(c as u32)).unwrap_or(&unknown),
                c as u32);
        }
    }

    text
}

