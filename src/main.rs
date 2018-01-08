#[macro_use]
extern crate nom;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

mod ast;
mod parse;

use parse::*;
use std::fs::*;
use std::io::*;
use structopt::*;

#[derive(StructOpt)]
struct Cinnamon {
    #[structopt(help = "Input file.")] filename: String,
}

fn main() {
    let args = Cinnamon::from_args();
    let mut file = File::open(args.filename).unwrap();
    let mut contents = vec![];
    file.read_to_end(&mut contents).unwrap();

    let ast = contents.parse().unwrap();

    println!("{:?}", ast);
}
