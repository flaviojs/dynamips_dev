//! x86-codegen.h: Macros for generating x86 code
//!
//! Authors:
//!   Paolo Molaro (lupus@ximian.com)
//!   Intel Corporation (ORP Project)
//!   Sergey Chaban (serge@wildwestsoftware.com)
//!   Dietmar Maurer (dietmar@ximian.com)
//!   Patrik Torstensson
//!
//! Copyright (C)  2000 Intel Corporation.  All rights reserved.
//! Copyright (C)  2001, 2002 Ximian, Inc.

use crate::_extra::*;
use std::ffi::c_int;
use std::ffi::c_uchar;

// x86 register numbers
pub type X86_Reg_No = c_int; // TODO enum
pub const X86_EAX: X86_Reg_No = 0;
pub const X86_ECX: X86_Reg_No = 1;
pub const X86_EDX: X86_Reg_No = 2;
pub const X86_EBX: X86_Reg_No = 3;
pub const X86_ESP: X86_Reg_No = 4;
pub const X86_EBP: X86_Reg_No = 5;
pub const X86_ESI: X86_Reg_No = 6;
pub const X86_EDI: X86_Reg_No = 7;
pub const X86_NREG: X86_Reg_No = 8;

// opcodes for alu instructions
pub type X86_ALU_Opcode = c_int; // TODO enum
pub const X86_ADD: X86_Reg_No = 0;
pub const X86_OR: X86_Reg_No = 1;
pub const X86_ADC: X86_Reg_No = 2;
pub const X86_SBB: X86_Reg_No = 3;
pub const X86_AND: X86_Reg_No = 4;
pub const X86_SUB: X86_Reg_No = 5;
pub const X86_XOR: X86_Reg_No = 6;
pub const X86_CMP: X86_Reg_No = 7;
pub const X86_NALU: X86_Reg_No = 8;

// opcodes for shift instructions
pub type X86_Shift_Opcode = c_int; // TODO enum
pub const X86_SHLD: X86_Shift_Opcode = 0;
pub const X86_SHLR: X86_Shift_Opcode = 1;
pub const X86_ROL: X86_Shift_Opcode = 0;
pub const X86_ROR: X86_Shift_Opcode = 1;
pub const X86_RCL: X86_Shift_Opcode = 2;
pub const X86_RCR: X86_Shift_Opcode = 3;
pub const X86_SHL: X86_Shift_Opcode = 4;
pub const X86_SHR: X86_Shift_Opcode = 5;
pub const X86_SAR: X86_Shift_Opcode = 7;
pub const X86_NSHIFT: X86_Shift_Opcode = 8;

// opcodes for floating-point instructions
pub type X86_FP_Opcode = c_int; // TODO enum
pub const X86_FADD: X86_FP_Opcode = 0;
pub const X86_FMUL: X86_FP_Opcode = 1;
pub const X86_FCOM: X86_FP_Opcode = 2;
pub const X86_FCOMP: X86_FP_Opcode = 3;
pub const X86_FSUB: X86_FP_Opcode = 4;
pub const X86_FSUBR: X86_FP_Opcode = 5;
pub const X86_FDIV: X86_FP_Opcode = 6;
pub const X86_FDIVR: X86_FP_Opcode = 7;
pub const X86_NFP: X86_FP_Opcode = 8;

// integer conditions codes
pub type X86_CC = c_int; // TODO enum
pub const X86_CC_EQ: X86_CC = 0;
pub const X86_CC_E: X86_CC = 0;
pub const X86_CC_Z: X86_CC = 0;
pub const X86_CC_NE: X86_CC = 1;
pub const X86_CC_NZ: X86_CC = 1;
pub const X86_CC_LT: X86_CC = 2;
pub const X86_CC_B: X86_CC = 2;
pub const X86_CC_C: X86_CC = 2;
pub const X86_CC_NAE: X86_CC = 2;
pub const X86_CC_LE: X86_CC = 3;
pub const X86_CC_BE: X86_CC = 3;
pub const X86_CC_NA: X86_CC = 3;
pub const X86_CC_GT: X86_CC = 4;
pub const X86_CC_A: X86_CC = 4;
pub const X86_CC_NBE: X86_CC = 4;
pub const X86_CC_GE: X86_CC = 5;
pub const X86_CC_AE: X86_CC = 5;
pub const X86_CC_NB: X86_CC = 5;
pub const X86_CC_NC: X86_CC = 5;
pub const X86_CC_LZ: X86_CC = 6;
pub const X86_CC_S: X86_CC = 6;
pub const X86_CC_GEZ: X86_CC = 7;
pub const X86_CC_NS: X86_CC = 7;
pub const X86_CC_P: X86_CC = 8;
pub const X86_CC_PE: X86_CC = 8;
pub const X86_CC_NP: X86_CC = 9;
pub const X86_CC_PO: X86_CC = 9;
pub const X86_CC_O: X86_CC = 10;
pub const X86_CC_NO: X86_CC = 11;
pub const X86_NCC: usize = 12;

// FP status
pub type _X86_FP_ENUM = c_int; // TODO enum
pub const X86_FP_C0: _X86_FP_ENUM = 0x100;
pub const X86_FP_C1: _X86_FP_ENUM = 0x200;
pub const X86_FP_C2: _X86_FP_ENUM = 0x400;
pub const X86_FP_C3: _X86_FP_ENUM = 0x4000;
pub const X86_FP_CC_MASK: _X86_FP_ENUM = 0x4500;

// FP control word
pub type _X86_FPCW_ENUM = c_int; // TODO enum
pub const X86_FPCW_INVOPEX_MASK: _X86_FPCW_ENUM = 0x1;
pub const X86_FPCW_DENOPEX_MASK: _X86_FPCW_ENUM = 0x2;
pub const X86_FPCW_ZERODIV_MASK: _X86_FPCW_ENUM = 0x4;
pub const X86_FPCW_OVFEX_MASK: _X86_FPCW_ENUM = 0x8;
pub const X86_FPCW_UNDFEX_MASK: _X86_FPCW_ENUM = 0x10;
pub const X86_FPCW_PRECEX_MASK: _X86_FPCW_ENUM = 0x20;
pub const X86_FPCW_PRECC_MASK: _X86_FPCW_ENUM = 0x300;
pub const X86_FPCW_ROUNDC_MASK: _X86_FPCW_ENUM = 0xc00;
// values for precision control
pub const X86_FPCW_PREC_SINGLE: _X86_FPCW_ENUM = 0;
pub const X86_FPCW_PREC_DOUBLE: _X86_FPCW_ENUM = 0x200;
pub const X86_FPCW_PREC_EXTENDED: _X86_FPCW_ENUM = 0x300;
// values for rounding control
pub const X86_FPCW_ROUND_NEAREST: _X86_FPCW_ENUM = 0;
pub const X86_FPCW_ROUND_DOWN: _X86_FPCW_ENUM = 0x400;
pub const X86_FPCW_ROUND_UP: _X86_FPCW_ENUM = 0x800;
pub const X86_FPCW_ROUND_TOZERO: _X86_FPCW_ENUM = 0xc00;

// prefix code
pub type X86_Prefix = c_int; // TODO enum
pub const X86_LOCK_PREFIX: X86_Prefix = 0xF0;
pub const X86_REPNZ_PREFIX: X86_Prefix = 0xF2;
pub const X86_REPZ_PREFIX: X86_Prefix = 0xF3;
pub const X86_REP_PREFIX: X86_Prefix = 0xF3;
pub const X86_CS_PREFIX: X86_Prefix = 0x2E;
pub const X86_SS_PREFIX: X86_Prefix = 0x36;
pub const X86_DS_PREFIX: X86_Prefix = 0x3E;
pub const X86_ES_PREFIX: X86_Prefix = 0x26;
pub const X86_FS_PREFIX: X86_Prefix = 0x64;
pub const X86_GS_PREFIX: X86_Prefix = 0x65;
pub const X86_UNLIKELY_PREFIX: X86_Prefix = 0x2E;
pub const X86_LIKELY_PREFIX: X86_Prefix = 0x3E;
pub const X86_OPERAND_PREFIX: X86_Prefix = 0x66;
pub const X86_ADDRESS_PREFIX: X86_Prefix = 0x67;

#[no_mangle]
pub static x86_cc_unsigned_map: [c_uchar; X86_NCC] = [
    0x74, // eq
    0x75, // ne
    0x72, // lt
    0x76, // le
    0x77, // gt
    0x73, // ge
    0x78, // lz
    0x79, // gez
    0x7a, // p
    0x7b, // np
    0x70, // o
    0x71, // no
];

#[no_mangle]
pub static x86_cc_signed_map: [c_uchar; X86_NCC] = [
    0x74, // eq
    0x75, // ne
    0x7c, // lt
    0x7e, // le
    0x7f, // gt
    0x7d, // ge
    0x78, // lz
    0x79, // gez
    0x7a, // p
    0x7b, // np
    0x70, // o
    0x71, // no
];

pub type x86_imm_buf = _x86_imm_buf;
#[repr(C)]
#[derive(Copy, Clone)]
pub union _x86_imm_buf {
    pub val: c_int,
    pub b: [c_uchar; 4],
}

pub const X86_NOBASEREG: c_int = -1;

// bitvector mask for callee-saved registers
pub const X86_ESI_MASK: c_int = 1 << X86_ESI;
pub const X86_EDI_MASK: c_int = 1 << X86_EDI;
pub const X86_EBX_MASK: c_int = 1 << X86_EBX;
pub const X86_EBP_MASK: c_int = 1 << X86_EBP;

pub const X86_CALLEE_REGS: c_int = (1 << X86_EAX) | (1 << X86_ECX) | (1 << X86_EDX);
pub const X86_CALLER_REGS: c_int = (1 << X86_EBX) | (1 << X86_EBP) | (1 << X86_ESI) | (1 << X86_EDI);
pub const X86_BYTE_REGS: c_int = (1 << X86_EAX) | (1 << X86_ECX) | (1 << X86_EDX) | (1 << X86_EBX);

macro_rules! X86_IS_SCRATCH {
    ($reg:expr) => {
        (X86_CALLER_REGS & (1 << ($reg))) // X86_EAX, X86_ECX, or X86_EDX // FIXME comment missmatch?
    };
}
pub(crate) use X86_IS_SCRATCH;
macro_rules! X86_IS_CALLEE {
    ($reg:expr) => {
        (X86_CALLEE_REGS & (1 << ($reg))) // X86_ESI, X86_EDI, X86_EBX, or X86_EBP // FIXME comment missmatch?
    };
}
pub(crate) use X86_IS_CALLEE;

macro_rules! X86_IS_BYTE_REG {
    ($reg:expr) => {
        (($reg) < 4)
    };
}
pub(crate) use X86_IS_BYTE_REG;

