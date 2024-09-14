//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! MIPS Coprocessor 0 (System Coprocessor) implementation.
//! We don't use the JIT here, since there is no high performance needed.

use crate::_private::*;
use crate::cpu::*;
use crate::dynamips::*;
use crate::dynamips_common::*;
use crate::memory::*;
use crate::mips64::*;
use crate::utils::*;

#[cfg(feature = "USE_UNSTABLE")]
pub const TLB_ZONE_ADD: c_int = 0;
#[cfg(feature = "USE_UNSTABLE")]
pub const TLB_ZONE_DELETE: c_int = 1;

/// Update the Context register with a faulty address
#[cfg(feature = "USE_UNSTABLE")]
#[inline]
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_update_context_reg(cpu: *mut cpu_mips_t, addr: m_uint64_t) {
    let mut badvpn2: m_uint64_t;

    badvpn2 = addr & MIPS_CP0_CONTEXT_VPN2_MASK;
    badvpn2 <<= MIPS_CP0_CONTEXT_BADVPN2_SHIFT;

    (*cpu).cp0.reg[MIPS_CP0_CONTEXT] &= !MIPS_CP0_CONTEXT_BADVPN2_MASK;
    (*cpu).cp0.reg[MIPS_CP0_CONTEXT] |= badvpn2;
}

/// Update the XContext register with a faulty address
#[cfg(feature = "USE_UNSTABLE")]
#[inline]
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_update_xcontext_reg(cpu: *mut cpu_mips_t, addr: m_uint64_t) {
    let mut rbadvpn2: m_uint64_t;

    rbadvpn2 = addr & MIPS_CP0_XCONTEXT_VPN2_MASK;
    rbadvpn2 <<= MIPS_CP0_XCONTEXT_BADVPN2_SHIFT;
    rbadvpn2 |= ((addr >> 62) & 0x03) << MIPS_CP0_XCONTEXT_R_SHIFT;

    (*cpu).cp0.reg[MIPS_CP0_XCONTEXT] &= !MIPS_CP0_XCONTEXT_RBADVPN2_MASK;
    (*cpu).cp0.reg[MIPS_CP0_XCONTEXT] |= rbadvpn2;
}

/// Get the CPU operating mode (User,Supervisor or Kernel)
#[cfg(feature = "USE_UNSTABLE")]
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_get_mode(cpu: *mut cpu_mips_t) -> u_int {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);
    let mut cpu_mode: u_int;

    cpu_mode = ((*cp0).reg[MIPS_CP0_STATUS] >> MIPS_CP0_STATUS_KSU_SHIFT) as u_int;
    cpu_mode &= MIPS_CP0_STATUS_KSU_MASK;
    cpu_mode
}

/// Get the VPN2 mask
#[cfg(feature = "USE_UNSTABLE")]
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_get_vpn2_mask(cpu: *mut cpu_mips_t) -> m_uint64_t {
    if (*cpu).addr_mode == 64 {
        MIPS_TLB_VPN2_MASK_64
    } else {
        MIPS_TLB_VPN2_MASK_32
    }
}

/// MIPS cp0 registers names
#[rustfmt::skip]
#[no_mangle]
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
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_get_reg_index(name: *mut c_char) -> c_int {
    for i in 0..MIPS64_CP0_REG_NR as c_int {
        if libc::strcmp(mips64_cp0_reg_names[i as usize], name) == 0 {
            return i;
        }
    }

    -1
}

/// Get the CPU operating mode (User,Supervisor or Kernel) - inline version
#[cfg(not(feature = "USE_UNSTABLE"))]
#[inline(always)]
unsafe fn mips64_cp0_get_mode_inline(cpu: *mut cpu_mips_t) -> u_int {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);
    let mut cpu_mode: u_int;

    cpu_mode = ((*cp0).reg[MIPS_CP0_STATUS] >> MIPS_CP0_STATUS_KSU_SHIFT) as u_int;
    cpu_mode &= MIPS_CP0_STATUS_KSU_MASK;
    cpu_mode
}

/// Get the CPU operating mode (User,Supervisor or Kernel)
#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_get_mode(cpu: *mut cpu_mips_t) -> u_int {
    mips64_cp0_get_mode_inline(cpu)
}

