use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug)]
struct Sid {
    code: u64,
    sat: u64
}

impl fmt::Display for Sid {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}:{}", self.code, self.sat)
    }
}

#[derive(Debug)]
struct Sids1 {
    sid_vec: Vec<Sid>,
}

#[derive(Debug)]
struct Msg {
    tow: u64,
    msg_type: u64,
    sid_vec: Vec<Sid>,
}

impl fmt::Display for Msg {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut sid_vec = String::new();
        for sid in self.sid_vec.iter() {
            sid_vec.push_str(&format!("{} ", sid));
        }
        write!(fmt, "{} {} {}", self.tow, self.msg_type, sid_vec)
    }
}

#[derive(Debug)]
struct Sids {
    msg_type: u64,
    sid_vec: Vec<Sid>,
}

impl fmt::Display for Sids {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut sid_vec = String::new();
        for sid in self.sid_vec.iter() {
            sid_vec.push_str(&format!("{} ", sid));
        }
        write!(fmt, "{} {}", self.msg_type, sid_vec)
    }
}

fn tow(value: &Value) -> Option<u64> {
    value["msg_type"].as_u64()
        .and_then(|msg_type|
                  match msg_type {
                      74 => value["header"]["t"]["tow"].as_u64().map(|tow| tow / 1000),
                      138 => value["common"]["toe"]["tow"].as_u64(),
                      149 => value["common"]["toe"]["tow"].as_u64(),
                      1501 => value["time"]["tow"].as_u64(),
                      1505 => value["time"]["tow"].as_u64(),
                      1510 => value["time"]["tow"].as_u64(),
                      _ => None,
                  })
}

fn sids(value: &Value) -> Option<Sids> {
    value["msg_type"].as_u64()
        .and_then(|msg_type|
                  match msg_type {
                      74 =>
                          value["obs"].as_array()
                          .and_then(|obs| {
                              let mut sid_vec: Vec<Sid> = Vec::new();
                              for ob in obs.iter() {
                                  if let Some(sat) = ob["sid"]["sat"].as_u64() {
                                      if let Some(code) = ob["sid"]["code"].as_u64() {
                                          sid_vec.push(Sid { code, sat })
                                      }
                                  }
                              }
                              Some(Sids { msg_type, sid_vec })
                          }),
                      138 =>
                          value["common"]["sid"]["sat"].as_u64()
                          .and_then(|sat|
                                    value["common"]["sid"]["code"].as_u64()
                                    .and_then(|code| Some(Sids { msg_type, sid_vec: vec![Sid { code, sat }] }))),
                      149 =>
                          value["common"]["sid"]["sat"].as_u64()
                          .and_then(|sat|
                                    value["common"]["sid"]["code"].as_u64()
                                    .and_then(|code| Some(Sids { msg_type, sid_vec: vec![Sid { code, sat }] }))),
                      1501 =>
                          value["sid"]["sat"].as_u64()
                          .and_then(|sat|
                                    value["sid"]["code"].as_u64()
                                    .and_then(|code| Some(Sids { msg_type, sid_vec: vec![Sid { code, sat }] }))),
                      1505 =>
                          value["sid"]["sat"].as_u64()
                          .and_then(|sat|
                                    value["sid"]["code"].as_u64()
                                    .and_then(|code| Some(Sids { msg_type, sid_vec: vec![Sid { code, sat }] }))),
                      1510 =>
                          value["sid"]["sat"].as_u64()
                          .and_then(|sat|
                                    value["sid"]["code"].as_u64()
                                    .and_then(|code| Some(Sids { msg_type, sid_vec: vec![Sid { code, sat }] }))),
                      _ => None,
                  })
}

