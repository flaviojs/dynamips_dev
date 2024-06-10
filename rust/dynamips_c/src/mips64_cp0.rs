//! MIPS Coprocessor 0 (System Coprocessor) implementation.
//! We don't use the JIT here, since there is no high performance needed.

use crate::cpu::*;
use crate::dynamips_common::*;
#[cfg(feature = "USE_UNSTABLE")]
use crate::memory::*;
use crate::mips64::*;
use crate::prelude::*;
use crate::utils::*;

/// MIPS cp0 registers names
#[no_mangle]
#[rustfmt::skip]
pub static mut mips64_cp0_reg_names: [*mut c_char; MIPS64_CP0_REG_NR] = [
    cstr!("index"), 
    cstr!("random"), 
    cstr!("entry_lo0"), 
    cstr!("entry_lo1"), 
    cstr!("context"), 
    cstr!("pagemask"),
    cstr!("wired"),
    cstr!("info"),
    cstr!("badvaddr"), 
    cstr!("count"), 
    cstr!("entry_hi"), 
    cstr!("compare"), 
    cstr!("status"), 
    cstr!("cause"),
    cstr!("epc"), 
    cstr!("prid"), 
    cstr!("config"), 
    cstr!("ll_addr"), 
    cstr!("watch_lo"), 
    cstr!("watch_hi"), 
    cstr!("xcontext"),
    cstr!("cp0_r21"),
    cstr!("cp0_r22"),
    cstr!("cp0_r23"),
    cstr!("cp0_r24"),
    cstr!("cp0_r25"),
    cstr!("ecc"), 
    cstr!("cache_err"), 
    cstr!("tag_lo"), 
    cstr!("tag_hi"), 
    cstr!("err_epc"),
    cstr!("cp0_r31"),
];

/// Get cp0 register index given its name
#[allow(clippy::needless_range_loop)]
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_get_reg_index(name: *mut c_char) -> c_int {
    for i in 0..MIPS64_CP0_REG_NR {
        if libc::strcmp(mips64_cp0_reg_names[i], name) == 0 {
            return i as c_int;
        }
    }

    -1
}

/// Get the CPU operating mode (User,Supervisor or Kernel) - inline version
#[cfg(not(feature = "USE_UNSTABLE"))]
#[inline(always)]
unsafe fn mips64_cp0_get_mode_inline(cpu: *mut cpu_mips_t) -> u_int {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);

    let mut cpu_mode: u_int = ((*cp0).reg[MIPS_CP0_STATUS] >> MIPS_CP0_STATUS_KSU_SHIFT) as u_int;
    cpu_mode &= MIPS_CP0_STATUS_KSU_MASK;
    cpu_mode
}

/// Get the CPU operating mode (User,Supervisor or Kernel)
#[cfg(not(feature = "USE_UNSTABLE"))]
unsafe fn mips64_cp0_get_mode(cpu: *mut cpu_mips_t) -> u_int {
    mips64_cp0_get_mode_inline(cpu)
}

/// Check that we are running in kernel mode
#[cfg(not(feature = "USE_UNSTABLE"))]
pub unsafe fn mips64_cp0_check_kernel_mode(cpu: *mut cpu_mips_t) -> c_int {
    let cpu_mode: u_int = mips64_cp0_get_mode(cpu);

    if cpu_mode != MIPS_CP0_STATUS_KM {
        // XXX Branch delay slot
        mips64_trigger_exception(cpu, MIPS_CP0_CAUSE_ILLOP, 0);
        return 1;
    }

    0
}

/// Get the CPU operating mode (User,Supervisor or Kernel)
#[cfg(feature = "USE_UNSTABLE")]
#[inline(always)]
unsafe fn mips64_cp0_get_mode(cpu: *mut cpu_mips_t) -> u_int {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);

    let mut cpu_mode: u_int = ((*cp0).reg[MIPS_CP0_STATUS] >> MIPS_CP0_STATUS_KSU_SHIFT) as u_int;
    cpu_mode &= MIPS_CP0_STATUS_KSU_MASK;
    cpu_mode
}

