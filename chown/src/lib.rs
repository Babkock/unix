/*
 * chown/lib.rs
 * Babkock/unix
 *
 * Copyright (c) 2019 Tanner Babcock.
 * MIT License.
*/
extern crate libc;
extern crate walkdir;

use walkdir::WalkDir;
use libc::{gid_t, lchown, uid_t};
use std::{env, fs, io};
use std::io::ErrorKind;
use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};
use std::convert::AsRef;
use std::ffi::CString;
use std::borrow::Cow;
use std::os::unix::ffi::OsStrExt;
use crate::group::*;

mod group;

pub const FTS_COMFOLLOW: u8 = 1;
pub const FTS_PHYSICAL: u8 = 1 << 1;
pub const FTS_LOGICAL: u8 = 1 << 2;

#[derive(Clone, PartialEq, Debug)]
pub enum Verbosity {
    Silent,
    Changes,
    Verbose,
    Normal
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IfFrom {
    All,
    User(u32),
    Group(u32),
    UserGroup(u32, u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalizeMode {
    None,
    Normal,
    Existing,
    Missing
}

#[derive(Clone, PartialEq, Debug)]
pub struct Owner {
    dest_uid: Option<u32>,
    dest_gid: Option<u32>,
    bit_flag: u8,
    verbosity: Verbosity,
    filter: IfFrom,
    files: Vec<String>,
    recurse: bool,
    preserve_root: bool,
    dereference: bool
}

macro_rules! unwrap {
    ($m:expr, $e:ident, $err:block) => (
        match $m {
            Ok(meta) => meta,
            Err($e) => $err
        }
    )
}

pub fn parse_spec(spec: &str) -> Result<(Option<u32>, Option<u32>), String> {
    let args = spec.split(':').collect::<Vec<_>>();
    let usr_only: bool = args.len() == 1;
    let grp_only: bool = args.len() == 2 && args[0].is_empty() && !args[1].is_empty();
    let usr_grp: bool = args.len() == 2 && !args[0].is_empty() && !args[1].is_empty();

    if usr_only {
        Ok((
            Some(match Passwd::locate(args[0]) {
                Ok(v) => v.uid(),
                _ => return Err(format!("invalid user: '{}'", spec)),
            }),
            None,
        ))
    } else if grp_only {
        Ok((
            None,
            Some(match Group::locate(args[1]) {
                Ok(v) => v.gid(),
                _ => return Err(format!("invalid group: '{}'", spec)),
            }),
        ))
    } else if usr_grp {
        Ok((
            Some(match Passwd::locate(args[0]) {
                Ok(v) => v.uid(),
                _ => return Err(format!("invalid user: '{}'", spec)),
            }),
            Some(match Group::locate(args[1]) {
                Ok(v) => v.gid(),
                _ => return Err(format!("invalid group: '{}'", spec))
            }),
        ))
    } else {
        Ok((None, None))
    }
}

impl Owner {
    pub fn exec(&self) -> i32 {
        let mut ret = 0;
        for f in &self.files {
            ret |= self.traverse(f);
        }
        ret
    }

    fn chown<P: AsRef<Path>>(
        &self,
        path: P,
        duid: uid_t,
        dgid: gid_t,
        follow: bool
    ) -> io::Result<()> {
        let path = path.as_ref();
        let s: CString = CString::new(path.as_os_str().as_bytes()).unwrap();
        let ret = unsafe {
            if follow {
                libc::chown(s.as_ptr(), duid, dgid)
            } else {
                lchown(s.as_ptr(), duid, dgid)
            }
        };
        if ret == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    fn traverse<P: AsRef<Path>>(&self, root: P) -> i32 {
        let follow_arg = self.dereference || self.bit_flag != FTS_PHYSICAL;
        let path = root.as_ref();
        let meta = match self.obtain_meta(path, follow_arg) {
            Some(m) => m,
            _ => return 1
        };

        // prohibit only if :
        // --preserve-root and -R present
        if self.recurse && self.preserve_root {
            let may_exist = if follow_arg {
                path.canonicalize().ok()
            } else {
                let real = resolve_relative_path(path);
                if real.is_dir() {
                    Some(real.canonicalize().expect("failed to get real path"))
                } else {
                    Some(real.into_owned())
                }
            };

            if let Some(p) = may_exist {
                if p.parent().is_none() {
                    println!("it is dangerous to operate recursively on '/'");
                    println!("use --no-preserve-root to override this failsafe");
                    return 1;
                }
            }
        }

        let ret = if self.matched(meta.uid(), meta.gid()) {
            self.wrap_chown(path, &meta, follow_arg)
        } else {
            0
        };

        if !self.recurse {
            ret
        } else {
            ret | self.dive_into(&root)
        }
    }

    fn dive_into<P: AsRef<Path>>(&self, root: P) -> i32 {
        let mut ret = 0;
        let root = root.as_ref();
        let follow = self.dereference || self.bit_flag & FTS_LOGICAL != 0;
        for entry in WalkDir::new(root).follow_links(follow).min_depth(1) {
            let entry = unwrap!(entry, e, {
                ret = 1;
                println!("{}", e);
                continue;
            });
            let path = entry.path();
            let meta = match self.obtain_meta(path, follow) {
                Some(m) => m,
                _ => {
                    ret = 1;
                    continue;
                }
            };

            if !self.matched(meta.uid(), meta.gid()) {
                continue;
            }

            ret = self.wrap_chown(path, &meta, follow);
        }
        ret
    }

    fn obtain_meta<P: AsRef<Path>>(
        &self,
        path: P,
        follow: bool
    ) -> Option<Metadata> {
        use self::Verbosity::*;
        let path = path.as_ref();
        let meta = if follow {
            unwrap!(path.metadata(), e, {
                match self.verbosity {
                    Silent => (),
                    _ => println!("cannot access '{}': {}", path.display(), e),
                }
                return None;
            })
        } else {
            unwrap!(path.symlink_metadata(), e, {
                match self.verbosity {
                    Silent => (),
                    _ => println!("cannot dereference '{}': {}", path.display(), e),
                }
                return None;
            })
        };
        Some(meta)
    }

    fn wrap_chown<P: AsRef<Path>>(
        &self,
        path: P,
        meta: &Metadata,
        follow: bool
    ) -> i32 {
        use self::Verbosity::*;
        let mut ret = 0;
        let dest_uid = self.dest_uid.unwrap_or(meta.uid());
        let dest_gid = self.dest_gid.unwrap_or(meta.gid());
        let path = path.as_ref();
        if let Err(e) = self.chown(path, dest_uid, dest_gid, follow) {
            match self.verbosity {
                Silent => (),
                _ => {
                    println!("changing ownership of '{}': {}", path.display(), e);
                    if self.verbosity == Verbose {
                        println!(
                            "failed to change ownership of {} from {}:{} to {}:{}",
                            path.display(),
                            group::uid2usr(meta.uid()).unwrap(),
                            group::gid2grp(meta.gid()).unwrap(),
                            group::uid2usr(dest_uid).unwrap(),
                            group::gid2grp(dest_gid).unwrap()
                        );
                    };
                }
            }
            ret = 1;
        } else {
            let changed = dest_uid != meta.uid() || dest_gid != meta.gid();
            if changed {
                match self.verbosity {
                    Changes | Verbose => {
                        println!(
                            "changed ownership of {} from {}:{} to {}:{}",
                            path.display(),
                            group::uid2usr(meta.uid()).unwrap(),
                            group::gid2grp(meta.gid()).unwrap(),
                            group::uid2usr(dest_uid).unwrap(),
                            group::gid2grp(dest_gid).unwrap()
                        );
                    },
                    _ => ()
                };
            } else if self.verbosity == Verbose {
                println!(
                    "ownership of {} retained as {}:{}",
                    path.display(),
                    group::uid2usr(dest_uid).unwrap(),
                    group::gid2grp(dest_gid).unwrap()
                );
            }
        }
        ret
    }

    #[inline]
    fn matched(&self, uid: uid_t, gid: gid_t) -> bool {
        match self.filter {
            IfFrom::All => true,
            IfFrom::User(u) => u == uid,
            IfFrom::Group(g) => g == gid,
            IfFrom::UserGroup(u, g) => u == uid && g == gid
        }
    }
}

pub fn resolve_relative_path(path: &Path) -> Cow<Path> {
    if path.components().all(|e| e != Component::ParentDir) {
        return path.into();
    }
    let root = Component::RootDir.as_os_str();
    let mut result = env::current_dir().unwrap_or(PathBuf::from(root));
    for c in path.components() {
        match c {
            Component::ParentDir => {
                if let Ok(p) = result.read_link() {
                    result = p;
                }
                result.pop();
            },
            Component::CurDir => (),
            Component::RootDir | Component::Normal(_) | Component::Prefix(_) => {
                result.push(c.as_os_str())
            }
        }
    }
    result.into()
}

/* do you know what this is? the future */
pub fn resolve<P: AsRef<Path>>(original: P) -> io::Result<PathBuf> {
    const MAX_LINKS: u32 = 255;
    let mut followed = 0;
    let mut result = original.as_ref().to_path_buf();
    loop {
        if followed == MAX_LINKS {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "maximum links followed"
            ));
        }

        match fs::symlink_metadata(&result) {
            Err(e) => return Err(e),
            Ok(ref m) if !m.file_type().is_symlink() => break,
            Ok(..) => {
                followed += 1;
                match fs::read_link(&result) {
                    Ok(path) => {
                        result.pop();
                        result.push(path);
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        }
    }
    Ok(result)
}

pub fn canonicalize<P: AsRef<Path>>(
    orig: P,
    can_mode: CanonicalizeMode
) -> io::Result<PathBuf> {
    let orig = orig.as_ref();
    let orig = if orig.is_absolute() {
        orig.to_path_buf()
    } else {
        env::current_dir().unwrap().join(orig)
    };

    let mut result: PathBuf = PathBuf::new();
    let mut parts = vec![];

    /* split path by directory separator; add root directory to final path
     * buffer; add remaining parts to temporary vector for canonicalization */
    for p in orig.components() {
        match p {
            Component::Prefix(_) | Component::RootDir => {
                result.push(p.as_os_str());
            }
            Component::CurDir => (),
            Component::ParentDir => {
                parts.pop();
            }
            Component::Normal(_) => {
                parts.push(p.as_os_str());
            }
        }
    }

    /* resolve symlinks where possible */
    if !parts.is_empty() {
        for p in parts[..parts.len() - 1].iter() {
            result.push(p);

            if can_mode == CanonicalizeMode::None {
                continue;
            }

            match resolve(&result) {
                Err(e) => match can_mode {
                    CanonicalizeMode::Missing => continue,
                    _ => return Err(e)
                },
                Ok(path) => {
                    result.pop();
                    result.push(path);
                }
            }
        }

        result.push(parts.last().unwrap());

        match resolve(&result) {
            Err(e) => {
                if can_mode == CanonicalizeMode::Existing {
                    return Err(e);
                }
            },
            Ok(path) => {
                result.pop();
                result.push(path);
            }
        }
    }
    Ok(result)
}

