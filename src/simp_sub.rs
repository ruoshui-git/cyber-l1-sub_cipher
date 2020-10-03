#![allow(dead_code)]

use rand::{prelude::ThreadRng, seq::SliceRandom};

#[derive(Clone)]
pub struct SimpSub {
    // map_keys: Vec<char>,
    mapper: HashMap<char, char, BuildCharHasher>,
    /// Useful for frequency based cracking
    rev_mapper: HashMap<char, char, BuildCharHasher>,
}

impl fmt::Debug for SimpSub {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (from, to) = self.key_as_str();
        writeln!(f, "SimpSub---")?;
        writeln!(f, "from: {}", from)?;
        writeln!(f, "  to: {}", to)?;
        Ok(())
    }
}

use std::{
    collections::HashMap,
    collections::HashSet,
    fmt,
    hash::{BuildHasher, Hasher},
};

use crate::{alpha, ngram::NGram};

use rayon::prelude::*;

/// Return byte value as hash
struct CharHasher(u64);

impl Hasher for CharHasher {
    fn finish(&self) -> u64 {
        self.0 as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        // we don't care about overflow, because we're only encoding ascii alphabets; they should all be encoded
        // within the first byte
        for byte in bytes {
            self.0 += *byte as u64
        }
    }
}

#[derive(Clone)]
struct BuildCharHasher;

impl BuildHasher for BuildCharHasher {
    type Hasher = CharHasher;

    fn build_hasher(&self) -> Self::Hasher {
        CharHasher(0)
    }
}

impl SimpSub {
    pub fn new(from: &str, to: &str) -> Result<Self, &'static str> {
        if from.len() != to.len() {
            return Err("Error creating a key for SimpSub: from.len() must match to.len()");
        }

        let mut map_keys = Vec::with_capacity(from.len());
        let mut mapper: HashMap<char, char, BuildCharHasher> =
            HashMap::with_capacity_and_hasher(from.len(), BuildCharHasher);
        let mut rev_mapper: HashMap<char, char, BuildCharHasher> =
            HashMap::with_capacity_and_hasher(from.len(), BuildCharHasher);
        for pair in from.chars().zip(to.chars()) {
            mapper.insert(pair.0, pair.1);
            rev_mapper.insert(pair.1, pair.0);
            map_keys.push(pair.0);
        }

