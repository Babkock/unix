#![allow(unused_imports)]
extern crate cat;
extern crate clap;

use clap::{Arg, App};
use cat::*;
use std::io::{self, stderr, stdin, stdout, BufWriter, Read, Write};

fn main() -> io::Result<()> {
    let matches = App::new("cat").about("Concatenate FILE(s), or standard input, to standard output\nReads from stdin if FILE is -")
        .arg(Arg::with_name("FILE")
             .help("The file to load")
             .required(false)
             .index(1)
             .multiple(true))
        .arg(Arg::with_name("show-all")
            .short("a")
            .long("show-all")
            .help("equivalent to -vET")
            .takes_value(false))
        .arg(Arg::with_name("number-nonblank")
            .short("b")
            .long("number-nonblank")
            .help("number non-empty output lines, overrides -n")
            .takes_value(false))
        .arg(Arg::with_name("show-ends")
            .short("E")
            .long("show-ends")
            .help("display $ at end of each line")
            .takes_value(false))
        .arg(Arg::with_name("number")
            .short("n")
            .long("number")
            .help("number all output lines")
            .takes_value(false))
        .arg(Arg::with_name("squeeze-blank")
            .short("s")
            .long("squeeze-blank")
            .help("suppress repeat empty lines in output")
            .takes_value(false))
        .arg(Arg::with_name("show-tabs")
            .short("T")
            .long("show-tabs")
            .help("display TAB characters as ^I")
            .takes_value(false))
        .arg(Arg::with_name("show-nonprinting")
            .short("v")
            .long("show-nonprinting")
            .help("use ^ and M- notation, except for \\n and \\t")
            .takes_value(false))
        .get_matches();

    let number_mode = if matches.occurrences_of("b") == 1 {
        NumMode::NumNonEmpty
    } else if matches.occurrences_of("n") == 1 {
        NumMode::NumAll
    } else {
        NumMode::NumNull
    };

    let show_nonprint: bool = match matches.occurrences_of("v") {
        0 => {
            if matches.occurrences_of("A") == 1 {
                true
            } else {
                false
            }
        },
        1 => true,
        _ => false
    };
    let show_ends: bool = match matches.occurrences_of("E") {
        0 => {
            if matches.occurrences_of("A") == 1 {
                true
            } else {
                false
            }
        },
        1 => true,
        _ => false
    };
    let show_tabs: bool = match matches.occurrences_of("T") {
        0 => {
            if matches.occurrences_of("A") == 1 {
                true
            } else {
                false
            }
        },
        1 => true,
        _ => false
    };
    let squeeze_blank: bool = match matches.occurrences_of("s") {
        0 => false,
        1 => true,
        _ => false
    };

    let mut files: Vec<&str> = Vec::new();

    match matches.values_of("FILE") {
        None => {
            files.push("-");
        },
        Some(n) => {
            files = n.collect();
        }
    };
   
    /* we can now assume files is a vector of files to read, otherwise just '-' noting stdin */
    
    
    Ok(())
}