/// Check that we are running in kernel mode
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_check_kernel_mode(cpu: *mut cpu_mips_t) -> c_int {
    let cpu_mode: u_int = mips64_cp0_get_mode(cpu);

    if cpu_mode != MIPS_CP0_STATUS_KM {
        #[cfg(not(feature = "USE_UNSTABLE"))]
        {
            // XXX Branch delay slot
            mips64_trigger_exception(cpu, MIPS_CP0_CAUSE_ILLOP, 0);
        }
        #[cfg(feature = "USE_UNSTABLE")]
        {
            mips64_general_exception(cpu, MIPS_CP0_CAUSE_ILLOP);
        }
        return 1;
    }

    0
}

/// Get value of random register
#[inline]
unsafe fn mips64_cp0_get_random_reg(cpu: *mut cpu_mips_t) -> u_int {
    // We use the virtual count register as a basic "random" value
    let wired: u_int = (*cpu).cp0.reg[MIPS_CP0_WIRED] as u_int;
    wired + ((*cpu).cp0_virt_cnt_reg % ((*cpu).cp0.tlb_entries - wired))
}

/// Get a cp0 register (fast version)
#[inline]
unsafe fn mips64_cp0_get_reg_fast(cpu: *mut cpu_mips_t, cp0_reg: u_int) -> m_uint64_t {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);
    let delta: m_uint32_t;
    let mut res: m_uint32_t;

    match cp0_reg as usize {
        MIPS_CP0_COUNT => {
            delta = (*cpu).cp0_virt_cmp_reg - (*cpu).cp0_virt_cnt_reg;
            res = (*cp0).reg[MIPS_CP0_COMPARE] as m_uint32_t;
            res -= (*(*cpu).vm).clock_divisor * delta;
            #[allow(clippy::needless_return)]
            {
                return sign_extend(res as m_int64_t, 32) as m_uint64_t;
            }
        }

        MIPS_CP0_COMPARE => {
            if true {
                #[allow(clippy::needless_return)]
                {
                    return sign_extend((*cp0).reg[MIPS_CP0_COMPARE] as m_int64_t, 32) as m_uint64_t;
                }
            } else {
                // really useful and logical ?
                delta = (*cpu).cp0_virt_cmp_reg - (*cpu).cp0_virt_cnt_reg;
                res = (*cp0).reg[MIPS_CP0_COUNT] as m_uint32_t;
                res += (*(*cpu).vm).clock_divisor * delta;
                #[allow(clippy::needless_return)]
                {
                    return res as m_uint64_t;
                }
            }
        }
        MIPS_CP0_INFO => {
            #[allow(clippy::needless_return)]
            {
                return MIPS64_R7000_TLB64_ENABLE as m_uint64_t;
            }
        }

        MIPS_CP0_RANDOM => {
            #[allow(clippy::needless_return)]
            {
                return mips64_cp0_get_random_reg(cpu) as m_uint64_t;
            }
        }

        _ => {
            #[allow(clippy::needless_return)]
            {
                return (*cp0).reg[cp0_reg as usize];
            }
        }
    }
}

/// Get a cp0 register
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_get_reg(cpu: *mut cpu_mips_t, cp0_reg: u_int) -> m_uint64_t {
    mips64_cp0_get_reg_fast(cpu, cp0_reg)
}

