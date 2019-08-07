#![allow(unused_imports)]
extern crate term_grid;
extern crate termsize;
extern crate time;
extern crate unicode_width;
extern crate number_prefix;
extern crate libc;
#[cfg(unix)]
#[macro_use]
extern crate lazy_static;

use number_prefix::{Standalone, Prefixed, decimal_prefix};
use term_grid::{Cell, Direction, Filling, Grid, GridOptions};
use time::{strftime, Timespec};
#[cfg(unix)]
use libc::{mode_t, S_ISGID, S_ISUID, S_ISVTX, S_IWOTH, S_IXGRP, S_IXOTH, S_IXUSR};

use std::{io, fs};
use std::time::UNIX_EPOCH;
use std::fs::{DirEntry, FileType, Metadata};
use std::path::{Path, PathBuf};
use std::cmp::Reverse;
#[cfg(unix)]
use std::collections::HashMap;

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

#[cfg(any(unix, target_os = "redox"))]
use std::os::unix::fs::MetadataExt;
#[cfg(unix)]
use std::os::unix::fs::FileTypeExt;
#[cfg(unix)]
use unicode_width::UnicodeWidthStr;

pub struct Options {
    pub dirs: Vec<String>,   // "required" arg, comes with no option
    
    pub show_hidden: bool,        // -a | --all
    pub ignore_implied: bool,     // -A | --almost-all
    pub dirs_themselves: bool,    // -d | --directory
    pub long_listing: bool,       // -l | --long
    pub dereference: bool,        // -L | --dereference
    pub reverse: bool,            // -r | --reverse
    pub recurse: bool,            // -R | --recursive

    pub sort_by_mtime: bool,      // -t (sort by modification time)
    pub sort_by_ctime: bool,      // -c (sort by change time)
    pub sort_by_size: bool,       // -S (sort by file size)
    pub no_sort: bool,            // -U (don't sort at all)
    pub ignore_backups: bool,     // -B (ignore files with '~')

    pub numeric_ids: bool,        // -n (user and group IDs)
    pub one_file_per_line: bool,  // -1
    pub human_readable: bool,     // -h | --human-readable
    pub classify: bool,           // -F | --classify

    pub color: bool,            // --color
}

#[cfg(unix)]
static DEFAULT_COLORS: &str = "rs=0:di=01;34:ln=01;36:mh=00:pi=40;33:so=01;35:bd=40;33;01:cd=40;33;01:or=40;31;01:mi=00:su=37;41:sg=30;43:ca=30;41:tw=30;42:ow=34;42:st=37;44:ex=01;32:*.tar=01;31:*.tgz=01;31:*.arc=01;31:*.arj=01;31:*.taz=01;31:*.lha=01;31:*.lz4=01;31:*.lzh=01;31:*.lzma=01;31:*.tlz=01;31";

#[cfg(unix)]
lazy_static! {
    static ref LS_COLORS: String = std::env::var("LS_COLORS").unwrap_or(DEFAULT_COLORS.to_string());
    static ref COLOR_MAP: HashMap<&'static str, &'static str> = {
        let codes = LS_COLORS.split(":");
        let mut map = HashMap::new();
        for c in codes {
            let p: Vec<_> = c.split("=").collect();
            if p.len() == 2 {
                map.insert(p[0], p[1]);
            }
        }
        map
    };
    static ref RESET_CODE: &'static str = COLOR_MAP.get("rs").unwrap_or(&"0");
    static ref LEFT_CODE: &'static str = COLOR_MAP.get("lc").unwrap_or(&"\x1b[");
    static ref RIGHT_CODE: &'static str = COLOR_MAP.get("rc").unwrap_or(&"m");
    static ref END_CODE: &'static str = COLOR_MAP.get("ec").unwrap_or(&"");
}

#[cfg(unix)]
macro_rules! has {
    ($mode:expr, $perm:expr) => {
        $mode & ($perm) != 0
    }
}

#[cfg(not(unix))]
#[allow(unused_variables)]
pub fn display_permissions(metadata: &fs::Metadata) -> String {
    String::from("---------")
}

#[cfg(unix)]
pub fn display_permissions(metadata: &fs::Metadata) -> String {
    let mode: mode_t = metadata.mode() as mode_t;
    display_permissions_unix(mode as u32)
}

/* This function is taken from the uucore crate
 * (c) Joseph Crail and (c) Jian Zeng */
#[cfg(unix)]
pub fn display_permissions_unix(mode: u32) -> String {
    let mut result: String = String::with_capacity(9);
    result.push(if has!(mode, libc::S_IRUSR) { 'r' } else { '-' });
    result.push(if has!(mode, libc::S_IWUSR) { 'w' } else { '-' });
    result.push(if has!(mode, libc::S_ISUID) {
        if has!(mode, libc::S_IXUSR) {
            's'
        } else {
            'S'
        }
    } else if has!(mode, libc::S_IXUSR) {
        'x'
    } else {
        '-'
    });

    result.push(if has!(mode, libc::S_IRGRP) { 'r' } else { '-' });
    result.push(if has!(mode, libc::S_IWGRP) { 'w' } else { '-' });
    result.push(if has!(mode, libc::S_ISGID) {
        if has!(mode, libc::S_IXGRP) {
            's'
        } else {
            'S'
        }
    } else if has!(mode, libc::S_IXGRP) {
        'x'
    } else {
        '-'
    });

    result.push(if has!(mode, libc::S_IROTH) { 'r' } else { '-' });
    result.push(if has!(mode, libc::S_IWOTH) { 'w' } else { '-' });
    result.push(if has!(mode, libc::S_ISVTX) {
        if has!(mode, libc::S_IXOTH) {
            't'
        } else {
            'T'
        }
    } else if has!(mode, libc::S_IXOTH) {
        'x'
    } else {
        '-'
    });

    result
}

