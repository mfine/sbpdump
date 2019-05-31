extern crate clap;

use clap::{App, Arg};
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
        .arg(Arg::with_name("matched").long("matched"))
        .arg(Arg::with_name("gps").long("gps"))
        .arg(Arg::with_name("gal").long("gal"))
        .get_matches();

    let file = matches.value_of("file").unwrap();
    let input = File::open(file)?;

    let matched = matches.is_present("matched");
    let gps = matches.is_present("gps");
    let gal = matches.is_present("gal");

    sbpdump::dump(&input, matched, gps || !gal, gal || !gps)
}
