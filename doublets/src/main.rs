use doublets;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use structopt::StructOpt;

fn read_words(path: &str, length: usize) -> HashSet<String> {
    let f = File::open(path).unwrap();
    let reader = BufReader::new(f);
    reader
        .lines()
        .map(|w| w.unwrap())
        .filter(|w| w.len() == length)
        .map(|w| w.to_string())
        .collect()
}

#[derive(StructOpt)]
#[structopt(name = "Doublets", about = "x")]
struct Opt {
    #[structopt(short, long, default_value = "10")]
    depth: usize,

    #[structopt(short, long, default_value = "/usr/share/dict/words")]
    words: String,

    head: String,
    tail: String,
}

fn main() {
    let opt = Opt::from_args();

    if opt.head.len() != opt.tail.len()
        || opt.head.len() != opt.head.as_bytes().len()
        || opt.head.as_bytes().len() != opt.tail.as_bytes().len()
    {
        println!("Error: the head and tail words must be the same length and be ascii only.");
        return;
    }

    let words = read_words(&opt.words, opt.head.len());

    println!(
        "Dictionary size: {} words of length {}",
        words.len(),
        opt.head.len()
    );

    doublets::find(&opt.head.to_lowercase(), &opt.tail.to_lowercase(), &words, opt.depth);
}
