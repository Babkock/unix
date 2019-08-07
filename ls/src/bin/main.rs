#![allow(unused_imports)]
extern crate ls;
extern crate clap;

use clap::{Arg, App};
use ls::*;
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
             .long("")
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
             .long("")
             .help("Sort by file size, largest first")
             .takes_value(false))
        .arg(Arg::with_name("sort-by-mtime")
             .short("t")
             .long("")
             .help("Sort by modification time, newest first")
             .takes_value(false))
        .arg(Arg::with_name("do-not-sort")
             .short("U")
             .long("")
             .help("Do not sort; list files in the directory order")
             .takes_value(false))
        .arg(Arg::with_name("color")
             .short("")
             .long("color")
             .help("Color output based on file type")
             .required(false)
             .takes_value(true))
        .get_matches();

    let show_hidden: bool = match matches.occurrences_of("a") {
        0 => false,
        1 => true,
        _ => false
    };
    let ignore_implied: bool = match matches.occurrences_of("A") {
        0 => false,
        1 => true,
        _ => false
    };
    let dirs_themselves: bool = match matches.occurrences_of("d") {
        0 => false,
        1 => true,
        _ => false
    };
    let long_listing: bool = match matches.occurrences_of("l") {
        0 => false,
        1 => true,
        _ => false
    };
    let dereference: bool = match matches.occurrences_of("L") {
        0 => false,
        1 => true,
        _ => false
    };
    let reverse: bool = match matches.occurrences_of("r") {
        0 => false,
        1 => true,
        _ => false
    };
    let recurse: bool = match matches.occurrences_of("R") {
        0 => false,
        1 => true,
        _ => false
    };
    let sort_by_mtime: bool = match matches.occurrences_of("t") {
        0 => false,
        1 => true,
        _ => false
    };
    let sort_by_ctime: bool = match matches.occurrences_of("c") {
        0 => false,
        1 => true,
        _ => false
    };
    let sort_by_size: bool = match matches.occurrences_of("S") {
        0 => false,
        1 => true,
        _ => false
    };
    let no_sort: bool = match matches.occurrences_of("U") {
        0 => false,
        1 => true,
        _ => false
    };
    let ignore_backups: bool = match matches.occurrences_of("B") {
        0 => false,
        1 => true,
        _ => false
    };
    let numeric_ids: bool = match matches.occurrences_of("n") {
        0 => false,
        1 => true,
        _ => false
    };
    let one_file_per_line: bool = match matches.occurrences_of("1") {
        0 => false,
        1 => true,
        _ => false
    };
    let human_readable: bool = match matches.occurrences_of("h") {
        0 => false,
        1 => true,
        _ => false
    };
    let classify: bool = match matches.occurrences_of("F") {
        0 => false,
        1 => true,
        _ => false
    };
    let color: bool = match matches.occurrences_of("color") {
        0 => false,
        1 => true,
        _ => false
    };

    let mut dirs: Vec<&str> = Vec::new();

    match matches.value_of("DIRECTORY") {
        None => {
            dirs.push(".");
        },
        Some(n) => {
            dirs = n.collect();
        }
    };

    let options: Options = Options {
        dirs,
        show_hidden,
        ignore_implied,
        dirs_themselves,
        long_listing,
        dereference,
        reverse,
        recurse,

        sort_by_mtime,
        sort_by_ctime,
        sort_by_size,
        no_sort,
        ignore_backups,

        numeric_ids,
        one_file_per_line,
        human_readable,
        classify,
        color
    };

    list(options);
    Ok(())
}

