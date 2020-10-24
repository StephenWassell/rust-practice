use doublets;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use structopt::StructOpt;

// Faster for small keys than the standard HashSet
// (see http://cglab.ca/%7Eabeinges/blah/hash-rs/)
use fnv::FnvHashSet;

// Read a dictionary file with one word per line.
// The search in doublets::find will only consider lowercase words, to exclude proper names.
fn read_words(path: &str, length: usize) -> FnvHashSet<String> {
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
#[structopt(
    name = "Doublets",
    about = "Find solutions for Lewis Carroll's Doublets puzzles."
)]
struct Opt {
    #[structopt(short, long, default_value = "0")]
    steps: usize,

    #[structopt(short, long, default_value = "/usr/share/dict/words")]
    dict: String,

    head: String,
    tail: String,
}

fn main() {
    let opt = Opt::from_args();

    if opt.head.len() != opt.tail.len() {
        println!("Error: the head and tail words must be the same length.");
        return;
    }

    let dict = read_words(&opt.dict, opt.head.len());

    println!(
        "Dictionary size: {} words of length {}",
        dict.len(),
        opt.head.len()
    );

    doublets::find(&opt.head, &opt.tail, dict, opt.steps);
}
