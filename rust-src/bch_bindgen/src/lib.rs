pub mod bcachefs;
pub mod btree;
pub mod errcode;
pub mod keyutils;
pub mod log;
pub mod rs;
pub mod fs;

pub mod c {
    pub use crate::bcachefs::*;
}

use c::bpos as Bpos;

pub const fn spos(inode: u64, offset: u64, snapshot: u32) -> Bpos {
    Bpos { inode, offset, snapshot }
}

pub const fn pos(inode: u64, offset: u64) -> Bpos {
    spos(inode, offset, 0)
}

pub const POS_MIN:  Bpos = spos(0, 0, 0);
pub const POS_MAX:  Bpos = spos(u64::MAX, u64::MAX, 0);
pub const SPOS_MAX: Bpos = spos(u64::MAX, u64::MAX, u32::MAX);

use std::cmp::Ordering;

impl PartialEq for Bpos {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Bpos {}

impl PartialOrd for Bpos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Bpos {
    fn cmp(&self, other: &Self) -> Ordering {
        let l_inode     = self.inode;
        let r_inode     = other.inode;
        let l_offset    = self.offset;
        let r_offset    = other.offset;
        let l_snapshot  = self.snapshot;
        let r_snapshot  = other.snapshot;

        l_inode.cmp(&r_inode)
            .then(l_offset.cmp(&r_offset))
            .then(l_snapshot.cmp(&r_snapshot))
    }
}

use std::ffi::CStr;
use std::fmt;

impl fmt::Display for c::btree_id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = unsafe { CStr::from_ptr(*c::bch2_btree_ids.get_unchecked(*self as usize)) };
        let s = s.to_str().unwrap();
        write!(f, "{}", s)
    }
}

use std::str::FromStr;
use std::ffi::CString;

use std::error::Error;

#[derive(Debug)]
pub struct InvalidBtreeId;

impl fmt::Display for InvalidBtreeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid btree id")
    }
}

impl Error for InvalidBtreeId {
}

impl FromStr for c::btree_id {
    type Err = InvalidBtreeId;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = CString::new(s).unwrap();
        let p: *const i8 = s.as_ptr();

        let v = unsafe {c::match_string(c::bch2_btree_ids[..].as_ptr(), (-(1 as isize)) as usize, p)};
        if v >= 0 {
            Ok(unsafe { std::mem::transmute(v) })
        } else {
            Err(InvalidBtreeId)
        }
    }
}

impl c::printbuf {
    fn new() -> c::printbuf {
        let mut buf: c::printbuf = Default::default();

        buf.set_heap_allocated(true);
        buf
    }
}

impl Drop for c::printbuf {
    fn drop(&mut self) {
        unsafe { c::bch2_printbuf_exit(self) }
    }             
}

impl fmt::Display for Bpos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = c::printbuf::new();

        unsafe { c::bch2_bpos_to_text(&mut buf, *self) };
 
        let s = unsafe { CStr::from_ptr(buf.buf) };
        let s = s.to_str().unwrap();
        write!(f, "{}", s)
    }
}

impl FromStr for c::bpos {
    type Err = InvalidBtreeId;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "POS_MIN" {
            return Ok(c::bpos { inode: 0, offset: 0, snapshot: 0 });
        }

        if s == "POS_MAX" {
            return Ok(c::bpos { inode: u64::MAX, offset: u64::MAX, snapshot: 0 });
        }

        if s == "SPOS_MAX" {
            return Ok(c::bpos { inode: u64::MAX, offset: u64::MAX, snapshot: u32::MAX });
        }

        let mut fields = s.split(':');
        let ino_str = fields.next().ok_or(InvalidBtreeId)?;
        let off_str = fields.next().ok_or(InvalidBtreeId)?;
        let snp_str = fields.next();

        let ino: u64    = ino_str.parse().map_err(|_| InvalidBtreeId)?;
        let off: u64    = off_str.parse().map_err(|_| InvalidBtreeId)?;
        let snp: u32    = snp_str.map(|s| s.parse().ok()).flatten().unwrap_or(0);

        Ok(c::bpos { inode: ino, offset: off, snapshot: snp })
    }
}

pub struct BkeySCToText<'a, 'b> {
    k:  &'a c::bkey_s_c,
    fs: &'b fs::Fs,
}

impl<'a, 'b> fmt::Display for BkeySCToText<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = c::printbuf::new();

        unsafe { c::bch2_bkey_val_to_text(&mut buf, self.fs.raw, *self.k) };
 
        let s = unsafe { CStr::from_ptr(buf.buf) };
        let s = s.to_str().unwrap();
        write!(f, "{}", s)
    }
}

impl c::bkey_s_c {
    pub fn to_text<'a, 'b>(&'a self, fs: &'b fs::Fs) -> BkeySCToText<'a, 'b> {
        BkeySCToText { k: self, fs }
    }
}
