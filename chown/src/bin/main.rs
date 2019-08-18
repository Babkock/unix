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
        .arg(Arg::with_name("FILE")
             .help("File or directory to change ownership for")
             .required(true)
             .index(1)
             .multiple(true))
        .arg(Arg::with_name("changes")
             .short("c")
             .long("changes")
             .help("Verbosity: only report when a change is made")
             .takes_value(false))
        .arg(Arg::with_name("silent")
             .short("f")
             .long("silent")
             .help("Verbosity: silent")
             .takes_value(false))
        .arg(Arg::with_name("verbose")
             .short("v")
             .long("verbose")
             .help("Verbosity: output a diagnostic for every file processed")
             .takes_value(false))
        .arg(Arg::with_name("dereference")
             .short("")
             .long("dereference")
             .help("Affect the refferer of each symbolic link")
             .takes_value(false))
        .arg(Arg::with_name("no-dereference")
             .short("h")
             .long("no-dereference")
             .help("Affect the symbolic links instead of any referenced file")
             .takes_value(false))
        .arg(Arg::with_name("from")
             .short("")
             .long("from")
             .help("Change the owner and/or group of each file only if its current owner and/or group match those specified. Either may be omitted, in which case a match is not required for the omitted attribute")
             .value_name("CURRENT_OWNER:CURRENT_GROUP")
             .takes_value(true))
        .arg(Arg::with_name("reference")
             .short("")
             .long("reference")
             .help("Use RFILE's owner and group rather than specifying OWNER:GROUP values")
             .value_name("RFILE")
             .takes_value(true))
        .arg(Arg::with_name("no-preserve-root")
             .short("")
             .long("no-preserve-root")
             .help("Do not treat '/' any different (default)")
             .takes_value(false))
        .arg(Arg::with_name("preserve-root")
             .short("")
             .long("preserve-root")
             .help("Fail to operate recursively on '/'")
             .takes_value(false))
        .arg(Arg::with_name("recursive")
             .short("R")
             .long("recursive")
             .help("Operate on files and directories recursively")
             .takes_value(false))
        .arg(Arg::with_name("traverse")
             .short("H")
             .long("traverse")
             .help("If a command line argument is a symbolic link to a directory, traverse it")
             .takes_value(false))
        .arg(Arg::with_name("traverse-all")
             .short("L")
             .long("traverse-all")
             .help("Traverse every symbolic link to a directory encountered")
             .takes_value(false))
        .arg(Arg::with_name("no-traverse")
             .short("P")
             .long("no-traverse")
             .help("Do not traverse any symbolic links (default)")
             .takes_value(false))
        .get_matches();

    let mut files: Vec<String> = Vec::new();

    match matches.value_of("FILE") {
        None => {
            println!("No argument supplied");
            return 1;
        },
        Some(n) => {
            files.push(n.to_string());
        }
    };

    let mut verbosity: Verbosity = if matches.occurrences_of("changes") != 0 {
        Verbosity::Changes
    } else if matches.occurrences_of("silent") != 0 || matches.occurrences_of("quiet") != 0 {
        Verbosity::Silent
    } else if matches.occurrences_of("verbose") != 0 {
        Verbosity::Verbose
    } else {
        Verbosity::Normal
    };
}
