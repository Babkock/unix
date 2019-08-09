extern crate libc;

use std::{io, fs, ptr, process};
use std::time::UNIX_EPOCH;
use std::fs::{DirEntry, FileType, Metadata};
use std::path::{Path, PathBuf};
use std::cmp::Reverse;
use std::ffi::{CStr, CString};
use crate::display::*;
use crate::Options;
use crate::sort_entries;

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
#[cfg(any(unix, target_os = "redox"))]
use std::os::unix::fs::MetadataExt;
#[cfg(unix)]
use std::os::unix::fs::FileTypeExt;

macro_rules! safe_unwrap(
    ($exp:expr) => (
        match $exp {
            Ok(m) => m,
            Err(f) => {
                println!("{}", f.to_string());
                process::exit(2);
            }
        }
    )
);

pub fn enter_directory(dir: &PathBuf, options: &Options) {
    let mut entries =
        safe_unwrap!(fs::read_dir(dir).and_then(|e| e.collect::<Result<Vec<_>, _>>()));

    entries.retain(|e| should_display(e, &options));

    let mut entries: Vec<_> = entries.iter().map(DirEntry::path).collect();
    sort_entries(&mut entries, &options);

    if options.show_hidden {
        let mut display_entries = entries.clone();
        display_entries.insert(0, dir.join(".."));
        display_entries.insert(0, dir.join("."));
        display_items(&display_entries, Some(dir), &options);
    } else {
        display_items(&entries, Some(dir), &options);
    }

    if options.recurse {
        for e in entries.iter().filter(|p| p.is_dir()) {
            println!("\n{}:", e.to_string_lossy());
            enter_directory(&e, options);
        }
    }
}

pub fn get_metadata(entry: &PathBuf, options: &Options) -> io::Result<Metadata> {
    if options.dereference {
        entry.metadata().or(entry.symlink_metadata())
    } else {
        entry.symlink_metadata()
    }
}

pub fn display_dir_entry_size(entry: &PathBuf, options: &Options) -> (usize, usize) {
    if let Ok(md) = get_metadata(entry, options) {
        (
            display_symlink_count(&md).len(),
            display_file_size(&md, options).len()
        )
    } else {
        (0, 0)
    }
}

#[cfg(unix)]
pub fn get_inode(metadata: &Metadata, options: &Options) -> String {
    if options.inode {
        format!("{:8} ", metadata.ino())
    } else {
        "".to_string()
    }
}

#[cfg(not(unix))]
pub fn get_inode(_metadata: &Metadata, _options: &Options) -> String {
    "".to_string()
}

pub fn get_file_name(name: &Path, strip: Option<&Path>) -> String {
    let mut name = match strip {
        Some(prefix) => name.strip_prefix(prefix).unwrap_or(name),
        None => name,
    };
    if name.as_os_str().len() == 0 {
        name = Path::new(".");
    }
    name.to_string_lossy().into_owned()
}

