/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate charname;
extern crate futures;
extern crate telegram_bot;
extern crate tokio_core;

use charname::get_name;

use futures::stream::Stream;

use telegram_bot::{Api, CanReplySendMessage, MessageKind, ParseMode, UpdateKind};

use tokio_core::reactor::Core;

use std::env;
use std::fmt::Write;

fn main() {
    let mut core = Core::new().unwrap();

    let token = env::var("TOKEN").unwrap();
    let api = Api::configure(token).build(core.handle()).unwrap();

    let future = api.stream().for_each(|update| {
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                api.spawn(
                    message
                        .text_reply(get_char_names(data))
                        .parse_mode(ParseMode::Markdown)
                        .disable_preview(),
                );
            }
        }

        Ok(())
    });

    core.run(future).unwrap();
}

fn get_char_names(string: &str) -> String {
    let mut text = String::with_capacity(4096); // max telegram message length

    for c in string.chars() {
        let name = get_name(c as u32);

        let new_part = format!(
            "`U+{val:04X}` [{}](http://www.fileformat.info/info/unicode/char/{val:X})\n",
            name,
            val = c as u32
        );

        if text.len() + new_part.len() >= 3900 {
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
