use crate::SPOS_MAX;
use crate::c;
use crate::fs::Fs;
use crate::errcode::{bch_errcode, errptr_to_result_c};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr;
use bitflags::bitflags;
use std::ffi::CStr;
use std::fmt;

pub struct BtreeTrans {
    raw:    c::btree_trans,
}

impl BtreeTrans {
    pub fn new<'a>(fs: &'a Fs) -> BtreeTrans {
        unsafe {
            let mut trans: MaybeUninit<BtreeTrans> = MaybeUninit::uninit();

            c::__bch2_trans_init(&mut (*trans.as_mut_ptr()).raw, fs.raw, 0);
            trans.assume_init()
        }
    }
}

impl Drop for BtreeTrans {
    fn drop(&mut self) {
        unsafe { c::bch2_trans_exit(&mut self.raw) }
    }             
}

bitflags! {
    pub struct BtreeIterFlags: u16 {
        const SLOTS = c::BTREE_ITER_SLOTS as u16;
        const ALL_LEVELS = c::BTREE_ITER_ALL_LEVELS as u16;
        const INTENT = c::BTREE_ITER_INTENT	 as u16;
        const PREFETCH = c::BTREE_ITER_PREFETCH as u16;
        const IS_EXTENTS = c::BTREE_ITER_IS_EXTENTS as u16;
        const NOT_EXTENTS = c::BTREE_ITER_NOT_EXTENTS as u16;
        const CACHED = c::BTREE_ITER_CACHED	as u16;
        const KEY_CACHED = c::BTREE_ITER_WITH_KEY_CACHE as u16;
        const WITH_UPDATES = c::BTREE_ITER_WITH_UPDATES as u16;
        const WITH_JOURNAL = c::BTREE_ITER_WITH_JOURNAL as u16;
        const __ALL_SNAPSHOTS = c::__BTREE_ITER_ALL_SNAPSHOTS as u16;
        const ALL_SNAPSHOTS = c::BTREE_ITER_ALL_SNAPSHOTS as u16;
        const FILTER_SNAPSHOTS = c::BTREE_ITER_FILTER_SNAPSHOTS as u16;
        const NOPRESERVE = c::BTREE_ITER_NOPRESERVE as u16;
        const CACHED_NOFILL = c::BTREE_ITER_CACHED_NOFILL as u16;
        const KEY_CACHE_FILL = c::BTREE_ITER_KEY_CACHE_FILL as u16;
    }
}

pub struct BtreeIter<'a> {
    raw:    c::btree_iter,
    trans:  PhantomData<&'a BtreeTrans>,
}

impl<'t> BtreeIter<'t> {
    pub fn new(trans: &'t BtreeTrans, btree: c::btree_id, pos: c::bpos, flags: BtreeIterFlags) -> BtreeIter {
        unsafe {
            let mut iter: MaybeUninit<c::btree_iter> = MaybeUninit::uninit();

            c::bch2_trans_iter_init_outlined(
                ptr::addr_of!(trans.raw).cast_mut(),
                &mut (*iter.as_mut_ptr()),
                btree as u32,
                pos,
                flags.bits as u32);

            BtreeIter { raw: iter.assume_init(), trans: PhantomData }
        }
    }

    pub fn peek_upto<'i>(&'i mut self, end: c::bpos) -> Result<Option<BkeySC>, bch_errcode> {
        unsafe {
            let k = c::bch2_btree_iter_peek_upto(&mut self.raw, end);
            errptr_to_result_c(k.k)
                .map(|_| if !k.k.is_null() { Some(BkeySC { k: &*k.k, v: &*k.v }) } else { None } )
        }
    }

    pub fn peek(&mut self) -> Result<Option<BkeySC>, bch_errcode> {
        self.peek_upto(SPOS_MAX)
    }

    pub fn peek_and_restart(&mut self) -> Result<Option<BkeySC>, bch_errcode> {
        unsafe {
            let k = c::bch2_btree_iter_peek_and_restart_outlined(&mut self.raw);

            errptr_to_result_c(k.k)
                .map(|_| if !k.k.is_null() { Some(BkeySC{ k: &*k.k, v: &*k.v }) } else { None } )
        }
    }

    pub fn advance(&mut self) {
        unsafe {
            c::bch2_btree_iter_advance(&mut self.raw);
        }
    }
}

impl<'a> Drop for BtreeIter<'a> {
    fn drop(&mut self) {
        unsafe { c::bch2_trans_iter_exit(self.raw.trans, &mut self.raw) }
    }             
}

pub struct BkeySC<'a> {
    pub k:  &'a c::bkey,
    pub v:  &'a c::bch_val,
}

impl<'a, 'b> BkeySC<'a> {
    unsafe fn to_raw(&self) -> c::bkey_s_c {
        c::bkey_s_c { k: self.k, v: self.v }
    }

    pub fn to_text(&'a self, fs: &'b Fs) -> BkeySCToText<'a, 'b> {
        BkeySCToText { k: self, fs }
    }
}

pub struct BkeySCToText<'a, 'b> {
    k:  &'a BkeySC<'a>,
    fs: &'b Fs,
}

impl<'a, 'b> fmt::Display for BkeySCToText<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = c::printbuf::new();

        unsafe { c::bch2_bkey_val_to_text(&mut buf, self.fs.raw, self.k.to_raw()) };
 
        let s = unsafe { CStr::from_ptr(buf.buf) };
        let s = s.to_str().unwrap();
        write!(f, "{}", s)
    }
}
