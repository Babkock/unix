/*
 * chown/tests.rs
 * Babkock/unix
 *
 * Copyright (c) 2019 Tanner Babcock.
 * MIT License.
*/
use super::*;
use std::fs::{metadata, FileType};
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
    match metadata("./Cargo.toml") {
        Ok(m) => {
            dest_gid = Some(m.gid());
            dest_uid = Some(m.uid());
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
    let ft: FileType;

    match metadata("./Cargo.toml") {
        Ok(m) => {
            dest_gid = Some(m.gid());
            dest_uid = Some(m.uid());
            ft = m.file_type();
        },
        Err(e) => {
            panic!("{}", e);
        }
    };

    assert_eq!(dest_uid.unwrap(), 1000);
    assert_eq!(dest_gid.unwrap(), 1000);
    assert!(!ft.is_dir());
    assert!(!ft.is_symlink());
}

#[test]
fn t_parser() {
    let c_uid: Option<u32>;
    let c_gid: Option<u32>;
    
    match parse_spec("root:audio") {
        Ok((u, g)) => {
            c_uid = u;
            c_gid = g;
        },
        Err(e) => {
            panic!("{}", e);
        }
    }

    assert_eq!(c_uid.unwrap(), 0);
    assert_eq!(c_gid.unwrap(), 12);
}