fn msg(value: &Value) -> Option<Msg> {
    value["msg_type"].as_u64()
        .and_then(|msg_type|
                  match msg_type {
                      74 =>
                          value["header"]["t"]["tow"].as_u64()
                          .and_then(|tow|
                                    value["obs"].as_array()
                                    .and_then(|obs| {
                                        let mut sid_vec: Vec<Sid> = Vec::new();
                                        for ob in obs.iter() {
                                            if let Some(sat) = ob["sid"]["sat"].as_u64() {
                                                if let Some(code) = ob["sid"]["code"].as_u64() {
                                                    sid_vec.push(Sid { code, sat })
                                                }
                                            }
                                        }
                                        Some(Msg { tow: tow / 1000, msg_type, sid_vec })})),
                      138 =>
                          value["common"]["toe"]["tow"].as_u64()
                          .and_then(|tow|
                                    value["common"]["sid"]["sat"].as_u64()
                                    .and_then(|sat|
                                              value["common"]["sid"]["code"].as_u64()
                                              .and_then(|code| Some(Msg { tow, msg_type, sid_vec: vec![Sid { code, sat }] })))),
                      149 =>
                          value["common"]["toe"]["tow"].as_u64()
                          .and_then(|tow|
                                    value["common"]["sid"]["sat"].as_u64()
                                    .and_then(|sat|
                                              value["common"]["sid"]["code"].as_u64()
                                              .and_then(|code| Some(Msg { tow, msg_type, sid_vec: vec![Sid { code, sat }] })))),
                      1501 =>
                          value["time"]["tow"].as_u64()
                          .and_then(|tow|
                                    value["sid"]["sat"].as_u64()
                                    .and_then(|sat|
                                              value["sid"]["code"].as_u64()
                                              .and_then(|code| Some(Msg { tow, msg_type, sid_vec: vec![Sid { code, sat }] })))),
                      1505 =>
                          value["time"]["tow"].as_u64()
                          .and_then(|tow|
                                    value["sid"]["sat"].as_u64()
                                    .and_then(|sat|
                                              value["sid"]["code"].as_u64()
                                              .and_then(|code| Some(Msg { tow, msg_type, sid_vec: vec![Sid { code, sat }] })))),
                      1510 =>
                          value["time"]["tow"].as_u64()
                          .and_then(|tow|
                                    value["sid"]["sat"].as_u64()
                                    .and_then(|sat|
                                              value["sid"]["code"].as_u64()
                                              .and_then(|code| Some(Msg { tow, msg_type, sid_vec: vec![Sid { code, sat }] })))),
                      _ => None,
                  })
}

fn main() -> Result<(), Error> {
    let input = File::open("sbp.json")?;
    let buffered = BufReader::new(input);

    // let mut tow_map: BTreeMap<u64, Vec<Sids>> = BTreeMap::new();
    // for line in buffered.lines() {
    //     let value: Value = serde_json::from_str(&(line?))?;
    //     if let Some(tow) = tow(&value) {
    //         if let Some(sids) = sids(&value) {
    //             tow_map.entry(tow)
    //                 .or_insert_with(Vec::new)
    //                 .push(sids);
    //         }
    //     }
    // }

    // for (key, values) in tow_map.iter() {
    //     println!("### {}", key);
    //     for value in values.iter() {
    //         println!("{}", value);
    //     }
    //     println!();
    // }

    // let mut tow_map: BTreeMap<u64, BTreeMap<u64, Vec<Sid>>> = BTreeMap::new();
    // for line in buffered.lines() {
    //     let value: Value = serde_json::from_str(&(line?))?;
    //     if let Some(tow) = tow(&value) {
    //         if let Some(mut sids) = sids(&value) {
    //             tow_map.entry(tow)
    //                 .or_insert_with(BTreeMap::new)
    //                 .entry(sids.msg_type)
    //                 .or_insert_with(Vec::new)
    //                 .append(&mut sids.sid_vec);
    //         }
    //     }
    // }

    // for (key, values) in tow_map.iter() {
    //     println!("### {}", key);
    //     for value in values.iter() {
    //         println!("{:?}", value);
    //     }
    //     println!();
    // }



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
        println!("*** {}", tow);
        for (msg_type, sid_vec) in msg_type_map.iter() {
            println!("### {} {} {:?}", tow, msg_type, sid_vec);
        }
        println!();
    }


    Ok(())
}
