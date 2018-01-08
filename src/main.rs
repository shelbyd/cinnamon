extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use std::fs::*;
use std::io::*;
use structopt::*;

#[derive(StructOpt)]
struct Cinnamon {
    #[structopt(help = "Input file.")] filename: String,
}

fn main() {
    let args = Cinnamon::from_args();
    let file = BufReader::new(File::open(args.filename).unwrap());

    for line in file.lines() {
      println!("{}", line.unwrap());
    }
}