        Ok(Self { mapper, rev_mapper })
    }

    /// A new SimpSub with random key
    fn with_rand_key() -> Self {
        let mut rand_key_arr = alpha::ASCII_LOWER;
        rand_key_arr.shuffle(&mut rand::thread_rng());
        Self::new(
            alpha::ASCII_LOWER_STR,
            &rand_key_arr.iter().collect::<String>(),
        )
        .unwrap()
    }

    pub fn swap_letters_at(&self, k0: char, k1: char) -> Self {
        let mut new_mapper = self.mapper.clone();
        let v0 = new_mapper.get(&k0).unwrap().to_owned();
        let v1 = new_mapper.get(&k1).unwrap().to_owned();
        new_mapper.insert(k0, v1);
        new_mapper.insert(k1, v0);

        let mut new_rev_mapper = self.rev_mapper.clone();
        new_rev_mapper.insert(v1, k0);
        new_rev_mapper.insert(v0, k1);

        Self {
            // map_keys: self.map_keys.clone(),
            mapper: new_mapper,
            rev_mapper: new_rev_mapper,
        }
    }

    pub fn swap_rand_letters(&self, rng: &mut ThreadRng) -> Self {
        // let keys: Vec<&char> = self.map_keys.choose_multiple(&mut rand::thread_rng(), 2).collect();
        let keys: Vec<&char> = self.mapper.keys().collect();
        let pair: Vec<&&char> = keys.choose_multiple(rng, 2).collect();
        let (k0, k1) = (**pair[0], **pair[1]);

        self.swap_letters_at(k0, k1)
    }

    fn key_as_str(&self) -> (String, String) {
        let mut values: Vec<(&char, &char)> = self.mapper.iter().collect();
        values.sort_unstable_by_key(|f| f.0);
        let (from, to): (Vec<char>, Vec<char>) = values.iter().cloned().unzip();
        (
            from.iter().collect::<String>(),
            to.iter().collect::<String>(),
        )
    }

    #[allow(dead_code)]
    /// Encrypt entire string with key; will turn all letters into lowercase
    pub fn encrypt(&self, text: &str) -> String {
        text.chars()
            .map(|c| {
                self.mapper
                    .get(&c.to_ascii_lowercase())
                    .or(Some(&c))
                    .unwrap()
                    .to_owned()
            })
            .collect()
    }

    pub fn decrypt(&self, text: &str) -> String {
        text.chars().map(|c| self.decrypt_char(c)).collect()
    }

    fn decrypt_char(&self, c: char) -> char {
        *self
            .rev_mapper
            .get(&c.to_ascii_lowercase())
            .or(Some(&c))
            .unwrap()
    }

    #[allow(dead_code)]
    // Assumes all words only contain lower alphabet
    fn decrypt_words(&self, words: &[String]) -> Vec<String> {
        words.iter().map(|word| self.decrypt(word)).collect()
    }

    fn decrypt_words_to_chars(&self, words: &[String]) -> Vec<char> {
        words
            .iter()
            .flat_map(|word| word.chars())
            .map(|c| self.decrypt_char(c))
            .collect()
    }

    /// Manually alter mapper; Assumes that `key` and `val` are both valid keys of mapper
    ///
    fn change_key(&mut self, key: char, val: char) {
        let old_val = *self.mapper.get(&key).unwrap();

        // so we want to swap val and old_val in our mapper
        self.mapper.insert(key, val);

        let key1 = *self.rev_mapper.get(&val).unwrap();

        self.mapper.insert(key1, old_val);

        // now update rev_mapper to match mapper
        self.rev_mapper.insert(val, key);
        self.rev_mapper.insert(old_val, key1);
    }

    fn with_key_change(&self, key: char, val: char) -> Self {
        let mut new_cipher = self.clone();
        new_cipher.change_key(key, val);
        new_cipher
    }

    /// Change mapper (key) so that crypt text will match crib; This is a generalized version of `change_key()`
    ///
    /// Assumes crypt.len() == crib.len()
    fn match_crib(
        &self,
        crypt: &str,
        crib: &str,
        char_used: &HashSet<char, BuildCharHasher>,
    ) -> Result<Self, &'static str> {
        let mut new_coder = self.clone();
        let mem: HashMap<char, char, BuildCharHasher> = HashMap::with_hasher(BuildCharHasher);
        for (crypt_c, crib_c) in crypt.chars().zip(crib.chars()) {
            if char_used.contains(&crypt_c) {
                return Err("crypt char already used");
            }
            if let Some(val) = mem.get(&crypt_c) {
                if *val == crib_c {
                    return Err("conflict in crypt and crib");
                }
            }

            new_coder.change_key(crypt_c, crib_c);
        }
        Ok(new_coder)
    }

    /// Decrypt text and calculate log ngram score
    fn de_score(&self, words: &[String], ngram: &NGram) -> f64 {
        ngram.score_chars(&self.decrypt_words_to_chars(words))
    }
}

/// Build a frequency list of strings in ascending order
fn freq_list(list: &[String]) -> Vec<(String, usize)> {
    let mut map: HashMap<String, usize> = HashMap::new();
    for word in list {
        *map.entry(word.to_owned()).or_insert(0) += 1;
    }

    let mut freqs: Vec<(String, usize)> = map.into_iter().collect();
    freqs.sort_unstable_by_key(|e| e.1);

    freqs
}

/// Run a random guess & check crack
pub fn rand_crack(text: &str, ngram: &NGram, nhills: usize) -> (SimpSub, f64) {
    let lower = text.to_lowercase();
    let words: Vec<String> = lower.split_whitespace().map(alpha::rm_punct).collect();

    let mut ciphers = Vec::with_capacity(nhills);
    for _ in 0..nhills {
        ciphers.push(SimpSub::with_rand_key());
    }

    ciphers
        .into_par_iter()
        .map(|cipher| improve(cipher, &words, ngram))
        .reduce(
            || improve(SimpSub::with_rand_key(), &words, ngram),
            |accu, cur| {
                if accu.1 > cur.1 {
                    accu
                } else {
                    cur
                }
            },
        )
}

