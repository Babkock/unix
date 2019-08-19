/*
 * chown/group.rs
 * Babkock/unix
 *
 * Copyright (c) 2019 Tanner Babcock.
 * MIT License.
*/
extern crate libc;

use std::{io, ptr};
use std::io::ErrorKind;
use std::ffi::{CStr, CString};
use libc::{gid_t, uid_t};
use libc::{getgrgid, getgrnam, getpwnam, getpwuid, group, passwd};
use std::borrow::Cow;

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

    /* the only difference between this group.rs and the one for ls, is
     * the uid() for Passwd, and the gid() for Group */
    pub fn uid(&self) -> uid_t {
        self.inner.pw_uid
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

