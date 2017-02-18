#![feature(plugin)]
#![plugin(phf_macros)]
#![plugin(clippy)]

extern crate phf;

mod unicode;
use unicode::UNICODE;

fn main() {
    do_a_thing("🤷‍♀️");
    do_a_thing("🥔");
    do_a_thing("🤷");
    do_a_thing("👊👊🏻");
    do_a_thing("🤷‍♂");
    do_a_thing("👨🏻‍💻");
}

fn do_a_thing(string: &str) {
    let aa = string.chars().map(|c|
        UNICODE.get(format!("{:0>4X}", c as u32).as_str())
    );

    let mut msg = String::new();

    for a in aa {
        msg.push_str(a.unwrap_or(&"UNKNOWN CHARACTER"));
        msg.push_str("\n");
    }

    println!("{}", msg);
}