fn improve(cipher: SimpSub, words: &[String], ngram: &NGram) -> (SimpSub, f64) {
    let mut cipher = cipher;
    let mut score = cipher.de_score(&words, ngram);
    let letters = cipher.key_as_str().0.chars().collect::<Vec<char>>();

    for (i, c1) in letters[0..letters.len()].iter().enumerate() {
        for c2 in letters[i..letters.len()].iter() {
            let new_cipher = cipher.swap_letters_at(*c1, *c2);

            let new_score = new_cipher.de_score(&words, ngram);
            if new_score > score {
                cipher = new_cipher;
                score = new_score;
            }
        }
    }
    (cipher, score)

    // let mut rep: usize = 0;

    // let mut rng = rand::thread_rng();

    // loop {
    //     let new_cipher = cipher.clone();
    //     new_cipher.swap_rand_letters(&mut rng);
    //     let new_score = new_cipher.de_score(&words, ngram);
    //     if new_score > score {
    //         cipher = new_cipher;
    //         score = new_score;
    //         rep = 0;
    //     } else {
    //         rep += 1;
    //     }

    //     if rep > rep_limit {
    //         break (cipher, score);
    //     }
    // }
}

/// Crack text under `limit` number of trials
pub fn crack(text: &str, limit: usize, ngram: &NGram) -> SimpSub {
    let lower = text.to_lowercase();
    let mut len3: Vec<String> = vec![];
    let mut len2: Vec<String> = vec![];
    let mut len1: Vec<String> = vec![];

    // get all short words
    let words: Vec<String> = lower
        .split_whitespace()
        .map(|word| {
            let word = alpha::rm_punct(word);
            match word.len() {
                1 => len1.push(word.clone()),
                2 => len2.push(word.clone()),
                3 => len3.push(word.clone()),
                _ => {}
            }
            word
        })
        .collect();

    let mut freqs1 = freq_list(&len1);

    let mut cipher = SimpSub::with_rand_key();

    // characters that shouldn't be touched
    let mut letters_used: HashSet<char, BuildCharHasher> = HashSet::with_hasher(BuildCharHasher);

    // the only one-letter words in english are 'a' and 'i', and 'a' is more common, so try both
    // if len1.len() == 1, it could be a or i;
    //      try both
    if freqs1.len() == 1 {
        let c = len1[0].chars().next().unwrap();
        cipher = choose_better(
            cipher.with_key_change(c, 'a'),
            cipher.with_key_change(c, 'i'),
            &words,
            ngram,
        )
        .cipher;
    } else if len1.len() == 2 {
        //  one is a and the other is i, but 'a' has higher probability
        cipher.change_key(freqs1.pop().unwrap().0.chars().next().unwrap(), 'a');
        cipher.change_key(freqs1.pop().unwrap().0.chars().next().unwrap(), 'i');
    }

    let freqs2 = freq_list(&len2);

    if !freqs2.is_empty() {
        // now deal with two letter words
        for (word2, _freq) in freqs2.into_iter().rev() {
            for crib in alpha::CRIBS_L2.iter() {
                letters_used.extend(crib.chars());
                if let Ok(new_cipher) = cipher.match_crib(word2.as_str(), crib, &letters_used) {
                    let result = choose_better(cipher, new_cipher, &words, ngram);
                    cipher = result.cipher;
                    let crib_not_used = result.first;
                    if crib_not_used {
                        crib.chars().for_each(|c| {
                            letters_used.remove(&c);
                        });
                    }
                }
            }
        }
    }
    println!("Done pre-cracking");
    // do the same for len3 cribs
    let mut rng = rand::thread_rng();
    for _ in 0..limit {
        let new_cipher = cipher.clone();
        new_cipher.swap_rand_letters(&mut rng);
        cipher = choose_better(cipher, new_cipher, &words, ngram).cipher
    }
    cipher
}

struct SubComp {
    cipher: SimpSub,
    first: bool,
    score: f64,
}

