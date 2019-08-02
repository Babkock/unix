#![allow(unused_imports)]
use std::{env, io, process};
use std::io::prelude::*;
use std::fs::File;

enum Mode {
    LINES(i32),
    BYTES(i32)
}

struct Options {
    mode: Mode,
    verbose: bool
}

impl Default for Options {
    fn default() -> Options {
        Options {
            mode: Mode::LINES(10),
            verbose: false
        }
    }
}

fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().collect();
    let mut options: Options = Default::default();
    let mut figure: i32 = -1;
    let stdin = io::stdin();
    let stderr = io::stderr();

    if args.len() > 1 {

        if args[1].as_bytes()[0] == '-' as u8 {
            if args[1].as_bytes()[1] == 'n' as u8 { // 'n' == lines
                match args[2].parse::<i32>() {
                    Ok(n) => {
                        options.mode = Mode::LINES(n);
                        figure = n;
                    },
                    Err(_e) => {
                        write!(stderr.lock(), "{}: Argument must be a number: given '{}'\n", args[0], args[2])?;
                        process::exit(2);
                    }
                }
            }
            if args[1].as_bytes()[1] == 'c' as u8 { // 'c' == bytes
                match args[2].parse::<i32>() {
                    Ok(n) => {
                        options.mode = Mode::BYTES(n);
                        figure = n;
                    },
                    Err(_e) => {
                        write!(stderr.lock(), "{}: Argument must be a number, given '{}'\n", args[0], args[2])?;
                        process::exit(2);
                    }
                }
            }
        }
        if (args[2].as_bytes()[0] == '-' as u8) && (args[2].as_bytes()[1] == 'v' as u8) {
            options.verbose = true;
        }

    }
    else {
        write!(stderr.lock(), "USAGE: {} [-v] [-c|-n] NUMBER\n", args[0])?;
        write!(stderr.lock(), "    Read from standard input\n")?;
        write!(stderr.lock(), "    c - print up until NUMBER of bytes\n")?;
        write!(stderr.lock(), "    n - print NUMBER of lines\n")?;
        write!(stderr.lock(), "    v - be verbose\n")?;
        process::exit(1);
    }

    let input = stdin.lock();
    let mut i: i32 = 0;
    for line in input.lines() {
        if i < figure {
            println!("{}", line.unwrap());
        }
        i += 1;
    }

    Ok(())
}
