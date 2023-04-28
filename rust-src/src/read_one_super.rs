/* read superblock: */

use std::{error::Error, ffi::CString, fmt};

use anyhow::bail;
use bch_bindgen::c::{
    bcachefs_metadata_version::{
        bcachefs_metadata_version_bkey_renumber, bcachefs_metadata_version_max,
        bcachefs_metadata_version_min,
    },
    bch2_bio_map, bch2_prt_printf, bch2_sb_realloc, bch_csum,
    bch_csum_type::BCH_CSUM_NR,
    bch_errcode::{
        BCH_ERR_invalid_sb_csum, BCH_ERR_invalid_sb_csum_type, BCH_ERR_invalid_sb_magic,
        BCH_ERR_invalid_sb_too_big, BCH_ERR_invalid_sb_version,
    },
    bch_sb_handle, bio_reset, fn_BCH_SB_CSUM_TYPE, bch2_crc_cmp, fn_csum_from_sb,
    fn_le16_to_cpu, fn_le32_to_cpu, fn_le64_to_cpu, fn_uuid_le_cmp, fn_vstruct_bytes, printbuf,
    req_opf::REQ_OP_READ,
    submit_bio_wait, BCACHE_MAGIC, BCHFS_MAGIC, BLK_REQ_META, BLK_REQ_SYNC,
};
use libc::{c_int, c_void, size_t};
use log::trace;

#[derive(Debug)]
enum ReadOneSuperErrorType {
    InvalidSuperBlockVersionError {
        version: u32,
        min_version: u32,
        max_version: u32,
    },
    IOError,
    NotABcachefsSuperBlockError,
    InvalidSuperBlockTooBigError {
        bytes: usize,
        max_size_bits: u8,
    },
    CannotReallocError,
    InvalidSuperBlockCsumType(u64),
    InvalidSuperBlockCsum,
}

#[derive(Debug)]
struct ReadOneSuperError {
    return_code: c_int,
    error: ReadOneSuperErrorType,
}

impl Error for ReadOneSuperError {}

