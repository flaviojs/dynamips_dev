//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! MIPS64 Step-by-step execution.

use crate::_private::*;
use crate::dynamips_common::*;
use crate::mips64::*;
use crate::utils::*;

extern "C" {
    fn mips64_cp0_exec_cfc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int);
    fn mips64_cp0_exec_ctc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int);
    fn mips64_cp0_exec_dmfc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int);
    fn mips64_cp0_exec_dmtc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int);
    fn mips64_cp0_exec_mfc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int);
    fn mips64_cp0_exec_mtc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int);
    fn mips64_cp0_exec_tlbp(cpu: *mut cpu_mips_t);
    fn mips64_cp0_exec_tlbr(cpu: *mut cpu_mips_t);
    fn mips64_cp0_exec_tlbwi(cpu: *mut cpu_mips_t);
    fn mips64_cp0_exec_tlbwr(cpu: *mut cpu_mips_t);
    fn mips64_exec_bdslot(cpu: *mut cpu_mips_t);
    fn mips64_exec_break(cpu: *mut cpu_mips_t, code: u_int);
    fn mips64_exec_dmfc1(cpu: *mut cpu_mips_t, gp_reg: u_int, cp1_reg: u_int);
    fn mips64_exec_dmtc1(cpu: *mut cpu_mips_t, gp_reg: u_int, cp1_reg: u_int);
    fn mips64_exec_eret(cpu: *mut cpu_mips_t);
    fn mips64_exec_mfc1(cpu: *mut cpu_mips_t, gp_reg: u_int, cp1_reg: u_int);
    fn mips64_exec_mtc1(cpu: *mut cpu_mips_t, gp_reg: u_int, cp1_reg: u_int);
    fn mips64_exec_syscall(cpu: *mut cpu_mips_t);
    fn mips64_trigger_trap_exception(cpu: *mut cpu_mips_t);
}

#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub type mips64_insn_exec_tag_exec = Option<unsafe extern "C" fn(_: *mut cpu_mips_t, _: mips_insn_t) -> c_int>;

/// MIPS instruction recognition
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mips64_insn_exec_tag {
    pub name: *mut c_char,
    pub exec: mips64_insn_exec_tag_exec,
    pub mask: m_uint32_t,
    pub value: m_uint32_t,
    pub delay_slot: c_int,
    pub instr_type: c_int,
    pub count: m_uint64_t,
}
impl mips64_insn_exec_tag {
    pub const fn new(name: *mut c_char, exec: mips64_insn_exec_tag_exec, mask: m_uint32_t, value: m_uint32_t, delay_slot: c_int, instr_type: c_int) -> Self {
        Self { name, exec, mask, value, delay_slot, instr_type, count: 0 }
    }
    pub const fn null() -> Self {
        Self { name: null_mut(), exec: None, mask: 0x00000000, value: 0x00000000, delay_slot: 1, instr_type: 0, count: 0 }
    }
}

/// Execute a memory operation (2)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
#[inline(always)]
pub unsafe extern "C" fn mips64_exec_memop2(cpu: *mut cpu_mips_t, memop: c_int, base: m_uint64_t, offset: c_int, dst_reg: u_int, keep_ll_bit: c_int) {
    let vaddr: m_uint64_t = (*cpu).gpr[base as usize].wrapping_add_signed(sign_extend(offset as m_int64_t, 16));

    if keep_ll_bit == 0 {
        (*cpu).ll_bit = 0;
    }
    let fn_: mips_memop_fn = (*cpu).mem_op_fn[memop as usize];
    fn_.unwrap()(cpu, vaddr, dst_reg);
}

/// ADD
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_ADD(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    // TODO: Exception handling
    let res: m_uint64_t = ((*cpu).gpr[rs as usize] as m_uint32_t).wrapping_add((*cpu).gpr[rt as usize] as m_uint32_t) as m_uint64_t;
    (*cpu).gpr[rd as usize] = sign_extend(res as m_int64_t, 32) as m_uint64_t;
    0
}

/// ADDI
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_ADDI(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let imm: c_int = bits(insn, 0, 15);
    let val: m_uint32_t = sign_extend(imm as m_int64_t, 16) as m_uint32_t;

    // TODO: Exception handling
    let res: m_uint32_t = ((*cpu).gpr[rs as usize] as m_uint32_t).wrapping_add(val);
    (*cpu).gpr[rt as usize] = sign_extend(res as m_int64_t, 32) as m_uint64_t;
    0
}

/// ADDIU
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_ADDIU(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let imm: c_int = bits(insn, 0, 15);
    let val: m_uint32_t = sign_extend(imm as m_int64_t, 16) as m_uint32_t;

    let res: m_uint32_t = ((*cpu).gpr[rs as usize] as m_uint32_t).wrapping_add(val);
    (*cpu).gpr[rt as usize] = sign_extend(res as m_int64_t, 32) as m_uint64_t;
    0
}

/// ADDU
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_ADDU(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    let res: m_uint32_t = ((*cpu).gpr[rs as usize] as m_uint32_t).wrapping_add((*cpu).gpr[rt as usize] as m_uint32_t);
    (*cpu).gpr[rd as usize] = sign_extend(res as m_int64_t, 32) as m_uint64_t;
    0
}

/// AND
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_AND(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    (*cpu).gpr[rd as usize] = (*cpu).gpr[rs as usize] & (*cpu).gpr[rt as usize];
    0
}

/// ANDI
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_ANDI(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let imm: c_int = bits(insn, 0, 15);

    (*cpu).gpr[rt as usize] = (*cpu).gpr[rs as usize] & imm as m_uint64_t;
    0
}

/// B (Branch, virtual instruction)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_B(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // exec the instruction in the delay slot
    mips64_exec_bdslot(cpu);

    // set the new pc in cpu structure
    (*cpu).pc = new_pc;
    1
}

/// BAL (Branch And Link, virtual instruction)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BAL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // set the return address (instruction after the delay slot)
    (*cpu).gpr[MIPS_GPR_RA] = (*cpu).pc + 8;

    // exec the instruction in the delay slot
    mips64_exec_bdslot(cpu);

    // set the new pc in cpu structure
    (*cpu).pc = new_pc;
    1
}

/// BEQ (Branch On Equal)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BEQ(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // take the branch if gpr[rs] == gpr[rt]
    let res: bool = (*cpu).gpr[rs as usize] == ((*cpu).gpr[rt as usize]);

    // exec the instruction in the delay slot
    mips64_exec_bdslot(cpu);

    // take the branch if the test result is true
    if res {
        (*cpu).pc = new_pc;
    } else {
        (*cpu).pc += 8;
    }

    1
}