// Frame structure:
//
//      +--------------------------------+
//      | in_arg[0]       = var[0]       |
//      | in_arg[1]       = var[1]       |
//      |         . . .                  |
//      | in_arg[n_arg-1] = var[n_arg-1] |
//      +--------------------------------+
//      |       return IP                |
//      +--------------------------------+
//      |       saved EBP                | <-- frame pointer (EBP)
//      +--------------------------------+
//      |            ...                 |  n_extra
//      +--------------------------------+
//      |       var[n_arg]               |
//      |       var[n_arg+1]             |  local variables area
//      |          . . .                 |
//      |       var[n_var-1]             |
//      +--------------------------------+
//      |                                |
//      |                                |
//      |       spill area               | area for spilling mimic stack
//      |                                |
//      +--------------------------------|
//      |          ebx                   |
//      |          ebp [ESP_Frame only]  |
//      |          esi                   |  0..3 callee-saved regs
//      |          edi                   | <-- stack pointer (ESP)
//      +--------------------------------+
//      |   stk0                         |
//      |   stk1                         |  operand stack area/
//      |   . . .                        |  out args
//      |   stkn-1                       |
//      +--------------------------------|
//
//

// useful building blocks
macro_rules! x86_modrm_mod {
    ($modrm:expr) => {
        $modrm >> 6
    };
}
pub(crate) use x86_modrm_mod;
macro_rules! x86_modrm_reg {
    ($modrm:expr) => {
        ($modrm >> 3) & 0x7
    };
}
pub(crate) use x86_modrm_reg;
macro_rules! x86_modrm_rm {
    ($modrm:expr) => {
        ($modrm & 0x7)
    };
}
pub(crate) use x86_modrm_rm;

