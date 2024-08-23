//! Rust code that is not available in C.
//!
//! cbindgen will ignore this module.
//! This module should be included as a prelude.
#![allow(unused_imports)]

use libc_alloc::LibcAlloc;
use std::any::TypeId;

/// Make rust memory compatible with C malloc/free/...
#[global_allocator]
static GLOBAL_ALLOCATOR: LibcAlloc = LibcAlloc;

/// Macro that mimics `__func__`.
/// Returns the function name as a C string (*mut c_char).
/// The function must have `#[named]` or `function_name!`.
#[macro_export]
macro_rules! cfunc {
    () => {
        concat!(function_name!(), "\0").as_ptr().cast::<c_char>().cast_mut()
    };
}
pub use cfunc;

/// Macro that concatenates expressions and a nul terminator.
#[macro_export]
macro_rules! str0 {
    () => {
        "\0"
    };
    ($($e:expr),*) => {
        constcat::concat!($($e),*, "\0")
    };
}
pub use str0;

/// Macro that converts a static string (&'static str) to a C string (*mut c_char).
#[macro_export]
macro_rules! cstr {
    ($($e:expr),*) => {
        str0!($($e),*).as_ptr().cast::<c_char>().cast_mut()
    };
}
pub use cstr;

/// Trait that converts a rust type to a C representation.
pub trait AsC<T, V> {
    fn as_c(&self) -> T;
    fn as_c_void(&self) -> V;
}
impl<T, const N: usize> AsC<*const T, *const c_void> for [T; N] {
    fn as_c(&self) -> *const T {
        self.as_ptr()
    }
    fn as_c_void(&self) -> *const c_void {
        self.as_c().cast::<_>()
    }
}
impl<T> AsC<*const T, *const c_void> for &[T] {
    fn as_c(&self) -> *const T {
        self.as_ptr()
    }
    fn as_c_void(&self) -> *const c_void {
        self.as_c().cast::<_>()
    }
}
impl AsC<*const c_char, *const c_void> for CStr {
    fn as_c(&self) -> *const c_char {
        self.as_ptr()
    }
    fn as_c_void(&self) -> *const c_void {
        self.as_c().cast::<_>()
    }
}
impl AsC<*const c_char, *const c_void> for CString {
    fn as_c(&self) -> *const c_char {
        self.as_c_str().as_ptr()
    }
    fn as_c_void(&self) -> *const c_void {
        self.as_c().cast::<_>()
    }
}
impl AsC<*const c_char, *const c_void> for &str {
    fn as_c(&self) -> *const c_char {
        self.as_ptr().cast::<_>()
    }
    fn as_c_void(&self) -> *const c_void {
        self.as_c().cast::<_>()
    }
}
impl AsC<*const c_char, *const c_void> for String {
    fn as_c(&self) -> *const c_char {
        self.as_str().as_c()
    }
    fn as_c_void(&self) -> *const c_void {
        self.as_c().cast::<_>()
    }
}

/// Trait that converts a mutable rust type to a C representation.
pub trait AsCMut<T, V> {
    fn as_c_mut(&mut self) -> T;
    fn as_c_void_mut(&mut self) -> V;
}
impl<T, const N: usize> AsCMut<*mut T, *mut c_void> for [T; N] {
    fn as_c_mut(&mut self) -> *mut T {
        self.as_ptr().cast_mut()
    }
    fn as_c_void_mut(&mut self) -> *mut c_void {
        self.as_c_mut().cast::<_>()
    }
}