/// Check that we are running in kernel mode
#[cfg(feature = "USE_UNSTABLE")]
pub unsafe fn mips64_cp0_check_kernel_mode(cpu: *mut cpu_mips_t) -> c_int {
    let cpu_mode: u_int = mips64_cp0_get_mode(cpu);

    if cpu_mode != MIPS_CP0_STATUS_KM {
        mips64_general_exception(cpu, MIPS_CP0_CAUSE_ILLOP);
        return 1;
    }

    0
}

/// Get value of random register
#[no_mangle] // TODO private
#[inline]
pub unsafe extern "C" fn mips64_cp0_get_random_reg(cpu: *mut cpu_mips_t) -> u_int {
    // We use the virtual count register as a basic "random" value
    let wired: u_int = (*cpu).cp0.reg[MIPS_CP0_WIRED] as u_int;
    wired + ((*cpu).cp0_virt_cnt_reg % ((*cpu).cp0.tlb_entries - wired))
}

/// Get a cp0 register (fast version)
#[no_mangle] // TODO private
#[inline]
pub unsafe extern "C" fn mips64_cp0_get_reg_fast(cpu: *mut cpu_mips_t, cp0_reg: u_int) -> m_uint64_t {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);
    let delta: m_uint32_t;
    let mut res: m_uint32_t;

    match cp0_reg as usize {
        MIPS_CP0_COUNT => {
            delta = (*cpu).cp0_virt_cmp_reg - (*cpu).cp0_virt_cnt_reg;
            res = (*cp0).reg[MIPS_CP0_COMPARE] as m_uint32_t;
            res -= (*(*cpu).vm).clock_divisor * delta;
            sign_extend(res as m_int64_t, 32) as m_uint64_t
        }

        MIPS_CP0_COMPARE => {
            if true {
                sign_extend((*cp0).reg[MIPS_CP0_COMPARE] as m_int64_t, 32) as m_uint64_t
            } else {
                // really useful and logical ?
                delta = (*cpu).cp0_virt_cmp_reg - (*cpu).cp0_virt_cnt_reg;
                res = (*cp0).reg[MIPS_CP0_COUNT] as m_uint32_t;
                res += (*(*cpu).vm).clock_divisor * delta;
                res as m_uint64_t
            }
        }

        MIPS_CP0_INFO => MIPS64_R7000_TLB64_ENABLE as m_uint64_t,

        MIPS_CP0_RANDOM => mips64_cp0_get_random_reg(cpu) as m_uint64_t,

        _ => (*cp0).reg[cp0_reg as usize],
    }
}

/// Get a cp0 register
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_get_reg(cpu: *mut cpu_mips_t, cp0_reg: u_int) -> m_uint64_t {
    mips64_cp0_get_reg_fast(cpu, cp0_reg)
}

/// DMFC0
#[no_mangle]
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_cp0_exec_dmfc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int) {
    (*cpu).gpr[gp_reg as usize] = mips64_cp0_get_reg_fast(cpu, cp0_reg);
}

