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
use crate::display::{display_permissions, display_permissions_unix, display_file_name, should_display, display_uname, display_item_long};
use crate::file::{get_metadata, display_dir_entry_size, get_inode, get_file_name};

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
    assert_eq!(display_uname(&m, &o), "root");
}

#[test]
#[cfg(not(unix))]
fn t_permissions() {
    assert_eq!((3+2), 5);
}