/// Returns the better cipher and true if cipher0 is used
fn choose_better(cipher0: SimpSub, cipher1: SimpSub, words: &[String], ngram: &NGram) -> SubComp {
    let plain0 = cipher0.decrypt_words_to_chars(words);
    // let wc0 = dict.count_words(&plain0);
    let plain1 = cipher1.decrypt_words_to_chars(words);
    // let wc1 = dict.count_words(&plain1);
    let score0 = ngram.score_chars(&plain0);
    let score1 = ngram.score_chars(&plain1);

    if score0 > score1 {
        SubComp {
            cipher: cipher0,
            first: true,
            score: score0,
        }
    } else {
        SubComp {
            cipher: cipher1,
            first: false,
            score: score1,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::alpha;

    #[test]
    fn test_change_key() {
        let mut key = SimpSub::new(alpha::ASCII_LOWER_STR, alpha::ASCII_LOWER_STR).unwrap();
        key.change_key('a', 't');
        assert_eq!(*key.mapper.get(&'a').unwrap(), 't');
        assert_eq!(*key.mapper.get(&'t').unwrap(), 'a');

        key.change_key('t', 'b');
        assert_eq!(*key.mapper.get(&'b').unwrap(), 'a');
        assert_eq!(*key.mapper.get(&'t').unwrap(), 'b');
    }

    #[test]
    fn test_encrypt() {
        let encoder = SimpSub::new(
            alpha::ASCII_LOWER_STR,
            "AZERTYUIOPQSDFGHJKLMWXCVBN".to_ascii_lowercase().as_str(),
        )
        .unwrap();
        assert_eq!(
            "REGRT EAF YOFR ASHIAZTMOEAS LWZLMOMWMOGFL DART MG A MTVM ZB AFASBNOFU STMMTK YKTJWTFEOTL."
                .to_ascii_lowercase(),
            encoder
                .encrypt("dCode can find alphabetical substitutions made to a text by analyzing letter frequencies."));
    }

    #[test]
    fn test_decrypt() {
        let encoder = SimpSub::new(
            "AZERTYUIOPQSDFGHJKLMWXCVBN".to_ascii_lowercase().as_str(),
            alpha::ASCII_LOWER_STR,
        )
        .unwrap();

        assert_eq!(
            "dcode can find alphabetical substitutions made to a text by analyzing letter frequencies."
                .to_ascii_lowercase(),
            encoder
                .encrypt("REGRT EAF YOFR ASHIAZTMOEAS LWZLMOMWMOGFL DART MG A MTVM ZB AFASBNOFU STMMTK YKTJWTFEOTL.")
        );

        let encoder = SimpSub::new(
            alpha::ASCII_LOWER_STR,
            "AZERTYUIOPQSDFGHJKLMWXCVBN".to_ascii_lowercase().as_str(),
        )
        .unwrap();

        assert_eq!(
            "dcode can find alphabetical substitutions made to a text by analyzing letter frequencies.",
            encoder.decrypt("REGRT EAF YOFR ASHIAZTMOEAS LWZLMOMWMOGFL DART MG A MTVM ZB AFASBNOFU STMMTK YKTJWTFEOTL.")
        );
    }

    #[test]
    fn test_keys_different_after_swap() {
        let cipher0 = SimpSub::new(alpha::ASCII_LOWER_STR, alpha::ASCII_LOWER_STR).unwrap();
        let cipher1 = cipher0.swap_rand_letters(&mut rand::thread_rng());

        let (f0, k0) = cipher0.key_as_str();
        let (f1, k1) = cipher1.key_as_str();
        assert_ne!(k0, k1);
        assert_eq!(f0, f1);
    }

    #[test]
    fn test_keys_diff_many_swaps() {
        let mut rng = rand::thread_rng();
        let mut cipher = SimpSub::with_rand_key();
        for i in 0..10000 {
            let new_cipher = cipher.swap_rand_letters(&mut rng);
            assert_ne!(
                cipher.key_as_str().1,
                new_cipher.key_as_str().1,
                "failed on iteration {}",
                i
            );
            cipher = new_cipher;
        }
    }
}
