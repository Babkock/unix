#![allow(unused_imports)]
#[macro_use]
extern crate quick_error;
extern crate clap;

use quick_error::ResultExt;
use clap::{Arg, App};
use std::{env, process};
use std::fs::{metadata, File};
use std::io::{self, stderr, stdin, stdout, BufWriter, Read, Write};

#[cfg(unix)]
//use unix_socket::UnixStream;

#[derive(PartialEq)]
enum NumMode {
    NumNull,
    NumNonEmpty,
    NumAll,
}

quick_error! {
    #[derive(Debug)]
    enum Errors {
        Input(err: io::Error, path: String) {
            display("cat: {0}: {1}", path, err)
            context(path: &'a str, err: io::Error) -> (err, path.to_owned())
            cause(err)
        }

        Output(err: io::Error) {
            display("cat: {0}", err) from()
            cause(err)
        }

        Filetype(p: String) {
            display("cat: {0}: unknown filetype", p)
        }

        Problem(c: usize) {
            display("cat: {0} problems", c)
        }

        Dir(p: String) {
            display("cat: {0}: Is a directory", p)
        }
    }
}

struct Options {
    number: NumMode,     // Line numbering mode
    squeeze_blank: bool, // Compress repeated empty lines
    show_tabs: bool,     // show TAB characters
    tab: String,         // string to show when show_tabs is on
    end_of_line: String, // show characters other than \n at line ends
    nonprint: bool,      // use ^ and M- notation
}

struct Handle {
    reader: Box<Read>
}

/* recognized file types */
enum Types {
    Directory,
    File,
    Stdin,
    SymLink,
    #[cfg(unix)]
    BlockDevice,
    #[cfg(unix)]
    CharDevice,
    #[cfg(unix)]
    Fifo,
    #[cfg(unix)]
    Socket,
}

type CatResult<T> = Result<T, Errors>;

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
    
    Ok(())
}

