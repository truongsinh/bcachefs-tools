pub mod key;
pub mod cmd_mount;

pub mod err {
    pub enum GError {
        Unknown {
            message: std::borrow::Cow<'static, String>,
        },
    }
    pub type GResult<T, E, OE> = ::core::result::Result<::core::result::Result<T, E>, OE>;
    pub type Result<T, E> = GResult<T, E, GError>;
}

#[macro_export]
macro_rules! c_str {
    ($lit:expr) => {
        unsafe {
            std::ffi::CStr::from_ptr(concat!($lit, "\0").as_ptr() as *const std::os::raw::c_char)
                .to_bytes_with_nul()
                .as_ptr() as *const std::os::raw::c_char
        }
    };
}

#[derive(Debug)]
struct ErrnoError(errno::Errno);
impl std::fmt::Display for ErrnoError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}
impl std::error::Error for ErrnoError {}