/// BEQL (Branch On Equal Likely)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BEQL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // take the branch if gpr[rs] == gpr[rt]
    let res: bool = (*cpu).gpr[rs as usize] == (*cpu).gpr[rt as usize];

    // take the branch if the test result is true
    if res {
        mips64_exec_bdslot(cpu);
        (*cpu).pc = new_pc;
    } else {
        (*cpu).pc += 8;
    }

    1
}

/// BEQZ (Branch On Equal Zero) - Virtual Instruction
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BEQZ(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // take the branch if gpr[rs] == 0
    let res: bool = (*cpu).gpr[rs as usize] == 0;

    // exec the instruction in the delay slot
    mips64_exec_bdslot(cpu);

    // take the branch if the test result is true
    if res {
        (*cpu).pc = new_pc;
    } else {
        (*cpu).pc += 8;
    }

    1
}

/// BNEZ (Branch On Not Equal Zero) - Virtual Instruction
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BNEZ(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // take the branch if gpr[rs] != 0
    let res: bool = (*cpu).gpr[rs as usize] != 0;

    // exec the instruction in the delay slot
    mips64_exec_bdslot(cpu);

    // take the branch if the test result is true
    if res {
        (*cpu).pc = new_pc;
    } else {
        (*cpu).pc += 8;
    }

    1
}

/// BGEZ (Branch On Greater or Equal Than Zero)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BGEZ(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // take the branch if gpr[rs] >= 0
    let res: bool = (*cpu).gpr[rs as usize] as m_int64_t >= 0;

    // exec the instruction in the delay slot
    mips64_exec_bdslot(cpu);

    // take the branch if the test result is true
    if res {
        (*cpu).pc = new_pc;
    } else {
        (*cpu).pc += 8;
    }

    1
}

/// BGEZAL (Branch On Greater or Equal Than Zero And Link)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BGEZAL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // set the return address (instruction after the delay slot)
    (*cpu).gpr[MIPS_GPR_RA] = (*cpu).pc + 8;

    // take the branch if gpr[rs] >= 0
    let res: bool = (*cpu).gpr[rs as usize] as m_int64_t >= 0;

    // exec the instruction in the delay slot
    mips64_exec_bdslot(cpu);

    // take the branch if the test result is true
    if res {
        (*cpu).pc = new_pc;
    } else {
        (*cpu).pc += 8;
    }

    1
}

/// BGEZALL (Branch On Greater or Equal Than Zero And Link Likely)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BGEZALL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // set the return address (instruction after the delay slot)
    (*cpu).gpr[MIPS_GPR_RA] = (*cpu).pc + 8;

    // take the branch if gpr[rs] >= 0
    let res: bool = (*cpu).gpr[rs as usize] as m_int64_t >= 0;

    // take the branch if the test result is true
    if res {
        mips64_exec_bdslot(cpu);
        (*cpu).pc = new_pc;
    } else {
        (*cpu).pc += 8;
    }

    1
}

/// BGEZL (Branch On Greater or Equal Than Zero Likely)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BGEZL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // take the branch if gpr[rs] >= 0 */
    let res: bool = (*cpu).gpr[rs as usize] as m_int64_t >= 0;

    // take the branch if the test result is true
    if res {
        mips64_exec_bdslot(cpu);
        (*cpu).pc = new_pc;
    } else {
        (*cpu).pc += 8;
    }

    1
}

/// BGTZ (Branch On Greater Than Zero)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BGTZ(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // take the branch if gpr[rs] > 0
    let res: bool = (*cpu).gpr[rs as usize] as m_int64_t > 0;

    // exec the instruction in the delay slot
    mips64_exec_bdslot(cpu);

    // take the branch if the test result is true
    if res {
        (*cpu).pc = new_pc;
    } else {
        (*cpu).pc += 8;
    }

    1
}

/// BGTZL (Branch On Greater Than Zero Likely)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BGTZL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // take the branch if gpr[rs] > 0
    let res: bool = (*cpu).gpr[rs as usize] as m_int64_t > 0;

    // take the branch if the test result is true
    if res {
        mips64_exec_bdslot(cpu);
        (*cpu).pc = new_pc;
    } else {
        (*cpu).pc += 8;
    }

    1
}

/// BLEZ (Branch On Less or Equal Than Zero)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BLEZ(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // take the branch if gpr[rs] <= 0
    let res: bool = (*cpu).gpr[rs as usize] as m_int64_t <= 0;

    // exec the instruction in the delay slot
    mips64_exec_bdslot(cpu);

    // take the branch if the test result is true
    if res {
        (*cpu).pc = new_pc;
    } else {
        (*cpu).pc += 8;
    }

    1
}

// BLEZL (Branch On Less or Equal Than Zero Likely)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BLEZL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // take the branch if gpr[rs] <= 0
    let res: bool = (*cpu).gpr[rs as usize] as m_int64_t <= 0;

    // take the branch if the test result is true
    if res {
        mips64_exec_bdslot(cpu);
        (*cpu).pc = new_pc;
    } else {
        (*cpu).pc += 8;
    }

    1
}

/// BLTZ (Branch On Less Than Zero)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BLTZ(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // take the branch if gpr[rs] < 0
    let res: bool = ((*cpu).gpr[rs as usize] as m_int64_t) < 0;

    // exec the instruction in the delay slot
    mips64_exec_bdslot(cpu);

    // take the branch if the test result is true
    if res {
        (*cpu).pc = new_pc;
    } else {
        (*cpu).pc += 8;
    }

    1
}

/// BLTZAL (Branch On Less Than Zero And Link)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BLTZAL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // set the return address (instruction after the delay slot)
    (*cpu).gpr[MIPS_GPR_RA] = (*cpu).pc + 8;

    // take the branch if gpr[rs] < 0
    let res: bool = ((*cpu).gpr[rs as usize] as m_int64_t) < 0;

    // exec the instruction in the delay slot
    mips64_exec_bdslot(cpu);

    // take the branch if the test result is true
    if res {
        (*cpu).pc = new_pc;
    } else {
        (*cpu).pc += 8;
    }

    1
}

/// BLTZALL (Branch On Less Than Zero And Link Likely)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BLTZALL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // set the return address (instruction after the delay slot)
    (*cpu).gpr[MIPS_GPR_RA] = (*cpu).pc + 8;

    // take the branch if gpr[rs] < 0
    let res: bool = ((*cpu).gpr[rs as usize] as m_int64_t) < 0;

    // take the branch if the test result is true
    if res {
        mips64_exec_bdslot(cpu);
        (*cpu).pc = new_pc;
    } else {
        (*cpu).pc += 8;
    }

    1
}

/// BLTZL (Branch On Less Than Zero Likely)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BLTZL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // take the branch if gpr[rs] < 0
    let res: bool = ((*cpu).gpr[rs as usize] as m_int64_t) < 0;

    // take the branch if the test result is true
    if res {
        mips64_exec_bdslot(cpu);
        (*cpu).pc = new_pc;
    } else {
        (*cpu).pc += 8;
    }

    1
}

