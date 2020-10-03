#![allow(dead_code)]

pub struct Caesar {}
const ASCII_LEN: i32 = 26;
use std::{char, collections::HashMap};

use crate::{alpha, dict::Dict};

/// Get a hashmap of percentage of letters. Assumes all chars are in lowercase.
pub fn get_percentage(text: &str) -> HashMap<char, f64> {
    let mut freq: HashMap<char, u32> = HashMap::with_capacity(26);
    let mut total: u32 = 0;
    for char in text.chars() {
        if alpha::ASCII_LOWER.contains(&char) {
            *freq.entry(char).or_insert(0) += 1;
            total += 1;
        }
    }

    let mut percent = HashMap::with_capacity(26);

    // this will ignore chars other than ASCII letters
    for c in alpha::ASCII_LOWER.iter() {
        match freq.get(c) {
            Some(letter_freq) => {
                percent.insert(*c, *letter_freq as f64 / total as f64);
            }
            None => {
                percent.insert(*c, 0.);
            }
        }
    }

    percent
}
pub fn order_percentage(percent: &HashMap<char, f64>) -> Vec<(char, f64)> {
    let mut entries: Vec<(char, f64)> = percent
        .iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect();

    entries.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    entries
}

impl Caesar {
    /// Shift a letter by a certain amount; assumes that letter is a valid lower ascii alphabet, otherwise leave it unchanged
    ///
    /// Negative shift will move backwards
    fn shift(letter: char, shift: i32) -> char {
        match alpha::get_letter_pos(letter) {
            Some(orig_pos) => {
                let new_pos = match (shift + orig_pos as i32) % ASCII_LEN {
                    x if x < 0 => x + ASCII_LEN,
                    x => x,
                };

                assert!(
                    new_pos >= 0 && new_pos <= ASCII_LEN,
                    "internal error: position should be in range 0 to 27"
                );

                (b'a' + new_pos as u8) as char
            }
            None => letter,
        }
    }

    /// Find the distance between two letters. Assume they are ascii alphabets.
    fn distance(a: char, b: char) -> i32 {
        a as i32 - b as i32
    }

    /// Shift an entire string with Caesar cipher. Assumes all chars are ascii lower.
    fn transform(text: &str, shift: i32) -> String {
        if shift == 0 {
            return text.to_owned();
        }

        let mut s = String::with_capacity(text.len());

        for c in text.chars() {
            s.push(Self::shift(c, shift));
        }

        s
    }

    pub fn crack(text: &str, dict: &Dict) -> Option<String> {
        // for now, primitive frequency analysis
        let lower = text.to_ascii_lowercase();
        let percent = get_percentage(&lower);
        let order = order_percentage(&percent);

        // try mapping all to 'e' until one is readable in english
        for trial in order {
            let dist = Self::distance('e', trial.0);
            let text = Self::transform(&lower, dist);
            if dict.str_contains_every_word(&text) {
                return Some(text);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alpha;

    #[test]
    fn ident_shifts() {
        for letter in alpha::ASCII_LOWER.iter() {
            assert_eq!(*letter, Caesar::shift(*letter, 0));
        }
    }
    #[test]
    fn regular_shifts() {
        assert_eq!('a', Caesar::shift('c', -2));
        assert_eq!('b', Caesar::shift('a', 1));

        assert_eq!('k', Caesar::shift('a', 10));

        assert_eq!('c', Caesar::shift('x', -21));
    }

    #[test]
    fn wrapped_shifts() {
        assert_eq!('a', Caesar::shift('z', 1));
        assert_eq!('u', Caesar::shift('e', -10))
    }

    #[test]
    fn multiple_wraps() {
        assert_eq!('a', Caesar::shift('b', 26 + 26 * 2 + 25));
        assert_eq!('z', Caesar::shift('r', -17 - 26 * 3 - 1));
    }
}
