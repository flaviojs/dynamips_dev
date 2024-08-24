//! Cisco router simulation platform.
//! Copyright (c) 2005 Christophe Fillot (cf@utc.fr)
//!
//! SB-1 I/O devices.
//!
//! XXX: just for tests!

use crate::_private::*;
use crate::cpu::*;
use crate::dev_vtty::*;
use crate::device::*;
use crate::dynamips_common::*;
use crate::memory::*;
use crate::ptask::*;
use crate::vm::*;

const DEBUG_UNKNOWN: c_int = 1;

/// DUART Status Register
const DUART_SR_RX_RDY: m_uint8_t = 0x01; // Receiver ready
const DUART_SR_RX_FFUL: m_uint8_t = 0x02; // Receive FIFO full
const DUART_SR_TX_RDY: m_uint8_t = 0x04; // Transmitter ready
const DUART_SR_TX_EMT: m_uint8_t = 0x08; // Transmitter empty

/// DUART Interrupt Status Register
const DUART_ISR_TXA: m_uint8_t = 0x01; // Channel A Transmitter Ready
const DUART_ISR_RXA: m_uint8_t = 0x02; // Channel A Receiver Ready
const DUART_ISR_TXB: m_uint8_t = 0x10; // Channel B Transmitter Ready
const DUART_ISR_RXB: m_uint8_t = 0x20; // Channel B Receiver Ready

/// DUART Interrupt Mask Register
const DUART_IMR_TXA: m_uint8_t = 0x01; // Channel A Transmitter Ready
const DUART_IMR_RXA: m_uint8_t = 0x02; // Channel A Receiver Ready
const DUART_IMR_TXB: m_uint8_t = 0x10; // Channel B Transmitter Ready
const DUART_IMR_RXB: m_uint8_t = 0x20; // Channel B Receiver Ready

/// SB-1 DUART channel
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sb1_duart_channel {
    pub mode: m_uint8_t,
    pub cmd: m_uint8_t,
}

/// SB-1 I/O private data
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sb1_io_data {
    pub vm_obj: vm_obj_t,
    pub dev: vdevice,

    /// Virtual machine
    pub vm: *mut vm_instance_t,
    pub duart_irq: u_int,
    pub duart_irq_seq: u_int,
    pub duart_isr: m_uint8_t,
    pub duart_imr: m_uint8_t,
    pub duart_chan: [sb1_duart_channel; 2],

    /// Periodic task to trigger dummy DUART IRQ
    pub duart_irq_tid: ptask_id_t,
}

/// Console port input
unsafe extern "C" fn tty_con_input(vtty: *mut vtty_t) {
    let d: *mut sb1_io_data = (*vtty).priv_data.cast::<_>();

    if ((*d).duart_imr & DUART_IMR_RXA) != 0 {
        (*d).duart_isr |= DUART_ISR_RXA;
        vm_set_irq((*d).vm, (*d).duart_irq);
    }
}

/// AUX port input
unsafe extern "C" fn tty_aux_input(vtty: *mut vtty_t) {
    let d: *mut sb1_io_data = (*vtty).priv_data.cast::<_>();

    if ((*d).duart_imr & DUART_IMR_RXB) != 0 {
        (*d).duart_isr |= DUART_ISR_RXB;
        vm_set_irq((*d).vm, (*d).duart_irq);
    }
}

/// IRQ trickery for Console and AUX ports
unsafe extern "C" fn tty_trigger_dummy_irq(d: *mut c_void, _arg: *mut c_void) -> c_int {
    let d: *mut sb1_io_data = d.cast::<_>();

    (*d).duart_irq_seq += 1;

    if (*d).duart_irq_seq == 2 {
        let mask: u_int = (DUART_IMR_TXA | DUART_IMR_TXB) as u_int;
        if ((*d).duart_imr as u_int & mask) != 0 {
            (*d).duart_isr |= DUART_ISR_TXA | DUART_ISR_TXB;
            vm_set_irq((*d).vm, (*d).duart_irq);
        }

        (*d).duart_irq_seq = 0;
    }

    0
}

