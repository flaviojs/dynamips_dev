//! Cisco 3600 simulation platform.
//! Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
//! Patched by Jeremy Grossmann for the GNS3 project (www.gns3.net)
//!
//! NS16552 DUART.

use crate::_private::*;
use crate::cpu::*;
use crate::dev_vtty::*;
use crate::device::*;
use crate::dynamips_common::*;
use crate::memory::*;
use crate::ptask::*;
use crate::vm::*;

/// Debugging flags
const DEBUG_UNKNOWN: c_int = 1;
const DEBUG_ACCESS: c_int = 0;

/// Interrupt Enable Register (IER)
const IER_ERXRDY: u_int = 0x1;
const IER_ETXRDY: u_int = 0x2;

/// Interrupt Identification Register
const IIR_NPENDING: u_char = 0x01; // 0: irq pending, 1: no irq pending
const IIR_TXRDY: u_char = 0x02;
const IIR_RXRDY: u_char = 0x04;

/// Line Status Register (LSR)
const LSR_RXRDY: u_char = 0x01;
const LSR_TXRDY: u_char = 0x20;
const LSR_TXEMPTY: u_char = 0x40;

/// Line Control Register
const LCR_WRL0: u_char = 0x01;
const LCR_WRL1: u_char = 0x02;
const LCR_NUMSTOP: u_char = 0x04;
const LCR_PARITYON: u_char = 0x08;
const LCR_PARITYEV: u_char = 0x10;
const LCR_STICKP: u_char = 0x20;
const LCR_SETBREAK: u_char = 0x40;
const LCR_DIVLATCH: u_char = 0x80;

/// Modem Control Register
const MCR_DTR: u_char = 0x01;
const MCR_RTS: u_char = 0x02;
const MCR_OUT1: u_char = 0x04;
const MCR_OUT2: u_char = 0x08;
const MCR_LOOP: u_char = 0x10;

/// UART channel
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ns16552_channel {
    pub ier: u_int,
    pub output: u_int,
    pub vtty: *mut vtty_t,
}

/* NS16552 structure */
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ns16552_data {
    pub vm_obj: vm_obj_t,
    pub dev: vdevice,
    pub vm: *mut vm_instance_t,
    pub irq: u_int,

    /// Register offset divisor
    pub reg_div: u_int,

    /// Periodic task to trigger DUART IRQ
    pub tid: ptask_id_t,

    pub channel: [ns16552_channel; 2],
    pub duart_irq_seq: u_int,

    pub line_control_reg: u_int,
    pub div_latch: u_int,
    pub baud_divisor: u_int,
}

/// Console port input
unsafe extern "C" fn tty_con_input(vtty: *mut vtty_t) {
    let d: *mut ns16552_data = (*vtty).priv_data.cast::<_>();

    if ((*d).channel[0].ier & IER_ERXRDY) != 0 {
        vm_set_irq((*d).vm, (*d).irq);
    }
}

/// AUX port input
unsafe extern "C" fn tty_aux_input(vtty: *mut vtty_t) {
    let d: *mut ns16552_data = (*vtty).priv_data.cast::<_>();

    if ((*d).channel[1].ier & IER_ERXRDY) != 0 {
        vm_set_irq((*d).vm, (*d).irq);
    }
}

/// IRQ trickery for Console and AUX ports
unsafe extern "C" fn tty_trigger_dummy_irq(d: *mut c_void, _arg: *mut c_void) -> c_int {
    let d: *mut ns16552_data = d.cast::<_>();
    (*d).duart_irq_seq += 1;

    if (*d).duart_irq_seq == 2 {
        if ((*d).channel[0].ier & IER_ETXRDY) != 0 {
            (*d).channel[0].output = TRUE as u_int;
            vm_set_irq((*d).vm, (*d).irq);
        }

        if ((*d).channel[1].ier & IER_ETXRDY) != 0 {
            (*d).channel[1].output = TRUE as u_int;
            vm_set_irq((*d).vm, (*d).irq);
        }

        (*d).duart_irq_seq = 0;
    }

    0
}