/// Set a cp0 register
#[inline]
unsafe fn mips64_cp0_set_reg(cpu: *mut cpu_mips_t, cp0_reg: u_int, val: m_uint64_t) {
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

            delta = ((*cp0).reg[MIPS_CP0_COMPARE] - val as m_uint64_t) as m_uint32_t;
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

/// Get a cp0 "set 1" register (R7000)
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_s1_get_reg(cpu: *mut cpu_mips_t, cp0_s1_reg: u_int) -> m_uint64_t {
    match cp0_s1_reg as usize {
        MIPS_CP0_S1_CONFIG => {
            #[allow(clippy::needless_return)]
            {
                return 0x7F << 25;
            }
        }

        MIPS_CP0_S1_IPLLO => {
            #[allow(clippy::needless_return)]
            {
                return (*cpu).cp0.ipl_lo as m_uint64_t;
            }
        }

        MIPS_CP0_S1_IPLHI => {
            #[allow(clippy::needless_return)]
            {
                return (*cpu).cp0.ipl_hi as m_uint64_t;
            }
        }

        MIPS_CP0_S1_INTCTL => {
            #[allow(clippy::needless_return)]
            {
                return (*cpu).cp0.int_ctl as m_uint64_t;
            }
        }

        MIPS_CP0_S1_DERRADDR0 => {
            #[allow(clippy::needless_return)]
            {
                return (*cpu).cp0.derraddr0 as m_uint64_t;
            }
        }

        MIPS_CP0_S1_DERRADDR1 => {
            #[allow(clippy::needless_return)]
            {
                return (*cpu).cp0.derraddr1 as m_uint64_t;
            }
        }

        _ => {
            // undefined register
            cpu_log!((*cpu).gen, cstr!("CP0_S1"), cstr!("trying to read unknown register %u\n"), cp0_s1_reg);
            #[allow(clippy::needless_return)]
            {
                return 0;
            }
        }
    }
}

/// Set a cp0 "set 1" register (R7000)
#[inline]
unsafe fn mips64_cp0_s1_set_reg(cpu: *mut cpu_mips_t, cp0_s1_reg: u_int, val: m_uint64_t) {
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

/// DMFC0
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_exec_dmfc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int) {
    (*cpu).gpr[gp_reg as usize] = mips64_cp0_get_reg_fast(cpu, cp0_reg);
}

/// DMTC0
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_exec_dmtc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int) {
    mips64_cp0_set_reg(cpu, cp0_reg, (*cpu).gpr[gp_reg as usize]);
}

/// MFC0
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_exec_mfc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int) {
    (*cpu).gpr[gp_reg as usize] = sign_extend(mips64_cp0_get_reg_fast(cpu, cp0_reg) as m_int64_t, 32) as m_uint64_t;
}

/// MTC0
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_exec_mtc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int) {
    mips64_cp0_set_reg(cpu, cp0_reg, (*cpu).gpr[gp_reg as usize] & 0xffffffff);
}

/// CFC0
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_exec_cfc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int) {
    (*cpu).gpr[gp_reg as usize] = sign_extend(mips64_cp0_s1_get_reg(cpu, cp0_reg) as m_int64_t, 32) as m_uint64_t;
}

/// CTC0
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_exec_ctc0(cpu: *mut cpu_mips_t, gp_reg: u_int, cp0_reg: u_int) {
    mips64_cp0_s1_set_reg(cpu, cp0_reg, (*cpu).gpr[gp_reg as usize] & 0xffffffff);
}

/// Get the page size corresponding to a page mask
#[inline]
unsafe fn get_page_size(page_mask: m_uint32_t) -> m_uint32_t {
    (page_mask + 0x2000) >> 1
}

/// Write page size in buffer
unsafe fn get_page_size_str(buffer: *mut c_char, len: size_t, page_mask: m_uint32_t) -> *mut c_char {
    let page_size: m_uint32_t = get_page_size(page_mask);

    // Mb ?
    if page_size >= (1024 * 1024) {
        libc::snprintf(buffer, len, cstr!("%uMB"), page_size >> 20);
    } else {
        libc::snprintf(buffer, len, cstr!("%uKB"), page_size >> 10);
    }

    buffer
}

/// Get the VPN2 mask
#[cfg(not(feature = "USE_UNSTABLE"))]
#[inline(always)]
unsafe fn mips64_cp0_get_vpn2_mask(cpu: *mut cpu_mips_t) -> m_uint64_t {
    if (*cpu).addr_mode == 64 {
        return MIPS_TLB_VPN2_MASK_64;
    } else {
        return MIPS_TLB_VPN2_MASK_32;
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

    asid = ((*cp0).reg[MIPS_CP0_TLB_HI] & MIPS_TLB_ASID_MASK as m_uint64_t) as u_int;

    for i in 0..(*cp0).tlb_entries as c_int {
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

                    pca = ((*entry).lo0 & MIPS_TLB_C_MASK as m_uint64_t) as m_uint32_t;
                    pca >>= MIPS_TLB_C_SHIFT;
                    (*res).cached = mips64_cca_cached(pca as m_uint8_t) as u_int;

                    (*res).tlb_index = i as u_int;
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
                    (*res).cached = mips64_cca_cached(pca as m_uint8_t) as u_int;

                    (*res).tlb_index = i as u_int;
                    return TRUE;
                }
            }

            // Invalid entry
            return FALSE;
        }
    }

    // No matching entry
    return FALSE;
}

/// Map a TLB entry into the MTS.
///
/// We apply the physical address bus masking here.
///
/// TODO: - Manage ASID
///       - Manage CPU Mode (user,supervisor or kernel)
#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
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

