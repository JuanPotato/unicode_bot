## Unicode Info Bot ([@UnicodeInfoBot](https://t.me/UnicodeInfoBot))

This is a neat little telegram bot that I made that tells you all the characters in your message.

![Example screenshot](https://user-images.githubusercontent.com/9531780/67625841-6ab6ad00-f811-11e9-9d8d-77c04dc6fcb3.png)

Commands
 - [x] Send any text and it will tell you the characters in it
 - [x] `/about, /start` - Tells the user about what the bot can do
 - [x] `/help` - Show a list of commands
 - [x] `/raw` - Reply to messages that were parsed as commands to get the characters in them
 - [x] `/filter` - Ignore specified characters. Reply with `/filter` and then a list of characters
                   that you don't want: `/filter abcdefghij`. Case sensitive
 - [x] `/codepoint <codepoint>` - Returns the character at the specified codepoint.
 - [ ] `/full` - Reply to messages that has more than 50 characters and you'll get a txt file
                 that has the entire character list. Will limit to 4096 chars or something.
 - [ ] `/unique` - Reply to a message, only shows each character once in order of occurence
