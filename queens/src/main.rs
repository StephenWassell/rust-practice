use structopt::StructOpt;
pub mod lib;

#[derive(StructOpt)]
#[structopt(name = "8 queens", about = "An example of StructOpt usage.")]
struct Opt {
    #[structopt(short, long, default_value = "8")]
    num: usize,

    #[structopt(short, long)]
    quiet: bool,
}

fn main() {
	let opt = Opt::from_args();
    println!("{}", lib::queens(opt.num, opt.quiet));
}
