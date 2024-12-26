//! Extra stuff that does not come from the dynamips C code.

use std::ffi::c_char;
use std::ffi::c_uchar;
use std::ffi::c_uint;
use std::ffi::c_ulong;
use std::ffi::CStr;
use std::ffi::CString;

pub mod _sys {
    //! Extra system symbols not included in libc, generated in the build script with bindgen.
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/_extra_sys.rs"));
}

// Extra C symbols, generated in the build script with cc.
unsafe extern "C" {
    pub fn c_stderr_set(x: *mut libc::FILE);
    pub fn c_stderr() -> *mut libc::FILE;
    pub fn c_stdout_set(x: *mut libc::FILE);
    pub fn c_stdout() -> *mut libc::FILE;
}

// Non-standard types. The C header that contains them is unknown.
pub type u_char = c_uchar;
pub type u_int = c_uint;
pub type u_long = c_ulong;

/// Wrapper for sprintf::Printf that formats `*const c_char` and `*mut c_char` as a string.
pub struct Printf<T>(pub T);
macro_rules! impl_same_printf {
    ($($T:ty),*) => {
        $(
            impl sprintf::Printf for Printf<$T> {
                fn format(&self, x: &sprintf::ConversionSpecifier) -> Result<String, sprintf::PrintfError> {
                    self.0.format(x)
                }
                fn as_int(&self) -> Option<i32> {
                    self.0.as_int()
                }
            }
        )*
    };
}
impl_same_printf!(u64, i64, u32, i32, u16, i16, u8, i8, usize, isize, f64, f32, char, String, CString, &str, &CStr);
impl<T: 'static> sprintf::Printf for Printf<*const T> {
    fn format(&self, spec: &sprintf::ConversionSpecifier) -> Result<String, sprintf::PrintfError> {
        if spec.conversion_type == sprintf::ConversionType::String {
            // format a string
            if std::any::TypeId::of::<T>() == std::any::TypeId::of::<c_char>() {
                let s: *const c_char = self.0.cast::<_>();
                if s.is_null() {
                    Err(sprintf::PrintfError::WrongType) // nul is not supported
                } else if let Ok(s) = unsafe { std::ffi::CStr::from_ptr(s).to_str() } {
                    s.format(spec) // utf8 is ok
                } else {
                    Err(sprintf::PrintfError::WrongType) // non-utf8 is not supported
                }
            } else {
                Err(sprintf::PrintfError::WrongType) // T must be c_char
            }
        } else {
            // format a pointer address
            self.0.format(spec)
        }
    }
    fn as_int(&self) -> Option<i32> {
        None
    }
}
impl<T: 'static> sprintf::Printf for Printf<*mut T> {
    fn format(&self, spec: &sprintf::ConversionSpecifier) -> Result<String, sprintf::PrintfError> {
        Printf(self.0.cast_const()).format(spec) // same as *const T
    }
    fn as_int(&self) -> Option<i32> {
        None
    }
}

/// Make sure cbindgen exports types by using them as arguments in this empty function.
#[rustfmt::skip]
#[no_mangle]
pub extern "C" fn _export(
    _: crate::dynamips_common::m_int16_t,
    _: crate::dynamips_common::m_int8_t,
    _: crate::net::n_eth_dot1q_hdr_t,
    _: crate::net::n_eth_hdr_t,
    _: crate::net::n_eth_isl_hdr_t,
    _: crate::net::n_eth_snap_hdr_t,
    _: crate::net::n_ip_network_t,
    _: crate::net::n_ipv6_network_t,
    _: crate::utils::hreg_map,
    _: crate::utils::insn_exec_page_t,
    _: crate::utils::insn_tblock_fptr,
    _: crate::utils::mips_insn_t,
    _: crate::utils::mts_map_t,
    _: crate::utils::mts32_entry_t,
    _: crate::utils::mts64_entry_t,
    _: crate::utils::ppc_insn_t,
    _: u_long,
) {
}
