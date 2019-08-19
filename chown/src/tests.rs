/*
 * chown/tests.rs
 * Babkock/unix
 *
 * Copyright (c) 2019 Tanner Babcock.
 * MIT License.
*/
extern crate assert_cli;

use super::*;
use std::io;
use std::fs::Metadata;
use crate::{Owner, Verbosity, IfFrom, parse_spec, FTS_PHYSICAL};
use crate::group::*;

#[test]
fn t_spec_parser() {
    let executor: Owner = Owner {
        bit_flag: FTS_PHYSICAL,
        dest_uid: 1000, // "you", or, first non-root user
        dest_gid: 12,   // "audio"
        verbosity: Verbosity::Silent,
        recurse: false,
        dereference: true,
        filter: IfFrom::UserGroup(1000, 12),
        preserve_root: false,
        files: vec!["./Cargo.toml"]
    };
    executor.exec();


}

