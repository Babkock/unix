extern crate libc;

use std::{io, ptr, process};
use std::io::ErrorKind;
use std::ffi::{CStr, CString};
use libc::{mode_t, S_ISGID, S_ISUID, S_IRUSR, S_IWUSR, S_ISVTX, S_IROTH, S_IRGRP, S_IWOTH, S_IWGRP, S_IXGRP, S_IXOTH, S_IXUSR};
use libc::{time_t, c_char, c_int, gid_t, uid_t};
use libc::{getgrgid, getgrnam, getgroups, getpwnam, getpwuid, group, passwd};
use std::borrow::Cow;

extern "C" {
    fn getgrouplist(
        name: *const c_char,
        gid: gid_t,
        groups: *mut gid_t,
        ngroups: *mut c_int
    ) -> c_int;
}

pub fn get_groups() -> io::Result<Vec<gid_t>> {
    let ngroups = unsafe { getgroups(0, ptr::null_mut()) };
    if ngroups == -1 {
        return Err(io::Error::last_os_error());
    }
    let mut groups = Vec::with_capacity(ngroups as usize);
    let ngroups = unsafe { getgroups(ngroups, groups.as_mut_ptr()) };
    if ngroups == -1 {
        Err(io::Error::last_os_error())
    } else {
        unsafe {
            groups.set_len(ngroups as usize);
        }
        Ok(groups)
    }
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
    fn locate(key: K) -> io::Result<Self>
    where
        Self: ::std::marker::Sized;
}

macro_rules! f {
    ($fnam:ident, $fid:ident, $t:ident, $st:ident) => (
        impl Locate<$t> for $st {
            fn locate(k: $t) -> io::Result<Self> {
                unsafe {
                    let data = $fid(k);
                    if !data.is_null() {
                        Ok($st {
                            inner: ptr::read(data as *const _)
                        })
                    } else {
                        Err(io::Error::new(ErrorKind::NotFound, format!("No such id: {}", k)))
                    }
                }
            }
        }

        impl<'a> Locate<&'a str> for $st {
            fn locate(k: &'a str) -> io::Result<Self> {
                if let Ok(id) = k.parse::<$t>() {
                    let data = unsafe { $fid(id) };
                    if !data.is_null() {
                        Ok($st {
                            inner: unsafe { ptr::read(data as *const _)}
                        })
                    } else {
                        Err(io::Error::new(ErrorKind::NotFound, format!("No such id: {}", id)))
                    }
                } else {
                    unsafe {
                        let data = $fnam(CString::new(k).unwrap().as_ptr());
                        if !data.is_null() {
                            Ok($st {
                                inner: ptr::read(data as *const _)
                            })
                        } else {
                            Err(io::Error::new(ErrorKind::NotFound, format!("Not found: {}", k)))
                        }
                    }
                }
            }
        }
    )
}

f!(getpwnam, getpwuid, uid_t, Passwd);
f!(getgrnam, getgrgid, gid_t, Group);

#[inline]
pub fn uid2usr(id: uid_t) -> io::Result<String> {
    Passwd::locate(id).map(|p| p.name().into_owned())
}

#[inline]
pub fn gid2grp(id: gid_t) -> io::Result<String> {
    Group::locate(id).map(|p| p.name().into_owned())
}

#[inline]
pub fn usr2uid(name: &str) -> io::Result<uid_t> {
    Passwd::locate(name).map(|p| p.uid())
}

#[inline]
pub fn grp2gid(name: &str) -> io::Result<gid_t> {
    Group::locate(name).map(|p| p.gid())
}

