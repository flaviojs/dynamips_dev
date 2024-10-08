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

/// Equivalent to the C code: `*(buf)++ = value`
/// Assumes buf is &mut *mut c_uchar or similar.
#[macro_export]
macro_rules! buf_push {
    ($buf:expr, $value:expr) => {
        **$buf = $value;
        *$buf = $buf.add(1);
    };
}
pub use buf_push;

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
    pub fn atm_bridge_delete_all() -> c_int;
    pub fn atm_bridge_start(filename: *mut c_char) -> c_int;
    pub fn atm_init();
    pub fn atmsw_delete_all() -> c_int;
    pub fn atmsw_start(filename: *mut c_char) -> c_int;
    pub fn c1700_platform_register() -> c_int;
    pub fn c2600_platform_register() -> c_int;
    pub fn c2691_platform_register() -> c_int;
    pub fn c3600_platform_register() -> c_int;
    pub fn c3725_platform_register() -> c_int;
    pub fn c3745_platform_register() -> c_int;
    pub fn c6msfc1_platform_register() -> c_int;
    pub fn c6sup1_platform_register() -> c_int;
    pub fn c7200_platform_register() -> c_int;
    pub fn dev_bswap_init(vm: *mut crate::vm::vm_instance_t, name: *mut c_char, paddr: crate::dynamips_common::m_uint64_t, len: crate::dynamips_common::m_uint32_t, remap_addr: crate::dynamips_common::m_uint64_t) -> c_int;
    pub fn dev_pcmcia_disk_init(vm: *mut crate::vm::vm_instance_t, name: *mut c_char, paddr: crate::dynamips_common::m_uint64_t, len: crate::dynamips_common::m_uint32_t, disk_size: u_int, mode: ::std::os::raw::c_int) -> *mut crate::vm::vm_obj_t;
    pub fn ethsw_delete_all() -> c_int;
    pub fn ethsw_start(filename: *mut c_char) -> c_int;
    pub fn frsw_delete_all() -> c_int;
    pub fn frsw_start(filename: *mut c_char) -> c_int;
    pub fn hypervisor_stopsig() -> c_int;
    pub fn hypervisor_tcp_server(ip_addr: *mut c_char, tcp_port: c_int) -> c_int;
    pub fn mips64_clear_irq(cpu: *mut crate::mips64::cpu_mips_t, irq: crate::dynamips_common::m_uint8_t);
    pub fn mips64_dump_insn(buffer: *mut c_char, buf_size: size_t, insn_name_size: size_t, pc: crate::dynamips_common::m_uint64_t, instruction: crate::utils::mips_insn_t) -> c_int;
    pub fn mips64_dump_stats(cpu: *mut crate::mips64::cpu_mips_t);
    pub fn mips64_exec_create_ilt();
    pub fn mips64_exec_run_cpu(cpu: *mut crate::cpu::cpu_gen_t) -> *mut c_void;
    pub fn mips64_jit_create_ilt();
    pub fn mips64_jit_flush(cpu: *mut crate::mips64::cpu_mips_t, threshold: u_int) -> u_int;
    pub fn mips64_jit_init(cpu: *mut crate::mips64::cpu_mips_t) -> c_int;
    pub fn mips64_jit_run_cpu(cpu: *mut crate::cpu::cpu_gen_t) -> *mut c_void;
    pub fn mips64_jit_shutdown(cpu: *mut crate::mips64::cpu_mips_t);
    pub fn mips64_mem_shutdown(cpu: *mut crate::mips64::cpu_mips_t);
    pub fn mips64_set_addr_mode(cpu: *mut crate::mips64::cpu_mips_t, addr_mode: u_int) -> c_int;
    pub fn mips64_set_irq(cpu: *mut crate::mips64::cpu_mips_t, irq: crate::dynamips_common::m_uint8_t);
    pub fn ppc32_delete(cpu: *mut crate::ppc32::cpu_ppc_t);
    pub fn ppc32_dump_regs(cpu: *mut crate::cpu::cpu_gen_t);
    pub fn ppc32_dump_stats(cpu: *mut crate::ppc32::cpu_ppc_t);
    pub fn ppc32_exec_create_ilt();
    pub fn ppc32_exec_run_cpu(gen: *mut crate::cpu::cpu_gen_t) -> *mut c_void;
    pub fn ppc32_get_bat_spr(cpu: *mut crate::ppc32::cpu_ppc_t, spr: u_int) -> crate::dynamips_common::m_uint32_t;
    pub fn ppc32_init(cpu: *mut crate::ppc32::cpu_ppc_t) -> c_int;
    pub fn ppc32_jit_alloc_hreg_forced(cpu: *mut crate::ppc32::cpu_ppc_t, hreg: c_int) -> c_int;
    pub fn ppc32_jit_alloc_hreg(cpu: *mut crate::ppc32::cpu_ppc_t, ppc_reg: c_int) -> c_int;
    pub fn ppc32_jit_close_hreg_seq(cpu: *mut crate::ppc32::cpu_ppc_t);
    pub fn ppc32_jit_create_ilt();
    pub fn ppc32_jit_init(cpu: *mut crate::ppc32::cpu_ppc_t) -> c_int;
    pub fn ppc32_jit_insert_hreg_mru(cpu: *mut crate::ppc32::cpu_ppc_t, map: *mut crate::utils::hreg_map);
    pub fn ppc32_jit_run_cpu(gen: *mut crate::cpu::cpu_gen_t) -> *mut c_void;
    pub fn ppc32_jit_start_hreg_seq(cpu: *mut crate::ppc32::cpu_ppc_t, insn: *mut c_char);
    pub fn ppc32_jit_tcb_recompile(cpu: *mut crate::ppc32::cpu_ppc_t, block: *mut crate::ppc32_jit::ppc32_jit_tcb_t) -> c_int;
    pub fn ppc32_jit_tcb_record_patch(block: *mut crate::ppc32_jit::ppc32_jit_tcb_t, iop: *mut crate::jit_op::jit_op_t, jit_ptr: *mut u_char, vaddr: crate::dynamips_common::m_uint32_t) -> c_int;
    pub fn ppc32_mem_invalidate_cache(cpu: *mut crate::ppc32::cpu_ppc_t);
    pub fn ppc32_run_breakpoint(cpu: *mut crate::ppc32::cpu_ppc_t);
    pub fn ppc32_set_bat_spr(cpu: *mut crate::ppc32::cpu_ppc_t, spr: u_int, val: crate::dynamips_common::m_uint32_t);
    pub fn ppc32_set_sdr1(cpu: *mut crate::ppc32::cpu_ppc_t, sdr1: crate::dynamips_common::m_uint32_t) -> c_int;
    pub fn ppc32_timer_irq_run(cpu: *mut crate::ppc32::cpu_ppc_t) -> *mut c_void;
    pub fn ppc32_trigger_exception(cpu: *mut crate::ppc32::cpu_ppc_t, exc_vector: u_int);
    pub fn ppc32_trigger_irq(cpu: *mut crate::ppc32::cpu_ppc_t);
    pub fn ppc32_trigger_timer_irq(cpu: *mut crate::ppc32::cpu_ppc_t);
    pub fn ppc32_update_cr_set_altered_hreg(cpu: *mut crate::ppc32::cpu_ppc_t);
    pub fn ppc32_vmtest_platform_register() -> c_int;
    pub fn tsg_show_stats();
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