/// BNE (Branch On Not Equal)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BNE(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // take the branch if gpr[rs] != gpr[rt]
    let res: bool = (*cpu).gpr[rs as usize] != (*cpu).gpr[rt as usize];

    // exec the instruction in the delay slot
    mips64_exec_bdslot(cpu);

    // take the branch if the test result is true
    if res {
        (*cpu).pc = new_pc;
    } else {
        (*cpu).pc += 8;
    }

    1
}

/// BNEL (Branch On Not Equal Likely)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BNEL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // take the branch if gpr[rs] != gpr[rt]
    let res: bool = (*cpu).gpr[rs as usize] != (*cpu).gpr[rt as usize];

    // take the branch if the test result is true
    if res {
        mips64_exec_bdslot(cpu);
        (*cpu).pc = new_pc;
    } else {
        (*cpu).pc += 8;
    }

    1
}

/// BREAK
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_BREAK(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let code: u_int = bits(insn, 6, 25) as u_int;

    mips64_exec_break(cpu, code);
    1
}

/// CACHE
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_CACHE(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let op: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_CACHE as c_int, base as m_uint64_t, offset, op as u_int, FALSE);
    0
}

/// CFC0
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_CFC0(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    mips64_cp0_exec_cfc0(cpu, rt as u_int, rd as u_int);
    0
}

/// CTC0
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_CTC0(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    mips64_cp0_exec_ctc0(cpu, rt as u_int, rd as u_int);
    0
}

/// DADDIU
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_DADDIU(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let imm: c_int = bits(insn, 0, 15);
    let val: m_uint64_t = sign_extend(imm as m_int64_t, 16) as m_uint64_t;

    (*cpu).gpr[rt as usize] = (*cpu).gpr[rs as usize].wrapping_add(val);
    0
}

/// DADDU: rd = rs + rt
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_DADDU(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    (*cpu).gpr[rd as usize] = (*cpu).gpr[rs as usize].wrapping_add((*cpu).gpr[rt as usize]);
    0
}

/// DIV
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_DIV(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);

    (*cpu).lo = (((*cpu).gpr[rs as usize] as m_int32_t) / ((*cpu).gpr[rt as usize] as m_int32_t)) as m_uint64_t;
    (*cpu).hi = (((*cpu).gpr[rs as usize] as m_int32_t) % ((*cpu).gpr[rt as usize] as m_int32_t)) as m_uint64_t;

    (*cpu).lo = sign_extend((*cpu).lo as m_int64_t, 32) as m_uint64_t;
    (*cpu).hi = sign_extend((*cpu).hi as m_int64_t, 32) as m_uint64_t;
    0
}

/// DIVU
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_DIVU(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);

    if (*cpu).gpr[rt as usize] == 0 {
        return 0;
    }

    (*cpu).lo = (((*cpu).gpr[rs as usize] as m_uint32_t) / ((*cpu).gpr[rt as usize] as m_uint32_t)) as m_uint64_t;
    (*cpu).hi = (((*cpu).gpr[rs as usize] as m_uint32_t) % ((*cpu).gpr[rt as usize] as m_uint32_t)) as m_uint64_t;

    (*cpu).lo = sign_extend((*cpu).lo as m_int64_t, 32) as m_uint64_t;
    (*cpu).hi = sign_extend((*cpu).hi as m_int64_t, 32) as m_uint64_t;
    0
}

/// DMFC0
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_DMFC0(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    mips64_cp0_exec_dmfc0(cpu, rt as u_int, rd as u_int);
    0
}

/// DMFC1
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_DMFC1(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    mips64_exec_dmfc1(cpu, rt as u_int, rd as u_int);
    0
}

/// DMTC0
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_DMTC0(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    mips64_cp0_exec_dmtc0(cpu, rt as u_int, rd as u_int);
    0
}

/// DMTC1
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_DMTC1(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    mips64_exec_dmtc1(cpu, rt as u_int, rd as u_int);
    0
}

/// DSLL
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_DSLL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);
    let sa: c_int = bits(insn, 6, 10);

    (*cpu).gpr[rd as usize] = (*cpu).gpr[rt as usize] << sa;
    0
}

/// DSLL32
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_DSLL32(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);
    let sa: c_int = bits(insn, 6, 10);

    (*cpu).gpr[rd as usize] = (*cpu).gpr[rt as usize] << (32 + sa);
    0
}

/// DSLLV
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_DSLLV(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    (*cpu).gpr[rd as usize] = (*cpu).gpr[rt as usize] << ((*cpu).gpr[rs as usize] & 0x3f);
    0
}

/// DSRA
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_DSRA(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);
    let sa: c_int = bits(insn, 6, 10);

    (*cpu).gpr[rd as usize] = ((*cpu).gpr[rt as usize] as m_int64_t >> sa) as m_uint64_t;
    0
}

/// DSRA32
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_DSRA32(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);
    let sa: c_int = bits(insn, 6, 10);

    (*cpu).gpr[rd as usize] = ((*cpu).gpr[rt as usize] as m_int64_t >> (32 + sa)) as m_uint64_t;
    0
}

/// DSRAV
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_DSRAV(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    (*cpu).gpr[rd as usize] = ((*cpu).gpr[rt as usize] as m_int64_t >> ((*cpu).gpr[rs as usize] & 0x3f)) as m_uint64_t;
    0
}

/// DSRL
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_DSRL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);
    let sa: c_int = bits(insn, 6, 10);

    (*cpu).gpr[rd as usize] = (*cpu).gpr[rt as usize] >> sa;
    0
}

/// DSRL32
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_DSRL32(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);
    let sa: c_int = bits(insn, 6, 10);

    (*cpu).gpr[rd as usize] = (*cpu).gpr[rt as usize] >> (32 + sa);
    0
}

/// DSRLV
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_DSRLV(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    (*cpu).gpr[rd as usize] = (*cpu).gpr[rt as usize] >> ((*cpu).gpr[rs as usize] & 0x3f);
    0
}

/// DSUBU
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_DSUBU(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    (*cpu).gpr[rd as usize] = (*cpu).gpr[rs as usize].wrapping_sub((*cpu).gpr[rt as usize]);
    0
}

/// ERET
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_ERET(cpu: *mut cpu_mips_t, _insn: mips_insn_t) -> c_int {
    mips64_exec_eret(cpu);
    1
}

/// J
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_J(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let instr_index: u_int = bits(insn, 0, 25) as u_int;

    /* compute the new pc */
    let mut new_pc: m_uint64_t = (*cpu).pc & !((1 << 28) - 1);
    new_pc |= (instr_index << 2) as m_uint64_t;

    // exec the instruction in the delay slot
    mips64_exec_bdslot(cpu);

    // set the new pc
    (*cpu).pc = new_pc;
    1
}