/// Unmap a TLB entry in the MTS.
#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_unmap_tlb_to_mts(cpu: *mut cpu_mips_t, index: c_int) {
    let v0_addr: m_uint64_t;
    let v1_addr: m_uint64_t;
    let page_size: m_uint32_t;
    let entry: *mut tlb_entry_t;

    entry = addr_of_mut!((*cpu).cp0.tlb[index as usize]);

    page_size = get_page_size((*entry).mask as m_uint32_t);
    v0_addr = (*entry).hi & mips64_cp0_get_vpn2_mask(cpu);
    v1_addr = v0_addr + page_size as m_uint64_t;

    if ((*entry).lo0 & MIPS_TLB_V_MASK as m_uint64_t) != 0 {
        (*cpu).mts_unmap.unwrap()(cpu, v0_addr, page_size, MTS_ACC_T, index);
    }

    if ((*entry).lo1 & MIPS_TLB_V_MASK as m_uint64_t) != 0 {
        (*cpu).mts_unmap.unwrap()(cpu, v1_addr, page_size, MTS_ACC_T, index);
    }
}

/// Map all TLB entries into the MTS
#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_map_all_tlb_to_mts(cpu: *mut cpu_mips_t) {
    for i in 0..(*cpu).cp0.tlb_entries as c_int {
        mips64_cp0_map_tlb_to_mts(cpu, i);
    }
}