macro_rules! x86_address_byte {
    ($inst:expr, $m:expr, $o:expr, $r:expr) => {{
        _bytes_push!($inst, ((($m & 0x03) << 6) | (($o & 0x07) << 3) | ($r & 0x07)) as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_address_byte;
macro_rules! x86_imm_emit32 {
    ($inst:expr, $imm:expr) => {{
        let imb: x86_imm_buf = x86_imm_buf { val: $imm as std::ffi::c_int };
        _bytes_push!($inst, imb.b[0]);
        _bytes_push!($inst, imb.b[1]);
        _bytes_push!($inst, imb.b[2]);
        _bytes_push!($inst, imb.b[3]);
    }};
}
pub(crate) use x86_imm_emit32;
macro_rules! x86_imm_emit16 {
    ($inst:expr, $imm:expr) => {{
        *((*$inst).cast::<std::ffi::c_short>()) = $imm; // FIXME assumes little endian host
        *$inst = $inst.add(2);
    }};
}
pub(crate) use x86_imm_emit16;
macro_rules! x86_imm_emit8 {
    ($inst:expr, $imm:expr) => {{
        _bytes_push!($inst, ($imm & 0xff) as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_imm_emit8;
macro_rules! x86_is_imm8 {
    ($imm:expr) => {
        ($imm as std::ffi::c_int >= -128) && ($imm as std::ffi::c_int <= 127)
    };
}
pub(crate) use x86_is_imm8;
macro_rules! x86_is_imm16 {
    ($imm:expr) => {
        ($imm as std::ffi::c_int >= -(1 << 16)) && ($imm as std::ffi::c_int <= (1 << 16) - 1)
    };
}
pub(crate) use x86_is_imm16;

macro_rules! x86_reg_emit {
    ($inst:expr, $r:expr, $regno:expr) => {{
        x86_address_byte!($inst, 3, $r, $regno);
    }};
}
pub(crate) use x86_reg_emit;
macro_rules! x86_reg8_emit {
    ($inst:expr, $r:expr, $regno:expr, $is_rh:expr, $is_rnoh:expr) => {{
        x86_address_byte!($inst, 3, if $is_rh { ($r | 4) } else { $r }, if $is_rnoh { ($regno | 4) } else { $regno });
    }};
}
pub(crate) use x86_reg8_emit;
macro_rules! x86_regp_emit {
    ($inst:expr, $r:expr, $regno:expr) => {{
        x86_address_byte!($inst, 0, $r, $regno);
    }};
}
pub(crate) use x86_regp_emit;
macro_rules! x86_mem_emit {
    ($inst:expr, $r:expr, $disp:expr) => {{
        x86_address_byte!($inst, 0, $r, 5);
        x86_imm_emit32!($inst, $disp);
    }};
}
pub(crate) use x86_mem_emit;

macro_rules! x86_membase_emit {
    ($inst:expr, $r:expr, $basereg:expr, $disp:expr) => {
        'block: {
            if $basereg == X86_ESP {
                if $disp == 0 {
                    x86_address_byte!($inst, 0, $r, X86_ESP);
                    x86_address_byte!($inst, 0, X86_ESP, X86_ESP);
                } else if x86_is_imm8!($disp) {
                    x86_address_byte!($inst, 1, $r, X86_ESP);
                    x86_address_byte!($inst, 0, X86_ESP, X86_ESP);
                    x86_imm_emit8!($inst, $disp);
                } else {
                    x86_address_byte!($inst, 2, $r, X86_ESP);
                    x86_address_byte!($inst, 0, X86_ESP, X86_ESP);
                    x86_imm_emit32!($inst, $disp);
                }
                break 'block;
            }
            if $disp == 0 && $basereg != X86_EBP {
                x86_address_byte!($inst, 0, $r, $basereg);
                break 'block;
            }
            if x86_is_imm8!($disp) {
                x86_address_byte!($inst, 1, $r, $basereg);
                x86_imm_emit8!($inst, $disp);
            } else {
                x86_address_byte!($inst, 2, $r, $basereg);
                x86_imm_emit32!($inst, $disp);
            }
        }
    };
}
pub(crate) use x86_membase_emit;

macro_rules! x86_memindex_emit {
    ($inst:expr, $r:expr, $basereg:expr, $disp:expr, $indexreg:expr, $shift:expr) => {{
        if $basereg == X86_NOBASEREG {
            x86_address_byte!($inst, 0, $r, 4);
            x86_address_byte!($inst, $shift, $indexreg, 5);
            x86_imm_emit32!($inst, $disp);
        } else if $disp == 0 && $basereg != X86_EBP {
            x86_address_byte!($inst, 0, $r, 4);
            x86_address_byte!($inst, $shift, $indexreg, $basereg);
        } else if x86_is_imm8!($disp) {
            x86_address_byte!($inst, 1, $r, 4);
            x86_address_byte!($inst, $shift, $indexreg, $basereg);
            x86_imm_emit8!($inst, $disp);
        } else {
            x86_address_byte!($inst, 2, $r, 4);
            x86_address_byte!($inst, $shift, $indexreg, 5);
            x86_imm_emit32!($inst, $disp);
        }
    }};
}
pub(crate) use x86_memindex_emit;

// target is the position in the code where to jump to:
// target = code;
// .. output loop code...
// x86_mov_reg_imm (code, X86_EAX, 0);
// loop = code;
// x86_loop (code, -1);
// ... finish method
//
// patch displacement
// x86_patch (loop, target);
//
// ins should point at the start of the instruction that encodes a target.
// the instruction is inspected for validity and the correct displacement
// is inserted.
macro_rules! x86_patch {
    ($ins:expr, $target:expr) => {{
        let mut pos: *mut std::ffi::c_uchar = $ins.add(1);
        let mut size: std::ffi::c_int = 0;
        match *$ins.cast::<std::ffi::c_uchar>() {
            0xe8 | 0xe9 => {
                size += 1;
                // call, jump32
            }
            0x0f => {
                if !(*pos >= 0x70 && *pos <= 0x8f) {
                    panic!("x86_patch::0x0f");
                }
                size += 1;
                pos = pos.add(1);
                // prefix for 32-bit disp
            }
            0xe0 | 0xe1 | 0xe2 | // loop
            0xeb | // jump8
            // conditional jump opcodes
            0x70 | 0x71 | 0x72 | 0x73 |
            0x74 | 0x75 | 0x76 | 0x77 |
            0x78 | 0x79 | 0x7a | 0x7b |
            0x7c | 0x7d | 0x7e | 0x7f => {}
            x => panic!("x86_patch::0x{:02x}", x),
        }
        let disp: std::ffi::c_int = $target.offset_from(pos) as std::ffi::c_int;
        if size != 0 {
            x86_imm_emit32!(&mut pos, disp - 4);
        } else if x86_is_imm8!(disp - 1) {
            x86_imm_emit8!(&mut pos, disp - 1);
        } else {
            panic!("x86_patch::size");
        }
    }};
}
pub(crate) use x86_patch;

macro_rules! x86_breakpoint {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xcc);
    }};
}
pub(crate) use x86_breakpoint;

macro_rules! x86_clc {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xf8 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_clc;
macro_rules! x86_cld {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xfc as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_cld;
macro_rules! x86_stosb {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xaa as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_stosb;
macro_rules! x86_stosl {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xab as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_stosl;
macro_rules! x86_stosd {
    ($inst:expr) => {
        x86_stosl!($inst);
    };
}
pub(crate) use x86_stosd;
macro_rules! x86_movsb {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xa4 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_movsb;
macro_rules! x86_movsl {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xa5 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_movsl;
macro_rules! x86_movsd {
    ($inst:expr) => {
        x86_movsl!($inst);
    };
}
pub(crate) use x86_movsd;

macro_rules! x86_prefix {
    ($inst:expr, $p:expr) => {{
        _bytes_push!($inst, $p as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_prefix;

macro_rules! x86_bswap {
    ($inst:expr, $reg:expr) => {{
        _bytes_push!($inst, 0x0f);
        _bytes_push!($inst, 0xc8 as std::ffi::c_uchar + $reg as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_bswap;

macro_rules! x86_rdtsc {
    ($inst:expr) => {{
        _bytes_push!($inst, 0x0f);
        _bytes_push!($inst, 0x31);
    }};
}
pub(crate) use x86_rdtsc;

macro_rules! x86_cmpxchg_reg_reg {
    ($inst:expr, $dreg:expr, $reg:expr) => {{
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        _bytes_push!($inst, 0xb1 as std::ffi::c_uchar);
        x86_reg_emit!($inst, $reg, $dreg);
    }};
}
pub(crate) use x86_cmpxchg_reg_reg;

macro_rules! x86_cmpxchg_mem_reg {
    ($inst:expr, $mem:expr, $reg:expr) => {{
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        _bytes_push!($inst, 0xb1 as std::ffi::c_uchar);
        x86_mem_emit!($inst, $reg, $mem);
    }};
}
pub(crate) use x86_cmpxchg_mem_reg;

macro_rules! x86_cmpxchg_membase_reg {
    ($inst:expr, $basereg:expr, $disp:expr, $reg:expr) => {{
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        _bytes_push!($inst, 0xb1 as std::ffi::c_uchar);
        x86_membase_emit!($inst, $reg, $basereg, $disp);
    }};
}
pub(crate) use x86_cmpxchg_membase_reg;

macro_rules! x86_xchg_reg_reg {
    ($inst:expr, $dreg:expr, $reg:expr, $size:expr) => {{
        if $size == 1 {
            _bytes_push!($inst, 0x86 as std::ffi::c_uchar);
        } else {
            _bytes_push!($inst, 0x87 as std::ffi::c_uchar);
        }
        x86_reg_emit!($inst, $reg, $dreg);
    }};
}
pub(crate) use x86_xchg_reg_reg;

macro_rules! x86_xchg_mem_reg {
    ($inst:expr, $mem:expr, $reg:expr, $size:expr) => {{
        if $size == 1 {
            _bytes_push!($inst, 0x86 as std::ffi::c_uchar);
        } else {
            _bytes_push!($inst, 0x87 as std::ffi::c_uchar);
        }
        x86_mem_emit!($inst, $reg, $mem);
    }};
}
pub(crate) use x86_xchg_mem_reg;

macro_rules! x86_xchg_membase_reg {
    ($inst:expr, $basereg:expr, $disp:expr, $reg:expr, $size:expr) => {{
        if $size == 1 {
            _bytes_push!($inst, 0x86 as std::ffi::c_uchar);
        } else {
            _bytes_push!($inst, 0x87 as std::ffi::c_uchar);
        }
        x86_membase_emit!($inst, $reg, $basereg, $disp);
    }};
}
pub(crate) use x86_xchg_membase_reg;

macro_rules! x86_xadd_reg_reg {
    ($inst:expr, $dreg:expr, $reg:expr, $size:expr) => {{
        _bytes_push!($inst, 0x0F as std::ffi::c_uchar);
        if $size == 1 {
            _bytes_push!($inst, 0xC0 as std::ffi::c_uchar);
        } else {
            _bytes_push!($inst, 0xC1 as std::ffi::c_uchar);
        }
        x86_reg_emit!($inst, $reg, $dreg);
    }};
}
pub(crate) use x86_xadd_reg_reg;

macro_rules! x86_xadd_mem_reg {
    ($inst:expr, $mem:expr, $reg:expr, $size:expr) => {{
        _bytes_push!($inst, 0x0F as std::ffi::c_uchar);
        if $size == 1 {
            _bytes_push!($inst, 0xC0 as std::ffi::c_uchar);
        } else {
            _bytes_push!($inst, 0xC1 as std::ffi::c_uchar);
        }
        x86_mem_emit!($inst, $reg, $mem);
    }};
}
pub(crate) use x86_xadd_mem_reg;

macro_rules! x86_xadd_membase_reg {
    ($inst:expr, $basereg:expr, $disp:expr, $reg:expr, $size:expr) => {{
        _bytes_push!($inst, 0x0F as std::ffi::c_uchar);
        if $size == 1 {
            _bytes_push!($inst, 0xC0 as std::ffi::c_uchar);
        } else {
            _bytes_push!($inst, 0xC1 as std::ffi::c_uchar);
        }
        x86_membase_emit!($inst, $reg, $basereg, $disp);
    }};
}
pub(crate) use x86_xadd_membase_reg;

macro_rules! x86_inc_mem {
    ($inst:expr, $mem:expr) => {{
        _bytes_push!($inst, 0xff as std::ffi::c_uchar);
        x86_mem_emit!($inst, 0, $mem);
    }};
}
pub(crate) use x86_inc_mem;

macro_rules! x86_inc_membase {
    ($inst:expr, $basereg:expr, $disp:expr) => {{
        _bytes_push!($inst, 0xff as std::ffi::c_uchar);
        x86_membase_emit!($inst, 0, $basereg, $disp);
    }};
}
pub(crate) use x86_inc_membase;

macro_rules! x86_inc_reg {
    ($inst:expr, $reg:expr) => {{
        _bytes_push!($inst, 0x40 + $reg as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_inc_reg;

macro_rules! x86_dec_mem {
    ($inst:expr, $mem:expr) => {{
        _bytes_push!($inst, 0xff as std::ffi::c_uchar);
        x86_mem_emit!($inst, 1, $mem);
    }};
}
pub(crate) use x86_dec_mem;

macro_rules! x86_dec_membase {
    ($inst:expr, $basereg:expr, $disp:expr) => {{
        _bytes_push!($inst, 0xff as std::ffi::c_uchar);
        x86_membase_emit!($inst, 1, $basereg, $disp);
    }};
}
pub(crate) use x86_dec_membase;

macro_rules! x86_dec_reg {
    ($inst:expr, $reg:expr) => {{
        _bytes_push!($inst, 0x48 + $reg as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_dec_reg;

macro_rules! x86_not_mem {
    ($inst:expr, $mem:expr) => {{
        _bytes_push!($inst, 0xf7 as std::ffi::c_uchar);
        x86_mem_emit!($inst, 2, $mem);
    }};
}
pub(crate) use x86_not_mem;

macro_rules! x86_not_membase {
    ($inst:expr, $basereg:expr, $disp:expr) => {{
        _bytes_push!($inst, 0xf7 as std::ffi::c_uchar);
        x86_membase_emit!($inst, 2, $basereg, $disp);
    }};
}
pub(crate) use x86_not_membase;

macro_rules! x86_not_reg {
    ($inst:expr, $reg:expr) => {{
        _bytes_push!($inst, 0xf7 as std::ffi::c_uchar);
        x86_reg_emit!($inst, 2, $reg);
    }};
}
pub(crate) use x86_not_reg;

macro_rules! x86_neg_mem {
    ($inst:expr, $mem:expr) => {{
        _bytes_push!($inst, 0xf7 as std::ffi::c_uchar);
        x86_mem_emit!($inst, 3, $mem);
    }};
}
pub(crate) use x86_neg_mem;

macro_rules! x86_neg_membase {
    ($inst:expr, $basereg:expr, $disp:expr) => {{
        _bytes_push!($inst, 0xf7 as std::ffi::c_uchar);
        x86_membase_emit!($inst, 3, $basereg, $disp);
    }};
}
pub(crate) use x86_neg_membase;

macro_rules! x86_neg_reg {
    ($inst:expr, $reg:expr) => {{
        _bytes_push!($inst, 0xf7 as std::ffi::c_uchar);
        x86_reg_emit!($inst, 3, $reg);
    }};
}
pub(crate) use x86_neg_reg;

macro_rules! x86_nop {
    ($inst:expr) => {{
        _bytes_push!($inst, 0x90 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_nop;

macro_rules! x86_alu_reg_imm {
    ($inst:expr, $opc:expr, $reg:expr, $imm:expr) => {
        'block: {
            if $reg == X86_EAX {
                _bytes_push!($inst, (($opc as std::ffi::c_uchar) << 3) + 5);
                x86_imm_emit32!($inst, $imm);
                break 'block;
            }
            if x86_is_imm8!($imm) {
                _bytes_push!($inst, 0x83 as std::ffi::c_uchar);
                x86_reg_emit!($inst, $opc, $reg);
                x86_imm_emit8!($inst, $imm);
            } else {
                _bytes_push!($inst, 0x81 as std::ffi::c_uchar);
                x86_reg_emit!($inst, $opc, $reg);
                x86_imm_emit32!($inst, $imm);
            }
        }
    };
}
pub(crate) use x86_alu_reg_imm;

macro_rules! x86_alu_mem_imm {
    ($inst:expr, $opc:expr, $mem:expr, $imm:expr) => {{
        if x86_is_imm8!($imm) {
            _bytes_push!($inst, 0x83 as std::ffi::c_uchar);
            x86_mem_emit!($inst, $opc, $mem);
            x86_imm_emit8!($inst, $imm);
        } else {
            _bytes_push!($inst, 0x81 as std::ffi::c_uchar);
            x86_mem_emit!($inst, $opc, $mem);
            x86_imm_emit32!($inst, $imm);
        }
    }};
}
pub(crate) use x86_alu_mem_imm;

macro_rules! x86_alu_membase_imm {
    ($inst:expr, $opc:expr, $basereg:expr, $disp:expr, $imm:expr) => {{
        if x86_is_imm8!($imm) {
            _bytes_push!($inst, 0x83 as std::ffi::c_uchar);
            x86_membase_emit!($inst, $opc, $basereg, $disp);
            x86_imm_emit8!($inst, $imm);
        } else {
            _bytes_push!($inst, 0x81 as std::ffi::c_uchar);
            x86_membase_emit!($inst, $opc, $basereg, $disp);
            x86_imm_emit32!($inst, $imm);
        }
    }};
}
pub(crate) use x86_alu_membase_imm;

macro_rules! x86_alu_membase8_imm {
    ($inst:expr, $opc:expr, $basereg:expr, $disp:expr, $imm:expr) => {{
        _bytes_push!($inst, 0x80 as std::ffi::c_uchar);
        x86_membase_emit!($inst, $opc, $basereg, $disp);
        x86_imm_emit8!($inst, $imm);
    }};
}
pub(crate) use x86_alu_membase8_imm;

macro_rules! x86_alu_mem_reg {
    ($inst:expr, $opc:expr, $mem:expr, $reg:expr) => {{
        _bytes_push!($inst, (($opc as std::ffi::c_uchar) << 3) + 1);
        x86_mem_emit!($inst, $reg, $mem);
    }};
}
pub(crate) use x86_alu_mem_reg;

macro_rules! x86_alu_membase_reg {
    ($inst:expr, $opc:expr, $basereg:expr, $disp:expr, $reg:expr) => {{
        _bytes_push!($inst, (($opc as std::ffi::c_uchar) << 3) + 1);
        x86_membase_emit!($inst, $reg, $basereg, $disp);
    }};
}
pub(crate) use x86_alu_membase_reg;

macro_rules! x86_alu_reg_reg {
    ($inst:expr, $opc:expr, $dreg:expr, $reg:expr) => {{
        _bytes_push!($inst, (($opc as std::ffi::c_uchar) << 3) + 3);
        x86_reg_emit!($inst, $dreg, $reg);
    }};
}
pub(crate) use x86_alu_reg_reg;

/// @x86_alu_reg8_reg8:
/// Supports ALU operations between two 8-bit registers.
/// dreg := dreg opc reg
/// X86_Reg_No enum is used to specify the registers.
/// Additionally is_*_h flags are used to specify what part
/// of a given 32-bit register is used - high (TRUE) or low (FALSE).
/// For example: dreg = X86_EAX, is_dreg_h = TRUE -> use AH
macro_rules! x86_alu_reg8_reg8 {
    ($inst:expr, $opc:expr, $dreg:expr, $reg:expr, $is_dreg_h:expr, $is_reg_h:expr) => {{
        _bytes_push!($inst, (($opc as std::ffi::c_uchar) << 3) + 2);
        x86_reg8_emit!($inst, $dreg, $reg, $is_dreg_h, $is_reg_h);
    }};
}
pub(crate) use x86_alu_reg8_reg8;

macro_rules! x86_alu_reg_mem {
    ($inst:expr, $opc:expr, $reg:expr, $mem:expr) => {{
        _bytes_push!($inst, (($opc as std::ffi::c_uchar) << 3) + 3);
        x86_mem_emit!($inst, $reg, $mem);
    }};
}
pub(crate) use x86_alu_reg_mem;

macro_rules! x86_alu_reg_membase {
    ($inst:expr, $opc:expr, $reg:expr, $basereg:expr, $disp:expr) => {{
        _bytes_push!($inst, (($opc as std::ffi::c_uchar) << 3) + 3);
        x86_membase_emit!($inst, $reg, $basereg, $disp);
    }};
}
pub(crate) use x86_alu_reg_membase;

macro_rules! x86_test_reg_imm {
    ($inst:expr, $reg:expr, $imm:expr) => {{
        if $reg == X86_EAX {
            _bytes_push!($inst, 0xa9 as std::ffi::c_uchar);
        } else {
            _bytes_push!($inst, 0xf7 as std::ffi::c_uchar);
            x86_reg_emit!($inst, 0, $reg);
        }
        x86_imm_emit32!($inst, $imm);
    }};
}
pub(crate) use x86_test_reg_imm;

macro_rules! x86_test_mem_imm {
    ($inst:expr, $mem:expr, $imm:expr) => {{
        _bytes_push!($inst, 0xf7 as std::ffi::c_uchar);
        x86_mem_emit!($inst, 0, $mem);
        x86_imm_emit32!($inst, $imm);
    }};
}
pub(crate) use x86_test_mem_imm;

macro_rules! x86_test_membase_imm {
    ($inst:expr, $basereg:expr, $disp:expr, $imm:expr) => {{
        _bytes_push!($inst, 0xf7 as std::ffi::c_uchar);
        x86_membase_emit!($inst, 0, $basereg, $disp);
        x86_imm_emit32!($inst, $imm);
    }};
}
pub(crate) use x86_test_membase_imm;

macro_rules! x86_test_reg_reg {
    ($inst:expr, $dreg:expr, $reg:expr) => {{
        _bytes_push!($inst, 0x85 as std::ffi::c_uchar);
        x86_reg_emit!($inst, $reg, $dreg);
    }};
}
pub(crate) use x86_test_reg_reg;

macro_rules! x86_test_mem_reg {
    ($inst:expr, $mem:expr, $reg:expr) => {{
        _bytes_push!($inst, 0x85 as std::ffi::c_uchar);
        x86_mem_emit!($inst, $reg, $mem);
    }};
}
pub(crate) use x86_test_mem_reg;

macro_rules! x86_test_membase_reg {
    ($inst:expr, $basereg:expr, $disp:expr, $reg:expr) => {{
        _bytes_push!($inst, 0x85 as std::ffi::c_uchar);
        x86_membase_emit!($inst, $reg, $basereg, $disp);
    }};
}
pub(crate) use x86_test_membase_reg;

macro_rules! x86_shift_reg_imm {
    ($inst:expr, $opc:expr, $reg:expr, $imm:expr) => {{
        if $imm == 1 {
            _bytes_push!($inst, 0xd1 as std::ffi::c_uchar);
            x86_reg_emit!($inst, $opc, $reg);
        } else {
            _bytes_push!($inst, 0xc1 as std::ffi::c_uchar);
            x86_reg_emit!($inst, $opc, $reg);
            x86_imm_emit8!($inst, $imm);
        }
    }};
}
pub(crate) use x86_shift_reg_imm;

macro_rules! x86_shift_mem_imm {
    ($inst:expr, $opc:expr, $mem:expr, $imm:expr) => {{
        if $imm == 1 {
            _bytes_push!($inst, 0xd1 as std::ffi::c_uchar);
            x86_mem_emit!($inst, $opc, $mem);
        } else {
            _bytes_push!($inst, 0xc1 as std::ffi::c_uchar);
            x86_mem_emit!($inst, $opc, $mem);
            x86_imm_emit8!($inst, $imm);
        }
    }};
}
pub(crate) use x86_shift_mem_imm;

macro_rules! x86_shift_membase_imm {
    ($inst:expr, $opc:expr, $basereg:expr, $disp:expr, $imm:expr) => {{
        if $imm == 1 {
            _bytes_push!($inst, 0xd1 as std::ffi::c_uchar);
            x86_membase_emit!($inst, $opc, $basereg, $disp);
        } else {
            _bytes_push!($inst, 0xc1 as std::ffi::c_uchar);
            x86_membase_emit!($inst, $opc, $basereg, $disp);
            x86_imm_emit8!($inst, $imm);
        }
    }};
}
pub(crate) use x86_shift_membase_imm;

macro_rules! x86_shift_reg {
    ($inst:expr, $opc:expr, $reg:expr) => {{
        _bytes_push!($inst, 0xd3 as std::ffi::c_uchar);
        x86_reg_emit!($inst, $opc, $reg);
    }};
}
pub(crate) use x86_shift_reg;

macro_rules! x86_shift_mem {
    ($inst:expr, $opc:expr, $mem:expr) => {{
        _bytes_push!($inst, 0xd3 as std::ffi::c_uchar);
        x86_mem_emit!($inst, $opc, $mem);
    }};
}
pub(crate) use x86_shift_mem;

macro_rules! x86_shift_membase {
    ($inst:expr, $opc:expr, $basereg:expr, $disp:expr) => {{
        _bytes_push!($inst, 0xd3 as std::ffi::c_uchar);
        x86_membase_emit!($inst, $opc, $basereg, $disp);
    }};
}
pub(crate) use x86_shift_membase;

// Multi op shift missing.

macro_rules! x86_shrd_reg {
    ($inst:expr, $dreg:expr, $reg:expr) => {{
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        _bytes_push!($inst, 0xad as std::ffi::c_uchar);
        x86_reg_emit!($inst, $reg, $dreg);
    }};
}
pub(crate) use x86_shrd_reg;

macro_rules! x86_shrd_reg_imm {
    ($inst:expr, $dreg:expr, $reg:expr, $shamt:expr) => {{
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        _bytes_push!($inst, 0xac as std::ffi::c_uchar);
        x86_reg_emit!($inst, $reg, $dreg);
        x86_imm_emit8!($inst, $shamt);
    }};
}
pub(crate) use x86_shrd_reg_imm;

macro_rules! x86_shld_reg {
    ($inst:expr, $dreg:expr, $reg:expr) => {{
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        _bytes_push!($inst, 0xa5 as std::ffi::c_uchar);
        x86_reg_emit!($inst, $reg, $dreg);
    }};
}
pub(crate) use x86_shld_reg;

macro_rules! x86_shld_reg_imm {
    ($inst:expr, $dreg:expr, $reg:expr, $shamt:expr) => {{
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        _bytes_push!($inst, 0xa4 as std::ffi::c_uchar);
        x86_reg_emit!($inst, $reg, $dreg);
        x86_imm_emit8!($inst, $shamt);
    }};
}
pub(crate) use x86_shld_reg_imm;

// EDX:EAX = EAX * rm
macro_rules! x86_mul_reg {
    ($inst:expr, $reg:expr, $is_signed:expr) => {{
        _bytes_push!($inst, 0xf7 as std::ffi::c_uchar);
        x86_reg_emit!($inst, 4 + if $is_signed { 1 } else { 0 }, $reg);
    }};
}
pub(crate) use x86_mul_reg;

macro_rules! x86_mul_mem {
    ($inst:expr, $mem:expr, $is_signed:expr) => {{
        _bytes_push!($inst, 0xf7 as std::ffi::c_uchar);
        x86_mem_emit!($inst, 4 + if $is_signed { 1 } else { 0 }, $mem);
    }};
}
pub(crate) use x86_mul_mem;

macro_rules! x86_mul_membase {
    ($inst:expr, $basereg:expr, $disp:expr, $is_signed:expr) => {{
        _bytes_push!($inst, 0xf7 as std::ffi::c_uchar);
        x86_membase_emit!($inst, 4 + if $is_signed { 1 } else { 0 }, $basereg, $disp);
    }};
}
pub(crate) use x86_mul_membase;

// r *= rm
macro_rules! x86_imul_reg_reg {
    ($inst:expr, $dreg:expr, $reg:expr) => {{
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        _bytes_push!($inst, 0xaf as std::ffi::c_uchar);
        x86_reg_emit!($inst, $dreg, $reg);
    }};
}
pub(crate) use x86_imul_reg_reg;

macro_rules! x86_imul_reg_mem {
    ($inst:expr, $reg:expr, $mem:expr) => {{
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        _bytes_push!($inst, 0xaf as std::ffi::c_uchar);
        x86_mem_emit!($inst, $reg, $mem);
    }};
}
pub(crate) use x86_imul_reg_mem;

macro_rules! x86_imul_reg_membase {
    ($inst:expr, $reg:expr, $basereg:expr, $disp:expr) => {{
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        _bytes_push!($inst, 0xaf as std::ffi::c_uchar);
        x86_membase_emit!($inst, $reg, $basereg, $disp);
    }};
}
pub(crate) use x86_imul_reg_membase;

// dreg = rm * imm
macro_rules! x86_imul_reg_reg_imm {
    ($inst:expr, $dreg:expr, $reg:expr, $imm:expr) => {{
        if x86_is_imm8!($imm) {
            _bytes_push!($inst, 0x6b as std::ffi::c_uchar);
            x86_reg_emit!($inst, $dreg, $reg);
            x86_imm_emit8!($inst, $imm);
        } else {
            _bytes_push!($inst, 0x69 as std::ffi::c_uchar);
            x86_reg_emit!($inst, $dreg, $reg);
            x86_imm_emit32!($inst, $imm);
        }
    }};
}
pub(crate) use x86_imul_reg_reg_imm;

macro_rules! x86_imul_reg_mem_imm {
    ($inst:expr, $reg:expr, $mem:expr, $imm:expr) => {{
        if x86_is_imm8!($imm) {
            _bytes_push!($inst, 0x6b as std::ffi::c_uchar);
            x86_mem_emit!($inst, $reg, $mem);
            x86_imm_emit8!($inst, $imm);
        } else {
            _bytes_push!($inst, 0x69 as std::ffi::c_uchar);
            x86_reg_emit!($inst, $reg, $mem);
            x86_imm_emit32!($inst, $imm);
        }
    }};
}
pub(crate) use x86_imul_reg_mem_imm;

macro_rules! x86_imul_reg_membase_imm {
    ($inst:expr, $reg:expr, $basereg:expr, $disp:expr, $imm:expr) => {{
        if x86_is_imm8!($imm) {
            _bytes_push!($inst, 0x6b as std::ffi::c_uchar);
            x86_membase_emit!($inst, $reg, $basereg, $disp);
            x86_imm_emit8!($inst, $imm);
        } else {
            _bytes_push!($inst, 0x69 as std::ffi::c_uchar);
            x86_membase_emit!($inst, $reg, $basereg, $disp);
            x86_imm_emit32!($inst, $imm);
        }
    }};
}
pub(crate) use x86_imul_reg_membase_imm;

// divide EDX:EAX by rm;
// eax = quotient, edx = remainder

macro_rules! x86_div_reg {
    ($inst:expr, $reg:expr, $is_signed:expr) => {{
        _bytes_push!($inst, 0xf7 as std::ffi::c_uchar);
        x86_reg_emit!($inst, 6 + if $is_signed { 1 } else { 0 }, $reg);
    }};
}
pub(crate) use x86_div_reg;

macro_rules! x86_div_mem {
    ($inst:expr, $mem:expr, $is_signed:expr) => {{
        _bytes_push!($inst, 0xf7 as std::ffi::c_uchar);
        x86_mem_emit!($inst, 6 + if $is_signed { 1 } else { 0 }, $mem);
    }};
}
pub(crate) use x86_div_mem;

macro_rules! x86_div_membase {
    ($inst:expr, $basereg:expr, $disp:expr, $is_signed:expr) => {{
        _bytes_push!($inst, 0xf7 as std::ffi::c_uchar);
        x86_membase_emit!($inst, 6 + if $is_signed { 1 } else { 0 }, $basereg, $disp);
    }};
}
pub(crate) use x86_div_membase;

macro_rules! x86_mov_mem_reg {
    ($inst:expr, $mem:expr, $reg:expr, $size:expr) => {{
        match $size {
            1 => _bytes_push!($inst, 0x88 as std::ffi::c_uchar),
            2 => {
                _bytes_push!($inst, 0x66 as std::ffi::c_uchar);
                // fall through
                _bytes_push!($inst, 0x89 as std::ffi::c_uchar);
            }
            4 => _bytes_push!($inst, 0x89 as std::ffi::c_uchar),
            x => panic!("x86_mov_mem_reg::{}", x),
        }
        x86_mem_emit!($inst, $reg, $mem);
    }};
}
pub(crate) use x86_mov_mem_reg;

macro_rules! x86_mov_regp_reg {
    ($inst:expr, $regp:expr, $reg:expr, $size:expr) => {{
        match $size {
            1 => _bytes_push!($inst, 0x88 as std::ffi::c_uchar),
            2 => {
                _bytes_push!($inst, 0x66 as std::ffi::c_uchar);
                // fall through
                _bytes_push!($inst, 0x89 as std::ffi::c_uchar);
            }
            4 => _bytes_push!($inst, 0x89 as std::ffi::c_uchar),
            x => panic!("x86_mov_regp_reg::{}", x),
        }
        x86_regp_emit!($inst, $reg, $regp);
    }};
}
pub(crate) use x86_mov_regp_reg;

macro_rules! x86_mov_membase_reg {
    ($inst:expr, $basereg:expr, $disp:expr, $reg:expr, $size:expr) => {{
        match $size {
            1 => _bytes_push!($inst, 0x88 as std::ffi::c_uchar),
            2 => {
                _bytes_push!($inst, 0x66 as std::ffi::c_uchar);
                // fall through
                _bytes_push!($inst, 0x89 as std::ffi::c_uchar);
            }
            4 => _bytes_push!($inst, 0x89 as std::ffi::c_uchar),
            x => panic!("x86_mov_membase_reg::{}", x),
        }
        x86_membase_emit!($inst, $reg, $basereg, $disp);
    }};
}
pub(crate) use x86_mov_membase_reg;

macro_rules! x86_mov_memindex_reg {
    ($inst:expr, $basereg:expr, $disp:expr, $indexreg:expr, $shift:expr, $reg:expr, $size:expr) => {{
        match $size {
            1 => _bytes_push!($inst, 0x88 as std::ffi::c_uchar),
            2 => {
                _bytes_push!($inst, 0x66 as std::ffi::c_uchar);
                // fall through
                _bytes_push!($inst, 0x89 as std::ffi::c_uchar);
            }
            4 => _bytes_push!($inst, 0x89 as std::ffi::c_uchar),
            x => panic!("x86_mov_memindex_reg::{}", x),
        }
        x86_memindex_emit!($inst, $reg, $basereg, $disp, $indexreg, $shift);
    }};
}
pub(crate) use x86_mov_memindex_reg;

macro_rules! x86_mov_reg_reg {
    ($inst:expr, $dreg:expr, $reg:expr, $size:expr) => {{
        match $size {
            1 => _bytes_push!($inst, 0x8a as std::ffi::c_uchar),
            2 => {
                _bytes_push!($inst, 0x66 as std::ffi::c_uchar);
                // fall through
                _bytes_push!($inst, 0x8b as std::ffi::c_uchar);
            }
            4 => _bytes_push!($inst, 0x8b as std::ffi::c_uchar),
            x => panic!("x86_mov_reg_reg::{}", x),
        }
        x86_reg_emit!($inst, $dreg, $reg);
    }};
}
pub(crate) use x86_mov_reg_reg;

macro_rules! x86_mov_reg_mem {
    ($inst:expr, $reg:expr, $mem:expr, $size:expr) => {{
        match $size {
            1 => _bytes_push!($inst, 0x8a as std::ffi::c_uchar),
            2 => {
                _bytes_push!($inst, 0x66 as std::ffi::c_uchar);
                // fall through
                _byts_push!($nst, 0x8b as std::ffi::c_uchar);
            }
            4 => _bytes_push!($inst, 0x8b as std::ffi::c_uchar),
            x => panic!("x86_mov_reg_mem::{}", x),
        }
        x86_mem_emit!($inst, $reg, $mem);
    }};
}
pub(crate) use x86_mov_reg_mem;

macro_rules! x86_mov_reg_membase {
    ($inst:expr, $reg:expr, $basereg:expr, $disp:expr, $size:expr) => {{
        match $size {
            1 => _bytes_push!($inst, 0x8a as std::ffi::c_uchar),
            2 => {
                _bytes_push!($inst, 0x66 as std::ffi::c_uchar);
                // fall through
                _bytes_push!($inst, 0x8b as std::ffi::c_uchar);
            }
            4 => _bytes_push!($inst, 0x8b as std::ffi::c_uchar),
            x => panic!("x86_mov_reg_membase::{}", x),
        }
        x86_membase_emit!($inst, $reg, $basereg, $disp);
    }};
}
pub(crate) use x86_mov_reg_membase;

macro_rules! x86_mov_reg_memindex {
    ($inst:expr, $reg:expr, $basereg:expr, $disp:expr, $indexreg:expr, $shift:expr, $size:expr) => {{
        match $size {
            1 => _bytes_push!($inst, 0x8a as std::ffi::c_uchar),
            2 => {
                _bytes_push!($inst, 0x66 as std::ffi::c_uchar);
                // fall through
                _bytes_push!($inst, 0x8b as std::ffi::c_uchar);
            }
            4 => _bytes_push!($inst, 0x8b as std::ffi::c_uchar),
            x => panic!("x86_mov_reg_memindex::{}", x),
        }
        x86_memindex_emit!($inst, $reg, $basereg, $disp, $indexreg, $shift);
    }};
}
pub(crate) use x86_mov_reg_memindex;

// Note: x86_clear_reg!() changes the condition code!
macro_rules! x86_clear_reg {
    ($inst:expr, $reg:expr) => {
        x86_alu_reg_reg!($inst, X86_XOR, $reg, $reg);
    };
}
pub(crate) use x86_clear_reg;

macro_rules! x86_mov_reg_imm {
    ($inst:expr, $reg:expr, $imm:expr) => {{
        _bytes_push!($inst, 0xb8 as std::ffi::c_uchar + $reg as std::ffi::c_uchar);
        x86_imm_emit32!($inst, $imm);
    }};
}
pub(crate) use x86_mov_reg_imm;

macro_rules! x86_mov_mem_imm {
    ($inst:expr, $mem:expr, $imm:expr, $size:expr) => {{
        if $size == 1 {
            _bytes_push!($inst, 0xc6 as std::ffi::c_uchar);
            x86_mem_emit!($inst, 0, $mem);
            x86_imm_emit8!($inst, $imm);
        } else if $size == 2 {
            _bytes_push!($inst, 0x66 as std::ffi::c_uchar);
            _bytes_push!($inst, 0xc7 as std::ffi::c_uchar);
            x86_mem_emit!($inst, 0, $mem);
            x86_imm_emit16!($inst, $imm);
        } else {
            _bytes_push!($inst, 0xc7 as std::ffi::c_uchar);
            x86_mem_emit!($inst, 0, $mem);
            x86_imm_emit32!($inst, $imm);
        }
    }};
}
pub(crate) use x86_mov_mem_imm;

macro_rules! x86_mov_membase_imm {
    ($inst:expr, $basereg:expr, $disp:expr, $imm:expr, $size:expr) => {{
        if $size == 1 {
            _bytes_push!($inst, 0xc6 as std::ffi::c_uchar);
            x86_membase_emit!($inst, 0, $basereg, $disp);
            x86_imm_emit8!($inst, $imm);
        } else if $size == 2 {
            _bytes_push!($inst, 0x66 as std::ffi::c_uchar);
            _bytes_push!($inst, 0xc7 as std::ffi::c_uchar);
            x86_membase_emit!($inst, 0, $basereg, $disp);
            x86_imm_emit16!($inst, $imm);
        } else {
            _bytes_push!($inst, 0xc7 as std::ffi::c_uchar);
            x86_membase_emit!($inst, 0, $basereg, $disp);
            x86_imm_emit32!($inst, $imm);
        }
    }};
}
pub(crate) use x86_mov_membase_imm;

macro_rules! x86_mov_memindex_imm {
    ($inst:expr, $basereg:expr, $disp:expr, $indexreg:expr, $shift:expr, $imm:expr, $size:expr) => {{
        if $size == 1 {
            _bytes_push!($inst, 0xc6 as std::ffi::c_uchar);
            x86_memindex_emit!($inst, 0, $basereg, $disp, $indexreg, $shift);
            x86_imm_emit8!($inst, $imm);
        } else if $size == 2 {
            _bytes_push!($inst, 0x66 as std::ffi::c_uchar);
            _bytes_push!($inst, 0xc7 as std::ffi::c_uchar);
            x86_memindex_emit!($inst, 0, $basereg, $disp, $indexreg, $shift);
            x86_imm_emit16!($inst, $imm);
        } else {
            _bytes_push!($inst, 0xc7 as std::ffi::c_uchar);
            x86_memindex_emit!($inst, 0, $basereg, $disp, $indexreg, $shift);
            x86_imm_emit32!($inst, $imm);
        }
    }};
}
pub(crate) use x86_mov_memindex_imm;

macro_rules! x86_lea_mem {
    ($inst:expr, $reg:expr, $mem:expr) => {{
        _bytes_push!($inst, 0x8d as std::ffi::c_uchar);
        x86_mem_emit!($inst, $reg, $mem);
    }};
}
pub(crate) use x86_lea_mem;

macro_rules! x86_lea_membase {
    ($inst:expr, $reg:expr, $basereg:expr, $disp:expr) => {{
        _bytes_push!($inst, 0x8d as std::ffi::c_uchar);
        x86_membase_emit!($inst, $reg, $basereg, $disp);
    }};
}
pub(crate) use x86_lea_membase;

macro_rules! x86_lea_memindex {
    ($inst:expr, $reg:expr, $basereg:expr, $disp:expr, $indexreg:expr, $shift:expr) => {{
        _bytes_push!($inst, 0x8d as std::ffi::c_uchar);
        x86_memindex_emit!($inst, $reg, $basereg, $disp, $indexreg, $shift);
    }};
}
pub(crate) use x86_lea_memindex;

macro_rules! x86_widen_reg {
    ($inst:expr, $dreg:expr, $reg:expr, $is_signed:expr, $is_half:expr) => {{
        let mut op: std::ffi::c_uchar = 0xb6;
        assert!($is_half || X86_IS_BYTE_REG!($reg));
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        if $is_signed {
            op += 0x08;
        }
        if $is_half {
            op += 0x01;
        }
        _bytes_push!($inst, op);
        x86_reg_emit!($inst, $dreg, $reg);
    }};
}
pub(crate) use x86_widen_reg;

macro_rules! x86_widen_mem {
    ($inst:expr, $dreg:expr, $mem:expr, $is_signed:expr, $is_half:expr) => {{
        let mut op: std::ffi::c_uchar = 0xb6;
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        if $is_signed {
            op += 0x08;
        }
        if $is_half {
            op += 0x01;
        }
        _bytes_push!($inst, op);
        x86_mem_emit!($inst, $dreg, $mem);
    }};
}
pub(crate) use x86_widen_mem;

macro_rules! x86_widen_membase {
    ($inst: expr, $dreg:expr, $basereg:expr, $disp:expr, $is_signed:expr, $is_half:expr) => {{
        let mut op: std::ffi::c_uchar = 0xb6;
        _bytes_push!($inst, 0x0f);
        if $is_signed {
            op += 0x08;
        }
        if $is_half {
            op += 0x01;
        }
        _bytes_push!($inst, op);
        x86_membase_emit!($inst, $dreg, $basereg, $disp);
    }};
}
pub(crate) use x86_widen_membase;

macro_rules! x86_widen_memindex {
    ($inst:expr, $dreg:expr, $basereg:expr, $disp:expr, $indexreg:expr, $shift:expr, $is_signed:expr, $is_half:expr) => {{
        let mut op: std::ffi::c_uchar = 0xb6;
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        if $is_signed {
            op += 0x08;
        }
        if $is_half {
            op += 0x01;
        }
        _bytes_push!($inst, op);
        x86_memindex_emit!($inst, $dreg, $basereg, $disp, $indexreg, $shift);
    }};
}
pub(crate) use x86_widen_memindex;

macro_rules! x86_lahf {
    ($inst:expr) => {{
        _bytes_push!($inst, 0x9f as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_lahf;
macro_rules! x86_sahf {
    //FIXME overridden later
    ($inst:expr) => {{
        _bytes_push!($inst, 0x9e as std::ffi::c_uchar);
    }};
}
//pub(crate) use x86_sahf;
macro_rules! x86_xchg_ah_al {
    ($inst:expr) => {{
        _bytes_push!($inst, 0x86 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xe0 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_xchg_ah_al;

macro_rules! x86_cdq {
    ($inst:expr) => {{
        _bytes_push!($inst, 0x99 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_cdq;
macro_rules! x86_wait {
    ($inst:expr) => {{
        _bytes_push!($inst, 0x9b as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_wait;

macro_rules! x86_fp_op_mem {
    ($inst:expr, $opc:expr, $mem:expr, $is_double:expr) => {{
        _bytes_push!($inst, if $is_double { 0xdc as std::ffi::c_uchar } else { 0xd8 as std::ffi::c_uchar });
        x86_mem_emit!($inst, $opc, $mem);
    }};
}
pub(crate) use x86_fp_op_mem;

macro_rules! x86_fp_op_membase {
    ($inst:expr, $opc:expr, $basereg:expr, $disp:expr, $is_double:expr) => {{
        _bytes_push!($inst, if $is_double { 0xdc as std::ffi::c_uchar } else { 0xd8 as std::ffi::c_uchar });
        x86_membase_emit!($inst, $opc, $basereg, $disp);
    }};
}
pub(crate) use x86_fp_op_membase;

macro_rules! x86_fp_op {
    ($inst:expr, $opc:expr, $index:expr) => {{
        _bytes_push!($inst, 0xd8 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xc0 as std::ffi::c_uchar + ($opc << 3) + ($index & 0x07));
    }};
}
pub(crate) use x86_fp_op;

macro_rules! x86_fp_op_reg {
    ($inst:expr, $opc:expr, $index:expr, $pop_stack:expr) => {{
        static map: [std::ffi::c_uchar; 9] = [0, 1, 2, 3, 5, 4, 7, 6, 8];
        _bytes_push!($inst, if $pop_stack { 0xde as std::ffi::c_uchar } else { 0xdc as std::ffi::c_uchar });
        _bytes_push!($inst, 0xc0 as std::ffi::c_uchar + (map[$opc as usize] << 3) + ($index & 0x07));
    }};
}
pub(crate) use x86_fp_op_reg;

/// @x86_fp_int_op_membase
/// Supports FPU operations between ST(0) and integer operand in memory.
/// Operation encoded using X86_FP_Opcode enum.
/// Operand is addressed by [basereg + disp].
/// is_int specifies whether operand is int32 (TRUE) or int16 (FALSE).
macro_rules! x86_fp_int_op_membase {
    ($inst:expr, $opc:expr, $basereg:expr, $disp:expr, $is_int:expr) => {{
        _bytes_push!($inst, if $is_int { 0xda as std::ffi::c_uchar } else { 0xde as std::ffi::c_uchar });
        x86_membase_emit!($inst, $opc, $basereg, $disp);
    }};
}
pub(crate) use x86_fp_int_op_membase;

macro_rules! x86_fstp {
    ($inst:expr, $index:expr) => {{
        _bytes_push!($inst, 0xdd as std::ffi::c_uchar);
        _bytes_push!($inst, 0xd8 as std::ffi::c_uchar + $index);
    }};
}
pub(crate) use x86_fstp;

macro_rules! x86_fcompp {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xde as std::ffi::c_uchar);
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_fcompp;

macro_rules! x86_fucompp {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xda as std::ffi::c_uchar);
        _bytes_push!($inst, 0xe9 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_fucompp;

macro_rules! x86_fnstsw {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xdf as std::ffi::c_uchar);
        _bytes_push!($inst, 0xe0 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_fnstsw;

macro_rules! x86_fnstcw {
    ($inst:expr, $mem:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        x86_mem_emit!($inst, 7, $mem);
    }};
}
pub(crate) use x86_fnstcw;

macro_rules! x86_fnstcw_membase {
    ($inst:expr, $basereg:expr, $disp:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        x86_membase_emit!($inst, 7, $basereg, $disp);
    }};
}
pub(crate) use x86_fnstcw_membase;

macro_rules! x86_fldcw {
    ($inst:expr, $mem:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        x86_mem_emit!($inst, 5, $mem);
    }};
}
pub(crate) use x86_fldcw;

macro_rules! x86_fldcw_membase {
    ($inst:expr, $basereg:expr, $disp:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        x86_membase_emit!($inst, 5, $basereg, $disp);
    }};
}
pub(crate) use x86_fldcw_membase;

macro_rules! x86_fchs {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xe0 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_fchs;

macro_rules! x86_frem {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xf8 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_frem;

macro_rules! x86_fxch {
    ($inst:expr, $index:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xc8 as std::ffi::c_uchar + ($index & 0x07));
    }};
}
pub(crate) use x86_fxch;

macro_rules! x86_fcomi {
    ($inst:expr, $index:expr) => {{
        _bytes_push!($inst, 0xdb as std::ffi::c_uchar);
        _bytes_push!($inst, 0xf0 as std::ffi::c_uchar + ($index & 0x07));
    }};
}
pub(crate) use x86_fcomi;

macro_rules! x86_fcomip {
    ($inst:expr, $index:expr) => {{
        _bytes_push!($inst, 0xdf as std::ffi::c_uchar);
        _bytes_push!($inst, 0xf0 as std::ffi::c_uchar + ($index & 0x07));
    }};
}
pub(crate) use x86_fcomip;

macro_rules! x86_fucomi {
    ($inst:expr, $index:expr) => {{
        _bytes_push!($inst, 0xdb as std::ffi::c_uchar);
        _bytes_push!($inst, 0xe8 as std::ffi::c_uchar + ($index & 0x07));
    }};
}
pub(crate) use x86_fucomi;

macro_rules! x86_fucomip {
    ($inst:expr, $index:expr) => {{
        _bytes_push!($inst, 0xdf as std::ffi::c_uchar);
        _bytes_push!($inst, 0xe8 as std::ffi::c_uchar + ($index & 0x07));
    }};
}
pub(crate) use x86_fucomip;

macro_rules! x86_fld {
    ($inst:expr, $mem:expr, $is_double:expr) => {{
        _bytes_push!($inst, if $is_double { 0xdd as std::ffi::c_uchar } else { 0xd9 as std::ffi::c_uchar });
        x86_mem_emit!($inst, 0, $mem);
    }};
}
pub(crate) use x86_fld;

macro_rules! x86_fld_membase {
    ($inst:expr, $basereg:expr, $disp:expr, $is_double:expr) => {{
        _bytes_push!($inst, if $is_double { 0xdd as std::ffi::c_uchar } else { 0xd9 as std::ffi::c_uchar });
        x86_membase_emit!($inst, 0, $basereg, $disp);
    }};
}
pub(crate) use x86_fld_membase;

macro_rules! x86_fld80_mem {
    ($inst:expr, $mem:expr) => {{
        _bytes_push!($inst, 0xdb as std::ffi::c_uchar);
        x86_mem_emit!($inst, 5, $mem);
    }};
}
pub(crate) use x86_fld80_mem;

macro_rules! x86_fld80_membase {
    ($inst:expr, $basereg:expr, $disp:expr) => {{
        _bytes_push!($inst, 0xdb as std::ffi::c_uchar);
        x86_membase_emit!($inst, 5, $basereg, $disp);
    }};
}
pub(crate) use x86_fld80_membase;

macro_rules! x86_fild {
    ($inst:expr, $mem:expr, $is_long:expr) => {{
        if $is_long {
            _bytes_push!($inst, 0xdf as std::ffi::c_uchar);
            x86_mem_emit!($inst, 5, $mem);
        } else {
            _bytes_push!($inst, 0xdb as std::ffi::c_uchar);
            x86_mem_emit!($inst, 0, $mem);
        }
    }};
}
pub(crate) use x86_fild;

macro_rules! x86_fild_membase {
    ($inst:expr, $basereg:expr, $disp:expr, $is_long:expr) => {{
        if $is_long {
            _bytes_push!($inst, 0xdf as std::ffi::c_uchar);
            x86_membase_emit!($inst, 5, $basereg, $disp);
        } else {
            _bytes_push!($inst, 0xdb as std::ffi::c_uchar);
            x86_membase_emit!($inst, 0, $basereg, $disp);
        }
    }};
}
pub(crate) use x86_fild_membase;

macro_rules! x86_fld_reg {
    ($inst:expr, $index:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xc0 as std::ffi::c_uchar + ($index & 0x07));
    }};
}
pub(crate) use x86_fld_reg;

macro_rules! x86_fldz {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xee as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_fldz;

macro_rules! x86_fld1 {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xe8 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_fld1;

macro_rules! x86_fldpi {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xeb as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_fldpi;

macro_rules! x86_fst {
    ($inst:expr, $mem:expr, $is_double:expr, $pop_stack:expr) => {{
        _bytes_push!($inst, if $is_double { 0xdd as std::ffi::c_uchar } else { 0xd9 as std::ffi::c_uchar });
        x86_mem_emit!($inst, 2 + (if $pop_stack { 1 } else { 0 }), $mem);
    }};
}
pub(crate) use x86_fst;

macro_rules! x86_fst_membase {
    ($inst:expr, $basereg:expr, $disp:expr, $is_double:expr, $pop_stack:expr) => {{
        _bytes_push!($inst, if $is_double { 0xdd as std::ffi::c_uchar } else { 0xd9 as std::ffi::c_uchar });
        x86_membase_emit!($inst, 2 + (if $pop_stack { 1 } else { 0 }), $basereg, $disp);
    }};
}
pub(crate) use x86_fst_membase;

macro_rules! x86_fst80_mem {
    ($inst:expr, $mem:expr) => {{
        _bytes_push!($inst, 0xdb as std::ffi::c_uchar);
        x86_mem_emit!($inst, 7, $mem);
    }};
}
pub(crate) use x86_fst80_mem;

macro_rules! x86_fst80_membase {
    ($inst:expr, $basereg:expr, $disp:expr) => {{
        _bytes_push!($inst, 0xdb as std::ffi::c_uchar);
        x86_membase_emit!($inst, 7, $basereg, $disp);
    }};
}
pub(crate) use x86_fst80_membase;

macro_rules! x86_fist_pop {
    ($inst:expr, $mem:expr, $is_long:expr) => {{
        if $is_long {
            _bytes_push!($inst, 0xdf as std::ffi::c_uchar);
            x86_mem_emit!($inst, 7, $mem);
        } else {
            _bytes_push!($inst, 0xdb as std::ffi::c_uchar);
            x86_mem_emit!($inst, 3, $mem);
        }
    }};
}
pub(crate) use x86_fist_pop;

macro_rules! x86_fist_pop_membase {
    ($inst:expr, $basereg:expr, $disp:expr, $is_long:expr) => {{
        if $is_long {
            _bytes_push!($inst, 0xdf as std::ffi::c_uchar);
            x86_membase_emit!($inst, 7, $basereg, $disp);
        } else {
            _bytes_push!($inst, 0xdb as std::ffi::c_uchar);
            x86_membase_emit!($inst, 3, $basereg, $disp);
        }
    }};
}
pub(crate) use x86_fist_pop_membase;

macro_rules! x86_fstsw {
    ($inst:expr) => {{
        _bytes_push!($inst, 0x9b as std::ffi::c_uchar);
        _bytes_push!($inst, 0xdf as std::ffi::c_uchar);
        _bytes_push!($inst, 0xe0 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_fstsw;

/// @x86_fist_membase
/// Converts content of ST(0) to integer and stores it at memory location
/// addressed by [basereg + disp].
/// is_int specifies whether destination is int32 (TRUE) or int16 (FALSE).
macro_rules! x86_fist_membase {
    ($inst:expr, $basereg:expr, $disp:expr, $is_int:expr) => {{
        if $is_int {
            _bytes_push!($inst, 0xdb as std::ffi::c_uchar);
            x86_membase_emit!($inst, 2, $basereg, $disp);
        } else {
            _bytes_push!($inst, 0xdf as std::ffi::c_uchar);
            x86_membase_emit!($inst, 2, $basereg, $disp);
        }
    }};
}
pub(crate) use x86_fist_membase;

macro_rules! x86_push_reg {
    ($inst:expr, $reg:expr) => {{
        _bytes_push!($inst, 0x50 as std::ffi::c_uchar + $reg as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_push_reg;

macro_rules! x86_push_regp {
    ($inst:expr, $reg:expr) => {{
        _bytes_push!($inst, 0xff as std::ffi::c_uchar);
        x86_regp_emit!($inst, 6, $reg);
    }};
}
pub(crate) use x86_push_regp;

macro_rules! x86_push_mem {
    ($inst:expr, $mem:expr) => {{
        _bytes_push!($inst, 0xff as std::ffi::c_uchar);
        x86_mem_emit!($inst, 6, $mem);
    }};
}
pub(crate) use x86_push_mem;

macro_rules! x86_push_membase {
    ($inst:expr, $basereg:expr, $disp:expr) => {{
        _bytes_push!($inst, 0xff as std::ffi::c_uchar);
        x86_membase_emit!($inst, 6, $basereg, $disp);
    }};
}
pub(crate) use x86_push_membase;

macro_rules! x86_push_memindex {
    ($inst:expr, $basereg:expr, $disp:expr, $indexreg:expr, $shift:expr) => {{
        _bytes_push!($inst, 0xff as std::ffi::c_uchar);
        x86_memindex_emit!($inst, 6, $basereg, $disp, $indexreg, $shift);
    }};
}
pub(crate) use x86_push_memindex;

macro_rules! x86_push_imm_template {
    ($inst:expr) => {
        x86_push_imm!($inst, 0xf0f0f0f0_u32);
    };
}
pub(crate) use x86_push_imm_template;

macro_rules! x86_push_imm {
    ($inst:expr, $imm:expr) => {{
        let _imm: std::ffi::c_int = $imm as std::ffi::c_int;
        if x86_is_imm8!(_imm) {
            _bytes_push!($inst, 0x6A as std::ffi::c_uchar);
            x86_imm_emit8!($inst, _imm);
        } else {
            _bytes_push!($inst, 0x68 as std::ffi::c_uchar);
            x86_imm_emit32!($inst, _imm);
        }
    }};
}
pub(crate) use x86_push_imm;

macro_rules! x86_pop_reg {
    ($inst:expr, $reg:expr) => {{
        _bytes_push!($inst, 0x58 as std::ffi::c_uchar + $reg as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_pop_reg;

macro_rules! x86_pop_mem {
    ($inst:expr, $mem:expr) => {{
        _bytes_push!($inst, 0x87 as std::ffi::c_uchar);
        x86_mem_emit!($inst, 0, $mem);
    }};
}
pub(crate) use x86_pop_mem;

macro_rules! x86_pop_membase {
    ($inst:expr, $basereg:expr, $disp:expr) => {{
        _bytes_push!($inst, 0x87 as std::ffi::c_uchar);
        x86_membase_emit!($inst, 0, $basereg, $disp);
    }};
}
pub(crate) use x86_pop_membase;

macro_rules! x86_pushad {
    ($inst:expr) => {{
        _bytes_push!($inst, 0x60 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_pushad;
macro_rules! x86_pushfd {
    ($inst:expr) => {{
        _bytes_push!($inst, 0x9c as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_pushfd;
macro_rules! x86_popad {
    ($inst:expr) => {{
        _bytes_push!($inst, 0x61 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_popad;
macro_rules! x86_popfd {
    ($inst:expr) => {{
        _bytes_push!($inst, 0x9d as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_popfd;

macro_rules! x86_loop {
    ($inst:expr, $imm:expr) => {{
        _bytes_push!($inst, 0xe2 as std::ffi::c_uchar);
        x86_imm_emit8!($inst, $imm);
    }};
}
pub(crate) use x86_loop;

macro_rules! x86_loope {
    ($inst:expr, $imm:expr) => {{
        _bytes_push!($inst, 0xe1 as std::ffi::c_uchar);
        x86_imm_emit8!($inst, $imm);
    }};
}
pub(crate) use x86_loope;

macro_rules! x86_loopne {
    ($inst:expr, $imm:expr) => {{
        _bytes_push!($inst, 0xe0 as std::ffi::c_uchar);
        x86_imm_emit8!($inst, $imm);
    }};
}
pub(crate) use x86_loopne;

macro_rules! x86_jump32 {
    ($inst:expr, $imm:expr) => {{
        _bytes_push!($inst, 0xe9 as std::ffi::c_uchar);
        x86_imm_emit32!($inst, $imm);
    }};
}
pub(crate) use x86_jump32;

macro_rules! x86_jump8 {
    ($inst:expr, $imm:expr) => {{
        _bytes_push!($inst, 0xeb as std::ffi::c_uchar);
        x86_imm_emit8!($inst, $imm);
    }};
}
pub(crate) use x86_jump8;

macro_rules! x86_jump_reg {
    ($inst:expr, $reg:expr) => {{
        _bytes_push!($inst, 0xff as std::ffi::c_uchar);
        x86_reg_emit!($inst, 4, $reg);
    }};
}
pub(crate) use x86_jump_reg;

macro_rules! x86_jump_mem {
    ($inst:expr, $mem:expr) => {{
        _bytes_push!($inst, 0xff as std::ffi::c_uchar);
        x86_mem_emit!($inst, 4, $mem);
    }};
}
pub(crate) use x86_jump_mem;

macro_rules! x86_jump_membase {
    ($inst:expr, $basereg:expr, $disp:expr) => {{
        _bytes_push!($inst, 0xff as std::ffi::c_uchar);
        x86_membase_emit!($inst, 4, $basereg, $disp);
    }};
}
pub(crate) use x86_jump_membase;

// target is a pointer in our buffer.
macro_rules! x86_jump_code {
    ($inst:expr, $target:expr) => {{
        let mut t: std::ffi::c_int = ($target.cast::<std::ffi::c_uchar>().offset_from(*$inst) - 2) as std::ffi::c_int;
        if x86_is_imm8!(t) {
            x86_jump8!($inst, t);
        } else {
            t -= 3;
            x86_jump32!($inst, t);
        }
    }};
}
pub(crate) use x86_jump_code;

macro_rules! x86_jump_disp {
    ($inst:expr, $disp:expr) => {{
        let mut t: std::ffi::c_int = ($disp - 2) as std::ffi::c_int;
        if x86_is_imm8!(t) {
            x86_jump8!($inst, t);
        } else {
            t -= 3;
            x86_jump32!($inst, t);
        }
    }};
}
pub(crate) use x86_jump_disp;

macro_rules! x86_branch8 {
    ($inst:expr, $cond:expr, $imm:expr, $is_signed:expr) => {{
        if $is_signed {
            _bytes_push!($inst, x86_cc_signed_map[$cond as usize]);
        } else {
            _bytes_push!($inst, x86_cc_unsigned_map[$cond as usize]);
        }
        x86_imm_emit8!($inst, $imm);
    }};
}
pub(crate) use x86_branch8;

macro_rules! x86_branch32 {
    ($inst:expr, $cond:expr, $imm:expr, $is_signed:expr) => {{
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        if $is_signed {
            _bytes_push!($inst, x86_cc_signed_map[$cond as usize] + 0x10);
        } else {
            _bytes_push!($inst, x86_cc_unsigned_map[$cond as usize] + 0x10);
        }
        x86_imm_emit32!($inst, $imm);
    }};
}
pub(crate) use x86_branch32;

macro_rules! x86_branch {
    ($inst:expr, $cond:expr, $target:expr, $is_signed:expr) => {{
        let mut offset: std::ffi::c_int = ($target.offset_from(*$inst) - 2) as std::ffi::c_int;
        if x86_is_imm8!(offset) {
            x86_branch8!($inst, $cond, offset, $is_signed);
        } else {
            offset -= 4;
            x86_branch32!($inst, $cond, offset, $is_signed);
        }
    }};
}
pub(crate) use x86_branch;

macro_rules! x86_branch_disp {
    ($inst:expr, $cond:expr, $disp:expr, $is_signed:expr) => {{
        let mut offset: std::ffi::c_int = $disp - 2;
        if x86_is_imm8!(offset) {
            x86_branch8!($inst, $cond, offset, $is_signed);
        } else {
            offset -= 4;
            x86_branch32!($inst, $cond, offset, $is_signed);
        }
    }};
}
pub(crate) use x86_branch_disp;

macro_rules! x86_set_reg {
    ($inst:expr, $cond:expr, $reg:expr, $is_signed:expr) => {{
        assert!(X86_IS_BYTE_REG!($reg));
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        if $is_signed {
            _bytes_push!($inst, x86_cc_signed_map[$cond as usize] + 0x20);
        } else {
            _bytes_push!($inst, x86_cc_unsigned_map[$cond as usize] + 0x20);
        }
        x86_reg_emit!($inst, 0, $reg);
    }};
}
pub(crate) use x86_set_reg;

macro_rules! x86_set_mem {
    ($inst:expr, $cond:expr, $mem:expr, $is_signed:expr) => {{
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        if $is_signed {
            _bytes_push!($inst, x86_cc_signed_map[$cond as usize] + 0x20);
        } else {
            _bytes_push!($inst, x86_cc_unsigned_map[$cond as usize] + 0x20);
        }
        x86_mem_emit!($inst, 0, $mem);
    }};
}
pub(crate) use x86_set_mem;

macro_rules! x86_set_membase {
    ($inst:expr, $cond:expr, $basereg:expr, $disp:expr, $is_signed:expr) => {{
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        if $is_signed {
            _bytes_push!($inst, x86_cc_signed_map[$cond as usize] + 0x20);
        } else {
            _bytes_push!($inst, x86_cc_unsigned_map[$cond as usize] + 0x20);
        }
        x86_membase_emit!($inst, 0, $basereg, $disp);
    }};
}
pub(crate) use x86_set_membase;

macro_rules! x86_call_imm {
    ($inst:expr, $disp:expr) => {{
        _bytes_push!($inst, 0xe8 as std::ffi::c_uchar);
        x86_imm_emit32!($inst, $disp as std::ffi::c_int);
    }};
}
pub(crate) use x86_call_imm;

macro_rules! x86_call_reg {
    ($inst:expr, $reg:expr) => {{
        _bytes_push!($inst, 0xff as std::ffi::c_uchar);
        x86_reg_emit!($inst, 2, $reg);
    }};
}
pub(crate) use x86_call_reg;

macro_rules! x86_call_mem {
    ($inst:expr, $mem:expr) => {{
        _bytes_push!($inst, 0xff as std::ffi::c_uchar);
        x86_mem_emit!($inst, 2, $mem);
    }};
}
pub(crate) use x86_call_mem;

macro_rules! x86_call_membase {
    ($inst:expr, $basereg:expr, $disp:expr) => {{
        _bytes_push!($inst, 0xff as std::ffi::c_uchar);
        x86_membase_emit!($inst, 2, $basereg, $disp);
    }};
}
pub(crate) use x86_call_membase;

macro_rules! x86_call_code {
    ($inst:expr, $target:expr) => {{
        let mut _x86_offset: std::ffi::c_int = $target.cast::<std::ffi::c_uchar>().offset_from(*$inst) as std::ffi::c_int;
        _x86_offset -= 5;
        x86_call_imm!($inst, _x86_offset);
    }};
}
pub(crate) use x86_call_code;

macro_rules! x86_ret {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xc3 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_ret;

macro_rules! x86_ret_imm {
    ($inst:expr, $imm:expr) => {{
        if $imm == 0 {
            x86_ret!($inst);
        } else {
            _bytes_push!($inst, 0xc2 as std::ffi::c_uchar);
            x86_imm_emit16!($inst, $imm);
        }
    }};
}
pub(crate) use x86_ret_imm;

macro_rules! x86_cmov_reg {
    ($inst:expr, $cond:expr, $is_signed:expr, $dreg:expr, $reg:expr) => {{
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        if $is_signed {
            _bytes_push!($inst, x86_cc_signed_map[$cond as usize] - 0x30);
        } else {
            _bytes_push!($inst, x86_cc_unsigned_map[$cond as usize] - 0x30);
        }
        x86_reg_emit!($inst, $dreg, $reg);
    }};
}
pub(crate) use x86_cmov_reg;

macro_rules! x86_cmov_mem {
    ($inst:expr, $cond:expr, $is_signed:expr, $reg:expr, $mem:expr) => {{
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        if $is_signed {
            _bytes_push!($inst, x86_cc_signed_map[$cond as usize] - 0x30);
        } else {
            _bytes_push!($inst, x86_cc_unsigned_map[$cond as usize] - 0x30);
        }
        x86_mem_emit!($inst, $reg, $mem);
    }};
}
pub(crate) use x86_cmov_mem;

macro_rules! x86_cmov_membase {
    ($inst:expr, $cond:expr, $is_signed:expr, $reg:expr, $basereg:expr, $disp:expr) => {{
        _bytes_push!($inst, 0x0f as std::ffi::c_uchar);
        if $is_signed {
            _bytes_push!($inst, x86_cc_signed_map[$cond as usize] - 0x30);
        } else {
            _bytes_push!($inst, x86_cc_unsigned_map[$cond as usize] - 0x30);
        }
        x86_membase_emit!($inst, $reg, $basereg, $disp);
    }};
}
pub(crate) use x86_cmov_membase;

macro_rules! x86_enter {
    ($inst:expr, $framesize:expr) => {{
        _bytes_push!($inst, 0xc8 as std::ffi::c_uchar);
        x86_imm_emit16!($inst, $framesize);
        _bytes_push!($inst, 0);
    }};
}
pub(crate) use x86_enter;

macro_rules! x86_leave {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xc9 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_leave;
macro_rules! x86_sahf {
    // FIXME overrides earlier version
    ($inst:expr) => {{
        _bytes_push!($inst, 0x9e as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_sahf;

macro_rules! x86_fsin {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xfe as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_fsin;
macro_rules! x86_fcos {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xff as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_fcos;
macro_rules! x86_fabs {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xe1 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_fabs;
macro_rules! x86_ftst {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xe4 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_ftst;
macro_rules! x86_fxam {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xe5 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_fxam;
macro_rules! x86_fpatan {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xf3 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_fpatan;
macro_rules! x86_fprem {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xf8 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_fprem;
macro_rules! x86_fprem1 {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xf5 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_fprem1;
macro_rules! x86_frndint {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xfc as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_frndint;
macro_rules! x86_fsqrt {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xfa as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_fsqrt;
macro_rules! x86_fptan {
    ($inst:expr) => {{
        _bytes_push!($inst, 0xd9 as std::ffi::c_uchar);
        _bytes_push!($inst, 0xf2 as std::ffi::c_uchar);
    }};
}
pub(crate) use x86_fptan;

macro_rules! x86_padding {
    ($inst:expr, $size:expr) => {{
        match $size {
            1 => x86_nop!($inst),
            2 => {
                _bytes_push!($inst, 0x8b);
                _bytes_push!($inst, 0xc0);
            }
            3 => {
                _bytes_push!($inst, 0x8d);
                _bytes_push!($inst, 0x6d);
                _bytes_push!($inst, 0x00);
            }
            4 => {
                _bytes_push!($inst, 0x8d);
                _bytes_push!($inst, 0x64);
                _bytes_push!($inst, 0x24);
                _bytes_push!($inst, 0x00);
            }
            5 => {
                _bytes_push!($inst, 0x8d);
                _bytes_push!($inst, 0x64);
                _bytes_push!($inst, 0x24);
                _bytes_push!($inst, 0x00);
                x86_nop!($inst);
            }
            6 => {
                _bytes_push!($inst, 0x8d);
                _bytes_push!($inst, 0xad);
                _bytes_push!($inst, 0x00);
                _bytes_push!($inst, 0x00);
                _bytes_push!($inst, 0x00);
                _bytes_push!($inst, 0x00);
            }
            7 => {
                _bytes_push!($inst, 0x8d);
                _bytes_push!($inst, 0xa4);
                _bytes_push!($inst, 0x24);
                _bytes_push!($inst, 0x00);
                _bytes_push!($inst, 0x00);
                _bytes_push!($inst, 0x00);
                _bytes_push!($inst, 0x00);
            }
            x => panic!("x86_padding::{}", x),
        }
    }};
}
pub(crate) use x86_padding;

macro_rules! x86_prolog {
    ($inst:expr, $frame_size:expr, $reg_mask:expr) => {{
        let mut m: std::ffi::c_uint = 1;
        x86_enter!($inst, $frame_size);
        for i in 0..X86_NREG as std::ffi::c_uint {
            if ($reg_mask & m) != 0 {
                x86_push_reg!($inst, i);
            }
            m <<= 1;
        }
    }};
}
pub(crate) use x86_prolog;

macro_rules! x86_epilog {
    ($inst:expr, $reg_mask:expr) => {{
        let mut i: std::ffi::c_uint = X86_EDI as std::ffi::c_uint;
        let mut m: std::ffi::c_uint = 1 << X86_EDI;
        while m != 0 {
            if ($reg_mask & m) != 0 {
                x86_pop_reg!($inst, i);
            }
            i -= 1;
            m >>= 1;
        }
        x86_leave!($inst);
        x86_ret!($inst);
    }};
}
pub(crate) use x86_epilog;

#[cfg(feature = "USE_UNSTABLE")]
#[inline]
#[no_mangle]
pub unsafe extern "C" fn x86_jump_code_fn(instp: *mut *mut u_char, target: *mut u_char) {
    x86_jump_code!(&mut *instp, target);
}

#[cfg(feature = "USE_UNSTABLE")]
#[inline]
#[no_mangle]
pub unsafe extern "C" fn x86_patch_fn(instp: *mut u_char, target: *mut u_char) {
    x86_patch!(instp, target);
}
