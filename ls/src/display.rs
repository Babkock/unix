extern crate libc;

use std::{io, fs};
use std::fs::{DirEntry, FileType, Metadata};
#[cfg(any(unix, target_os ="redox"))]
use std::os::unix::fs::MetadataExt;
#[cfg(unix)]
use std::os::unix::fs::FileTypeExt;

#[cfg(unix)]
use libc::{mode_t, S_ISGID, S_ISUID, S_IRUSR, S_IWUSR, S_ISVTX, S_IROTH, S_IRGRP, S_IWOTH, S_IWGRP, S_IXGRP, S_IXOTH, S_IXUSR};
use libc::{time_t, c_char, c_int, gid_t, uid_t};
use libc::{getgrgid, getgrnam, getgroups, getpwnam, getpwuid, group, passwd};

#[cfg(unix)]
macro_rules! has {
    ($mode:expr, $perm:expr) => (
        $mode & ($perm) != 0
    )
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