/// dev_ns16552_access()
unsafe extern "C" fn dev_ns16552_access(cpu: *mut cpu_gen_t, dev: *mut vdevice, mut offset: m_uint32_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let d: *mut ns16552_data = (*dev).priv_data.cast::<_>();
    let mut channel: c_int = 0;
    let mut odata: u_char;

    if op_type == MTS_READ {
        *data = 0;
    }

    if DEBUG_ACCESS != 0 {
        if op_type == MTS_READ {
            cpu_log!(cpu, cstr!("NS16552"), cstr!("read from 0x%x, pc=0x%llx\n"), offset, cpu_get_pc(cpu));
        } else {
            cpu_log!(cpu, cstr!("NS16552"), cstr!("write to 0x%x, value=0x%llx, pc=0x%llx\n"), offset, *data, cpu_get_pc(cpu));
        }
    }

    offset >>= (*d).reg_div;

    if offset >= 0x08 {
        channel = 1;
    }

    // From the NS16552V datasheet, the following is known about the registers
    // Bit 4 is channel
    // Value 0 Receive or transmit buffer
    // Value 1 Interrupt enable
    // Value 2 Interrupt identification (READ), FIFO Config (Write)
    // Value 3 Line Control (Appears in IOS)
    //    0x1 - Word Length Selector bit 0
    //    0x2 - Word Length Selector bit 1
    //    0x4 - Num stop bits
    //    0x8 - Parity Enable
    //    0x16 - Parity even
    //    0x32 - Stick Parity
    //    0x64 - Set Break
    //    0x128 - Division Latch
    // Value 4 Modem Control (Appears in IOS)
    // Value 5 Line status
    // Value 6 Modem Status
    // Value 7 Scratch

    match offset {
        // Receiver Buffer Reg. (RBR) / Transmitting Holding Reg. (THR)
        0x00 | 0x08 => {
            if (*d).div_latch == 0 {
                if op_type == MTS_WRITE {
                    vtty_put_char((*d).channel[channel as usize].vtty, *data as c_char);

                    if ((*d).channel[channel as usize].ier & IER_ETXRDY) != 0 {
                        vm_set_irq((*d).vm, (*d).irq);
                    }

                    (*d).channel[channel as usize].output = TRUE as u_int;
                } else {
                    *data = vtty_get_char((*d).channel[channel as usize].vtty) as m_uint64_t;
                }
            } else {
                #[allow(clippy::collapsible_else_if)]
                if op_type == MTS_WRITE {
                    (*d).baud_divisor = (((*data) & 0x00ff) | ((*d).baud_divisor & 0xff00) as m_uint64_t) as u_int;
                }
            }
        }

        // Interrupt Enable Register (IER)
        0x01 | 0x09 => {
            if (*d).div_latch == 0 {
                if op_type == MTS_READ {
                    *data = (*d).channel[channel as usize].ier as m_uint64_t;
                } else {
                    (*d).channel[channel as usize].ier = (*data & 0xFF) as u_int;

                    if (*data & 0x02) == 0 {
                        // transmit holding register
                        (*(*d).channel[channel as usize].vtty).managed_flush = TRUE;
                        vtty_flush((*d).channel[channel as usize].vtty);
                    }
                }
            } else {
                #[allow(clippy::collapsible_else_if)]
                if op_type == MTS_WRITE {
                    (*d).baud_divisor = ((((*data) & 0xff) << 8) | ((*d).baud_divisor & 0xff) as m_uint64_t) as u_int;
                }
            }
        }

        // Interrupt Ident Register (IIR)
        0x02 | 0x0A => {
            if (*d).div_latch == 0 {
                vm_clear_irq((*d).vm, (*d).irq);
                if op_type == MTS_READ {
                    odata = IIR_NPENDING;

                    if vtty_is_char_avail((*d).channel[channel as usize].vtty) != 0 {
                        odata = IIR_RXRDY;
                    } else {
                        #[allow(clippy::collapsible_else_if)]
                        if (*d).channel[channel as usize].output != 0 {
                            odata = IIR_TXRDY;
                            (*d).channel[channel as usize].output = 0;
                        }
                    }

                    *data = odata as m_uint64_t;
                }
            }
        }

        // Line Control Register (LCR)
        0x03 | 0x0B => {
            if op_type == MTS_READ {
                *data = (*d).line_control_reg as m_uint64_t;
            } else {
                (*d).line_control_reg = *data as u_int;
                let mut bits: u_int = 5;
                let mut stop: *mut c_char = cstr!("1");
                let mut parity: *mut c_char = cstr!("no ");
                let mut parityeven: *mut c_char = cstr!("odd");
                if (*data & LCR_WRL0 as m_uint64_t) != 0 {
                    bits += 1;
                }
                if (*data & LCR_WRL1 as m_uint64_t) != 0 {
                    bits += 2;
                }

                if (*data & LCR_NUMSTOP as m_uint64_t) != 0 {
                    if bits >= 6 {
                        stop = cstr!("2");
                    } else {
                        stop = cstr!("1.5");
                    }
                }

                if (*data & LCR_PARITYON as m_uint64_t) != 0 {
                    parity = cstr!(""); // Parity on
                }
                if (*data & LCR_PARITYEV as m_uint64_t) != 0 {
                    parityeven = cstr!("even");
                }

                // DIV LATCH changes the behavior of 0x0,0x1,and 0x2
                if (*data & LCR_DIVLATCH as m_uint64_t) != 0 {
                    (*d).div_latch = 1;
                } else {
                    (*d).div_latch = 0;
                    //  1200 divisor was 192
                    //  9600 divisor was  24
                    // 19200 divisor was  12
                    // Suggests a crystal of 3686400 hz
                    let baud: u_int = if (*d).baud_divisor > 0 { 3686400 / ((*d).baud_divisor * 16) } else { 0 };
                    let _ = baud;
                }
                let _ = (bits, stop, parity, parityeven);
            }
        }

        // MODEM Control Register (MCR)
        0x04 | 0x0C => {
            if op_type != MTS_READ {
                let mut f1: *mut c_char = cstr!("");
                let mut f2: *mut c_char = cstr!("");
                let mut f3: *mut c_char = cstr!("");
                let mut f4: *mut c_char = cstr!("");
                let mut f5: *mut c_char = cstr!("");
                if (*data & MCR_DTR as m_uint64_t) != 0 {
                    f1 = cstr!("DTR ");
                }
                if (*data & MCR_RTS as m_uint64_t) != 0 {
                    f2 = cstr!("RTS ");
                }
                if (*data & MCR_OUT1 as m_uint64_t) != 0 {
                    f3 = cstr!("OUT1 ");
                }
                if (*data & MCR_OUT2 as m_uint64_t) != 0 {
                    f4 = cstr!("OUT2 ");
                }
                if (*data & MCR_LOOP as m_uint64_t) != 0 {
                    f5 = cstr!("LOOP ");
                }
                let _ = (f1, f2, f3, f4, f5);
            }
        }

        // Line Status Register (LSR)
        0x05 | 0x0D => {
            if op_type == MTS_READ {
                odata = 0;

                if vtty_is_char_avail((*d).channel[channel as usize].vtty) != 0 {
                    odata |= LSR_RXRDY;
                }

                odata |= LSR_TXRDY | LSR_TXEMPTY;
                *data = odata as m_uint64_t;
            }
        }

        // MODEM Status Register (MSR)?
        #[cfg(if_0)]
        0x06 | 0x0E => {}

        // Scratch Register (SCR)?
        #[cfg(if_0)]
        0x07 | 0x0F => {}

        _ => {
            if DEBUG_UNKNOWN != 0 {
                if op_type == MTS_READ {
                    cpu_log!(cpu, cstr!("NS16552"), cstr!("read from addr 0x%x, pc=0x%llx (size=%u)\n"), offset, cpu_get_pc(cpu), op_size);
                } else {
                    cpu_log!(cpu, cstr!("NS16552"), cstr!("write to addr 0x%x, value=0x%llx, pc=0x%llx (size=%u)\n"), offset, *data, cpu_get_pc(cpu), op_size);
                }
            }
        }
    }

    null_mut()
}

