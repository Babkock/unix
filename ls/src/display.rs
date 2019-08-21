/*
 * ls/display.rs
 * Babkock/unix
 *
 * Copyright (c) 2019 Tanner Babcock.
 * MIT License.
*/
extern crate libc;
extern crate term_grid;
extern crate termsize;
extern crate time;
extern crate unicode_width;
extern crate number_prefix;

use number_prefix::{Standalone, Prefixed, decimal_prefix};
use term_grid::{Cell, Direction, Filling, Grid, GridOptions};
use time::{strftime, Timespec};

use std::fs::{DirEntry, FileType, Metadata};
use std::path::{Path, PathBuf};
use crate::{max, Options, pad_left, color_name};
use crate::file::*;
use crate::group::*;

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
#[cfg(any(unix, target_os ="redox"))]
use std::os::unix::fs::MetadataExt;
#[cfg(unix)]
use std::os::unix::fs::FileTypeExt;
#[cfg(unix)]
use unicode_width::UnicodeWidthStr;

#[cfg(unix)]
use libc::{mode_t, S_ISGID, S_ISUID, S_IRUSR, S_IWUSR, S_ISVTX, S_IROTH, S_IRGRP, S_IWOTH, S_IWGRP, S_IXGRP, S_IXOTH, S_IXUSR};

#[cfg(unix)]
macro_rules! has {
    ($mode:expr, $perm:expr) => (
        $mode & ($perm) != 0
    )
}

#[cfg(not(unix))]
#[allow(unused_variables)]
pub fn display_permissions(metadata: &Metadata) -> String {
    String::from("---------")
}

/// Wrapper for display_permissions_unix in unix, prints string of hyphens on Windows
#[cfg(unix)]
pub fn display_permissions(metadata: &Metadata) -> String {
    let mode: mode_t = metadata.mode() as mode_t;
    display_permissions_unix(mode as u32)
}

/// Interpret file and group permissions given the 32-bit *mode*. Returns
/// the formatted string.
#[cfg(unix)]
pub fn display_permissions_unix(mode: u32) -> String {
    let mut result: String = String::with_capacity(9);
    result.push(if has!(mode, S_IRUSR) { 'r' } else { '-' });
    result.push(if has!(mode, S_IWUSR) { 'w' } else { '-' });
    result.push(if has!(mode, S_ISUID) {
        if has!(mode, S_IXUSR) {
            's'
        } else {
            'S'
        }
    } else if has!(mode, S_IXUSR) {
        'x'
    } else {
        '-'
    });

    result.push(if has!(mode, S_IRGRP) { 'r' } else { '-' });
    result.push(if has!(mode, S_IWGRP) { 'w' } else { '-' });
    result.push(if has!(mode, S_ISGID) {
        if has!(mode, S_IXGRP) {
            's'
        } else {
            'S'
        }
    } else if has!(mode, S_IXGRP) {
        'x'
    } else {
        '-'
    });

    result.push(if has!(mode, S_IROTH) { 'r' } else { '-' });
    result.push(if has!(mode, S_IWOTH) { 'w' } else { '-' });
    result.push(if has!(mode, S_ISVTX) {
        if has!(mode, S_IXOTH) {
            't'
        } else {
            'T'
        }
    } else if has!(mode, S_IXOTH) {
        'x'
    } else {
        '-'
    });

    result
}

