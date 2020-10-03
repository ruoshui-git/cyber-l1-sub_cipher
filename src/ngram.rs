use std::{
    collections::HashMap,
    fs::File,
    io::{self, prelude::*},
};

use io::BufReader;

pub struct NGram {
    n: usize,
    prob: HashMap<String, f64>,
    floor: f64,
}

impl NGram {
    /// Calculate the log probability of given seq of text; Assumes text only contains lower alphabet
    pub fn score_chars(&self, chars: &[char]) -> f64 {
        chars.windows(self.n).fold(0., |accu, quad| {
            match self.prob.get(quad.iter().collect::<String>().as_str()) {
                Some(prob) => accu + prob,
                None => accu + self.floor,
            }
        })
    }
}

impl NGram {
    pub fn load_default_quad() -> io::Result<Self> {
        let fin = BufReader::new(File::open("quadgrams.txt")?);

        let mut prob = HashMap::with_capacity(389373);

        let mut words: Vec<String> = vec![];
        let mut freqs: Vec<usize> = vec![];

        for line in fin.lines() {
            let line = line?;
            let mut parts = line.split_whitespace();
            words.push(parts.next().unwrap().to_owned());
            freqs.push(parts.next().unwrap().trim_start().parse().unwrap());
        }

        let total_quadgrams: f64 = freqs.iter().sum::<usize>() as f64;

        for (word, freq) in words.into_iter().zip(freqs.iter()) {
            prob.insert(word, (*freq as f64 / total_quadgrams).log10());
        }
        let floor = (0.01 / total_quadgrams).log10();

        // dbg!(&prob);

        Ok(Self { n: 4, prob, floor })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_rel_freq() {
        let quads = NGram::load_default_quad().unwrap();

        let eng: Vec<char> = "ATTACK THE EAST WALL OF THE CASTLE AT DAWN"
            .to_ascii_lowercase()
            .chars()
            .collect();
        let nonsense: Vec<char> = "FYYFHP YMJ JFXY BFQQ TK YMJ HFXYQJ FY IFBS"
            .to_ascii_lowercase()
            .chars()
            .collect();

        let (score_eng, score_nonsense) = (quads.score_chars(&eng), quads.score_chars(&nonsense));
        assert!(score_eng > score_nonsense);
        dbg!(score_eng, score_nonsense);
    }
}