/// Wrapper to allow custom formatting in the sprintf crate.
/// Add string format for `*const c_char` and `*mut c_char` and `&[c_char]`.
pub struct CustomPrintf<T>(pub T);
macro_rules! impl_CustomPrintf {
    ($($T:ty),*) => {
        $(
            impl sprintf::Printf for CustomPrintf<$T> {
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
impl_CustomPrintf!(u64, i64, u32, i32, u16, i16, u8, i8, usize, isize, f64, f32, char, String, CString, &str, &CStr);
impl<T: 'static> sprintf::Printf for CustomPrintf<*const T> {
    fn format(&self, spec: &sprintf::ConversionSpecifier) -> Result<String, sprintf::PrintfError> {
        CustomPrintf(self.0.cast_mut()).format(spec) // same as mut
    }
    fn as_int(&self) -> Option<i32> {
        CustomPrintf(self.0.cast_mut()).as_int() // same as mut
    }
}
impl<T: 'static> sprintf::Printf for CustomPrintf<*mut T> {
    fn format(&self, spec: &sprintf::ConversionSpecifier) -> Result<String, sprintf::PrintfError> {
        if spec.conversion_type == sprintf::ConversionType::String {
            // string
            if TypeId::of::<T>() != TypeId::of::<c_char>() {
                return Err(sprintf::PrintfError::WrongType); // T must be c_char
            }
            let ptr: *mut c_char = self.0.cast::<_>();
            if ptr.is_null() {
                Err(sprintf::PrintfError::WrongType) // does not support null
            } else if let Ok(s) = unsafe { CStr::from_ptr(ptr).to_str() } {
                s.format(spec) // supports utf8
            } else {
                Err(sprintf::PrintfError::WrongType) // does not support non-utf8
            }
        } else {
            // pointer address
            self.0.format(spec)
        }
    }
    fn as_int(&self) -> Option<i32> {
        None
    }
}
impl sprintf::Printf for CustomPrintf<&[c_char]> {
    fn format(&self, spec: &sprintf::ConversionSpecifier) -> Result<String, sprintf::PrintfError> {
        let p: *const c_char = self.0.as_c();
        CustomPrintf(p).format(spec)
    }
    fn as_int(&self) -> Option<i32> {
        None
    }
}

// dynamips C functions
extern "C" {
    pub fn dev_pcmcia_disk_init(vm: *mut crate::vm::vm_instance_t, name: *mut c_char, paddr: crate::dynamips_common::m_uint64_t, len: crate::dynamips_common::m_uint32_t, disk_size: u_int, mode: ::std::os::raw::c_int) -> *mut crate::vm::vm_obj_t;
    pub fn mips64_delete(cpu: *mut crate::mips64::cpu_mips_t);
    pub fn mips64_exec_run_cpu(cpu: *mut crate::cpu::cpu_gen_t) -> *mut c_void;
    pub fn mips64_init(cpu: *mut crate::mips64::cpu_mips_t) -> c_int;
    pub fn mips64_jit_init(cpu: *mut crate::mips64::cpu_mips_t) -> c_int;
    pub fn mips64_jit_run_cpu(cpu: *mut crate::cpu::cpu_gen_t) -> *mut c_void;
    pub fn pci_io_add(d: *mut crate::pci_io::pci_io_data, start: crate::dynamips_common::m_uint32_t, end: crate::dynamips_common::m_uint32_t, dev: *mut crate::device::vdevice, handler: crate::device::dev_handler_t) -> *mut crate::pci_dev::pci_io_device;
    pub fn pci_io_remove(dev: *mut crate::pci_dev::pci_io_device);
    pub fn physmem_copy_u16_from_vm(vm: *mut crate::vm::vm_instance_t, paddr: crate::dynamips_common::m_uint64_t) -> crate::dynamips_common::m_uint16_t;
    pub fn ppc32_delete(cpu: *mut crate::ppc32::cpu_ppc_t);
    pub fn ppc32_exec_run_cpu(gen: *mut crate::cpu::cpu_gen_t) -> *mut c_void;
    pub fn ppc32_init(cpu: *mut crate::ppc32::cpu_ppc_t) -> c_int;
    pub fn ppc32_jit_init(cpu: *mut crate::ppc32::cpu_ppc_t) -> c_int;
    pub fn ppc32_jit_run_cpu(gen: *mut crate::cpu::cpu_gen_t) -> *mut c_void;
    pub fn ppc32_update_cr_set_altered_hreg(cpu: *mut crate::ppc32::cpu_ppc_t);
    pub fn vm_alloc_host_page(vm: *mut crate::vm::vm_instance_t) -> *mut c_void;
    pub fn vm_bind_device(vm: *mut crate::vm::vm_instance_t, dev: *mut crate::device::vdevice) -> c_int;
    pub fn vm_build_filename(vm: *mut crate::vm::vm_instance_t, name: *mut c_char) -> *mut c_char;
    pub fn vm_ghost_image_get(filename: *mut c_char, ptr: *mut *mut u_char, fd: *mut c_int) -> c_int;
    pub fn vm_ghost_image_release(fd: c_int) -> c_int;
    pub fn vm_mmap_close_file(fd: c_int, ptr: *mut u_char, len: size_t) -> c_int;
    pub fn vm_mmap_create_file(vm: *mut crate::vm::vm_instance_t, name: *mut c_char, len: size_t, ptr: *mut *mut u_char) -> c_int;
    pub fn vm_mmap_open_file(vm: *mut crate::vm::vm_instance_t, name: *mut c_char, ptr: *mut *mut u_char, fsize: *mut libc::off_t) -> c_int;
    pub fn vm_object_add(vm: *mut crate::vm::vm_instance_t, obj: *mut crate::vm::vm_obj_t);
    pub fn vm_object_init(obj: *mut crate::vm::vm_obj_t);
    pub fn vm_unbind_device(vm: *mut crate::vm::vm_instance_t, dev: *mut crate::device::vdevice) -> c_int;
}

// _private C functions
extern "C" {
    pub fn c_errno_set(x: c_int);
    pub fn c_errno() -> c_int;
    pub fn c_INET6_ADDRSTRLEN() -> libc::socklen_t;
    pub fn c_optarg() -> *mut c_char;
    pub fn c_opterr_set(x: c_int);
    pub fn c_opterr() -> c_int;
    pub fn c_optind() -> c_int;
    pub fn c_stderr() -> *mut libc::FILE;
    pub fn c_stdout() -> *mut libc::FILE;
    pub fn c_timezone() -> c_long;
}

// system C functions
extern "C" {
    pub fn gethostbyname(name: *const c_char) -> *mut libc::hostent;
    pub fn htonl(x: u32) -> u32;
    pub fn htons(x: u16) -> u16;
    pub fn inet_addr(cp: *const libc::c_char) -> libc::in_addr_t;
    pub fn inet_aton(cp: *const c_char, inp: *mut libc::in_addr) -> c_int;
    pub fn inet_ntop(af: c_int, src: *const c_void, dst: *mut c_char, size: libc::socklen_t) -> *const c_char;
    pub fn inet_pton(af: c_int, src: *const c_char, dst: *mut c_void) -> c_int;
    pub fn ntohl(x: u32) -> u32;
    pub fn ntohs(x: u16) -> u16;
}

#[cfg(feature = "ENABLE_GEN_ETH")]
pub mod _pcap {
    // Based on https://stackoverflow.com/a/68202788
    // auto rebuild when _pcap.h changes
    const _: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/_private_pcap.h"));
    include!(concat!(env!("OUT_DIR"), "/_private_pcap.rs"));

    pub use pcap_direction_t_PCAP_D_IN as PCAP_D_IN;
    pub use pcap_direction_t_PCAP_D_INOUT as PCAP_D_INOUT;
    pub use pcap_direction_t_PCAP_D_OUT as PCAP_D_OUT;
}

pub use crate::_export::*;
pub use constcat;
pub use function_name::named;
pub use libc;
pub use libc::size_t;
pub use libc::ssize_t;
pub use paste::paste;
pub use setjmp;
pub use std::ffi::c_char;
pub use std::ffi::c_double;
pub use std::ffi::c_float;
pub use std::ffi::c_int;
pub use std::ffi::c_long;
pub use std::ffi::c_longlong;
pub use std::ffi::c_schar;
pub use std::ffi::c_short;
pub use std::ffi::c_uchar;
pub use std::ffi::c_uint;
pub use std::ffi::c_ulong;
pub use std::ffi::c_ulonglong;
pub use std::ffi::c_ushort;
pub use std::ffi::c_void;
pub use std::ffi::CStr;
pub use std::ffi::CString;
pub use std::marker::PhantomData;
pub use std::mem::offset_of;
pub use std::mem::size_of;
pub use std::mem::zeroed;
pub use std::ptr::addr_of;
pub use std::ptr::addr_of_mut;
pub use std::ptr::null_mut;
