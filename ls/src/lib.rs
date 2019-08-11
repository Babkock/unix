/* 
 * ls/lib.rs
 * Babkock/unix
 * 
 * Copyright (c) 2019 Tanner Babcock.
 * MIT License.
*/
//!
//! # **`ls`**`
//!
//! List all files in the specified directory. Uses the current directory (".")
//! if no path is specified.
//!
//! ```rust
//! extern crate ls;
//! ```
//!
//! This command-line example uses four options: long listing, human-readable, all, and classify.
//!
//! ```
//! $ ls -lhaF
//! drwxr-xr-x 3 user user  4.10K 2019-08-09 01:01 ./
//! drwxr-xr-x 4 user user  4.10K 2019-08-06 16:58 ../
//! -rw-r--r-- 1 user user 11.50K 2019-08-08 22:54 display.rs
//! -rw-r--r-- 1 user user  2.67K 2019-08-08 22:57 file.rs
//! -rw-r--r-- 1 user user  5.10K 2019-08-08 22:59 group.rs
//! -rw-r--r-- 1 user user  6.49K 2019-08-08 22:55 lib.rs
//! drwxr-xr-x 2 user user  4.10K 2019-08-08 04:04 bin/
//! ```
//!
extern crate term_grid;
extern crate termsize;
extern crate time;
extern crate unicode_width;
extern crate number_prefix;
extern crate libc;
#[cfg(unix)]
#[macro_use]
extern crate lazy_static;

use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use std::cmp::Reverse;
#[cfg(unix)]
use std::collections::HashMap;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

/// Big struct of all options for ls, including the specified
/// directories themselves.
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
    pub inode: bool,              // -i | --inode

    pub color: bool,              // --color
}

mod display;
mod file;
mod group;

#[cfg(unix)]
static DEFAULT_COLORS: &str = "dir=01;94:no=00:fi=00:di=01;34:ln=01;36:pi=40;33:so=01;35:do=01;35:bd=40;33;01:cd=40;33;01:or=40;31;01:mi=01;05;37;41:su=37;41:sg=30;43:ca=30;41:tw=30;42:ow=34;42:st=37;44:ex=01;32:*.tar=01;31:*.tgz=01;31:*.svgz=01;31:*.arj=01;31:*.taz=01;31:*.lzh=01;31:*.lzma=01;31:*.zip=01;31:*.z=01;31:*.Z=01;31:*.dz=01;31:*.gz=01;31:*.bz2=01;31:*.tbz2=01;31:*.bz=01;31:*.tz=01;31:*.deb=01;31:*.rpm=01;31:*.jar=01;31:*.rar=01;31:*.ace=01;31:*.zoo=01;31:*.cpio=01;31:*.7z=01;31:*.rz=01;31:*.jpg=01;35:*.jpeg=01;35:*.gif=01;35:*.bmp=01;35:*.pbm=01;35:*.pgm=01;35:*.ppm=01;35:*.tga=01;35:*.xbm=01;35:*.xpm=01;35:*.tif=01;35:*.tiff=01;35:*.png=01;35:*.mng=01;35:*.pcx=01;35:*.mov=01;35:*.mpg=01;35:*.mpeg=01;35:*.m2v=01;35:*.ogm=01;35:*.mp4=01;35:*.m4v=01;35:*.mp4v=01;35:*.vob=01;35:*.qt=01;35:*.nuv=01;35:*.wmv=01;35:*.asf=01;35:*.rm=01;35:*.rmvb=01;35:*.flc=01;35:*.avi=01;35:*.fli=01;35:*.gl=01;35:*.dl=01;35:*.xcf=01;35:*.xwd=01;35:*.yuv=01;35:*.svg=01;35:*.aac=00;36:*.au=00;36:*.flac=00;36:*.mid=00;36:*.midi=00;36:*.mka=00;36:*.mp3=00;36:*.mpc=00;36:*.ogg=00;36:*.ra=00;36:*.wav=00;36:*.mkv=1;31:*.conf=1;93:*.d=0;33;40:*.rlib=0;33;40:*.txt=1;93:*.log=1;93:*.php=1;31;40:*.js=1;32;40:*.bin=1;32;40:*.asm=1;31;40:*.json=1;93:*.html=0;35;40:*.xml=0;35;40:*.yaml=0;35;40:*.toml=0;35;40:*.shtml=0;35;40:*.ini=1;33:*.sh=1;32;40:*.lua=1;32:*.css=0;36;40:*.scss=0;36;40:*.less=0;36;40:*.c=1;93:*.h=1;31:*.cpp=1;32;40:*.rs=1;31:*.rb=1;31:*.py=1;31;40:*.pl=1;32;40:*.md=1;93:*.rtf=1;93;40:*.o=0;33;40:*.so=0;33;40:*.lock=1;93:*.yml=0;35;40";

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
        // make directories always blue. if this bothers you just delete the next line
        map.insert("dir", "01;94");
        map
    };
    static ref RESET_CODE: &'static str = COLOR_MAP.get("rs").unwrap_or(&"0");
    static ref LEFT_CODE: &'static str = COLOR_MAP.get("lc").unwrap_or(&"\x1b[");
    static ref RIGHT_CODE: &'static str = COLOR_MAP.get("rc").unwrap_or(&"m");
    static ref END_CODE: &'static str = COLOR_MAP.get("ec").unwrap_or(&"");
}


