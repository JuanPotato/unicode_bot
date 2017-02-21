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
        get_name(first_c), first_c);

    for c in chars {
        if text.len() >= 3800 {
            let _ = write!(text, "\n\nYour mesage has been truncated because it was too big");
            break;
        } else {
            let _ = write!(text, "\n[{}](http://unic.gq/{:X})", 
                get_name(c as u32), c as u32);
        }
    }

    text
}

fn get_name(c: u32) -> &'static str {
    match UNICODE.get(&c) {
        Some(s) => s,
        None => {
            match c {
                0x3400   ... 0x4DB5   => "CJK Ideograph Extension A",
                0x4E00   ... 0x9FD5   => "CJK Ideograph",
                0xAC00   ... 0xD7A3   => "Hangul Syllable",
                0xD800   ... 0xDB7F   => "Non Private Use High Surrogate",
                0xDB80   ... 0xDBFF   => "Private Use High Surrogate",
                0xDC00   ... 0xDFFF   => "Low Surrogate",
                0xE000   ... 0xF8FF   => "Private Use",
                0x17000  ... 0x187EC  => "Tangut Ideograph",
                0x20000  ... 0x2A6D6  => "CJK Ideograph Extension B",
                0x2A700  ... 0x2B734  => "CJK Ideograph Extension C",
                0x2B740  ... 0x2B81D  => "CJK Ideograph Extension D",
                0x2B820  ... 0x2CEA1  => "CJK Ideograph Extension E",
                0xF0000  ... 0xFFFFD  => "Plane 15 Private Use",
                0x100000 ... 0x10FFFD => "Plane 16 Private Use",
                                    _ => "UNKNOWN CHARACTER",
            }
        }
    }
}