/// JAL
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_JAL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let instr_index: u_int = bits(insn, 0, 25) as u_int;

    // compute the new pc
    let mut new_pc: m_uint64_t = (*cpu).pc & !((1 << 28) - 1);
    new_pc |= (instr_index << 2) as m_uint64_t;

    // set the return address (instruction after the delay slot)
    (*cpu).gpr[MIPS_GPR_RA] = (*cpu).pc + 8;

    // exec the instruction in the delay slot
    mips64_exec_bdslot(cpu);

    // set the new pc
    (*cpu).pc = new_pc;
    1
}

/// JALR
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_JALR(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rd: c_int = bits(insn, 11, 15);

    // set the return pc (instruction after the delay slot) in GPR[rd]
    (*cpu).gpr[rd as usize] = (*cpu).pc + 8;

    // get the new pc
    let new_pc: m_uint64_t = (*cpu).gpr[rs as usize];

    // exec the instruction in the delay slot
    mips64_exec_bdslot(cpu);

    // set the new pc
    (*cpu).pc = new_pc;
    1
}

/// JR
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_JR(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);

    // get the new pc
    let new_pc: m_uint64_t = (*cpu).gpr[rs as usize];

    // exec the instruction in the delay slot
    mips64_exec_bdslot(cpu);

    // set the new pc
    (*cpu).pc = new_pc;
    1
}

/// LB (Load Byte)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_LB(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_LB as c_int, base as m_uint64_t, offset, rt as u_int, TRUE);
    0
}

/// LBU (Load Byte Unsigned)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_LBU(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_LBU as c_int, base as m_uint64_t, offset, rt as u_int, TRUE);
    0
}

/// LD (Load Double-Word)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_LD(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_LD as c_int, base as m_uint64_t, offset, rt as u_int, TRUE);
    0
}

/// LDC1 (Load Double-Word to Coprocessor 1)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_LDC1(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let ft: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_LDC1 as c_int, base as m_uint64_t, offset, ft as u_int, TRUE);
    0
}

/// LDL (Load Double-Word Left)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_LDL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_LDL as c_int, base as m_uint64_t, offset, rt as u_int, TRUE);
    0
}

/// LDR (Load Double-Word Right)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_LDR(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_LDR as c_int, base as m_uint64_t, offset, rt as u_int, TRUE);
    0
}

/// LH (Load Half-Word)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_LH(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_LH as c_int, base as m_uint64_t, offset, rt as u_int, TRUE);
    0
}

/// LHU (Load Half-Word Unsigned)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_LHU(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_LHU as c_int, base as m_uint64_t, offset, rt as u_int, TRUE);
    0
}

/// LI (virtual)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_LI(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let imm: c_int = bits(insn, 0, 15);

    (*cpu).gpr[rt as usize] = sign_extend(imm as m_int64_t, 16) as m_uint64_t;
    0
}

/// LL (Load Linked)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_LL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_LL as c_int, base as m_uint64_t, offset, rt as u_int, TRUE);
    0
}

/// LUI
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_LUI(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let imm: c_int = bits(insn, 0, 15);

    (*cpu).gpr[rt as usize] = (sign_extend(imm as m_int64_t, 16) << 16) as m_uint64_t;
    0
}

/// LW (Load Word)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_LW(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_LW as c_int, base as m_uint64_t, offset, rt as u_int, TRUE);
    0
}

/// LWL (Load Word Left)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_LWL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_LWL as c_int, base as m_uint64_t, offset, rt as u_int, TRUE);
    0
}

/// LWR (Load Word Right)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_LWR(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_LWR as c_int, base as m_uint64_t, offset, rt as u_int, TRUE);
    0
}

/// LWU (Load Word Unsigned)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_LWU(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_LWU as c_int, base as m_uint64_t, offset, rt as u_int, TRUE);
    0
}

/// MFC0
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_MFC0(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    mips64_cp0_exec_mfc0(cpu, rt as u_int, rd as u_int);
    0
}

/// MFC1
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_MFC1(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    mips64_exec_mfc1(cpu, rt as u_int, rd as u_int);
    0
}

/// MFHI
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_MFHI(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rd: c_int = bits(insn, 11, 15);

    if rd != 0 {
        (*cpu).gpr[rd as usize] = (*cpu).hi;
    }
    0
}

/// MFLO
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_MFLO(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rd: c_int = bits(insn, 11, 15);

    if rd != 0 {
        (*cpu).gpr[rd as usize] = (*cpu).lo;
    }
    0
}

/// MOVE (virtual instruction, real: ADDU)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_MOVE(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rd: c_int = bits(insn, 11, 15);

    (*cpu).gpr[rd as usize] = sign_extend((*cpu).gpr[rs as usize] as m_int64_t, 32) as m_uint64_t;
    0
}

/// MOVZ
#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_MOVZ(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    if (*cpu).gpr[rt as usize] != 0 {
        (*cpu).gpr[rd as usize] = (*cpu).gpr[rs as usize];
    }

    0
}

/// MTC0
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_MTC0(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    mips64_cp0_exec_mtc0(cpu, rt as u_int, rd as u_int);
    0
}

/// MTC1
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_MTC1(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    mips64_exec_mtc1(cpu, rt as u_int, rd as u_int);
    0
}

/// MTHI
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_MTHI(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);

    (*cpu).hi = (*cpu).gpr[rs as usize];
    0
}

/// MTLO
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_MTLO(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);

    (*cpu).lo = (*cpu).gpr[rs as usize];
    0
}

/// MUL
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_MUL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    // note: after this instruction, HI/LO regs are undefined
    let val: m_int32_t = (*cpu).gpr[rs as usize] as m_int32_t * (*cpu).gpr[rt as usize] as m_int32_t;
    (*cpu).gpr[rd as usize] = sign_extend(val as m_int64_t, 32) as m_uint64_t;
    0
}

/// MULT
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_MULT(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);

    let mut val: m_int64_t = (*cpu).gpr[rs as usize] as m_int32_t as m_int64_t;
    val *= (*cpu).gpr[rt as usize] as m_int32_t as m_int64_t;

    (*cpu).lo = sign_extend(val, 32) as m_uint64_t;
    (*cpu).hi = sign_extend(val >> 32, 32) as m_uint64_t;
    0
}

/// MULTU
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_MULTU(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);

    let mut val: m_uint64_t = (*cpu).gpr[rs as usize] as m_uint32_t as m_uint64_t;
    val *= (*cpu).gpr[rt as usize] as m_uint32_t as m_uint64_t;
    (*cpu).lo = sign_extend(val as m_int64_t, 32) as m_uint64_t;
    (*cpu).hi = sign_extend((val >> 32) as m_int64_t, 32) as m_uint64_t;
    0
}

