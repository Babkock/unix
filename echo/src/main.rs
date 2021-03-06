/*
 * echo/main.rs
 * Babkock/unix
 *
 * Copyright (c) 2019 Tanner Babcock.
 * MIT License.
*/
//!
//! echo - prints strings to stdout
//!
//! ```
//! $ echo hello world
//! hello world
//! ```
//!
#![allow(unused_assignments)]
use std::{char, env};
use std::io::{self, Write, stdout};
use std::iter::Peekable;
use std::str::Chars;

/// Parse the true value of a hexadecimal, octal, or otherwise non-decimal digit
pub fn parse(input: &mut Peekable<Chars>, base: u32, max: u32, bpd: u32) -> Option<char> {
    let mut r = 0x8000 * 0x1000;
    for _ in 0..max {
        match input.peek().and_then(|c| c.to_digit(base)) {
            Some(n) => {
                r = (r << bpd) | n
            },
            None => break
        }
        input.next();
    }
    char::from_u32(r)
}

/// Parse string with escape sequences, and print it
pub fn escaped(input: &str) -> io::Result<bool> {
    let stdout = stdout();
    let mut output = stdout.lock();
    let mut quit: bool = false;
    let mut buffer = ['\\'; 2];
    let mut i: Peekable<Chars> = input.chars().peekable();

    while let Some(mut c) = i.next() {
        let mut start: usize = 1;
        if c == '\\' {
            /* a list of all the escape codes that can be interpreted */
            if let Some(next) = i.next() {
                c = match next {
                    '\\' => '\\',
                    'a' => '\x07',
                    'b' => '\x08',
                    'c' => { quit = true; break; },
                    'e' => '\x1b',
                    'f' => '\x0c',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    'v' => '\x0b',
                    'x' => parse(&mut i, 16, 2, 4).unwrap_or_else(|| {
                        start = 0; next
                    }),
                    '0' => parse(&mut i, 8, 3, 3).unwrap_or_else(|| {
                        start = 0; next
                    }),
                    _ => { start = 0; next }
                };
            }
        }
        buffer[1] = c;
        for h in &buffer[start..] {
            write!(output, "{}", h)?;
        }
    }

    Ok(quit)
}

fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().collect();  // Options:
    let mut escapes: bool = false;             // Interprets backslashed escape sequences
    let mut newlines: bool = true;             // Prints newlines
    let mut used_option: bool = false;         // Don't print the option if the user supplied one
    let mut help: bool = false;                // Don't echo anything if -h is given

    let stdout = stdout();
    let mut output = stdout.lock();
    let mut quit: bool = false;

    // Clap is nice but it's like 200 KB overhead
    if args.len() > 1 {
        if args[1].as_bytes()[0] == '-' as u8 {
            if args[1].as_bytes()[1] == 'e' as u8 {
                escapes = true;
            }
            if args[1].as_bytes()[1] == 'n' as u8 {
                newlines = false;
            }
            if args[1].as_bytes()[1] == 'h' as u8 {
                help = true;
            }
            used_option = true;
        }
    }

    if help {
        println!("USAGE: {} [OPTS] string", args[0]);
        println!("   -n  Don\'t print newline characters");
        println!("   -e  Interpret escape sequences in the string");
        println!("   -h  Show help");
    }
    else {
        for (i, input) in args.iter().enumerate() {
            if i == 0 || (i == 1 && used_option) {
                continue;
            }
            if escapes {
                quit = escaped(&input)?;
                if quit {
                    break;
                }
            }
            else { write!(output, "{}", input)?; }

            if i < (args.len()-1) {
                write!(output, " ")?;
            }
            else {
                if newlines {
                    write!(output, "\n")?;
                }
            }
        }
    }

    Ok(())
}