pub fn list(options: Options) {
    let locs: Vec<String> = if options.dirs[0] == "." {
        vec![String::from(".")]
    } else {
        options.dirs.iter().cloned().collect()
    };

    let mut sfiles = Vec::<PathBuf>::new();
    let mut sdirs = Vec::<PathBuf>::new();
    for loc in locs {
        let p = PathBuf::from(&loc);
        let mut dir = false;

        if p.is_dir() && !options.dirs_themselves {
            dir = true;
            if options.long_listing && !(options.dereference) {
                if let Ok(md) = p.symlink_metadata() {
                    if md.file_type().is_symlink() && !p.ends_with("/") {
                        dir = false;
                    }
                }
            }
        }
        if dir {
            sdirs.push(p);
        } else {
            sfiles.push(p);
        }
    }
    sort_entries(&mut sfiles, &options);
    display_items(&sfiles, None, &options);

    sort_entries(&mut sdirs, &options);
    for d in sdirs {
        if options.dirs.len() > 1 {
            println!("\n{}:", d.to_string_lossy());
        }
        enter_directory(&d, &options);
    }
}

#[cfg(unix)]
pub fn sort_entries(entries: &mut Vec<PathBuf>, options: &Options) {
    let mut rev = options.reverse;
    if options.sort_by_mtime {
        if options.sort_by_ctime {
            entries.sort_by_key(|k| {
                Reverse(
                    get_metadata(k, &options).map(|md| md.ctime()).unwrap_or(0)
                )
            });
        } else {
            entries.sort_by_key(|k| {
                Reverse(
                    get_metadata(k, &options).and_then(|md| md.modified())
                        .unwrap_or(UNIX_EPOCH)
                )
            })
        }
    } else if options.sort_by_size {
        entries.sort_by_key(|k| {
            get_metadata(k, &options).map(|md| md.size()).unwrap_or(0)
        });
        rev = !rev;
    } else if options.no_sort {
        entries.sort();
    }

    if rev {
        entries.reverse();
    }
}

#[cfg(windows)]
pub fn sort_entries(entries: &mut Vec<PathBuf>, options: &Options) {
    let mut rev = options.reverse;
    if options.sort_by_mtime {
        entries.sort_by_key(|k| {
            Reverse(
                get_metadata(k, &options).and_then(|md| md.modified())
                    .unwrap_or(UNIX_EPOCH)
            )
        });
    } else if options.sort_by_ctime {
        entries.sort_by_key(|k| {
            get_metadata(k, &options).map(|md| md.file_size()).unwrap_or(0)
        });
        rev = !rev;
    } else if !options.no_sort {
        entries.sort();
    }

    if rev {
        entries.reverse();
    }
}

pub fn max(l: usize, r: usize) -> usize {
    if l > r { l } else { r }
}

pub fn should_display(entry: &DirEntry, options: &Options) -> bool {
    let ffi_name = entry.file_name();
    let name = ffi_name.to_string_lossy();
    if !options.show_hidden && !options.ignore_implied {
        if name.starts_with('.') {
            return false;
        }
    }
    if options.ignore_backups && name.ends_with('~') {
        return false;
    }
    return true;
}

pub fn enter_directory(dir: &PathBuf, options: &Options) {

}

pub fn get_metadata(entry: &PathBuf, options: &Options) -> io::Result<Metadata> {

}

pub fn display_dir_entry_size(entry: &PathBuf, options: &Options) -> (usize, usize) {
    if let Ok(md) = get_metadata(entry, options) {
        (
            display_symlink_count(&md).len(),
            display_file_size(&md, &options).len()
        )
    } else {
        (0, 0)
    }
}

pub fn pad_left(string: String, count: usize) -> String {
    if count > string.len() {
        let pad = count - string.len();
        let pad = String::from_utf8(vec![' ' as u8; pad]).unwrap();
        format!("{}{}", pad, string)
    } else {
        string
    }
}

pub fn display_items(items: &Vec<PathBuf>, strip: Option<&Path>, options: Options) {
    if options.long_listing || options.numeric_ids {
        let (mut max_links, mut max_size) = (1, 1);
        for i in items {
            let (links, size) = display_dir_entry_size(i, &options);
            max_links = max(links, max_links);
            max_size = max(size, max_size);
        }
        for i in items {
            display_item_long(i, strip, max_links, max_size, &options);
        }
    } else {
        if !options.one_file_per_line {
            let names = items.iter().filter_map(|i| {
                let m = get_metadata(i, &options);
                match m {
                    Err(e) => {
                        let filename = get_file_name(i, strip);
                        show_error!("{}: {}", filename, e);
                        None
                    }
                    Ok(m) => {
                        Some(display_file_name(&i, strip, &m, &options))
                    }
                }
            });

            if let Some(size) = termsize::get() {
                let mut grid = Grid::new(GridOptions {
                    filling: Filling::Spaces(2),
                    direction: Direction::TopToBottom
                });

                for n in names {
                    grid.add(n);
                }

                if let Some(output) = grid.fit_into_width(size.cols as usize) {
                    print!("{}", output);
                    return;
                }
            }
        }

        /* couldn't display a grid */
        for i in items {
            let m = get_metadata(i, &options);
            if let Ok(m) = m {
                println!("{}", display_file_name(&i, strip, &md, &options).contents);
            }
        }
    }
}

pub fn display_item_long(
    item: &PathBuf,
    strip: Option<&Path>,
    max_links: usize,
    max_size: usize,
    options: &Options
) {
    let m = match get_metadata(item, options) {
        Err(e) => {
            let filename = get_file_name(&item, strip);
            show_error!("{}: {}", filename, e);
            return;
        },
        Ok(m) => m
    };

    // todo
}