/// NOP
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_NOP(_cpu: *mut cpu_mips_t, _insn: mips_insn_t) -> c_int {
    0
}

/// NOR
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_NOR(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    (*cpu).gpr[rd as usize] = !((*cpu).gpr[rs as usize] | (*cpu).gpr[rt as usize]);
    0
}

/// OR
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_OR(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    (*cpu).gpr[rd as usize] = (*cpu).gpr[rs as usize] | (*cpu).gpr[rt as usize];
    0
}

/// ORI
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_ORI(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let imm: c_int = bits(insn, 0, 15);

    (*cpu).gpr[rt as usize] = (*cpu).gpr[rs as usize] | imm as m_int64_t as m_uint64_t;
    0
}

/// PREF
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_PREF(_cpu: *mut cpu_mips_t, _insn: mips_insn_t) -> c_int {
    0
}

/// PREFI
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_PREFI(_cpu: *mut cpu_mips_t, _insn: mips_insn_t) -> c_int {
    0
}

/// SB (Store Byte)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SB(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_SB as c_int, base as m_uint64_t, offset, rt as u_int, FALSE);
    0
}

/// SC (Store Conditional)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SC(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_SC as c_int, base as m_uint64_t, offset, rt as u_int, TRUE);
    0
}

/// SD (Store Double-Word)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SD(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_SD as c_int, base as m_uint64_t, offset, rt as u_int, FALSE);
    0
}

/// SDL (Store Double-Word Left)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SDL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_SDL as c_int, base as m_uint64_t, offset, rt as u_int, FALSE);
    0
}

/// SDR (Store Double-Word Right)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SDR(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_SDR as c_int, base as m_uint64_t, offset, rt as u_int, FALSE);
    0
}

/// SDC1 (Store Double-Word from Coprocessor 1)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SDC1(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let ft: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_SDC1 as c_int, base as m_uint64_t, offset, ft as u_int, FALSE);
    0
}

/// SH (Store Half-Word)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SH(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_SH as c_int, base as m_uint64_t, offset, rt as u_int, FALSE);
    0
}

/// SLL
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SLL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);
    let sa: c_int = bits(insn, 6, 10);

    let res: m_uint32_t = ((*cpu).gpr[rt as usize] << sa) as m_uint32_t;
    (*cpu).gpr[rd as usize] = sign_extend(res as m_int64_t, 32) as m_uint64_t;
    0
}

/// SLLV
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SLLV(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    let res: m_uint32_t = ((*cpu).gpr[rt as usize] << ((*cpu).gpr[rs as usize] & 0x1f)) as m_uint32_t;
    (*cpu).gpr[rd as usize] = sign_extend(res as m_int64_t, 32) as m_uint64_t;
    0
}

/// SLT
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SLT(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    if ((*cpu).gpr[rs as usize] as m_int64_t) < (*cpu).gpr[rt as usize] as m_int64_t {
        (*cpu).gpr[rd as usize] = 1;
    } else {
        (*cpu).gpr[rd as usize] = 0;
    }

    0
}

/// SLTI
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SLTI(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let imm: c_int = bits(insn, 0, 15);
    let val: m_int64_t = sign_extend(imm as m_int64_t, 16);

    if ((*cpu).gpr[rs as usize] as m_int64_t) < val {
        (*cpu).gpr[rt as usize] = 1;
    } else {
        (*cpu).gpr[rt as usize] = 0;
    }

    0
}

/// SLTIU
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SLTIU(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let imm: c_int = bits(insn, 0, 15);
    let val: m_uint64_t = sign_extend(imm as m_int64_t, 16) as m_uint64_t;

    if (*cpu).gpr[rs as usize] < val {
        (*cpu).gpr[rt as usize] = 1;
    } else {
        (*cpu).gpr[rt as usize] = 0;
    }

    0
}

/// SLTU
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SLTU(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    if (*cpu).gpr[rs as usize] < (*cpu).gpr[rt as usize] {
        (*cpu).gpr[rd as usize] = 1;
    } else {
        (*cpu).gpr[rd as usize] = 0;
    }

    0
}

/// SRA
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SRA(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);
    let sa: c_int = bits(insn, 6, 10);

    let res: m_int32_t = ((*cpu).gpr[rt as usize] >> sa) as m_int32_t;
    (*cpu).gpr[rd as usize] = sign_extend(res as m_int64_t, 32) as m_uint64_t;
    0
}

/// SRAV
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SRAV(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    let res: m_int32_t = ((*cpu).gpr[rt as usize] >> ((*cpu).gpr[rs as usize] & 0x1f)) as m_int32_t;
    (*cpu).gpr[rd as usize] = sign_extend(res as m_int64_t, 32) as m_uint64_t;
    0
}

/// SRL
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SRL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);
    let sa: c_int = bits(insn, 6, 10);

    let res: m_uint32_t = ((*cpu).gpr[rt as usize] as m_uint32_t) >> sa;
    (*cpu).gpr[rd as usize] = sign_extend(res as m_int64_t, 32) as m_uint64_t;
    0
}

/// SRLV
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SRLV(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    let res: m_uint32_t = ((*cpu).gpr[rt as usize] as m_uint32_t) >> ((*cpu).gpr[rs as usize] & 0x1f);
    (*cpu).gpr[rd as usize] = sign_extend(res as m_int64_t, 32) as m_uint64_t;
    0
}

/// SUB
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SUB(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    /* TODO: Exception handling */
    let res: m_uint32_t = ((*cpu).gpr[rs as usize] as m_uint32_t).wrapping_sub((*cpu).gpr[rt as usize] as m_uint32_t);
    (*cpu).gpr[rd as usize] = sign_extend(res as m_int64_t, 32) as m_uint64_t;
    0
}

/// SUBU
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SUBU(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    let res: m_uint32_t = ((*cpu).gpr[rs as usize] as m_uint32_t).wrapping_sub((*cpu).gpr[rt as usize] as m_uint32_t);
    (*cpu).gpr[rd as usize] = sign_extend(res as m_int64_t, 32) as m_uint64_t;
    0
}

/// SW (Store Word)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SW(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_SW as c_int, base as m_uint64_t, offset, rt as u_int, FALSE);
    0
}

/// SWL (Store Word Left)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SWL(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_SWL as c_int, base as m_uint64_t, offset, rt as u_int, FALSE);
    0
}

/// SWR (Store Word Right)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SWR(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let base: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let offset: c_int = bits(insn, 0, 15);

    mips64_exec_memop2(cpu, MIPS_MEMOP_SWR as c_int, base as m_uint64_t, offset, rt as u_int, FALSE);
    0
}

/// SYNC
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SYNC(_cpu: *mut cpu_mips_t, _insn: mips_insn_t) -> c_int {
    0
}

/// SYSCALL
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_SYSCALL(cpu: *mut cpu_mips_t, _insn: mips_insn_t) -> c_int {
    mips64_exec_syscall(cpu);
    1
}

