/*
 * chmod/main.rs
 * Babkock/unix
 *
 * Copyright (c) 2019 Tanner Babcock.
 * MIT License.
*/
extern crate chown;
extern crate clap;

use clap::{Arg, App};
use chown::{Verbosity, Owner, IfFrom, parse_spec, FTS_COMFOLLOW, FTS_PHYSICAL, FTS_LOGICAL};
use std::{io, fs};
use std::io::{Error, ErrorKind};
use std::os::unix::fs::MetadataExt;

fn main() -> io::Result<()> {
    let matches = App::new("chown").about("Change the file owner and group")
        .template("{bin} - {about}\nUSAGE:\n\t{bin} [OPTION]... [OWNER][:[GROUP]] FILE...\n\t{bin} [OPTION]... --reference=RFILE FILE...\n\nFLAGS:\n\n{flags}\n\nOPTIONS:\n\n{options}\n")
        // this first argument, the "user:group" string, will get passed to parse_spec
        .arg(Arg::with_name("spec")
             .help("Specification string in format OWNER:GROUP")
             .required(false)
             .index(1)
             .multiple(false))
        .arg(Arg::with_name("FILE")
             .help("File or directory to change ownership for")
             .required(false)
             .index(2)
             .multiple(true))
        .arg(Arg::with_name("changes")
             .short("c")
             .long("changes")
             .help("Verbosity: only report when a change is made")
             .takes_value(false))
        .arg(Arg::with_name("silent")
             .short("f")
             .long("silent")
             .help("Verbosity: silent")
             .takes_value(false))
        .arg(Arg::with_name("verbose")
             .short("v")
             .long("verbose")
             .help("Verbosity: output a diagnostic for every file processed")
             .takes_value(false))
        .arg(Arg::with_name("dereference")
             .short("")
             .long("dereference")
             .help("Affect the refferer of each symbolic link")
             .takes_value(false))
        .arg(Arg::with_name("no-dereference")
             .short("h")
             .long("no-dereference")
             .help("Affect the symbolic links instead of any referenced file")
             .takes_value(false))
        .arg(Arg::with_name("from")
             .short("")
             .long("from")
             .help("Change the owner and/or group of each file only if its current owner and/or group match those specified. Either may be omitted, in which case a match is not required for the omitted attribute")
             .value_name("CURRENT_OWNER:CURRENT_GROUP")
             .takes_value(true))
        .arg(Arg::with_name("reference")
             .short("")
             .long("reference")
             .help("Use RFILE's owner and group rather than specifying OWNER:GROUP values")
             .value_name("RFILE")
             .takes_value(true))
        .arg(Arg::with_name("no-preserve-root")
             .short("")
             .long("no-preserve-root")
             .help("Do not treat '/' any different (default)")
             .takes_value(false))
        .arg(Arg::with_name("preserve-root")
             .short("")
             .long("preserve-root")
             .help("Fail to operate recursively on '/'")
             .takes_value(false))
        .arg(Arg::with_name("recursive")
             .short("R")
             .long("recursive")
             .help("Operate on files and directories recursively")
             .takes_value(false))
        .arg(Arg::with_name("traverse")
             .short("H")
             .long("traverse")
             .help("If a command line argument is a symbolic link to a directory, traverse it")
             .takes_value(false))
        .arg(Arg::with_name("traverse-all")
             .short("L")
             .long("traverse-all")
             .help("Traverse every symbolic link to a directory encountered")
             .takes_value(false))
        .arg(Arg::with_name("no-traverse")
             .short("P")
             .long("no-traverse")
             .help("Do not traverse any symbolic links (default)")
             .takes_value(false))
        .get_matches();

    let mut files: Vec<String> = Vec::new();

    let mut bit_flag: u8;
    let preserve_root: bool = if matches.occurrences_of("no-preserve-root") != 0 {
        false
    } else if matches.occurrences_of("preserve-root") != 0 {
        true
    } else {
        false
    };
    let mut derefer: i8 = if matches.occurrences_of("dereference") != 0 {
        1
    } else if matches.occurrences_of("no-dereference") != 0 {
        0
    } else {
        -1
    };
    //let flags: &[char] = &['H', 'L', 'P'];

    bit_flag = if matches.occurrences_of("traverse") != 0 {
        FTS_COMFOLLOW | FTS_PHYSICAL
    } else if matches.occurrences_of("traverse-all") != 0 {
        FTS_LOGICAL
    } else if matches.occurrences_of("no-traverse") != 0 {
        FTS_PHYSICAL
    } else {
        FTS_PHYSICAL
    };

    let recurse: bool = if matches.occurrences_of("recursive") != 0 {
        true
    } else {
        false
    };
    if recurse {
        if bit_flag == FTS_PHYSICAL {
            if derefer == 1 {
                return Err(Error::new(ErrorKind::Other, "-R --dereference requires -H or -L"));
            }
            derefer = 0;
        }
    } else {
        bit_flag = FTS_PHYSICAL;
    }

    if matches.occurrences_of("spec") != 0 && matches.occurrences_of("FILE") == 0 {
        if matches.occurrences_of("reference") == 0 {
            return Err(Error::new(ErrorKind::Other, "Please specify an OWNER:GROUP argument or a reference"));
        }
    }
    else if matches.occurrences_of("spec") != 0 && matches.occurrences_of("FILE") != 0 {
        match matches.value_of("FILE") {
            None => {
                files.push(matches.value_of("spec").unwrap().to_string());
            },
            Some(n) => {
                files.push(n.to_string());
            }
        }
    }
    else {
        return Err(Error::new(ErrorKind::Other, "Missing operand - Try with --help for more information."));
    }
    
    //println!("yay");

    let verbosity: Verbosity = if matches.occurrences_of("changes") != 0 {
        Verbosity::Changes
    } else if matches.occurrences_of("silent") != 0 || matches.occurrences_of("quiet") != 0 {
        Verbosity::Silent
    } else if matches.occurrences_of("verbose") != 0 {
        Verbosity::Verbose
    } else {
        Verbosity::Normal
    };

    let filter = if let Some(spec) = matches.value_of("from") {
        match parse_spec(&spec) {
            Ok((Some(uid), None)) => IfFrom::User(uid),
            Ok((None, Some(gid))) => IfFrom::Group(gid),
            Ok((Some(uid), Some(gid))) => IfFrom::UserGroup(uid, gid),
            Ok((None, None)) => IfFrom::All,
            Err(e) => {
                return Err(Error::new(ErrorKind::Other, e));
            }
        }
    }
    else {
        IfFrom::All
    };

    let dest_uid: Option<u32>;
    let dest_gid: Option<u32>;
    if let Some(file) = matches.value_of("reference") {
        match fs::metadata(&file) {
            Ok(meta) => {
                dest_gid = Some(meta.gid());
                dest_uid = Some(meta.uid());
            },
            Err(e) => {
                return Err(Error::new(ErrorKind::Other, format!("failed to get attributes of {}: {}", file, e)));
            }
        }
        // in the other, this is files = matches.free;
        match matches.value_of("spec") {
            None => {
                return Err(Error::new(ErrorKind::Other, "No file supplied for changing"));
            },
            Some(n) => {
                files.push(n.to_string());
            }
        }
    } else {
        match parse_spec(&matches.value_of("spec").unwrap()) {
            Ok((u, g)) => {
                dest_uid = u;
                dest_gid = g;
            }
            Err(e) => {
                return Err(Error::new(ErrorKind::Other, e));
            }
        }
    }
    let executor: Owner = Owner {
        bit_flag,
        dest_uid,
        dest_gid,
        verbosity,
        recurse,
        dereference: derefer != 0,
        filter,
        preserve_root,
        files
    };
    executor.exec();

    Ok(())
}

