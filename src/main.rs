#![feature(plugin)]
#![plugin(phf_macros)]

extern crate phf;

mod unicode;
use unicode::UNICODE;
use std::fmt::Write;


fn main() {
    println!("{}", get_char_names(" 123🤳"));
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
        let _ = write!(text, "\n[{}](http://unic.gq/{:X})", 
            UNICODE.get(&(c as u32)).unwrap_or(&unknown),
            c as u32);
    }

    text
}

