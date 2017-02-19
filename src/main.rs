#![feature(plugin)]
#![plugin(phf_macros)]

extern crate phf;

mod unicode;
use unicode::UNICODE;

fn main() {
    println!("{}", get_char_names(" 🤳"));
}

fn get_char_names(string: &str) -> String {
    let unknown = "UNKNOWN CHARACTER";
    let mut chars = string.chars();
        
    let (lower, _) = chars.size_hint();
    let mut text = String::with_capacity(4 * lower);
    // 3 for "\n" + 3 is the lowest the name could be

    text.push_str(UNICODE.get(&(chars.next().unwrap() as u32)).unwrap_or(&unknown));
 
    for c in chars {
        text.push_str("\n");
        text.push_str(
            match UNICODE.get(&(c as u32)) {
                Some(text) => text,
                None => "UNKNOWN CHARACTER",
            }
        );
    }

    text
}