/// The initial list function. Takes ownership of an **Options** struct, and allows
/// its called functions to borrow it. Prints all entries in all specified dirs to standard out.
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
    display::display_items(&sfiles, None, &options);

    sort_entries(&mut sdirs, &options);
    for d in sdirs {
        if options.dirs.len() > 1 {
            println!("\n{}:", d.to_string_lossy());
        }
        file::enter_directory(&d, &options);
    }
}

/// Sorts the directory *entries* (a vector of paths) given the **Options** specified in *options*.
/// Can sort by modified time, change time, file size, or can default to no sorting. Reverses if
/// specified.
#[cfg(unix)]
pub fn sort_entries(entries: &mut Vec<PathBuf>, options: &Options) {
    let mut rev = options.reverse;
    if options.sort_by_mtime {
        if options.sort_by_ctime {
            entries.sort_by_key(|k| {
                Reverse(
                    file::get_metadata(k, options).map(|md| md.ctime()).unwrap_or(0)
                )
            });
        } else {
            entries.sort_by_key(|k| {
                Reverse(
                    file::get_metadata(k, options).and_then(|md| md.modified())
                        .unwrap_or(UNIX_EPOCH)
                )
            })
        }
    } else if options.sort_by_size {
        entries.sort_by_key(|k| {
            file::get_metadata(k, options).map(|md| md.size()).unwrap_or(0)
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
                file::get_metadata(k, options).and_then(|md| md.modified())
                    .unwrap_or(UNIX_EPOCH)
            )
        });
    } else if options.sort_by_ctime {
        entries.sort_by_key(|k| {
            file::get_metadata(k, options).map(|md| md.file_size()).unwrap_or(0)
        });
        rev = !rev;
    } else if !options.no_sort {
        entries.sort();
    }

    if rev {
        entries.reverse();
    }
}

/// Compares *l* to *r*, two usizes.
pub fn max(l: usize, r: usize) -> usize {
    if l > r { l } else { r }
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

/// Parse colors.
#[cfg(unix)]
pub fn color_name(name: String, typ: &str) -> String {
    let mut typ = typ;
    if !COLOR_MAP.contains_key(typ) {
        if typ == "or" {
            typ = "ln";
        } else if typ == "mi" {
            typ = "fi";
        }
    };
    if let Some(code) = COLOR_MAP.get(typ) {
        format!(
            "{}{}{}{}{}{}{}{}",
            *LEFT_CODE,
            code,
            *RIGHT_CODE,
            name,
            *END_CODE,
            *LEFT_CODE,
            *RESET_CODE,
            *RIGHT_CODE
        )
    } else {
        name
    }
}

