use queens;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "Queens", about = "Find solutions to the n-queens problem.")]
struct Opt {
    #[structopt(short, long, default_value = "8")]
    num: usize,

    #[structopt(short, long)]
    quiet: bool,
}

fn main() {
    let opt = Opt::from_args();
    println!(
        "Found {} solutions for board size {}",
        queens::queens(opt.num, opt.quiet),
        opt.num
    );
}
