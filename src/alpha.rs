#![allow(dead_code)]

pub const ASCII_LOWER: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];

pub const ASCII_LOWER_STR: &str = "abcdefghijklmnopqrstuvwxyz";

/// Returns of the position of letter in the alphabet. None if it is not a lowercase ascii letter.
pub fn get_letter_pos(letter: char) -> Option<u8> {
    if letter.is_ascii_lowercase() {
        Some(letter as u8 - b'a')
    } else {
        None
    }
}

pub fn rm_punct(word: &str) -> String {
    word.replace(
        |x| match x {
            ',' | '.' | '!' | ':' | ';' => true,
            _ => false,
        },
        "",
    )
}

/// Common list of two-letter words
///
/// From: [https://www3.nd.edu/~busiforc/handouts/cryptography/cryptography%20hints.html](https://www3.nd.edu/~busiforc/handouts/cryptography/cryptography%20hints.html)
pub const CRIBS_L2: [&str; 24] = [
    "of", "to", "in", "it", "is", "be", "as", "at", "so", "we", "he", "by", "or", "on", "do", "if",
    "me", "my", "up", "an", "go", "no", "us", "am",
];
