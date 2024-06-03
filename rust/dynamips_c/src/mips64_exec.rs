//! MIPS64 Step-by-step execution.

use crate::dynamips_common::*;
use crate::mips64::*;
use crate::prelude::*;
use crate::utils::*;

extern "C" {
    fn mips64_cp0_exec_cfc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int);
    fn mips64_cp0_exec_ctc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int);
    fn mips64_cp0_exec_dmfc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int);
    fn mips64_cp0_exec_dmtc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int);
    fn mips64_cp0_exec_mfc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int);
    fn mips64_cp0_exec_mtc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int);
    fn mips64_cp0_exec_tlbp(cpu: *mut cpu_mips_t);
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
