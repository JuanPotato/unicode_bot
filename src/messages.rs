pub const ABOUT_MESSAGE: &str = "Hi, I am Unicode Info Bot.

I'll tell you exactly what characters are in your message. \
Just send me any message. I have to stop at 50 characters otherwise the links would no \
longer work. If you need to see more than 50 characters, check out my extra commands \
by sending /help.

To see an example of how I work, just send any text that isn't a command.

Made in Rust by @JuanPotato
https://github.com/JuanPotato/unicode\\_bot";

pub const HELP_MESSAGE: &str = "`/about or /start` - Returns a little blurb about the bot.
`/help` - Returns this message showing all the commands.
`/raw` - Reply to a message that was interpreted as a command with `/raw` to get that message's \
breakdown.

All other non-command messages are responded with a breakdown of that message's characters \
(up to 50 characters).";

pub const NEED_REPLY_MESSAGE: &str = "You need to reply to a message to use this command.";

pub const NEED_REPLY_TEXT_MESSAGE: &str = "Couldn't find any text in the message you replied to.";
