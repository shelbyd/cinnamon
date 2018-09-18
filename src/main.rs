extern crate failure;
#[macro_use]
extern crate nom;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

mod ast;
mod parse;

use failure::*;
use parse::*;
use std::fs::*;
use std::io::Read;
use structopt::*;

#[derive(StructOpt)]
struct Cinnamon {
    #[structopt(help = "Input file.")]
    filename: String,
}

fn main() -> Result<(), Error> {
    let args = Cinnamon::from_args();
    let mut file = File::open(args.filename)?;
    let mut contents = vec![];
    file.read_to_end(&mut contents)?;

    let ast = contents
        .parse()
        .map_err(|_| err_msg("Could not parse file"))?;

    for statement in ast {
        statement.execute();
    }

    Ok(())
}
