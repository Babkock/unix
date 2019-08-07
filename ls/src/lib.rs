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

use std::fs;
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

}
