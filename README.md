## Unicode Info Bot ([@UnicodeInfoBot](t.me/UnicodeInfoBot))

This is a neat little telegram bot that I made that tells you all the characters in your message.

Commands
 - [x] Send any text and it will tell you the characters in it
 - [ ] `/start, /help` - Tells the user about what the bot can do
 - [ ] `/raw` - Reply to messages that were parsed as commands to get the characters in them
 - [ ] `/full` - Reply to messages that are moret than 50 characters and you'll get a txt file
                 that has the entire character list. Will limit to 4096 chars or something.     
 - [ ] `/unique` - Reply to a message, only shows each character once in order of occurence
 - [ ] `/filter` - Ignore specified characters. Reply with `/filter` and then some sort of syntax,
                   maybe regex for range. or just all the characters that you dont want, and unicode
                   escape is allowed. `/filter abcdefghij\x00\u{0001}`