/// Display a collection of *items* (pointer to vector of paths), given the
/// **Options** specified in *options*.
pub fn display_items(items: &Vec<PathBuf>, strip: Option<&Path>, options: &Options) {
    if options.long_listing || options.numeric_ids {
        let (mut max_links, mut max_size) = (1, 1);
        for i in items {
            let (links, size) = display_dir_entry_size(i, options);
            max_links = max(links, max_links);
            max_size = max(size, max_size);
        }
        for i in items {
            display_item_long(i, strip, max_links, max_size, options);
        }
    } else {
        if !options.one_file_per_line {
            let names = items.iter().filter_map(|i| {
                let m = get_metadata(i, options);
                match m {
                    Err(e) => {
                        let filename = get_file_name(i, strip);
                        println!("{}: {}", filename, e);
                        None
                    }
                    Ok(m) => {
                        Some(display_file_name(&i, strip, &m, options))
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
            let m = get_metadata(i, options);
            if let Ok(m) = m {
                println!("{}", display_file_name(&i, strip, &m, options).contents);
            }
        }
    }
}

/// Display *item* in a long listing. Takes the maximum number of symbolic links in *max_links*,
/// the maximum file size in *max_size*, and the user-specified **Options** in *options*.
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
            println!("{}: {}", filename, e);
            return;
        },
        Ok(m) => m
    };

    println!(
        "{}{}{} {} {} {} {} {} {}",
        get_inode(&m, options),
        display_file_type(m.file_type()),
        display_permissions(&m),
        pad_left(display_symlink_count(&m), max_links),
        display_uname(&m, options),
        display_group(&m, options),
        pad_left(display_file_size(&m, options), max_size),
        display_date(&m, options),
        display_file_name(&item, strip, &m, options).contents
    );
}

/// Displays the name of a single file at *path*. Interprets *metadata* and recognizes
/// *options*.
#[cfg(unix)]
pub fn display_file_name(
    path: &Path,
    strip: Option<&Path>,
    metadata: &Metadata,
    options: &Options
) -> Cell {
    let mut name = get_file_name(path, strip);
    if !options.long_listing {
        name = get_inode(metadata, options) + &name;
    }
    let mut width = UnicodeWidthStr::width(&*name);

    let color = options.color;
    let classify = options.classify;
    let ext;

    if color || classify {
        let file_type = metadata.file_type();

        let (code, sym) = if file_type.is_dir() {
            ("dir", Some('/'))
        } else if file_type.is_symlink() {
            if path.exists() {
                ("ln", Some('@'))
            } else {
                ("or", Some('@'))
            }
        } else if file_type.is_socket() {
            ("so", Some('='))
        } else if file_type.is_fifo() {
            ("pi", Some('|'))
        } else if file_type.is_block_device() {
            ("bd", None)
        } else if file_type.is_char_device() {
            ("cd", None)
        } else if file_type.is_file() {
            let mode = metadata.mode() as mode_t;
            let sym = if has!(mode, S_IXUSR | S_IXGRP | S_IXOTH) {
                Some('*')
            } else {
                None
            };
            if has!(mode, S_ISUID) {
                ("su", sym)
            } else if has!(mode, S_ISGID) {
                ("sg", sym)
            } else if has!(mode, S_ISVTX) && has!(mode, S_IWOTH) {
                ("tw", sym)
            } else if has!(mode, S_ISVTX) {
                ("st", sym)
            } else if has!(mode, S_IWOTH) {
                ("ow", sym)
            } else if has!(mode, S_IXUSR | S_IXGRP | S_IXOTH) {
                ("ex", sym)
            } else if metadata.nlink() > 1 {
                ("mh", sym)
            } else if let Some(e) = path.extension() {
                ext = format!("*.{}", e.to_string_lossy());
                (ext.as_str(), None)
            } else {
                ("fi", None)
            }
        } else {
            ("", None)
        };

        if color {
            name = color_name(name, code);
        }
        if classify {
            if let Some(s) = sym {
                name.push(s);
                width += 1;
            }
        }
    }

    if options.long_listing && metadata.file_type().is_symlink() {
        if let Ok(target) = path.read_link() {
            // Don't bother updating width here because it's not used
            let code = if target.exists() { "fi" } else { "mi" };
            let target_name = color_name(target.to_string_lossy().to_string(), code);
            name.push_str(" -> ");
            name.push_str(&target_name);
        }
    }

    Cell {
        contents: name,
        width: width
    }
}

#[cfg(not(unix))]
#[allow(unused_variables)]
pub fn display_symlink_count(metadata: &Metadata) -> String {
    String::from("1")
}

#[cfg(unix)]
pub fn display_symlink_count(metadata: &Metadata) -> String {
    metadata.nlink().to_string()
}

#[cfg(not(unix))]
#[allow(unused_variables)]
pub fn display_uname(metadata: &Metadata, _options: Options) -> String {
    "somebody".to_string()
}

#[cfg(not(unix))]
pub fn display_group(metadata: &Metadata, _options: Options) -> String {
    "somegroup".to_string()
}

/// Return a formatted string for the file's date - either ctime() or mtime()
/// from *metadata*, depending on *options*.
#[cfg(unix)]
pub fn display_date(metadata: &Metadata, options: &Options) -> String {
    let secs = if options.sort_by_ctime {
        metadata.ctime()
    } else {
        metadata.mtime()
    };
    let time = time::at(Timespec::new(secs, 0));
    strftime("%F %R", &time).unwrap()
}

#[cfg(not(unix))]
#[allow(unused_variables)]
pub fn display_date(metadata: &Metadata, options: &Options) -> String {
    if let Ok(mtime) = metadata.modified() {
        let time = time::at(Timespec::new(
            mtime
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            0,
        ));
        strftime("%F %R", &time).unwrap()
    } else {
        "???".to_string()
    }
}

/// Display the file size in bytes, rounding up and printing the highest prefix 
/// if human_readable is specified in *options*.
pub fn display_file_size(metadata: &Metadata, options: &Options) -> String {
    if options.human_readable {
        match decimal_prefix(metadata.len() as f64) {
            Standalone(bytes) => bytes.to_string(),
            Prefixed(prefix, bytes) => format!("{:.2}{}", bytes, prefix).to_uppercase()
        }
    } else {
        metadata.len().to_string()
    }
}

/// Is the file a directory, a symbolic link, or a regular file?
pub fn display_file_type(file_type: FileType) -> String {
    if file_type.is_dir() {
        "d".to_string()
    } else if file_type.is_symlink() {
        "l".to_string()
    } else {
        "-".to_string()
    }
}

/// Returns the username that owns the file described in *metadata*. Returns a UID if numeric_ids
/// had been specified.
#[cfg(unix)]
pub fn display_uname(metadata: &Metadata, options: &Options) -> String {
    if options.numeric_ids {
        metadata.uid().to_string()
    } else {
        uid2usr(metadata.uid()).unwrap_or(metadata.uid().to_string())
    }
}

/// Like display_uname, but for the file's group.
#[cfg(unix)]
pub fn display_group(metadata: &Metadata, options: &Options) -> String {
    if options.numeric_ids {
        metadata.gid().to_string()
    } else {
        gid2grp(metadata.gid()).unwrap_or(metadata.gid().to_string())
    }
}

#[cfg(not(unix))]
pub fn display_file_name(
    path: &Path,
    strip: Option<&Path>,
    metadata: &Metadata,
    options: &Options
) -> Cell {
    let mut name = get_file_name(path, strip);

    if !options.long_listing {
        name = get_inode(metadata, options) + &name;
    }

    if options.classify {
        let file_type = metadata.file_type();
        if file_type.is_dir() {
            name.push('/');
        } else if file_type.is_symlink() {
            name.push('@');
        }
    }

    if options.long_listing && metadata.file_type().is_symlink() {
        if let Ok(target) = path.read_link() {
            let target_name = target.to_string_lossy().to_string();
            name.push_str(" -> ");
            name.push_str(&target_name);
        }
    }

    name.into()
}

/// Decides if a particular directory entry should be displayed or not.
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