/// dev_sb1_io_access()
unsafe extern "C" fn dev_sb1_io_access(cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, _op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let d: *mut sb1_io_data = (*dev).priv_data.cast::<_>();
    let mut odata: u_char;

    if op_type == MTS_READ {
        *data = 0;
    }

    match offset {
        0x390 => {
            // DUART Interrupt Status Register
            if op_type == MTS_READ {
                *data = (*d).duart_isr as m_uint64_t;
            }
        }

        0x320 => {
            // DUART Channel A Only Interrupt Status Register
            if op_type == MTS_READ {
                *data = ((*d).duart_isr & 0x0F) as m_uint64_t;
            }
        }

        0x340 => {
            // DUART Channel B Only Interrupt Status Register
            if op_type == MTS_READ {
                *data = (((*d).duart_isr >> 4) & 0x0F) as m_uint64_t;
            }
        }

        0x3a0 => {
            // DUART Interrupt Mask Register
            if op_type == MTS_READ {
                *data = (*d).duart_imr as m_uint64_t;
            } else {
                (*d).duart_imr = *data as m_uint8_t;
            }
        }

        0x330 => {
            // DUART Channel A Only Interrupt Mask Register
            if op_type == MTS_READ {
                *data = ((*d).duart_imr & 0x0F) as m_uint64_t;
            } else {
                (*d).duart_imr &= !0x0F;
                (*d).duart_imr |= (*data & 0x0F) as m_uint8_t;
            }
        }

        0x350 => {
            // DUART Channel B Only Interrupt Mask Register
            if op_type == MTS_READ {
                *data = (((*d).duart_imr >> 4) & 0x0F) as m_uint64_t;
            } else {
                (*d).duart_imr &= !0xF0;
                (*d).duart_imr |= ((*data & 0x0F) << 4) as m_uint8_t;
            }
        }

        0x100 => {
            // DUART Mode (Channel A)
            if op_type == MTS_READ {
                (*d).duart_chan[0].mode = *data as m_uint8_t;
            } else {
                *data = (*d).duart_chan[0].mode as m_uint64_t;
            }
        }

        0x200 => {
            // DUART Mode (Channel B)
            if op_type == MTS_READ {
                (*d).duart_chan[1].mode = *data as m_uint8_t;
            } else {
                *data = (*d).duart_chan[1].mode as m_uint64_t;
            }
        }

        0x150 => {
            // DUART Command Register (Channel A)
            if op_type == MTS_READ {
                (*d).duart_chan[0].cmd = *data as m_uint8_t;
            } else {
                *data = (*d).duart_chan[0].cmd as m_uint64_t;
            }
        }

        0x250 => {
            // DUART Command Register (Channel B)
            if op_type == MTS_READ {
                (*d).duart_chan[1].cmd = *data as m_uint8_t;
            } else {
                *data = (*d).duart_chan[1].cmd as m_uint64_t;
            }
        }

        0x120 => {
            // DUART Status Register (Channel A)
            if op_type == MTS_READ {
                odata = 0;

                if vtty_is_char_avail((*(*d).vm).vtty_con) != 0 {
                    odata |= DUART_SR_RX_RDY;
                }

                odata |= DUART_SR_TX_RDY;

                vm_clear_irq((*d).vm, (*d).duart_irq);
                *data = odata as m_uint64_t;
            }
        }

        0x220 => {
            // DUART Status Register (Channel B)
            if op_type == MTS_READ {
                odata = 0;

                if vtty_is_char_avail((*(*d).vm).vtty_aux) != 0 {
                    odata |= DUART_SR_RX_RDY;
                }

                odata |= DUART_SR_TX_RDY;

                if false {
                    vm_clear_irq((*d).vm, (*d).duart_irq);
                }
                *data = odata as m_uint64_t;
            }
        }

        0x160 => {
            // DUART Received Data Register (Channel A)
            if op_type == MTS_READ {
                *data = vtty_get_char((*(*d).vm).vtty_con) as m_uint64_t;
                (*d).duart_isr &= !DUART_ISR_RXA;
            }
        }

        0x260 => {
            // DUART Received Data Register (Channel B)
            if op_type == MTS_READ {
                *data = vtty_get_char((*(*d).vm).vtty_aux) as m_uint64_t;
                (*d).duart_isr &= !DUART_ISR_RXB;
            }
        }

        0x170 => {
            // DUART Transmit Data Register (Channel A)
            if op_type == MTS_WRITE {
                vtty_put_char((*(*d).vm).vtty_con, *data as c_char);
                (*d).duart_isr &= !DUART_ISR_TXA;
            }
        }

        0x270 => {
            // DUART Transmit Data Register (Channel B)
            if op_type == MTS_WRITE {
                vtty_put_char((*(*d).vm).vtty_aux, *data as c_char);
                (*d).duart_isr &= !DUART_ISR_TXB;
            }
        }

        0x1a76 => {
            // pcmcia status
            if op_type == MTS_READ {
                *data = 0xFF;
            }
        }

        _ => {
            if DEBUG_UNKNOWN != 0 {
                if op_type == MTS_READ {
                    cpu_log!(cpu, cstr!("SB1_IO"), cstr!("read from addr 0x%x, pc=0x%llx\n"), offset, cpu_get_pc(cpu));
                } else {
                    cpu_log!(cpu, cstr!("SB1_IO"), cstr!("write to addr 0x%x, value=0x%llx, pc=0x%llx\n"), offset, *data, cpu_get_pc(cpu));
                }
            }
        }
    }

    null_mut()
}

