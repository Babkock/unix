/*
 * chown/lib.rs
 * Babkock/unix
 *
 * Copyright (c) 2019 Tanner Babcock.
 * MIT License.
*/
extern crate libc;
extern crate walkdir;
use walkdir::WalkDir;
use libc::{self, gid_t, lchown, uid_t};
use std::fs::{self, Metadata};
use std::os::unix::fs::MetadataExt;
use std::io;
use std::path::Path;
use std::convert::AsRef;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;

// mod group;

const FTS_COMFOLLOW: u8 = 1;
const FTS_PHYSICAL: u8 = 1 << 1;
const FTS_LOGICAL: u8 = 1 << 2;

#[derive(PartialEq, Debug)]
pub enum Verbosity {
    Silent,
    Changes,
    Verbose,
    Normal
}

pub enum IfFrom {
    All,
    User(u32),
    Group(u32),
    UserGroup(u32, u32),
}

pub struct Owner {
    dest_uid: Option<u32>,
    dest_gid: Option<u32>,
    bit_flag: u8,
    verbosity: Verbosity,
    filter: IfFrom,
    files: Vec<String>,
    recurse: bool,
    preserve_root: bool,
    dereference: bool
}

impl Owner {
    fn exec(&self) -> i32 {
        let mut ret = 0;
        for f in &self.files {
            ret |= self.traverse(f);
        }
        ret
    }

    fn chown<P: AsRef<Path>>(
        &self,
        path: P,
        duid: uid_t,
        dgid: gid_t,
        follow: bool
    ) -> io::Result<()> {
        let path = path.as_ref();
        // ...
    }
}

#[derive(PartialEq, Debug)]
pub struct Options {
    pub files: Vec<String>,

    pub from: String,
    pub reference: String,

    pub verbosity: Verbosity,
    pub dereference: bool,        // --dereference or -h | --no-dereference
    
    pub no_preserve_root: bool,   // --no-preserve-root
    pub recurse: bool,            // -R | --recursive
    pub traverse_it: bool,        // -H
    pub traverse_all: bool,       // -L
    pub traverse_none: bool,      // -P
}
