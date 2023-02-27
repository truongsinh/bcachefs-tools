use crate::SPOS_MAX;
use crate::c;
use crate::fs::Fs;
use crate::errcode::{bch_errcode, errptr_to_result_c};
use std::mem::MaybeUninit;
use std::ptr;

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

pub struct BtreeIter {
    raw:    c::btree_iter,
}

impl BtreeIter {
    pub fn new<'a>(trans: &'a BtreeTrans, btree: c::btree_id, pos: c::bpos, flags: u32) -> BtreeIter {
        unsafe {
            let mut iter: MaybeUninit<BtreeIter> = MaybeUninit::uninit();

            c::bch2_trans_iter_init_outlined(
                ptr::addr_of!(trans.raw).cast_mut(),
                &mut (*iter.as_mut_ptr()).raw,
                btree as u32,
                pos,
                flags);
            iter.assume_init()
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

impl Drop for BtreeIter {
    fn drop(&mut self) {
        unsafe { c::bch2_trans_iter_exit(self.raw.trans, &mut self.raw) }
    }             
}
