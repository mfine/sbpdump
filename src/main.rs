use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use std::process;

#[derive(Ord)]
#[derive(PartialOrd)]
#[derive(Eq)]
#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Debug)]
struct Sid {
    sat: u64,
    code: u64,
}

impl fmt::Display for Sid {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:>2}:{:<2}", self.sat, self.code)
    }
}

#[derive(Debug)]
struct Msg {
    tow: u64,
    msg_type: u64,
    sid_vec: Vec<Sid>,
}

fn msg(value: &Value) -> Option<Msg> {
    value["msg_type"].as_u64().and_then(
        |msg_type|
        match msg_type {
            74 =>
                value["header"]["t"]["tow"].as_u64().and_then(
                    |tow|
                    value["obs"].as_array().and_then(|obs| {
                        let mut sid_vec: Vec<Sid> = Vec::new();
                        for ob in obs.iter() {
                            if let Some(sat) = ob["sid"]["sat"].as_u64() {
                                if let Some(code) = ob["sid"]["code"].as_u64() {
                                    sid_vec.push(Sid { sat, code })
                                }
                            }
                        }
                        Some(Msg { tow: tow / 1000, msg_type, sid_vec })
                    })),
            138 =>
                value["common"]["toe"]["tow"].as_u64().and_then(
                    |tow|
                    value["common"]["sid"]["sat"].as_u64().and_then(
                        |sat|
                        value["common"]["sid"]["code"].as_u64().and_then(
                            |code|
                            Some(Msg { tow, msg_type, sid_vec: vec![Sid { sat, code }] })))),
            149 =>
                value["common"]["toe"]["tow"].as_u64().and_then(
                    |tow|
                    value["common"]["sid"]["sat"].as_u64().and_then(
                        |sat|
                        value["common"]["sid"]["code"].as_u64().and_then(
                            |code|
                            Some(Msg { tow, msg_type, sid_vec: vec![Sid { sat, code }] })))),
            1501 =>
                value["time"]["tow"].as_u64().and_then(
                    |tow|
                    value["sid"]["sat"].as_u64().and_then(
                        |sat|
                        value["sid"]["code"].as_u64().and_then(
                            |code|
                            Some(Msg { tow, msg_type, sid_vec: vec![Sid { sat, code }] })))),
            1505 =>
                value["time"]["tow"].as_u64().and_then(
                    |tow|
                    value["sid"]["sat"].as_u64().and_then(
                        |sat|
                        value["sid"]["code"].as_u64().and_then(
                            |code|
                            Some(Msg { tow, msg_type, sid_vec: vec![Sid { sat, code }] })))),
            1510 =>
                value["time"]["tow"].as_u64().and_then(
                    |tow|
                    value["sid"]["sat"].as_u64().and_then(
                        |sat|
                        value["sid"]["code"].as_u64().and_then(
                            |code|
                            Some(Msg { tow, msg_type, sid_vec: vec![Sid { sat, code }] })))),
            _ => None,
        })
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("not enough arguments");
        process::exit(1);
    }

    let filename = args[1].clone();
    let input = File::open(filename)?;
    let buffered = BufReader::new(input);

    let mut tow_map: BTreeMap<u64, BTreeMap<u64, Vec<Sid>>> = BTreeMap::new();
    for line in buffered.lines() {
        let value: Value = serde_json::from_str(&(line?))?;
        if let Some(msg) = msg(&value) {
            let mut sid_vec = msg.sid_vec;
            tow_map.entry(msg.tow)
                .or_insert_with(BTreeMap::new)
                .entry(msg.msg_type)
                .or_insert_with(Vec::new)
                .append(&mut sid_vec);
        }
    }

    for (tow, msg_type_map) in tow_map.iter() {
        for (msg_type, sid_vec) in msg_type_map.iter() {
            let mut sid_vec_sort = sid_vec.to_vec();
            sid_vec_sort.retain(
                |sid|
                [0, 1, 5, 6, 7, 8, 9, 10, 11, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 56, 57, 58, 61].contains(&sid.code));
            sid_vec_sort.sort();
            sid_vec_sort.dedup();
            let mut sid_vec_str = String::new();
            for sid in sid_vec_sort.iter() {
                sid_vec_str.push_str(&format!("{} ", sid));
            }
            println!("{:>6} {:>4} {}", tow, msg_type, sid_vec_str);
        }
        println!();
        if tow % 60 == 0 {
            println!()
        }
        if tow % 600 == 0 {
            println!()
        }
    }


    Ok(())
}
