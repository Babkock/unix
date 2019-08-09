/*
 * ls/main.rs
 * Babkock/unix
 *
 * Copyright (c) 2019 Tanner Babcock.
 * MIT License.
*/
#![allow(unused_imports)]
extern crate ls;
extern crate clap;

use clap::{Arg, App};
use ls::{Options, list};
use std::io;

fn main() -> io::Result<()> {
    let matches = App::new("ls").about("List information about the FILEs (the current directory by default)\nSort entries alphabetically if none of -cftuvSUX nor --sort is specified.")
        .arg(Arg::with_name("DIRECTORY")
             .help("The directory to list, current directory by default")
             .required(false)
             .index(1)
             .multiple(true))
        .arg(Arg::with_name("one-file-per-line")
             .short("1")
             .long("")
             .help("List one file per line")
             .takes_value(false))
        .arg(Arg::with_name("all")
             .short("a")
             .long("all")
             .help("Do not ignore hidden files (names starting with '.')")
             .takes_value(false))
        .arg(Arg::with_name("almost-all")
             .short("A")
             .long("almost-all")
             .help("Do not list implied . and .. entries")
             .takes_value(false))
        .arg(Arg::with_name("ignore-backups")
             .short("B")
             .long("ignore-backups")
             .help("Do not list implied entries ending in ~")
             .takes_value(false))
        .arg(Arg::with_name("ctime")
             .short("c")
             .long("ctime")
             .help(
                 "If the long listing format (e.g. -l, -o) is being used, print the status \
                 change time (the 'ctime' in the inode) instead of the modification time. When explicitly \
                 sorting by time (--sort=time or -t) or when not using a long listing \
                 format, sort according to the status change time."
             )
             .takes_value(false))
        .arg(Arg::with_name("directory")
             .short("d")
             .long("directory")
             .help("List the directories themselves, not their contents")
             .takes_value(false))
        .arg(Arg::with_name("classify")
             .short("F")
             .long("classify")
             .help("Append indicator to entries (*/=>@|)")
             .takes_value(false))
        .arg(Arg::with_name("human-readable")
             .short("h")
             .long("human-readable")
             .help("With -l and -s, print sizes like 2M, 100K, 4G etc.")
             .takes_value(false))
        .arg(Arg::with_name("inode")
             .short("i")
             .long("inode")
             .help("Print the index number of each file")
             .takes_value(false))
        .arg(Arg::with_name("dereference")
             .short("L")
             .long("dereference")
             .help("When showing information for a symbolic link, show info for the file the link references, rather than the link itself")
             .takes_value(false))
        .arg(Arg::with_name("long")
             .short("l")
             .long("long")
             .help("Use a long listing format")
             .takes_value(false))
        .arg(Arg::with_name("numeric-uid-gid")
             .short("n")
             .long("numeric-uid-gid")
             .help("Like -l, but list numeric user and group IDs")
             .takes_value(false))
        .arg(Arg::with_name("reverse")
             .short("r")
             .long("reverse")
             .help("Reverse order while sorting")
             .takes_value(false))
        .arg(Arg::with_name("recursive")
             .short("R")
             .long("recursive")
             .help("List subdirectories recursively")
             .takes_value(false))
        .arg(Arg::with_name("sort-by-file-size")
             .short("S")
             .long("filesize")
             .help("Sort by file size, largest first")
             .takes_value(false))
        .arg(Arg::with_name("sort-by-mtime")
             .short("t")
             .long("mtime")
             .help("Sort by modification time, newest first")
             .takes_value(false))
        .arg(Arg::with_name("do-not-sort")
             .short("U")
             .long("none")
             .help("Do not sort; list files in the directory order")
             .takes_value(false))
        .arg(Arg::with_name("color")
             .short("")
             .long("color")
             .help("Color output based on file type")
             .required(false)
             .takes_value(true))
        .get_matches();

    let mut dirs: Vec<String> = Vec::new();

    match matches.value_of("DIRECTORY") {
        None => {
            dirs.push(".".to_string());
        },
        Some(n) => {
            dirs.push(n.to_string());
        }
    };

    let options: Options = Options {
        dirs,
        show_hidden: matches.occurrences_of("all") != 0,
        ignore_implied: matches.occurrences_of("almost-all") != 0,
        dirs_themselves: matches.occurrences_of("directory") != 0,
        long_listing: matches.occurrences_of("long") != 0,
        dereference: matches.occurrences_of("dereference") != 0,
        reverse: matches.occurrences_of("reverse") != 0,
        recurse: matches.occurrences_of("recursive") != 0,

        sort_by_mtime: matches.occurrences_of("sort-by-mtime") != 0,
        sort_by_ctime: matches.occurrences_of("ctime") != 0,
        sort_by_size: matches.occurrences_of("sort-by-file-size") != 0,
        no_sort: matches.occurrences_of("do-not-sort") != 0,
        ignore_backups: matches.occurrences_of("ignore-backups") != 0,

        numeric_ids: matches.occurrences_of("numeric-uid-gid") != 0,
        one_file_per_line: matches.occurrences_of("one-file-per-line") != 0,
        human_readable: matches.occurrences_of("human-readable") != 0,
        classify: matches.occurrences_of("classify") != 0,
        inode: matches.occurrences_of("inode") != 0,
        color: matches.occurrences_of("color") != 0
    };

    list(options);
    Ok(())
}

