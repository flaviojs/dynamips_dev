//! MIPS64 JIT compiler.

// TODO mips64_jit_tcb is not defined in unstable, but mips64_jit_tcb_t is still referenced in cpu_mips_t
pub type mips64_jit_tcb_t = mips64_jit_tcb;

/// cbindgen:no-export
#[repr(C)]
pub struct mips64_jit_tcb {
    _todo: u8,
}