/// Execute a callback for the specified entry
#[cfg(feature = "USE_UNSTABLE")]
#[inline]
unsafe fn mips64_cp0_tlb_callback(cpu: *mut cpu_mips_t, entry: *mut tlb_entry_t, action: c_int) {
    let vaddr: m_uint64_t = (*entry).hi & mips64_cp0_get_vpn2_mask(cpu);
    let psize: m_uint32_t = get_page_size((*entry).mask as m_uint32_t);

    if ((*entry).lo0 & MIPS_TLB_V_MASK as m_uint64_t) != 0 {
        let paddr0: m_uint64_t = ((*entry).lo0 & MIPS_TLB_PFN_MASK as m_uint64_t) << 6;

        if false {
            libc::printf(cstr!("TLB: vaddr=0x%8.8llx -> paddr0=0x%10.10llx (size=0x%8.8x), action=%s\n"), vaddr, paddr0, psize, if action == 0 { cstr!("ADD") } else { cstr!("DELETE") });
        }
    }

    if ((*entry).lo1 & MIPS_TLB_V_MASK as m_uint64_t) != 0 {
        let paddr1: m_uint64_t = ((*entry).lo1 & MIPS_TLB_PFN_MASK as m_uint64_t) << 6;

        if false {
            libc::printf(cstr!("TLB: vaddr=0x%8.8llx -> paddr1=0x%10.10llx (size=0x%8.8x), action=%s\n"), vaddr, paddr1, psize, if action == 0 { cstr!("ADD") } else { cstr!("DELETE") });
        }
    }
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

    for i in 0..(*cp0).tlb_entries as c_int {
        entry = addr_of_mut!((*cp0).tlb[i as usize]);

        page_mask = !(*entry).mask;
        hi_addr = (*entry).hi & vpn2_mask & page_mask;

        if ((vpn_addr & page_mask) == hi_addr) && (((*entry).hi & MIPS_TLB_G_MASK as m_uint64_t) != 0 || (((*entry).hi & MIPS_TLB_ASID_MASK as m_uint64_t) == asid as m_uint64_t)) {
            page_size = get_page_size((*entry).mask as m_uint32_t);

            if (vaddr & page_size as m_uint64_t) == 0 {
                // Even Page
                if ((*entry).lo0 & MIPS_TLB_V_MASK) != 0 {
                    // Check write protection
                    if (op_type == MTS_WRITE) && ((*entry).lo0 & MIPS_TLB_D_MASK) == 0 {
                        return MIPS_TLB_LOOKUP_MOD;
                    }

                    (*res).flags = 0;
                    (*res).vaddr = vaddr & MIPS_MIN_PAGE_MASK;
                    (*res).paddr = ((*entry).lo0 & MIPS_TLB_PFN_MASK) << 6;
                    (*res).paddr += (*res).vaddr & (page_size - 1) as m_uint64_t;
                    (*res).paddr &= (*cpu).addr_bus_mask;

                    (*res).offset = (vaddr & MIPS_MIN_PAGE_IMASK) as m_uint32_t;

                    pca = ((*entry).lo0 & MIPS_TLB_C_MASK) as m_uint32_t;
                    pca >>= MIPS_TLB_C_SHIFT;
                    (*res).cached = mips64_cca_cached(pca as m_uint8_t) as m_uint32_t;

                    if ((*entry).lo0 & MIPS_TLB_D_MASK) == 0 {
                        (*res).flags |= MTS_FLAG_RO;
                    }

                    return MIPS_TLB_LOOKUP_OK;
                }
            } else {
                // Odd Page
                if ((*entry).lo1 & MIPS_TLB_V_MASK) != 0 {
                    // Check write protection
                    if (op_type == MTS_WRITE) && ((*entry).lo1 & MIPS_TLB_D_MASK) == 0 {
                        return MIPS_TLB_LOOKUP_MOD;
                    }

                    (*res).flags = 0;
                    (*res).vaddr = vaddr & MIPS_MIN_PAGE_MASK;
                    (*res).paddr = ((*entry).lo1 & MIPS_TLB_PFN_MASK) << 6;
                    (*res).paddr += (*res).vaddr & (page_size - 1) as m_uint64_t;
                    (*res).paddr &= (*cpu).addr_bus_mask;

                    (*res).offset = (vaddr & MIPS_MIN_PAGE_IMASK) as m_uint32_t;

                    pca = ((*entry).lo1 & MIPS_TLB_C_MASK) as m_uint32_t;
                    pca >>= MIPS_TLB_C_SHIFT;
                    (*res).cached = mips64_cca_cached(pca as m_uint8_t) as m_uint32_t;

                    if ((*entry).lo1 & MIPS_TLB_D_MASK) == 0 {
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

/// TLBP: Probe a TLB entry
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_exec_tlbp(cpu: *mut cpu_mips_t) {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);
    let mut entry: *mut tlb_entry_t;

    let vpn2_mask: m_uint64_t = mips64_cp0_get_vpn2_mask(cpu);
    let hi_reg: m_uint64_t = (*cp0).reg[MIPS_CP0_TLB_HI];
    let asid: m_uint64_t = hi_reg & MIPS_TLB_ASID_MASK as m_uint64_t;
    let vpn2: m_uint64_t = hi_reg & vpn2_mask;

    (*cp0).reg[MIPS_CP0_INDEX] = 0xffffffff80000000_u64;

    for i in 0..(*cp0).tlb_entries as c_int {
        entry = addr_of_mut!((*cp0).tlb[i as usize]);

        #[cfg(not(feature = "USE_UNSTABLE"))]
        {
            if (((*entry).hi & vpn2_mask) == vpn2) && (((*entry).hi & MIPS_TLB_G_MASK as m_uint64_t) != 0 || (((*entry).hi & MIPS_TLB_ASID_MASK as m_uint64_t) == asid)) {
                (*cp0).reg[MIPS_CP0_INDEX] = i as m_uint64_t;
                if DEBUG_TLB_ACTIVITY != 0 {
                    libc::printf(cstr!("CPU: CP0_TLBP returned %u\n"), i);
                    mips64_tlb_dump((*cpu).gen);
                }
            }
        }
        #[cfg(feature = "USE_UNSTABLE")]
        {
            let page_mask: m_uint64_t = !(*entry).mask;

            if (((*entry).hi & vpn2_mask & page_mask) == vpn2) && (((*entry).hi & MIPS_TLB_G_MASK as m_uint64_t) != 0 || (((*entry).hi & MIPS_TLB_ASID_MASK as m_uint64_t) == asid)) {
                (*cp0).reg[MIPS_CP0_INDEX] = i as m_uint64_t;
                if DEBUG_TLB_ACTIVITY != 0 {
                    libc::printf(cstr!("CPU: CP0_TLBP returned %u\n"), i);
                    mips64_tlb_dump((*cpu).gen);
                }
            }
        }
    }
}

/* TLBR: Read Indexed TLB entry */
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_exec_tlbr(cpu: *mut cpu_mips_t) {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);
    let entry: *mut tlb_entry_t;

    let index: u_int = (*cp0).reg[MIPS_CP0_INDEX] as u_int;

    if DEBUG_TLB_ACTIVITY != 0 {
        cpu_log!((*cpu).gen, cstr!("TLB"), cstr!("CP0_TLBR: reading entry %u.\n"), index);
    }

    if index < (*cp0).tlb_entries {
        entry = addr_of_mut!((*cp0).tlb[index as usize]);

        (*cp0).reg[MIPS_CP0_PAGEMASK] = (*entry).mask;
        (*cp0).reg[MIPS_CP0_TLB_HI] = (*entry).hi;
        (*cp0).reg[MIPS_CP0_TLB_LO_0] = (*entry).lo0;
        (*cp0).reg[MIPS_CP0_TLB_LO_1] = (*entry).lo1;

        // The G bit must be reported in both Lo0 and Lo1 registers,
        // and cleared in Hi register.
        if ((*entry).hi & MIPS_TLB_G_MASK as m_uint64_t) != 0 {
            (*cp0).reg[MIPS_CP0_TLB_LO_0] |= MIPS_CP0_LO_G_MASK;
            (*cp0).reg[MIPS_CP0_TLB_LO_1] |= MIPS_CP0_LO_G_MASK;
            (*cp0).reg[MIPS_CP0_TLB_HI] &= !(MIPS_TLB_G_MASK as m_uint64_t);
        }
    }
}

/// TLBW: Write a TLB entry
#[inline]
unsafe fn mips64_cp0_exec_tlbw(cpu: *mut cpu_mips_t, index: u_int) {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);
    let entry: *mut tlb_entry_t;

    if DEBUG_TLB_ACTIVITY != 0 {
        cpu_log!((*cpu).gen, cstr!("TLB"), cstr!("CP0_TLBWI: writing entry %u [mask=0x%8.8llx,hi=0x%8.8llx,lo0=0x%8.8llx,lo1=0x%8.8llx]\n"), index, (*cp0).reg[MIPS_CP0_PAGEMASK], (*cp0).reg[MIPS_CP0_TLB_HI], (*cp0).reg[MIPS_CP0_TLB_LO_0], (*cp0).reg[MIPS_CP0_TLB_LO_1]);
    }

    if index < (*cp0).tlb_entries {
        entry = addr_of_mut!((*cp0).tlb[index as usize]);

        #[cfg(not(feature = "USE_UNSTABLE"))]
        {
            // Unmap the old entry if it was valid
            mips64_cp0_unmap_tlb_to_mts(cpu, index as c_int);
        }
        #[cfg(feature = "USE_UNSTABLE")]
        {
            mips64_cp0_tlb_callback(cpu, entry, TLB_ZONE_ADD);
        }

        (*entry).mask = (*cp0).reg[MIPS_CP0_PAGEMASK] & MIPS_TLB_PAGE_MASK;
        #[cfg(not(feature = "USE_UNSTABLE"))]
        {
            (*entry).hi = (*cp0).reg[MIPS_CP0_TLB_HI] & !(*entry).mask;
            (*entry).hi &= MIPS_CP0_HI_SAFE_MASK; // clear G bit
        }
        #[cfg(feature = "USE_UNSTABLE")]
        {
            (*entry).hi = (*cp0).reg[MIPS_CP0_TLB_HI];
        }
        (*entry).lo0 = (*cp0).reg[MIPS_CP0_TLB_LO_0];
        (*entry).lo1 = (*cp0).reg[MIPS_CP0_TLB_LO_1];

        // if G bit is set in lo0 and lo1, set it in hi
        if (((*entry).lo0 & (*entry).lo1) & MIPS_CP0_LO_G_MASK as m_uint64_t) != 0 {
            (*entry).hi |= MIPS_TLB_G_MASK as m_uint64_t;
        } else {
            #[cfg(feature = "USE_UNSTABLE")]
            {
                (*entry).hi &= !MIPS_TLB_G_MASK;
            }
        }

        // Clear G bit in TLB lo0 and lo1
        (*entry).lo0 &= !MIPS_CP0_LO_G_MASK;
        (*entry).lo1 &= !MIPS_CP0_LO_G_MASK;

        #[cfg(not(feature = "USE_UNSTABLE"))]
        {
            // Inform the MTS subsystem
            mips64_cp0_map_tlb_to_mts(cpu, index as c_int);
        }
        #[cfg(feature = "USE_UNSTABLE")]
        {
            // Inform the MTS subsystem
            (*cpu).mts_invalidate.unwrap()(cpu);

            mips64_cp0_tlb_callback(cpu, entry, TLB_ZONE_DELETE);
        }

        if DEBUG_TLB_ACTIVITY != 0 {
            mips64_tlb_dump_entry(cpu, index);
        }
    }
}

/// TLBWI: Write Indexed TLB entry
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_exec_tlbwi(cpu: *mut cpu_mips_t) {
    mips64_cp0_exec_tlbw(cpu, (*cpu).cp0.reg[MIPS_CP0_INDEX] as u_int);
}

/// TLBWR: Write Random TLB entry
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_exec_tlbwr(cpu: *mut cpu_mips_t) {
    mips64_cp0_exec_tlbw(cpu, mips64_cp0_get_random_reg(cpu));
}

/// Raw dump of the TLB
#[no_mangle]
pub unsafe extern "C" fn mips64_tlb_raw_dump(cpu: *mut cpu_gen_t) {
    let mcpu: *mut cpu_mips_t = CPU_MIPS64(cpu);
    let mut entry: *mut tlb_entry_t;

    libc::printf(cstr!("TLB dump:\n"));

    for i in 0..(*mcpu).cp0.tlb_entries as u_int {
        entry = addr_of_mut!((*mcpu).cp0.tlb[i as usize]);
        libc::printf(cstr!(" %2d: mask=0x%16.16llx hi=0x%16.16llx lo0=0x%16.16llx lo1=0x%16.16llx\n"), i, (*entry).mask, (*entry).hi, (*entry).lo0, (*entry).lo1);
    }

    libc::printf(cstr!("\n"));
}

/// Dump the specified TLB entry
#[no_mangle]
pub unsafe extern "C" fn mips64_tlb_dump_entry(cpu: *mut cpu_mips_t, index: u_int) {
    let mut buffer: [c_char; 256] = [0; 256];

    let entry: *mut tlb_entry_t = addr_of_mut!((*cpu).cp0.tlb[index as usize]);

    // virtual Address
    libc::printf(cstr!(" %2d: vaddr=0x%8.8llx "), index, (*entry).hi & mips64_cp0_get_vpn2_mask(cpu));

    // global or ASID
    if ((*entry).hi & MIPS_TLB_G_MASK as m_uint64_t) != 0 {
        libc::printf(cstr!("(global)    "));
    } else {
        libc::printf(cstr!("(asid 0x%2.2llx) "), (*entry).hi & MIPS_TLB_ASID_MASK as m_uint64_t);
    }

    // 1st page: Lo0
    libc::printf(cstr!("p0="));

    if ((*entry).lo0 & MIPS_TLB_V_MASK as m_uint64_t) != 0 {
        libc::printf(cstr!("0x%9.9llx"), ((*entry).lo0 & MIPS_TLB_PFN_MASK as m_uint64_t) << 6);
    } else {
        libc::printf(cstr!("(invalid)  "));
    }

    libc::printf(cstr!(" %c "), if ((*entry).lo0 & MIPS_TLB_D_MASK as m_uint64_t) != 0 { b'D' as c_int } else { b' ' as c_int });

    // 2nd page: Lo1
    libc::printf(cstr!("p1="));

    if ((*entry).lo1 & MIPS_TLB_V_MASK as m_uint64_t) != 0 {
        libc::printf(cstr!("0x%9.9llx"), ((*entry).lo1 & MIPS_TLB_PFN_MASK as m_uint64_t) << 6);
    } else {
        libc::printf(cstr!("(invalid)  "));
    }

    libc::printf(cstr!(" %c "), if ((*entry).lo1 & MIPS_TLB_D_MASK as m_uint64_t) != 0 { b'D' as c_int } else { b' ' as c_int });

    // page size
    libc::printf(cstr!(" (%s)\n"), get_page_size_str(buffer.as_c_mut(), buffer.len(), (*entry).mask as m_uint32_t));
}

/// Human-Readable dump of the TLB
#[no_mangle]
pub unsafe extern "C" fn mips64_tlb_dump(cpu: *mut cpu_gen_t) {
    let mcpu: *mut cpu_mips_t = CPU_MIPS64(cpu);

    libc::printf(cstr!("TLB dump:\n"));

    for i in 0..(*mcpu).cp0.tlb_entries as u_int {
        mips64_tlb_dump_entry(mcpu, i);
    }

    libc::printf(cstr!("\n"));
}