/// Set a cp0 register
#[no_mangle] // TODO private
#[inline]
pub unsafe extern "C" fn mips64_cp0_set_reg(cpu: *mut cpu_mips_t, cp0_reg: u_int, val: m_uint64_t) {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);
    let delta: m_uint32_t;

    match cp0_reg as usize {
        MIPS_CP0_STATUS | MIPS_CP0_CAUSE => {
            (*cp0).reg[cp0_reg as usize] = val;
            mips64_update_irq_flag(cpu);
        }

        MIPS_CP0_PAGEMASK => {
            (*cp0).reg[cp0_reg as usize] = val & MIPS_TLB_PAGE_MASK;
        }

        MIPS_CP0_COMPARE => {
            mips64_clear_irq(cpu, 7);
            mips64_update_irq_flag(cpu);
            (*cp0).reg[cp0_reg as usize] = val;

            delta = (val - (*cp0).reg[MIPS_CP0_COUNT]) as m_uint32_t;
            (*cpu).cp0_virt_cnt_reg = 0;
            (*cpu).cp0_virt_cmp_reg = delta / (*(*cpu).vm).clock_divisor;
        }

        MIPS_CP0_COUNT => {
            (*cp0).reg[cp0_reg as usize] = val;

            delta = ((*cp0).reg[MIPS_CP0_COMPARE] - val) as m_uint32_t;
            (*cpu).cp0_virt_cnt_reg = 0;
            (*cpu).cp0_virt_cmp_reg = delta / (*(*cpu).vm).clock_divisor;
        }

        MIPS_CP0_TLB_HI => {
            (*cp0).reg[cp0_reg as usize] = val & MIPS_CP0_HI_SAFE_MASK;
        }

        MIPS_CP0_TLB_LO_0 | MIPS_CP0_TLB_LO_1 => {
            (*cp0).reg[cp0_reg as usize] = val & MIPS_CP0_LO_SAFE_MASK;
        }

        MIPS_CP0_RANDOM | MIPS_CP0_PRID | MIPS_CP0_CONFIG => {
            // read only registers
        }

        MIPS_CP0_WIRED => {
            (*cp0).reg[cp0_reg as usize] = val & MIPS64_TLB_IDX_MASK;
        }

        _ => {
            (*cp0).reg[cp0_reg as usize] = val;
        }
    }
}

/// DMTC0
#[no_mangle]
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_cp0_exec_dmtc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int) {
    mips64_cp0_set_reg(cpu, cp0_reg, (*cpu).gpr[gp_reg as usize]);
}

/// MFC0
#[no_mangle]
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_cp0_exec_mfc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int) {
    (*cpu).gpr[gp_reg as usize] = sign_extend(mips64_cp0_get_reg_fast(cpu, cp0_reg) as m_int64_t, 32) as m_uint64_t;
}

/// MTC0
#[no_mangle]
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_cp0_exec_mtc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int) {
    mips64_cp0_set_reg(cpu, cp0_reg, (*cpu).gpr[gp_reg as usize] & 0xffffffff);
}

/// Get a cp0 "set 1" register (R7000)
unsafe fn mips64_cp0_s1_get_reg(cpu: *mut cpu_mips_t, cp0_s1_reg: u_int) -> m_uint64_t {
    match cp0_s1_reg as usize {
        MIPS_CP0_S1_CONFIG => 0x7F << 25,

        MIPS_CP0_S1_IPLLO => (*cpu).cp0.ipl_lo as m_uint64_t,

        MIPS_CP0_S1_IPLHI => (*cpu).cp0.ipl_hi as m_uint64_t,

        MIPS_CP0_S1_INTCTL => (*cpu).cp0.int_ctl as m_uint64_t,

        MIPS_CP0_S1_DERRADDR0 => (*cpu).cp0.derraddr0 as m_uint64_t,

        MIPS_CP0_S1_DERRADDR1 => (*cpu).cp0.derraddr1 as m_uint64_t,

        _ => {
            // undefined register
            cpu_log!((*cpu).gen, cstr!("CP0_S1"), cstr!("trying to read unknown register %u\n"), cp0_s1_reg);
            0
        }
    }
}

/// Set a cp0 "set 1" register (R7000)
#[no_mangle] // TODO private
#[inline]
pub unsafe extern "C" fn mips64_cp0_s1_set_reg(cpu: *mut cpu_mips_t, cp0_s1_reg: u_int, val: m_uint64_t) {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);

    match cp0_s1_reg as usize {
        MIPS_CP0_S1_IPLLO => {
            (*cp0).ipl_lo = val as m_uint32_t;
        }

        MIPS_CP0_S1_IPLHI => {
            (*cp0).ipl_hi = val as m_uint32_t;
        }

        MIPS_CP0_S1_INTCTL => {
            (*cp0).int_ctl = val as m_uint32_t;
        }

        MIPS_CP0_S1_DERRADDR0 => {
            (*cp0).derraddr0 = val as m_uint32_t;
        }

        MIPS_CP0_S1_DERRADDR1 => {
            (*cp0).derraddr1 = val as m_uint32_t;
        }

        _ => {
            cpu_log!((*cpu).gen, cstr!("CP0_S1"), cstr!("trying to set unknown register %u (val=0x%llx)\n"), cp0_s1_reg, val);
        }
    }
}

