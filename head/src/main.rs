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

    if args.len() > 2 {
        if (args[2].as_bytes()[0] == '-' as u8) && (args[2].as_bytes()[1] == 'v' as u8) {
            options.verbose = true;
        }
    }

    if args.len() > 1 {
        if args[1].as_bytes()[0] == '-' as u8 {
            if args[1].as_bytes()[1] == 'n' as u8 { // 'n' == lines
                if args[2].is_empty() == false {
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
                else {
                    options.mode = Mode::LINES(10);
                    figure = 10;
                }
            }
            if args[1].as_bytes()[1] == 'c' as u8 { // 'c' == bytes
                if args[2].is_empty() == false {
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
                else {
                    options.mode = Mode::BYTES(60);
                    figure = 60;
                }
            }
        }
    }
    else {
        options.mode = Mode::LINES(10);
        figure = 10;
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

