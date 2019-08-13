/* 
 * cat/tests.rs
 * Babkock/unix
 *
 * Copyright (c) 2019 Tanner Babcock.
 * MIT License.
*/
#[cfg(unix)]
extern crate unix_socket;
extern crate assert_cli;

use super::*;
use crate::{Options, Type, CatResult, get_input_type, open, write_fast};
use assert_cli::Assert;

#[test]
#[cfg(unix)]
fn t_input_type() {
    let mut t: Type = match metadata("/usr").context("/usr").unwrap().file_type() {
        ft if ft.is_dir() => {
            Type::Directory
        }
        ft if ft.is_file() => {
            Type::File
        }
        _ => {
            Type::Stdin
        }
    };

    assert_eq!(t, Type::Directory);
    assert_ne!(t, Type::File);

    t = match metadata("/dev/null").context("/dev/null").unwrap().file_type() {
        ft if ft.is_block_device() => {
            Type::BlockDevice
        }
        ft if ft.is_char_device() => {
            Type::CharDevice
        }
        ft if ft.is_dir() => {
            Type::Directory
        }
        ft if ft.is_file() => {
            Type::File
        }
        _ => {
            Type::Stdin
        }
    };

    assert_eq!(t, Type::CharDevice);
    assert_ne!(t, Type::BlockDevice);
}

#[test]
#[cfg(windows)]
fn t_input_type() {
    let t: Type = match metadata("C:\\Users").context("C:\\Users").unwrap().file_type() {
        ft if ft.is_dir() => {
            Type::Directory
        }
        ft if ft.is_file() => {
            Type::File
        }
        _ => {
            Type::Stdin
        }
    };

    assert_eq!(t, Type::Directory);
    assert_ne!(t, Type::File);
}

#[test]
fn t_options() {
    let o: Options = Options {
        number: NumMode::NumAll,
        squeeze_blank: true,
        show_tabs: true,
        tab: "x".to_string(),
        end_of_line: "\\n".to_string(),
        show_nonprint: false
    };

    assert_eq!(
        o,
        Options {
            number: NumMode::NumAll,
            squeeze_blank: true,
            show_tabs: true,
            tab: "x".to_string(),
            end_of_line: "\\n".to_string(),
            show_nonprint: false
        }
    );
}

#[test]
fn t_print_usage() {
    Assert::main_binary()
        .with_args(&["--help"])
        .stdout().contains(
            "Concatenate FILE(s), or standard input, to standard output\nReads from stdin if FILE is -\n\nUSAGE:\n"
        )
        .unwrap();
}

#[test]
fn t_read_file() {
    let x = Assert::main_binary()
        .with_args(&["Cargo.toml"])
        .stdout().contains(
            "[package]\nname = \"cat\"\nversion = \"0.1.0\"\n"
        )
        .execute();
    if x.is_err() {
        Assert::main_binary()
            .with_args(&["../Cargo.toml"])
            .stdout().contains(
                "[package]\nname = \"cat\"\nversion = \"0.1.0\"\n"
            )
            .unwrap();
    }
}