/// CFC0
#[no_mangle]
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_cp0_exec_cfc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int) {
    (*cpu).gpr[gp_reg as usize] = sign_extend(mips64_cp0_s1_get_reg(cpu, cp0_reg) as m_int64_t, 32) as m_uint64_t;
}

/// CTC0
#[no_mangle]
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_cp0_exec_ctc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int) {
    mips64_cp0_s1_set_reg(cpu, cp0_reg, (*cpu).gpr[gp_reg as usize] & 0xffffffff);
}

/// Get the page size corresponding to a page mask
#[inline]
#[no_mangle] // TODO private
pub unsafe extern "C" fn get_page_size(page_mask: m_uint32_t) -> m_uint32_t {
    (page_mask + 0x2000) >> 1
}

/// Get the VPN2 mask
#[no_mangle] // TODO private
#[cfg_attr(feature = "USE_UNSTABLE", inline(always))]
pub unsafe extern "C" fn mips64_cp0_get_vpn2_mask(cpu: *mut cpu_mips_t) -> m_uint64_t {
    if (*cpu).addr_mode == 64 {
        MIPS_TLB_VPN2_MASK_64
    } else {
        MIPS_TLB_VPN2_MASK_32
    }
}

/// TLB lookup
#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_tlb_lookup(cpu: *mut cpu_mips_t, vaddr: m_uint64_t, res: *mut mts_map_t) -> c_int {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);
    let vpn_addr: m_uint64_t;
    let vpn2_mask: m_uint64_t;
    let mut page_mask: m_uint64_t;
    let mut hi_addr: m_uint64_t;
    let page_size: m_uint32_t;
    let mut pca: m_uint32_t;
    let mut entry: *mut tlb_entry_t;
    let asid: u_int;

    vpn2_mask = mips64_cp0_get_vpn2_mask(cpu);
    vpn_addr = vaddr & vpn2_mask;

    asid = ((*cp0).reg[MIPS_CP0_TLB_HI] & MIPS_TLB_ASID_MASK as m_uint64_t) as m_uint32_t;

    for i in 0..(*cp0).tlb_entries {
        entry = addr_of_mut!((*cp0).tlb[i as usize]);

        page_mask = !((*entry).mask + 0x1FFF);
        hi_addr = (*entry).hi & vpn2_mask;

        if ((vpn_addr & page_mask) == hi_addr) && (((*entry).hi & MIPS_TLB_G_MASK as m_uint64_t) != 0 || (((*entry).hi & MIPS_TLB_ASID_MASK as m_uint64_t) == asid as m_uint64_t)) {
            page_size = get_page_size((*entry).mask as m_uint32_t);

            if (vaddr & page_size as m_uint64_t) == 0 {
                // Even Page
                if ((*entry).lo0 & MIPS_TLB_V_MASK as m_uint64_t) != 0 {
                    (*res).vaddr = vaddr & MIPS_MIN_PAGE_MASK;
                    (*res).paddr = ((*entry).lo0 & MIPS_TLB_PFN_MASK as m_uint64_t) << 6;
                    (*res).paddr += (*res).vaddr & (page_size - 1) as m_uint64_t;
                    (*res).paddr &= (*cpu).addr_bus_mask;

                    (*res).offset = (vaddr & MIPS_MIN_PAGE_IMASK) as m_uint32_t;

                    pca = (*entry).lo0 as m_uint32_t & MIPS_TLB_C_MASK;
                    pca >>= MIPS_TLB_C_SHIFT;
                    (*res).cached = mips64_cca_cached(pca as m_uint8_t) as m_uint32_t;

                    (*res).tlb_index = i;
                    return TRUE;
                }
            } else {
                // Odd Page
                if ((*entry).lo1 & MIPS_TLB_V_MASK as m_uint64_t) != 0 {
                    (*res).vaddr = vaddr & MIPS_MIN_PAGE_MASK;
                    (*res).paddr = ((*entry).lo1 & MIPS_TLB_PFN_MASK as m_uint64_t) << 6;
                    (*res).paddr += (*res).vaddr & (page_size - 1) as m_uint64_t;
                    (*res).paddr &= (*cpu).addr_bus_mask;

                    (*res).offset = (vaddr & MIPS_MIN_PAGE_IMASK) as m_uint32_t;

                    pca = ((*entry).lo1 & MIPS_TLB_C_MASK as m_uint64_t) as m_uint32_t;
                    pca >>= MIPS_TLB_C_SHIFT;
                    (*res).cached = mips64_cca_cached(pca as m_uint8_t) as m_uint32_t;

                    (*res).tlb_index = i;
                    return TRUE;
                }
            }

            // Invalid entry
            return FALSE;
        }
    }

    // No matching entry
    FALSE
}

