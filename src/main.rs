#![feature(plugin)]
#![plugin(phf_macros)]

extern crate phf;

mod unicode;
use unicode::UNICODE;

fn main() {
    println!("{}", get_char_names(" 🤳"));
}

fn get_char_names(string: &str) -> String {
    let unk = "UNKNOWN CHARACTER";
    let char_names = string.chars().map(|c|
        UNICODE.get(&(c as u32)).unwrap_or(&unk)
    );

    join_string(char_names, "\n")
}


fn join_string<'a, I>(mut set: I, del: &str) -> String
        where I: Iterator<Item=&'a&'a str> {

    let (lower, _) = set.size_hint();

    let mut text = String::with_capacity((3 + del.len()) * lower);

    text.push_str(set.next().unwrap());

    for string in set {
        text.push_str(del);
        text.push_str(string);
    }

    text
}
