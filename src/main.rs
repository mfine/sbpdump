use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::env;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use std::process;

const MSG_OBS: u64 = 74;
const MSG_EPHEMERIS_GPS: u64 = 138;
const MSG_EPHEMERIS_GAL: u64 = 149;
const MSG_SSR_ORBIT_CLOCK: u64 = 1501;
const MSG_SSR_CODE_BIASES: u64 = 1505;
const MSG_SSR_PHASE_BIASES: u64 = 1510;

const GPS_CODES: [u64; 12] = [0, 1, 5, 6, 7, 8, 9, 10, 11, 56, 57, 58];
const GAL_CODES: [u64; 16] = [
    14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 61,
];

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
struct Sid {
    sat: u64,
    code: u64,
}

impl fmt::Display for Sid {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:>2}:{:<2}", self.sat, self.code)
    }
}

struct Msg {
    msg_type: u64,
    sender: u64,
    tow: u64,
    sid_set: BTreeSet<Sid>,
}

impl Msg {
    fn observations(value: &Value, msg_type: u64, sender: u64) -> Option<Msg> {
        value["header"]["t"]["tow"].as_u64().and_then(|tow| {
            value["obs"].as_array().and_then(|obs| {
                let mut sid_set: BTreeSet<Sid> = BTreeSet::new();
                for ob in obs.iter() {
                    if let Some(sat) = ob["sid"]["sat"].as_u64() {
                        if let Some(code) = ob["sid"]["code"].as_u64() {
                            sid_set.insert(Sid { sat, code });
                        }
                    }
                }
                Some(Msg {
                    msg_type,
                    sender,
                    tow: tow / 1000,
                    sid_set,
                })
            })
        })
    }

    fn ephemerides(value: &Value, msg_type: u64, sender: u64) -> Option<Msg> {
        value["common"]["toe"]["tow"].as_u64().and_then(|tow| {
            value["common"]["sid"]["sat"].as_u64().and_then(|sat| {
                value["common"]["sid"]["code"].as_u64().and_then(|code| {
                    let mut sid_set: BTreeSet<Sid> = BTreeSet::new();
                    sid_set.insert(Sid { sat, code });
                    Some(Msg {
                        msg_type,
                        sender,
                        tow,
                        sid_set,
                    })
                })
            })
        })
    }

    fn corrections(value: &Value, msg_type: u64, sender: u64) -> Option<Msg> {
        value["time"]["tow"].as_u64().and_then(|tow| {
            value["sid"]["sat"].as_u64().and_then(|sat| {
                value["sid"]["code"].as_u64().and_then(|code| {
                    let mut sid_set: BTreeSet<Sid> = BTreeSet::new();
                    sid_set.insert(Sid { sat, code });
                    Some(Msg {
                        msg_type,
                        sender,
                        tow,
                        sid_set,
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

    let mut tow_map: BTreeMap<u64, BTreeMap<u64, BTreeMap<u64, BTreeSet<Sid>>>> = BTreeMap::new();
    for line in buffered.lines() {
        let value: Value = serde_json::from_str(&(line?))?;
        if let Some(msg) = Msg::new(&value) {
            let mut sid_set = msg.sid_set;
            tow_map
                .entry(msg.tow)
                .or_insert_with(BTreeMap::new)
                .entry(msg.sender)
                .or_insert_with(BTreeMap::new)
                .entry(msg.msg_type)
                .or_insert_with(BTreeSet::new)
                .append(&mut sid_set);
        }
    }

    let mut code_set: HashSet<u64> = HashSet::new();
    for code in GPS_CODES.iter() {
        code_set.insert(*code);
    }
    for code in GAL_CODES.iter() {
        code_set.insert(*code);
    }

    for (tow, sender_map) in tow_map.iter() {
        for (sender, msg_type_map) in sender_map.iter() {
            for (msg_type, sid_set) in msg_type_map.iter() {
                let mut sid_set_str = String::new();
                for sid in sid_set.iter() {
                    if code_set.contains(&sid.code) {
                        sid_set_str.push_str(&format!("{} ", sid));
                    }
                }
                println!("{:>6} {:>5} {:>4} {}", tow, sender, msg_type, sid_set_str);
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
