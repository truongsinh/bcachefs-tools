use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use crate::c;
use crate::errcode::{bch_errcode, errptr_to_result};

pub struct Fs {
    pub raw: *mut c::bch_fs,
}

impl Fs {
    pub fn open(devices: &Vec<PathBuf>, opts: c::bch_opts) -> Result<Fs, bch_errcode> {
        let devices: Vec<_> = devices.iter()
            .map(|i| CString::new(i.as_os_str().as_bytes()).unwrap()).collect();
        let dev_c_strs: Vec<_> = devices.iter()
            .map(|i| { let p: *const i8 = i.as_ptr(); p })
            .collect();
        let dev_c_strarray: *const *mut i8 = dev_c_strs[..].as_ptr() as *const *mut i8;

        let ret = unsafe { c::bch2_fs_open(dev_c_strarray, dev_c_strs.len() as u32, opts) };

        errptr_to_result(ret).map(|fs| Fs { raw: fs})
    }
}

impl Drop for Fs {
    fn drop(&mut self) {
        unsafe { c::bch2_fs_stop(self.raw) }
    }             
}