/// Shutdown a NS16552 device
unsafe extern "C" fn dev_ns16552_shutdown(vm: *mut vm_instance_t, d: *mut c_void) -> *mut c_void {
    let d: *mut ns16552_data = d.cast::<_>();
    if !d.is_null() {
        (*(*d).channel[0].vtty).read_notifier = None;
        (*(*d).channel[1].vtty).read_notifier = None;

        // Remove the periodic task
        ptask_remove((*d).tid);

        // Remove the device
        dev_remove(vm, addr_of_mut!((*d).dev));

        // Free the structure itself
        libc::free(d.cast::<_>());
    }

    null_mut()
}

/// Create a NS16552 device
#[no_mangle]
pub unsafe extern "C" fn dev_ns16552_init(vm: *mut vm_instance_t, paddr: m_uint64_t, len: m_uint32_t, reg_div: u_int, irq: u_int, vtty_A: *mut vtty_t, vtty_B: *mut vtty_t) -> c_int {
    // Allocate private data structure
    let d: *mut ns16552_data = libc::malloc(size_of::<ns16552_data>()).cast::<_>();
    if d.is_null() {
        libc::fprintf(c_stderr(), cstr!("NS16552: out of memory\n"));
        return -1;
    }

    libc::memset(d.cast::<_>(), 0, size_of::<ns16552_data>());
    (*d).vm = vm;
    (*d).irq = irq;
    (*d).reg_div = reg_div;
    (*d).channel[0].vtty = vtty_A;
    (*d).channel[1].vtty = vtty_B;
    (*d).line_control_reg = 0;
    (*d).div_latch = 0;
    (*d).baud_divisor = 0;

    vm_object_init(addr_of_mut!((*d).vm_obj));
    (*d).vm_obj.name = cstr!("ns16552");
    (*d).vm_obj.data = d.cast::<_>();
    (*d).vm_obj.shutdown = Some(dev_ns16552_shutdown);

    // Set device properties
    dev_init(addr_of_mut!((*d).dev));
    (*d).dev.name = cstr!("ns16552");
    (*d).dev.phys_addr = paddr;
    (*d).dev.phys_len = len;
    (*d).dev.handler = Some(dev_ns16552_access);
    (*d).dev.priv_data = d.cast::<_>();

    (*vtty_A).priv_data = d.cast::<_>();
    (*vtty_B).priv_data = d.cast::<_>();
    (*vtty_A).read_notifier = Some(tty_con_input);
    (*vtty_B).read_notifier = Some(tty_aux_input);

    // Trigger periodically a dummy IRQ to flush buffers
    (*d).tid = ptask_add(Some(tty_trigger_dummy_irq), d.cast::<_>(), null_mut());

    // Map this device to the VM
    vm_bind_device(vm, addr_of_mut!((*d).dev));
    vm_object_add(vm, addr_of_mut!((*d).vm_obj));
    0
}
