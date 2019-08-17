/*
 * chmod/main.rs
 * Babkock/unix
 *
 * Copyright (c) 2019 Tanner Babcock.
 * MIT License.
*/
#![allow(unused_imports)]
extern crate chown;
extern crate clap;

use clap::{Arg, App};
use chown::Options;
use std::io;

fn main() -> io::Result<()> {
    let matches = App::new("chown").about("Change the file owner and group")
        .arg(Arg::with_name)
}
