extern crate clap;

use clap::{App, Arg};
use sbpdump;
use std::fs::File;
use std::io::Error;

fn main() -> Result<(), Error> {
    let matches = App::new("sbpdump")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let file = matches.value_of("file").unwrap();
    let input = File::open(file)?;

    sbpdump::matched(&input)
}