/// TEQ (Trap if Equal)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_TEQ(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);

    if unlikely((*cpu).gpr[rs as usize] == (*cpu).gpr[rt as usize]) {
        mips64_trigger_trap_exception(cpu);
        return 1;
    }

    0
}

/// TEQI (Trap if Equal Immediate)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_TEQI(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let imm: c_int = bits(insn, 0, 15);
    let val: m_uint64_t = sign_extend(imm as m_int64_t, 16) as m_uint64_t;

    if unlikely((*cpu).gpr[rs as usize] == val) {
        mips64_trigger_trap_exception(cpu);
        return 1;
    }

    0
}

/// TLBP
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_TLBP(cpu: *mut cpu_mips_t, _insn: mips_insn_t) -> c_int {
    mips64_cp0_exec_tlbp(cpu);
    0
}

/// TLBR
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_TLBR(cpu: *mut cpu_mips_t, _insn: mips_insn_t) -> c_int {
    mips64_cp0_exec_tlbr(cpu);
    0
}

/// TLBWI
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_TLBWI(cpu: *mut cpu_mips_t, _insn: mips_insn_t) -> c_int {
    mips64_cp0_exec_tlbwi(cpu);
    0
}

/// TLBWR
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_TLBWR(cpu: *mut cpu_mips_t, _insn: mips_insn_t) -> c_int {
    mips64_cp0_exec_tlbwr(cpu);
    0
}

/// XOR
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_XOR(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    (*cpu).gpr[rd as usize] = (*cpu).gpr[rs as usize] ^ (*cpu).gpr[rt as usize];
    0
}

/// XORI
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_XORI(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let imm: c_int = bits(insn, 0, 15);

    (*cpu).gpr[rt as usize] = (*cpu).gpr[rs as usize] ^ imm as m_uint64_t;
    0
}

/// Unknown opcode
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_unknown(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    libc::printf(cstr!("MIPS64: unknown opcode 0x%8.8x at pc = 0x%llx\n"), insn, (*cpu).pc);
    mips64_dump_regs((*cpu).gen);
    0
}

