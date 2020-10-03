mod alpha;
mod caesar;
mod dict;
mod ngram;
mod simp_sub;

use std::{
    env,
    fs::File,
    io::{self, prelude::*},
    process,
    time::Instant,
};

#[allow(unused_imports)]
use caesar::{Caesar, get_percentage, order_percentage};
// use dict::Dict;
use dict::Dict;
use ngram::NGram;

fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() < 3 {
        eprintln!("program analyze|decode [file] [hills]?");
        process::exit(1);
    }

    let mut buf = String::new();

    let mut fin = File::open(&args[2])?;

    fin.read_to_string(&mut buf)?;
    buf.make_ascii_lowercase();

    match args[1].as_str() {
        "analyze" => {
            let percent = get_percentage(&buf);
            let mut order = order_percentage(&percent);
            order.reverse();

            println!("Percent: {:?}", order);
        }

        "decode" => {
            let nhills: usize = if args.len() >= 4 {
                args[3].parse().expect("provide a valid number of hills to climb")
            } else {
                500
            };

            // open dict
            println!("loading dict...");
            let mut dict_in = File::open("words_alpha.txt")?;
            let mut dict_buf = String::with_capacity(4264901); // we know this beforehand
            dict_in.read_to_string(&mut dict_buf)?;
            let dict = Dict::new(dict_buf.lines().collect());
            
            println!("loading quadgrams...");
            let quadgrams = NGram::load_default_quad()?;

            println!("cracking -------------------");
            let start = Instant::now();
            let result = Caesar::crack(&buf, &dict);
            let duration = start.elapsed();

            match result {
                Some(attempt) => {
                    println!("Ciphertext: {}", &buf);
                    println!("Best guess: {}", attempt);
                    println!("Time taken: {:?}", duration);
                    process::exit(0);
                }
                None => {
                    println!("Failed to crack with caesar cipher");
                }
            }

            println!("climbing hills...");

            let start = Instant::now();
            let result = simp_sub::rand_crack(&buf, &quadgrams, nhills);
            let duration = start.elapsed();

            println!(
                "Best result: {}, quad_gram_score: {}",
                result.0.decrypt(&buf),
                result.1
            );
            println!("time taken: {:?}", duration);

        }

        cmd => println!("unknown command: {}", cmd),
    }
    Ok(())
}
