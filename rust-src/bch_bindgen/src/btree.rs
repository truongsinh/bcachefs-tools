use crate::SPOS_MAX;
use crate::c;
use crate::fs::Fs;
use crate::errcode::{bch_errcode, errptr_to_result_c};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr;
use bitflags::bitflags;

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

impl<'a> BtreeIter<'a> {
    pub fn new(trans: &'a BtreeTrans, btree: c::btree_id, pos: c::bpos, flags: BtreeIterFlags) -> BtreeIter {
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

    pub fn peek_upto(&mut self, end: c::bpos) -> Result<c::bkey_s_c, bch_errcode> {
        unsafe {
            let k = c::bch2_btree_iter_peek_upto(&mut self.raw, end);
            errptr_to_result_c(k.k).map(|_| k)
        }
    }

    pub fn peek(&mut self) -> Result<c::bkey_s_c, bch_errcode> {
        self.peek_upto(SPOS_MAX)
    }

    pub fn peek_and_restart(&mut self) -> Result<Option<c::bkey_s_c>, bch_errcode> {
        unsafe {
            let k = c::bch2_btree_iter_peek_and_restart_outlined(&mut self.raw);

            errptr_to_result_c(k.k)
                .map(|_| if !k.k.is_null() { Some(k) } else { None } )
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
