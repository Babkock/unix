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
use libc::{time_t, c_char, c_int, gid_t, uid_t};
use libc::{getgrgid, getgrnam, getgroups, getpwnam, getpwuid, group, passwd};

use std::{io, fs, ptr};
use std::time::UNIX_EPOCH;
use std::io::ErrorKind;
use std::io::Error as IOError;
use std::io::Result as IoResult;
use std::fs::{DirEntry, FileType, Metadata};
use std::path::{Path, PathBuf};
use std::cmp::Reverse;
use std::ffi::{CStr, CString};
use std::borrow::Cow;
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

extern "C" {
    fn getgrouplist(
        name: *const c_char,
        gid: gid_t,
        groups: *mut gid_t,
        ngroups: *mut c_int
    ) -> c_int;
}

pub fn get_groups() -> IOResult<Vec<gid_t>> {
    let ngroups = unsafe { getgroups(0, ptr::null_mut()) };
    if ngroups == -1 {
        return Err(IOError::last_os_error());
    }
    // todo...
}

macro_rules! cstr2cow {
    ($v:expr) => (
        unsafe { CStr::from_ptr($v).to_string_lossy() }
    )
}

pub struct Passwd {
    inner: passwd
}

impl Passwd {
    pub fn name(&self) -> Cow<str> {
        cstr2cow!(self.inner.pw_name)
    }

    pub fn uid(&self) -> uid_t {
        self.inner.pw_uid
    }

    pub fn gid(&self) -> gid_t {
        self.inner.pw_gid
    }

    pub fn user_info(&self) -> Cow<str> {
        cstr2cow!(self.inner.pw_gecos)
    }

    pub fn user_shell(&self) -> Cow<str> {
        cstr2cow!(self.inner.pw_shell)
    }

    pub fn user_dir(&self) -> Cow<str> {
        cstr2cow!(self.inner.pw_dir)
    }

    pub fn user_passwd(&self) -> Cow<str> {
        cstr2cow!(self.inner.pw_passwd)
    }

    #[cfg(any(target_os = "freebsd", target_os = "macos"))]
    pub fn user_access_class(&self) -> Cow<str> {
        cstr2cow!(self.inner.pw_class)
    }

    pub fn as_inner(&self) -> &passwd {
        &self.inner
    }

    pub fn into_inner(self) -> passwd {
        self.inner
    }

    pub fn belongs_to(&self) -> Vec<gid_t> {
        let mut ngroups: c_int = 8;
        let mut groups = Vec::with_capacity(ngroups as usize);
        let gid = self.inner.pw_gid;
        let name = self.inner.pw_name;
        unsafe {
            if getgrouplist(name, gid, groups.as_mut_ptr(), &mut ngroups) == -1 {
                groups.resize(ngroups as usize, 0);
                getgrouplist(name, gid, groups.as_mut_ptr(), &mut ngroups);
            }
            groups.set_len(ngroups as usize);
        }
        groups.truncate(ngroups as usize);
        groups
    }
}

pub struct Group {
    inner: group
}

impl Group {
    pub fn name(&self) -> Cow<str> {
        cstr2cow!(self.inner.gr_name)
    }

    pub fn gid(&self) -> gid_t {
        self.inner.gr_gid
    }

    pub fn as_inner(&self) -> &group {
        &self.inner
    }

    pub fn into_inner(self) -> group {
        self.inner
    }
}

pub trait Locate<K> {
    fn locate(key: K) -> IOResult<Self>
    where
        Self: ::std::marker::Sized;
}

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
    let mut entries =
        safe_unwrap!(fs::read_dir(dir).and_then(|e| e.collect::<Result<Vec<_>, _>>()));

    entries.retain(|e| should_display(e, &options));

    let mut entries: Vec<_> = entries.iter().map(DirEntry::path).collect();
    sort_entries(&mut entries, &options);

    if options.show_hidden {
        let mut display_entries = entries.clone();
        display_entries.insert(0, dir.join(".."));
        display_entries.insert(0, dir.join("."));
        display_items(&display_entries, Some(dir), options);
    } else {
        display_items(&entries, Some(dir), options);
    }

    if options.recurse {
        for e in entries.iter().filter(|p| p.is_dir()) {
            println!("\n{}:", e.to_string_lossy());
            enter_directory(&e, &options);
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

pub fn display_items(items: &Vec<PathBuf>, strip: Option<&Path>, options: &Options) {
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
                println!("{}", display_file_name(&i, strip, &m, &options).contents);
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

    println!(
        "{}{}{} {} {} {} {} {} {}",
        get_inode(&m, &options),
        display_file_type(m.file_type()),
        display_permissions(&m),
        pad_left(display_symlink_count(&m), max_links),
        display_uname(&m, &options),
        display_group(&m, &options),
        pad_left(display_file_size(&m, &options), max_size),
        display_date(&m, &options),
        display_file_name(&item, strip, &m, &options).contents
    );
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

/* we're defining the f macro later */
f!(getpwnam, getpwuid, uid_t, Passwd);
f!(getgrnam, getgrgid, gid_t, Group);

#[inline]
pub fn uid2usr(id: uid_t) -> IOResult<String> {

}

#[inline]
pub fn gid2grp(id: gid_t) -> IOResult<String> {

}

#[cfg(unix)]
pub fn display_uname(metadata: &Metadata, options: &Options) -> String {
    if options.numeric_ids {
        metadata.uid().to_string()
    } else {
        uid2usr(metadata.uid()).unwrap_or(metadata.uid().to_string())
    }
}

#[cfg(unix)]
pub fn display_group(metadata: &Metadata, options: &Options) -> String {
    if options.numeric_ids {
        metadata.gid().to_string()
    } else {
        gid2grp(metadata.gid()).unwrap_or(metadata.gid().to_string())
    }
}

#[cfg(not(unix))]
#[allow(unused_variables)]
pub fn display_uname(metadata: &Metadata, _options: &Options) -> String {
    "somebody".to_string()
}

#[cfg(not(unix))]
pub fn display_group(metadata: &Metadata, _options: &Options) -> String {
    "somegroup".to_string()
}

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

pub fn display_file_type(file_type: FileType) -> String {
    if file_type.is_dir() {
        "d".to_string()
    } else if file_type.is_symlink() {
        "l".to_string()
    } else {
        "-".to_string()
    }
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

#[cfg(not(unix))]
pub fn display_file_name(
    path: &Path,
    strip: Option<&Path>,
    metadata: &Metadata,
    options: &Options
) -> Cell {
    let mut name = get_file_name(path, strip);

    if !options.long_listing {
        name = get_inode(metadata, &options) + &name;
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

#[cfg(unix)]
macro_rules! has {
    ($mode:expr, $perm:expr) => (
        $mode & ($perm as mode_t) != 0
    )
}

#[cfg(unix)]
pub fn display_file_name(
    path: &Path,
    strip: Option<&Path>,
    metadata: &Metadata,
    options: &Options
) -> Cell {
    let mut name = get_file_name(path, strip);
    if !options.long_listing {
        name = get_inode(metadata, &options) + &name;
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