/// MIPS instruction array
#[no_mangle] // TODO private
pub static mut mips64_exec_tags: [mips64_insn_exec_tag; 122] = [
    mips64_insn_exec_tag::new(cstr!("li"), Some(mips64_exec_LI), 0xffe00000, 0x24000000, 1, 16),
    mips64_insn_exec_tag::new(cstr!("move"), Some(mips64_exec_MOVE), 0xfc1f07ff, 0x00000021, 1, 15),
    mips64_insn_exec_tag::new(cstr!("b"), Some(mips64_exec_B), 0xffff0000, 0x10000000, 0, 10),
    mips64_insn_exec_tag::new(cstr!("bal"), Some(mips64_exec_BAL), 0xffff0000, 0x04110000, 0, 10),
    mips64_insn_exec_tag::new(cstr!("beqz"), Some(mips64_exec_BEQZ), 0xfc1f0000, 0x10000000, 0, 9),
    mips64_insn_exec_tag::new(cstr!("bnez"), Some(mips64_exec_BNEZ), 0xfc1f0000, 0x14000000, 0, 9),
    mips64_insn_exec_tag::new(cstr!("add"), Some(mips64_exec_ADD), 0xfc0007ff, 0x00000020, 1, 3),
    mips64_insn_exec_tag::new(cstr!("addi"), Some(mips64_exec_ADDI), 0xfc000000, 0x20000000, 1, 6),
    mips64_insn_exec_tag::new(cstr!("addiu"), Some(mips64_exec_ADDIU), 0xfc000000, 0x24000000, 1, 6),
    mips64_insn_exec_tag::new(cstr!("addu"), Some(mips64_exec_ADDU), 0xfc0007ff, 0x00000021, 1, 3),
    mips64_insn_exec_tag::new(cstr!("and"), Some(mips64_exec_AND), 0xfc0007ff, 0x00000024, 1, 3),
    mips64_insn_exec_tag::new(cstr!("andi"), Some(mips64_exec_ANDI), 0xfc000000, 0x30000000, 1, 5),
    mips64_insn_exec_tag::new(cstr!("beq"), Some(mips64_exec_BEQ), 0xfc000000, 0x10000000, 0, 8),
    mips64_insn_exec_tag::new(cstr!("beql"), Some(mips64_exec_BEQL), 0xfc000000, 0x50000000, 0, 8),
    mips64_insn_exec_tag::new(cstr!("bgez"), Some(mips64_exec_BGEZ), 0xfc1f0000, 0x04010000, 0, 9),
    mips64_insn_exec_tag::new(cstr!("bgezal"), Some(mips64_exec_BGEZAL), 0xfc1f0000, 0x04110000, 0, 9),
    mips64_insn_exec_tag::new(cstr!("bgezall"), Some(mips64_exec_BGEZALL), 0xfc1f0000, 0x04130000, 0, 9),
    mips64_insn_exec_tag::new(cstr!("bgezl"), Some(mips64_exec_BGEZL), 0xfc1f0000, 0x04030000, 0, 9),
    mips64_insn_exec_tag::new(cstr!("bgtz"), Some(mips64_exec_BGTZ), 0xfc1f0000, 0x1c000000, 0, 9),
    mips64_insn_exec_tag::new(cstr!("bgtzl"), Some(mips64_exec_BGTZL), 0xfc1f0000, 0x5c000000, 0, 9),
    mips64_insn_exec_tag::new(cstr!("blez"), Some(mips64_exec_BLEZ), 0xfc1f0000, 0x18000000, 0, 9),
    mips64_insn_exec_tag::new(cstr!("blezl"), Some(mips64_exec_BLEZL), 0xfc1f0000, 0x58000000, 0, 9),
    mips64_insn_exec_tag::new(cstr!("bltz"), Some(mips64_exec_BLTZ), 0xfc1f0000, 0x04000000, 0, 9),
    mips64_insn_exec_tag::new(cstr!("bltzal"), Some(mips64_exec_BLTZAL), 0xfc1f0000, 0x04100000, 0, 9),
    mips64_insn_exec_tag::new(cstr!("bltzall"), Some(mips64_exec_BLTZALL), 0xfc1f0000, 0x04120000, 0, 9),
    mips64_insn_exec_tag::new(cstr!("bltzl"), Some(mips64_exec_BLTZL), 0xfc1f0000, 0x04020000, 0, 9),
    mips64_insn_exec_tag::new(cstr!("bne"), Some(mips64_exec_BNE), 0xfc000000, 0x14000000, 0, 8),
    mips64_insn_exec_tag::new(cstr!("bnel"), Some(mips64_exec_BNEL), 0xfc000000, 0x54000000, 0, 8),
    mips64_insn_exec_tag::new(cstr!("break"), Some(mips64_exec_BREAK), 0xfc00003f, 0x0000000d, 1, 0),
    mips64_insn_exec_tag::new(cstr!("cache"), Some(mips64_exec_CACHE), 0xfc000000, 0xbc000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("cfc0"), Some(mips64_exec_CFC0), 0xffe007ff, 0x40400000, 1, 18),
    mips64_insn_exec_tag::new(cstr!("ctc0"), Some(mips64_exec_CTC0), 0xffe007ff, 0x40600000, 1, 18),
    mips64_insn_exec_tag::new(cstr!("daddiu"), Some(mips64_exec_DADDIU), 0xfc000000, 0x64000000, 1, 5),
    mips64_insn_exec_tag::new(cstr!("daddu"), Some(mips64_exec_DADDU), 0xfc0007ff, 0x0000002d, 1, 3),
    mips64_insn_exec_tag::new(cstr!("div"), Some(mips64_exec_DIV), 0xfc00ffff, 0x0000001a, 1, 17),
    mips64_insn_exec_tag::new(cstr!("divu"), Some(mips64_exec_DIVU), 0xfc00ffff, 0x0000001b, 1, 17),
    mips64_insn_exec_tag::new(cstr!("dmfc0"), Some(mips64_exec_DMFC0), 0xffe007f8, 0x40200000, 1, 18),
    mips64_insn_exec_tag::new(cstr!("dmfc1"), Some(mips64_exec_DMFC1), 0xffe007ff, 0x44200000, 1, 19),
    mips64_insn_exec_tag::new(cstr!("dmtc0"), Some(mips64_exec_DMTC0), 0xffe007f8, 0x40a00000, 1, 18),
    mips64_insn_exec_tag::new(cstr!("dmtc1"), Some(mips64_exec_DMTC1), 0xffe007ff, 0x44a00000, 1, 19),
    mips64_insn_exec_tag::new(cstr!("dsll"), Some(mips64_exec_DSLL), 0xffe0003f, 0x00000038, 1, 7),
    mips64_insn_exec_tag::new(cstr!("dsll32"), Some(mips64_exec_DSLL32), 0xffe0003f, 0x0000003c, 1, 7),
    mips64_insn_exec_tag::new(cstr!("dsllv"), Some(mips64_exec_DSLLV), 0xfc0007ff, 0x00000014, 1, 4),
    mips64_insn_exec_tag::new(cstr!("dsra"), Some(mips64_exec_DSRA), 0xffe0003f, 0x0000003b, 1, 7),
    mips64_insn_exec_tag::new(cstr!("dsra32"), Some(mips64_exec_DSRA32), 0xffe0003f, 0x0000003f, 1, 7),
    mips64_insn_exec_tag::new(cstr!("dsrav"), Some(mips64_exec_DSRAV), 0xfc0007ff, 0x00000017, 1, 4),
    mips64_insn_exec_tag::new(cstr!("dsrl"), Some(mips64_exec_DSRL), 0xffe0003f, 0x0000003a, 1, 7),
    mips64_insn_exec_tag::new(cstr!("dsrl32"), Some(mips64_exec_DSRL32), 0xffe0003f, 0x0000003e, 1, 7),
    mips64_insn_exec_tag::new(cstr!("dsrlv"), Some(mips64_exec_DSRLV), 0xfc0007ff, 0x00000016, 1, 4),
    mips64_insn_exec_tag::new(cstr!("dsubu"), Some(mips64_exec_DSUBU), 0xfc0007ff, 0x0000002f, 1, 3),
    mips64_insn_exec_tag::new(cstr!("eret"), Some(mips64_exec_ERET), 0xffffffff, 0x42000018, 0, 1),
    mips64_insn_exec_tag::new(cstr!("j"), Some(mips64_exec_J), 0xfc000000, 0x08000000, 0, 11),
    mips64_insn_exec_tag::new(cstr!("jal"), Some(mips64_exec_JAL), 0xfc000000, 0x0c000000, 0, 11),
    mips64_insn_exec_tag::new(cstr!("jalr"), Some(mips64_exec_JALR), 0xfc1f003f, 0x00000009, 0, 15),
    mips64_insn_exec_tag::new(cstr!("jr"), Some(mips64_exec_JR), 0xfc1ff83f, 0x00000008, 0, 13),
    mips64_insn_exec_tag::new(cstr!("lb"), Some(mips64_exec_LB), 0xfc000000, 0x80000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("lbu"), Some(mips64_exec_LBU), 0xfc000000, 0x90000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("ld"), Some(mips64_exec_LD), 0xfc000000, 0xdc000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("ldc1"), Some(mips64_exec_LDC1), 0xfc000000, 0xd4000000, 1, 3),
    mips64_insn_exec_tag::new(cstr!("ldl"), Some(mips64_exec_LDL), 0xfc000000, 0x68000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("ldr"), Some(mips64_exec_LDR), 0xfc000000, 0x6c000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("lh"), Some(mips64_exec_LH), 0xfc000000, 0x84000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("lhu"), Some(mips64_exec_LHU), 0xfc000000, 0x94000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("ll"), Some(mips64_exec_LL), 0xfc000000, 0xc0000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("lui"), Some(mips64_exec_LUI), 0xffe00000, 0x3c000000, 1, 16),
    mips64_insn_exec_tag::new(cstr!("lw"), Some(mips64_exec_LW), 0xfc000000, 0x8c000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("lwl"), Some(mips64_exec_LWL), 0xfc000000, 0x88000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("lwr"), Some(mips64_exec_LWR), 0xfc000000, 0x98000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("lwu"), Some(mips64_exec_LWU), 0xfc000000, 0x9c000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("mfc0"), Some(mips64_exec_MFC0), 0xffe007ff, 0x40000000, 1, 18),
    mips64_insn_exec_tag::new(cstr!("mfc0_1"), Some(mips64_exec_CFC0), 0xffe007ff, 0x40000001, 1, 19),
    mips64_insn_exec_tag::new(cstr!("mfc1"), Some(mips64_exec_MFC1), 0xffe007ff, 0x44000000, 1, 19),
    mips64_insn_exec_tag::new(cstr!("mfhi"), Some(mips64_exec_MFHI), 0xffff07ff, 0x00000010, 1, 14),
    mips64_insn_exec_tag::new(cstr!("mflo"), Some(mips64_exec_MFLO), 0xffff07ff, 0x00000012, 1, 14),
    #[cfg(feature = "USE_UNSTABLE")]
    mips64_insn_exec_tag::new(cstr!("movz"), Some(mips64_exec_MOVZ), 0xfc0007ff, 0x0000000a, 1, 3),
    mips64_insn_exec_tag::new(cstr!("mtc0"), Some(mips64_exec_MTC0), 0xffe007ff, 0x40800000, 1, 18),
    mips64_insn_exec_tag::new(cstr!("mtc1"), Some(mips64_exec_MTC1), 0xffe007ff, 0x44800000, 1, 19),
    mips64_insn_exec_tag::new(cstr!("mthi"), Some(mips64_exec_MTHI), 0xfc1fffff, 0x00000011, 1, 13),
    mips64_insn_exec_tag::new(cstr!("mtlo"), Some(mips64_exec_MTLO), 0xfc1fffff, 0x00000013, 1, 13),
    mips64_insn_exec_tag::new(cstr!("mul"), Some(mips64_exec_MUL), 0xfc0007ff, 0x70000002, 1, 4),
    mips64_insn_exec_tag::new(cstr!("mult"), Some(mips64_exec_MULT), 0xfc00ffff, 0x00000018, 1, 17),
    mips64_insn_exec_tag::new(cstr!("multu"), Some(mips64_exec_MULTU), 0xfc00ffff, 0x00000019, 1, 17),
    mips64_insn_exec_tag::new(cstr!("nop"), Some(mips64_exec_NOP), 0xffffffff, 0x00000000, 1, 1),
    mips64_insn_exec_tag::new(cstr!("nor"), Some(mips64_exec_NOR), 0xfc0007ff, 0x00000027, 1, 3),
    mips64_insn_exec_tag::new(cstr!("or"), Some(mips64_exec_OR), 0xfc0007ff, 0x00000025, 1, 3),
    mips64_insn_exec_tag::new(cstr!("ori"), Some(mips64_exec_ORI), 0xfc000000, 0x34000000, 1, 5),
    mips64_insn_exec_tag::new(cstr!("pref"), Some(mips64_exec_PREF), 0xfc000000, 0xcc000000, 1, 0),
    mips64_insn_exec_tag::new(cstr!("prefi"), Some(mips64_exec_PREFI), 0xfc0007ff, 0x4c00000f, 1, 0),
    mips64_insn_exec_tag::new(cstr!("sb"), Some(mips64_exec_SB), 0xfc000000, 0xa0000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("sc"), Some(mips64_exec_SC), 0xfc000000, 0xe0000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("sd"), Some(mips64_exec_SD), 0xfc000000, 0xfc000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("sdc1"), Some(mips64_exec_SDC1), 0xfc000000, 0xf4000000, 1, 3),
    mips64_insn_exec_tag::new(cstr!("sdl"), Some(mips64_exec_SDL), 0xfc000000, 0xb0000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("sdr"), Some(mips64_exec_SDR), 0xfc000000, 0xb4000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("sh"), Some(mips64_exec_SH), 0xfc000000, 0xa4000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("sll"), Some(mips64_exec_SLL), 0xffe0003f, 0x00000000, 1, 7),
    mips64_insn_exec_tag::new(cstr!("sllv"), Some(mips64_exec_SLLV), 0xfc0007ff, 0x00000004, 1, 4),
    mips64_insn_exec_tag::new(cstr!("slt"), Some(mips64_exec_SLT), 0xfc0007ff, 0x0000002a, 1, 3),
    mips64_insn_exec_tag::new(cstr!("slti"), Some(mips64_exec_SLTI), 0xfc000000, 0x28000000, 1, 5),
    mips64_insn_exec_tag::new(cstr!("sltiu"), Some(mips64_exec_SLTIU), 0xfc000000, 0x2c000000, 1, 5),
    mips64_insn_exec_tag::new(cstr!("sltu"), Some(mips64_exec_SLTU), 0xfc0007ff, 0x0000002b, 1, 3),
    mips64_insn_exec_tag::new(cstr!("sra"), Some(mips64_exec_SRA), 0xffe0003f, 0x00000003, 1, 7),
    mips64_insn_exec_tag::new(cstr!("srav"), Some(mips64_exec_SRAV), 0xfc0007ff, 0x00000007, 1, 4),
    mips64_insn_exec_tag::new(cstr!("srl"), Some(mips64_exec_SRL), 0xffe0003f, 0x00000002, 1, 7),
    mips64_insn_exec_tag::new(cstr!("srlv"), Some(mips64_exec_SRLV), 0xfc0007ff, 0x00000006, 1, 4),
    mips64_insn_exec_tag::new(cstr!("sub"), Some(mips64_exec_SUB), 0xfc0007ff, 0x00000022, 1, 3),
    mips64_insn_exec_tag::new(cstr!("subu"), Some(mips64_exec_SUBU), 0xfc0007ff, 0x00000023, 1, 3),
    mips64_insn_exec_tag::new(cstr!("sw"), Some(mips64_exec_SW), 0xfc000000, 0xac000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("swl"), Some(mips64_exec_SWL), 0xfc000000, 0xa8000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("swr"), Some(mips64_exec_SWR), 0xfc000000, 0xb8000000, 1, 2),
    mips64_insn_exec_tag::new(cstr!("sync"), Some(mips64_exec_SYNC), 0xfffff83f, 0x0000000f, 1, 1),
    mips64_insn_exec_tag::new(cstr!("syscall"), Some(mips64_exec_SYSCALL), 0xfc00003f, 0x0000000c, 1, 1),
    mips64_insn_exec_tag::new(cstr!("teq"), Some(mips64_exec_TEQ), 0xfc00003f, 0x00000034, 1, 17),
    mips64_insn_exec_tag::new(cstr!("teqi"), Some(mips64_exec_TEQI), 0xfc1f0000, 0x040c0000, 1, 20),
    mips64_insn_exec_tag::new(cstr!("tlbp"), Some(mips64_exec_TLBP), 0xffffffff, 0x42000008, 1, 1),
    mips64_insn_exec_tag::new(cstr!("tlbr"), Some(mips64_exec_TLBR), 0xffffffff, 0x42000001, 1, 1),
    mips64_insn_exec_tag::new(cstr!("tlbwi"), Some(mips64_exec_TLBWI), 0xffffffff, 0x42000002, 1, 1),
    mips64_insn_exec_tag::new(cstr!("tlbwr"), Some(mips64_exec_TLBWR), 0xffffffff, 0x42000006, 1, 1),
    mips64_insn_exec_tag::new(cstr!("xor"), Some(mips64_exec_XOR), 0xfc0007ff, 0x00000026, 1, 3),
    mips64_insn_exec_tag::new(cstr!("xori"), Some(mips64_exec_XORI), 0xfc000000, 0x38000000, 1, 5),
    mips64_insn_exec_tag::new(cstr!("unknown"), Some(mips64_exec_unknown), 0x00000000, 0x00000000, 1, 0),
    mips64_insn_exec_tag::null(),
    #[cfg(not(feature = "USE_UNSTABLE"))]
    mips64_insn_exec_tag::null(), // TODO extra value to sync array size; currently cbindgen does not support different sizes
];
