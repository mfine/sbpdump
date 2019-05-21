use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use std::process;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
struct Sid {
    sat: u64,
    code: u64,
}

impl fmt::Display for Sid {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:>2}:{:<2}", self.sat, self.code)
    }
}

const MSG_OBS: u64 = 74;
const MSG_EPHEMERIS_GPS: u64 = 138;
const MSG_EPHEMERIS_GAL: u64 = 149;
const MSG_SSR_ORBIT_CLOCK: u64 = 1501;
const MSG_SSR_CODE_BIASES: u64 = 1505;
const MSG_SSR_PHASE_BIASES: u64 = 1510;

#[derive(Debug)]
struct Msg {
    msg_type: u64,
    sender: u64,
    tow: u64,
    sid_vec: Vec<Sid>,
}

impl Msg {
    fn observations(value: &Value, msg_type: u64, sender: u64) -> Option<Msg> {
        value["header"]["t"]["tow"].as_u64().and_then(|tow| {
            value["obs"].as_array().and_then(|obs| {
                let mut sid_vec: Vec<Sid> = Vec::new();
                for ob in obs.iter() {
                    if let Some(sat) = ob["sid"]["sat"].as_u64() {
                        if let Some(code) = ob["sid"]["code"].as_u64() {
                            sid_vec.push(Sid { sat, code })
                        }
                    }
                }
                Some(Msg {
                    msg_type,
                    sender,
                    tow: tow / 1000,
                    sid_vec,
                })
            })
        })
    }

    fn ephemerides(value: &Value, msg_type: u64, sender: u64) -> Option<Msg> {
        value["common"]["toe"]["tow"].as_u64().and_then(|tow| {
            value["common"]["sid"]["sat"].as_u64().and_then(|sat| {
                value["common"]["sid"]["code"].as_u64().and_then(|code| {
                    Some(Msg {
                        msg_type,
                        sender,
                        tow,
                        sid_vec: vec![Sid { sat, code }],
                    })
                })
            })
        })
    }

    fn corrections(value: &Value, msg_type: u64, sender: u64) -> Option<Msg> {
        value["time"]["tow"].as_u64().and_then(|tow| {
            value["sid"]["sat"].as_u64().and_then(|sat| {
                value["sid"]["code"].as_u64().and_then(|code| {
                    Some(Msg {
                        msg_type,
                        sender,
                        tow,
                        sid_vec: vec![Sid { sat, code }],
                    })
                })
            })
        })
    }

    fn new(value: &Value) -> Option<Msg> {
        value["msg_type"].as_u64().and_then(|msg_type| {
            value["sender"].as_u64().and_then(|sender| match msg_type {
                MSG_OBS => Self::observations(value, msg_type, sender),
                MSG_EPHEMERIS_GPS => Self::ephemerides(value, msg_type, sender),
                MSG_EPHEMERIS_GAL => Self::ephemerides(value, msg_type, sender),
                MSG_SSR_ORBIT_CLOCK => Self::corrections(value, msg_type, sender),
                MSG_SSR_CODE_BIASES => Self::corrections(value, msg_type, sender),
                MSG_SSR_PHASE_BIASES => Self::corrections(value, msg_type, sender),
                _ => None,
            })
        })
    }
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

    let mut tow_map: BTreeMap<u64, BTreeMap<u64, BTreeMap<u64, Vec<Sid>>>> = BTreeMap::new();
    for line in buffered.lines() {
        let value: Value = serde_json::from_str(&(line?))?;
        if let Some(msg) = Msg::new(&value) {
            let mut sid_vec = msg.sid_vec;
            tow_map
                .entry(msg.tow)
                .or_insert_with(BTreeMap::new)
                .entry(msg.sender)
                .or_insert_with(BTreeMap::new)
                .entry(msg.msg_type)
                .or_insert_with(Vec::new)
                .append(&mut sid_vec);
        }
    }

    for (tow, sender_map) in tow_map.iter() {
        for (sender, msg_type_map) in sender_map.iter() {
            for (msg_type, sid_vec) in msg_type_map.iter() {
                let mut sid_vec_sort = sid_vec.to_vec();
                sid_vec_sort.retain(|sid| {
                    [
                        0, 1, 5, 6, 7, 8, 9, 10, 11, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
                        25, 26, 27, 28, 56, 57, 58, 61,
                    ]
                    .contains(&sid.code)
                });
                sid_vec_sort.sort();
                sid_vec_sort.dedup();
                let mut sid_vec_str = String::new();
                for sid in sid_vec_sort.iter() {
                    sid_vec_str.push_str(&format!("{} ", sid));
                }
                println!("{:>6} {:>5} {:>4} {}", tow, sender, msg_type, sid_vec_str);
            }
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
