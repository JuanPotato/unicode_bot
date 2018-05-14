/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate phf;

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

pub fn get_name(c: u32) -> &'static str {
    match UNICODE.get(&c) {
        Some(s) => s,
        None => match c {
            0x3400...0x4DB5 => "CJK Ideograph Extension A",
            0x4E00...0x9FD5 => "CJK Ideograph",
            0xAC00...0xD7A3 => "Hangul Syllable",
            0xD800...0xDB7F => "Non Private Use High Surrogate",
            0xDB80...0xDBFF => "Private Use High Surrogate",
            0xDC00...0xDFFF => "Low Surrogate",
            0xE000...0xF8FF => "Private Use",
            0x17000...0x187EC => "Tangut Ideograph",
            0x20000...0x2A6D6 => "CJK Ideograph Extension B",
            0x2A700...0x2B734 => "CJK Ideograph Extension C",
            0x2B740...0x2B81D => "CJK Ideograph Extension D",
            0x2B820...0x2CEA1 => "CJK Ideograph Extension E",
            0x2CEB0...0x2EBE0 => "CJK Ideograph Extension F",
            0xF0000...0xFFFFD => "Plane 15 Private Use",
            0x100000...0x10FFFD => "Plane 16 Private Use",
            _ => "UNKNOWN CHARACTER",
        },
    }
}
