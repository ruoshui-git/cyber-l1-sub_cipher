#![allow(dead_code)]


use std::collections::HashSet;

use crate::alpha;

pub struct Dict<'a> {
    words: HashSet<&'a str>,
}

impl<'a> Dict<'a> {
    /// Note that HashSet is moved; This is necesary to ensure it can't accidentally be changed
    pub fn new(words: HashSet<&'a str>) -> Self {
        Self { words }
    }
}

impl Dict<'_> {
    #[allow(dead_code)]
    /// Assumes that every node of vec contains a word with only lower alphabets
    pub fn contains_every_word(&self, list: &[String]) -> bool {
        list.iter().all(|word| self.words.contains(word.as_str()))
    }

    /// Assumes that str is in lower alphabets; can contain some punctuations
    pub fn str_contains_every_word(&self, string: &str) -> bool {
        string
            .split_whitespace()
            .all(|word| self.words.contains(alpha::rm_punct(word).as_str()))
    }

    pub fn count_words(&self, list: &[String]) -> u32 {
        list.iter().fold(0u32, |accu, word| {
            if self.words.contains(word.as_str()) {
                accu + 1
            } else {
                accu
            }
        })
    }
}
