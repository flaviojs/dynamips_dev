//! PowerPC MMU.

use crate::_private::*;
use crate::dynamips_common::*;
use crate::ppc32::*;

extern "C" {
    pub fn ppc32_get_bat_spr(cpu: *mut cpu_ppc_t, spr: u_int) -> m_uint32_t;
    pub fn ppc32_mem_invalidate_cache(cpu: *mut cpu_ppc_t);
    pub fn ppc32_mem_restart(cpu: *mut cpu_ppc_t) -> c_int;
    pub fn ppc32_set_bat_spr(cpu: *mut cpu_ppc_t, spr: u_int, val: m_uint32_t);
    pub fn ppc32_set_sdr1(cpu: *mut cpu_ppc_t, sdr1: m_uint32_t) -> c_int;
}

/// Set a BAT register
#[no_mangle]
pub unsafe extern "C" fn ppc32_set_bat(cpu: *mut cpu_ppc_t, bp: *mut ppc32_bat_prog) -> c_int {
    if ((*bp).type_ != PPC32_IBAT_IDX) && ((*bp).type_ != PPC32_DBAT_IDX) {
        return -1;
    }

    if (*bp).index >= PPC32_BAT_NR as c_int {
        return -1;
    }

    let bat: *mut ppc32_bat_reg = addr_of_mut!((*cpu).bat[(*bp).type_ as usize][(*bp).index as usize]);
    (*bat).reg[0] = (*bp).hi;
    (*bat).reg[1] = (*bp).lo;
    0
}