/// TLB lookup
#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_tlb_lookup(cpu: *mut cpu_mips_t, vaddr: m_uint64_t, op_type: u_int, res: *mut mts_map_t) -> c_int {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);
    let mut page_mask: m_uint64_t;
    let mut hi_addr: m_uint64_t;
    let page_size: m_uint32_t;
    let mut pca: m_uint32_t;
    let mut entry: *mut tlb_entry_t;

    let vpn2_mask: m_uint64_t = mips64_cp0_get_vpn2_mask(cpu);
    let vpn_addr: m_uint64_t = vaddr & vpn2_mask;

    let asid: u_int = ((*cp0).reg[MIPS_CP0_TLB_HI] & MIPS_TLB_ASID_MASK as m_uint64_t) as m_uint32_t;

    for i in 0..(*cp0).tlb_entries {
        entry = addr_of_mut!((*cp0).tlb[i as usize]);

        page_mask = !(*entry).mask;
        hi_addr = (*entry).hi & vpn2_mask & page_mask;

        if ((vpn_addr & page_mask) == hi_addr) && (((*entry).hi & MIPS_TLB_G_MASK as m_uint64_t) != 0 || (((*entry).hi & MIPS_TLB_ASID_MASK as m_uint64_t) == asid as m_uint64_t)) {
            page_size = get_page_size((*entry).mask as m_uint32_t);

            if (vaddr & page_size as m_uint64_t) == 0 {
                // Even Page
                if ((*entry).lo0 & MIPS_TLB_V_MASK as m_uint64_t) != 0 {
                    // Check write protection
                    if (op_type == MTS_WRITE) && ((*entry).lo0 & MIPS_TLB_D_MASK as m_uint64_t) == 0 {
                        return MIPS_TLB_LOOKUP_MOD;
                    }

                    (*res).flags = 0;
                    (*res).vaddr = vaddr & MIPS_MIN_PAGE_MASK;
                    (*res).paddr = ((*entry).lo0 & MIPS_TLB_PFN_MASK as m_uint64_t) << 6;
                    (*res).paddr += (*res).vaddr & (page_size - 1) as m_uint64_t;
                    (*res).paddr &= (*cpu).addr_bus_mask;

                    (*res).offset = (vaddr & MIPS_MIN_PAGE_IMASK) as m_uint32_t;

                    pca = ((*entry).lo0 & MIPS_TLB_C_MASK as m_uint64_t) as m_uint32_t;
                    pca >>= MIPS_TLB_C_SHIFT;
                    (*res).cached = mips64_cca_cached(pca as m_uint8_t) as m_uint32_t;

                    if ((*entry).lo0 & MIPS_TLB_D_MASK as m_uint64_t) == 0 {
                        (*res).flags |= MTS_FLAG_RO;
                    }

                    return MIPS_TLB_LOOKUP_OK;
                }
            } else {
                // Odd Page
                if ((*entry).lo1 & MIPS_TLB_V_MASK as m_uint64_t) != 0 {
                    // Check write protection
                    if (op_type == MTS_WRITE) && ((*entry).lo1 & MIPS_TLB_D_MASK as m_uint64_t) == 0 {
                        return MIPS_TLB_LOOKUP_MOD;
                    }

                    (*res).flags = 0;
                    (*res).vaddr = vaddr & MIPS_MIN_PAGE_MASK;
                    (*res).paddr = ((*entry).lo1 & MIPS_TLB_PFN_MASK as m_uint64_t) << 6;
                    (*res).paddr += (*res).vaddr & (page_size - 1) as m_uint64_t;
                    (*res).paddr &= (*cpu).addr_bus_mask;

                    (*res).offset = (vaddr & MIPS_MIN_PAGE_IMASK) as m_uint32_t;

                    pca = ((*entry).lo1 & MIPS_TLB_C_MASK as m_uint64_t) as m_uint32_t;
                    pca >>= MIPS_TLB_C_SHIFT;
                    (*res).cached = mips64_cca_cached(pca as m_uint8_t) as m_uint32_t;

                    if ((*entry).lo1 & MIPS_TLB_D_MASK as m_uint64_t) == 0 {
                        (*res).flags |= MTS_FLAG_RO;
                    }

                    return MIPS_TLB_LOOKUP_OK;
                }
            }

            // Invalid entry
            return MIPS_TLB_LOOKUP_INVALID;
        }
    }

    // No matching entry
    MIPS_TLB_LOOKUP_MISS
}

