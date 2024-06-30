//! ATM Virtual Segmentation & Reassembly Engine.

use crate::dynamips_common::*;
use crate::prelude::*;

pub const ATM_REAS_MAX_SIZE: usize = 16384;

/// Reassembly Context
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct atm_reas_context {
    pub buffer: [m_uint8_t; ATM_REAS_MAX_SIZE],
    pub buf_pos: size_t,
    pub len: size_t,
}

/// Reset a receive context
#[no_mangle]
pub unsafe extern "C" fn atm_aal5_recv_reset(arc: *mut atm_reas_context) {
    (*arc).buf_pos = 0;
    (*arc).len = 0;
}
