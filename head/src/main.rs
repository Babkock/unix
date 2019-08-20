/*
 * head/main.rs
 * Babkock/unix
 *
 * Copyright (c) 2019 Tanner Babcock.
 * MIT License.
*/
use std::{env, io, process};
use std::io::prelude::*;
use std::io::{Read, stdin, stderr};
//use std::fs::File;

enum Mode {
    Lines(i32),
    Bytes(i32)
}

struct Options {
    mode: Mode
}

impl Default for Options {
    fn default() -> Options {
        Options {
            mode: Mode::Lines(10)
        }
    }
}

fn usage() {
    let args: Vec<_> = env::args().collect();
    println!("USAGE: {} [-c|-n] NUMBER", args[0]);
    println!("    -c  print up to NUMBER of bytes");
    println!("    -n  print up to NUMBER of lines");
    println!("    -h  show help");
}

fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().collect();
    let mut options: Options = Default::default();
    let stdin = stdin();
    let stderr = stderr();

    if args.len() > 1 {
        if args[1].as_bytes()[0] == '-' as u8 {
            if args[1].as_bytes()[1] == 'n' as u8 { // 'n' == lines
                if args[2].is_empty() == false {
                    match args[2].parse::<i32>() {
                        Ok(n) => {
                            options.mode = Mode::Lines(n);
                        },
                        Err(_e) => {
                            write!(stderr.lock(), "{}: Argument must be a number: given '{}'\n", args[0], args[2])?;
                            process::exit(2);
                        }
                    }
                }
                else {
                    options.mode = Mode::Lines(10);
                }
            }
            if args[1].as_bytes()[1] == 'c' as u8 { // 'c' == bytes
                if args[2].is_empty() == false {
                    match args[2].parse::<i32>() {
                        Ok(n) => {
                            options.mode = Mode::Bytes(n);
                        },
                        Err(_e) => {
                            write!(stderr.lock(), "{}: Argument must be a number, given '{}'\n", args[0], args[2])?;
                            process::exit(2);
                        }
                    
                    }
                }
                else {
                    options.mode = Mode::Bytes(60);
                }
            }
        }
        if (args[1].as_bytes()[0] == '-' as u8) && (args[1].as_bytes()[1] == 'h' as u8) {
            usage();
            process::exit(0);
        }
    }
    else {
        options.mode = Mode::Lines(10);
    }

    let input = stdin.lock();
    let mut i: i32 = 0;
    match options.mode {
        Mode::Bytes(n) => {
            for byte in input.bytes().take(n as usize) {
                print!("{}", byte.unwrap() as char);
            }
        },
        Mode::Lines(n) => {
            for line in input.lines() {
                if i < n {
                    println!("{}", line.unwrap());
                    i += 1;
                }
                else {
                    break;
                }
            }
        }
    }

    Ok(())
}

