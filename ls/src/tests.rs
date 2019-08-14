/*
 * ls/tests.rs
 * Babkock/unix
 *
 * Copyright (c) 2019 Tanner Babcock.
 * MIT License.
*/
#![allow(unused_imports)]
extern crate assert_cli;

use super::*;
use std::io;
use std::fs::Metadata;
use crate::Options;
use crate::display::{display_permissions, display_file_name, display_file_size, display_uname, display_file_type, display_date};
use crate::file::get_metadata;

#[test]
#[cfg(unix)]
fn t_permissions() {
    let o: Options = Options {
        dirs: vec![String::from(".")],
        show_hidden: false,
        ignore_implied: false,
        dirs_themselves: false,
        long_listing: false,
        dereference: false,
        reverse: false,
        recurse: false,
        sort_by_mtime: false,
        sort_by_ctime: false,
        sort_by_size: false,
        no_sort: true,
        ignore_backups: true,
        numeric_ids: false,
        one_file_per_line: false,
        human_readable: false,
        classify: false,
        inode: false,
        color: false
    };

    let mut m = match get_metadata(&PathBuf::from(r"/home"), &o) {
        Err(e) => {
            panic!("{}", e);
        },
        Ok(m) => m
    };

    // the permissions string for /home ought to be...
    assert_eq!(display_permissions(&m), "rwxr-xr-x");
    assert_eq!(display_file_type(m.file_type()), "d");
    // and it's owned by root
    assert_eq!(display_uname(&m, &o), "root");

    // it's safe to assume every Unix/Linux user has this file
    m = match get_metadata(&PathBuf::from(r"/usr/include/errno.h"), &o) {
        Err(e) => {
            panic!("{}", e);
        },
        Ok(m) => m
    };

    assert_eq!(display_permissions(&m), "rw-r--r--");
    assert_eq!(display_file_type(m.file_type()), "-");
    assert_eq!(display_uname(&m, &o), "root");
}

#[test]
#[cfg(not(unix))]
fn t_permissions() {
    assert_eq!((3+2), 5);
}

// Note: if Cargo.toml is changed, the constant file size in this test will need to be changed
#[test]
fn t_file_size() {
    let o: Options = Options {
        dirs: vec![String::from(".")],
        show_hidden: false,
        ignore_implied: false,
        dirs_themselves: false,
        long_listing: false,
        dereference: false,
        reverse: false,
        recurse: false,
        sort_by_mtime: false,
        sort_by_ctime: false,
        sort_by_size: false,
        no_sort: true,
        ignore_backups: true,
        numeric_ids: false,
        one_file_per_line: false,
        human_readable: false,
        classify: false,
        inode: false,
        color: false
    };

    let m = match get_metadata(&PathBuf::from(r"./Cargo.toml"), &o) {
        Err(_e) => {
            match get_metadata(&PathBuf::from(r"../Cargo.toml"), &o) {
                Err(e) => {
                    panic!("{}", e);
                },
                Ok(m) => m
            }
        },
        Ok(m) => m
    };

    let size = display_file_size(&m, &o);
    
    assert_eq!(size, "298");
}

#[test]
fn t_dir_size() {
    let o: Options = Options {
        dirs: vec![String::from(".")],
        show_hidden: false,
        ignore_implied: false,
        dirs_themselves: false,
        long_listing: false,
        dereference: false,
        reverse: false,
        recurse: false,
        sort_by_mtime: false,
        sort_by_ctime: false,
        sort_by_size: false,
        no_sort: true,
        ignore_backups: true,
        numeric_ids: false,
        one_file_per_line: false,
        human_readable: false,
        classify: false,
        inode: false,
        color: false
    };

    let m = match get_metadata(&PathBuf::from("./bin"), &o) {
        Err(_e) => {
            match get_metadata(&PathBuf::from("./src"), &o) {
                Err(e) => {
                    panic!("{}", e);
                },
                Ok(m) => m
            }
        },
        Ok(m) => m
    };

    // ls always shows file size for directories as 4096
    let size = display_file_size(&m, &o);

    assert_eq!(size, "4096");
}

// as long as Cargo.toml was last modified in 2019 this will work
#[test]
fn t_last_modified() {
    let o: Options = Options {
        dirs: vec![String::from(".")],
        show_hidden: false,
        ignore_implied: false,
        dirs_themselves: false,
        long_listing: false,
        dereference: false,
        reverse: false,
        recurse: false,
        sort_by_mtime: true,
        sort_by_ctime: false,
        sort_by_size: false,
        no_sort: false,
        ignore_backups: true,
        numeric_ids: false,
        one_file_per_line: false,
        human_readable: false,
        classify: false,
        inode: false,
        color: false
    };

    let m = match get_metadata(&PathBuf::from("./Cargo.toml"), &o) {
        Err(_e) => {
            match get_metadata(&PathBuf::from("../Cargo.toml"), &o) {
                Err(e) => {
                    panic!("{}", e);
                },
                Ok(m) => m
            }
        },
        Ok(m) => m
    };

    assert!(display_date(&m, &o).contains("2019-"));
}

