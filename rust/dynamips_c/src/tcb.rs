//! Translation Sharing Groups.

extern "C" {
    pub fn tsg_show_stats();
}

pub type cpu_tb_t = cpu_tb;
pub type cpu_tc_t = cpu_tc;

/// cbindgen:no-export
#[repr(C)]
pub struct cpu_tb {
    _todo: u8,
}

/// cbindgen:no-export
#[repr(C)]
pub struct cpu_tc {
    _todo: u8,
}