/// Map a TLB entry into the MTS.
///
/// We apply the physical address bus masking here.
///
/// TODO: - Manage ASID
///       - Manage CPU Mode (user,supervisor or kernel)
#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle] // TODO private
pub unsafe extern "C" fn mips64_cp0_map_tlb_to_mts(cpu: *mut cpu_mips_t, index: c_int) {
    let v0_addr: m_uint64_t;
    let v1_addr: m_uint64_t;
    let p0_addr: m_uint64_t;
    let p1_addr: m_uint64_t;
    let page_size: m_uint32_t;
    let mut pca: m_uint32_t;
    let entry: *mut tlb_entry_t;
    let mut cacheable: c_int;

    entry = addr_of_mut!((*cpu).cp0.tlb[index as usize]);

    page_size = get_page_size((*entry).mask as m_uint32_t);
    v0_addr = (*entry).hi & mips64_cp0_get_vpn2_mask(cpu);
    v1_addr = v0_addr + page_size as m_uint64_t;

    if ((*entry).lo0 & MIPS_TLB_V_MASK as m_uint64_t) != 0 {
        pca = ((*entry).lo0 & MIPS_TLB_C_MASK as m_uint64_t) as m_uint32_t;
        pca >>= MIPS_TLB_C_SHIFT;
        cacheable = mips64_cca_cached(pca as m_uint8_t);

        p0_addr = ((*entry).lo0 & MIPS_TLB_PFN_MASK as m_uint64_t) << 6;
        (*cpu).mts_map.unwrap()(cpu, v0_addr, p0_addr & (*cpu).addr_bus_mask, page_size, cacheable, index);
    }

    if ((*entry).lo1 & MIPS_TLB_V_MASK as m_uint64_t) != 0 {
        pca = ((*entry).lo1 & MIPS_TLB_C_MASK as m_uint64_t) as m_uint32_t;
        pca >>= MIPS_TLB_C_SHIFT;
        cacheable = mips64_cca_cached(pca as m_uint8_t);

        p1_addr = ((*entry).lo1 & MIPS_TLB_PFN_MASK as m_uint64_t) << 6;
        (*cpu).mts_map.unwrap()(cpu, v1_addr, p1_addr & (*cpu).addr_bus_mask, page_size, cacheable, index);
    }
}

/// Map all TLB entries into the MTS
#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_map_all_tlb_to_mts(cpu: *mut cpu_mips_t) {
    for i in 0..(*cpu).cp0.tlb_entries {
        mips64_cp0_map_tlb_to_mts(cpu, i as c_int);
    }
}