/// Shutdown the SB-1 I/O devices
unsafe extern "C" fn dev_sb1_io_shutdown(vm: *mut vm_instance_t, d: *mut c_void) -> *mut c_void {
    let d: *mut sb1_io_data = d.cast::<_>();
    if !d.is_null() {
        // Remove the device
        dev_remove(vm, addr_of_mut!((*d).dev));

        // Free the structure itself
        libc::free(d.cast::<_>());
    }

    null_mut()
}

/// Create SB-1 I/O devices
#[no_mangle]
pub unsafe extern "C" fn dev_sb1_io_init(vm: *mut vm_instance_t, duart_irq: u_int) -> c_int {
    // allocate private data structure
    let d: *mut sb1_io_data = libc::malloc(size_of::<sb1_io_data>()).cast::<_>();
    if d.is_null() {
        libc::fprintf(c_stderr(), cstr!("SB1_IO: out of memory\n"));
        return -1;
    }

    libc::memset(d.cast::<_>(), 0, size_of::<sb1_io_data>());
    (*d).vm = vm;
    (*d).duart_irq = duart_irq;

    vm_object_init(addr_of_mut!((*d).vm_obj));
    (*d).vm_obj.name = cstr!("sb1_io");
    (*d).vm_obj.data = d.cast::<_>();
    (*d).vm_obj.shutdown = Some(dev_sb1_io_shutdown);

    // Set device properties
    dev_init(addr_of_mut!((*d).dev));
    (*d).dev.name = cstr!("sb1_io");
    (*d).dev.priv_data = d.cast::<_>();
    (*d).dev.phys_addr = 0x10060000_u64;
    (*d).dev.phys_len = 0x10000;
    (*d).dev.handler = Some(dev_sb1_io_access);

    // Set console and AUX port notifying functions
    (*(*vm).vtty_con).priv_data = d.cast::<_>();
    (*(*vm).vtty_aux).priv_data = d.cast::<_>();
    (*(*vm).vtty_con).read_notifier = Some(tty_con_input);
    (*(*vm).vtty_aux).read_notifier = Some(tty_aux_input);

    // Trigger periodically a dummy IRQ to flush buffers
    (*d).duart_irq_tid = ptask_add(Some(tty_trigger_dummy_irq), d.cast::<_>(), null_mut());

    // Map this device to the VM
    vm_bind_device(vm, addr_of_mut!((*d).dev));
    vm_object_add(vm, addr_of_mut!((*d).vm_obj));
    0
}