impl fmt::Display for ReadOneSuperError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.error {
            ReadOneSuperErrorType::InvalidSuperBlockVersionError {
                version,
                min_version,
                max_version,
            } => {
                write!(
                    f,
                    "Unsupported superblock version {:?} (min {:?}, max {:?})",
                    version, min_version, max_version
                )
            }
            ReadOneSuperErrorType::IOError => {
                write!(f, "IO error: {}", self.return_code)
            }
            ReadOneSuperErrorType::NotABcachefsSuperBlockError => {
                write!(f, "Not a bcachefs superblock")
            }
            ReadOneSuperErrorType::InvalidSuperBlockTooBigError {
                bytes,
                max_size_bits,
            } => {
                write!(
                    f,
                    "Invalid superblock: too big (got {} bytes, layout max {}))",
                    bytes,
                    512u64 << max_size_bits
                )
            }
            ReadOneSuperErrorType::CannotReallocError => Ok(()),
            ReadOneSuperErrorType::InvalidSuperBlockCsumType(t) => {
                write!(f, "unknown checksum type {}", t)
            }
            ReadOneSuperErrorType::InvalidSuperBlockCsum => {
                write!(f, "bad checksum",)
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn read_one_super_rust(sb: &mut bch_sb_handle, offset: u64, err: *mut printbuf) -> c_int {
    match read_one_super_result(sb, offset) {
        Ok(_) => 0,
        Err(e) => match e.downcast::<ReadOneSuperError>() {
            Ok(e) => {
                let s = CString::new(format!("{}", e)).unwrap();
                unsafe { bch2_prt_printf(err, s.as_ptr()) };
                e.return_code
            }
            Err(_) => {
                let s = CString::new("read_one_super unknown error").unwrap();
                unsafe { bch2_prt_printf(err, s.as_ptr()) };
                -1
            }
        },
    }
}
fn read_one_super_result(sb: &mut bch_sb_handle, offset: u64) -> anyhow::Result<()> {
    let csum: bch_csum;
    let mut version: u32;
    let mut version_min: u32;
    let mut bytes: size_t;
    let mut ret: c_int;

    trace!("using Rust version of read_one_super");

    unsafe {
        loop {
            // reread:
            bio_reset(
                sb.bio,
                sb.bdev,
                (REQ_OP_READ as u32) | BLK_REQ_SYNC | BLK_REQ_META,
            );
            (*sb.bio).bi_iter.bi_sector = offset;
            bch2_bio_map(sb.bio, sb.sb as *mut c_void, sb.buffer_size);

            ret = submit_bio_wait(sb.bio);
            if ret != 0 {
                bail!(ReadOneSuperError {
                    return_code: ret,
                    error: ReadOneSuperErrorType::IOError
                })
            }

            if fn_uuid_le_cmp((*sb.sb).magic, BCACHE_MAGIC)
                && fn_uuid_le_cmp((*sb.sb).magic, BCHFS_MAGIC)
            {
                bail!(ReadOneSuperError {
                    return_code: -(BCH_ERR_invalid_sb_magic as c_int),
                    error: ReadOneSuperErrorType::NotABcachefsSuperBlockError
                })
            }

            version = fn_le16_to_cpu((*sb.sb).version);
            version_min = if version >= bcachefs_metadata_version_bkey_renumber as u32 {
                fn_le16_to_cpu((*sb.sb).version_min)
            } else {
                version
            };

            if version >= bcachefs_metadata_version_max as u32 {
                bail!(ReadOneSuperError {
                    return_code: -(BCH_ERR_invalid_sb_version as c_int),
                    error: ReadOneSuperErrorType::InvalidSuperBlockVersionError {
                        version: version as u32,
                        min_version: bcachefs_metadata_version_min as u32,
                        max_version: bcachefs_metadata_version_max as u32,
                    }
                })
            }

            if version_min < bcachefs_metadata_version_min as u32 {
                bail!(ReadOneSuperError {
                    return_code: -(BCH_ERR_invalid_sb_version as c_int),
                    error: ReadOneSuperErrorType::InvalidSuperBlockVersionError {
                        version: version,
                        min_version: bcachefs_metadata_version_min as u32,
                        max_version: bcachefs_metadata_version_max as u32,
                    }
                })
            }

            bytes = fn_vstruct_bytes(sb.sb);

            if bytes > 512 << (*sb.sb).layout.sb_max_size_bits {
                bail!(ReadOneSuperError {
                    return_code: -(BCH_ERR_invalid_sb_too_big as c_int),
                    error: ReadOneSuperErrorType::InvalidSuperBlockTooBigError {
                        bytes: bytes,
                        max_size_bits: (*sb.sb).layout.sb_max_size_bits
                    }
                })
            }

            if bytes > sb.buffer_size {
                ret = bch2_sb_realloc(
                    sb as *mut _ as *mut bch_sb_handle,
                    fn_le32_to_cpu((*sb.sb).u64s),
                );
                if ret != 0 {
                    bail!(ReadOneSuperError {
                        return_code: ret,
                        error: ReadOneSuperErrorType::CannotReallocError
                    })
                }
            // goto reread;
            } else {
                break;
            }
        }

        if fn_BCH_SB_CSUM_TYPE(sb.sb) >= BCH_CSUM_NR as u64 {
            bail!(ReadOneSuperError {
                return_code: -(BCH_ERR_invalid_sb_csum_type as c_int),
                error: ReadOneSuperErrorType::InvalidSuperBlockCsumType(fn_BCH_SB_CSUM_TYPE(sb.sb))
            })
        }

        /* XXX: verify MACs */
        csum = fn_csum_from_sb(sb.sb);

        if bch2_crc_cmp(csum, (*sb.sb).csum) {
            // prt_printf(err, "bad checksum");
            bail!(ReadOneSuperError {
                return_code: -(BCH_ERR_invalid_sb_csum as c_int),
                error: ReadOneSuperErrorType::InvalidSuperBlockCsum
            })
        }

        sb.seq = fn_le64_to_cpu((*sb.sb).seq);
    }

    return Ok(());
}
