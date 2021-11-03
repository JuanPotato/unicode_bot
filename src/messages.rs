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
`/filter CHARACTERS` - Reply to a message to break it down while ignoring all given CHARACTERS
`/codepoint <codepoint>` - Returns the character at the specified codepoint.

All other non-command messages are responded with a breakdown of that message's characters \
(up to 50 characters).";

pub const NEED_REPLY_MESSAGE: &str = "You need to reply to a text message to use this command.";

pub const INVALID_CODEPOINT: &str = "Didn't recognise that codepoint.";

pub const NO_CODEPOINT: &str = "You need to specify a codepoint to use this command.";
