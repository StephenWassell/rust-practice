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
#[structopt(
    name = "Doublets",
    about = "Find solutions for Lewis Carroll's Doublets puzzles."
)]
struct Opt {
    #[structopt(
        short,
        long,
        default_value = "0",
        about = "Maximum number of steps between the head and tail words; default is the length of the words."
    )]
    steps: usize,

    #[structopt(short, long, default_value = "/usr/share/dict/words")]
    dict: String,

    #[structopt(about = "The starting word.")]
    head: String,
    #[structopt(about = "The ending word.")]
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
