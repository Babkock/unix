/*
 * chown/tests.rs
 * Babkock/unix
 *
 * Copyright (c) 2019 Tanner Babcock.
 * MIT License.
*/
#![allow(unused_imports)]
extern crate assert_cli;

use super::*;
use crate::{Owner, Verbosity, IfFrom, parse_spec, FTS_PHYSICAL};

#[test]
fn t_group_change() {
    let executor: Owner = Owner {
        bit_flag: FTS_PHYSICAL,
        dest_uid: Some(1000 as u32), // "you", or, first non-root user
        dest_gid: Some(12 as u32),   // "audio"
        verbosity: Verbosity::Silent,
        recurse: false,
        dereference: true,
        filter: IfFrom::UserGroup(1000, 1000),
        preserve_root: false,
        files: vec!["./Cargo.toml".to_string()]
    };
    executor.exec();

    let dest_uid: Option<u32>;
    let dest_gid: Option<u32>;
    match fs::metadata("./Cargo.toml") {
        Ok(meta) => {
            dest_gid = Some(meta.gid());
            dest_uid = Some(meta.uid());
        },
        Err(e) => {
            panic!("{}", e);
        }
    };

    assert_eq!(dest_gid.unwrap(), 12);
    assert_eq!(dest_uid.unwrap(), 1000);
}

#[test]
fn t_revert_change() {
    let executor: Owner = Owner {
        bit_flag: FTS_PHYSICAL,
        dest_uid: Some(1000 as u32),
        dest_gid: Some(1000 as u32),
        verbosity: Verbosity::Silent,
        recurse: false,
        dereference: true,
        filter: IfFrom::UserGroup(1000, 12),
        preserve_root: false,
        files: vec!["./Cargo.toml".to_string()]
    };

    executor.exec();

    let dest_uid: Option<u32>;
    let dest_gid: Option<u32>;
    match fs::metadata("./Cargo.toml") {
        Ok(meta) => {
            dest_gid = Some(meta.gid());
            dest_uid = Some(meta.uid());
        },
        Err(e) => {
            panic!("{}", e);
        }
    };

    assert_eq!(dest_uid.unwrap(), 1000);
    assert_eq!(dest_gid.unwrap(), 1000);
}

