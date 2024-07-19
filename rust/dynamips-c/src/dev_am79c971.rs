//! Cisco router simulation platform.
//! Copyright (C) 2006 Christophe Fillot.  All rights reserved.
//!
//! AMD Am79c971 FastEthernet chip emulation.

use crate::_private::*;
use crate::cpu::*;
use crate::device::*;
use crate::dynamips_common::*;
use crate::memory::*;
use crate::net::*;
use crate::net_io::*;
use crate::pci_dev::*;
use crate::ptask::*;
use crate::utils::*;
use crate::vm::*;
use std::cmp::min;

/// Interface type
pub const AM79C971_TYPE_100BASE_TX: c_int = 1; // 100baseTX
pub const AM79C971_TYPE_10BASE_T: c_int = 2; // 10baseT

/// Debugging flags
const DEBUG_CSR_REGS: u_int = 0;
const DEBUG_BCR_REGS: u_int = 0;
const DEBUG_PCI_REGS: u_int = 0;
const DEBUG_ACCESS: u_int = 0;
const DEBUG_TRANSMIT: u_int = 0;
const DEBUG_RECEIVE: u_int = 0;
const DEBUG_UNKNOWN: u_int = 0;

/// AMD Am79c971 PCI vendor/product codes
const AM79C971_PCI_VENDOR_ID: u_int = 0x1022;
const AM79C971_PCI_PRODUCT_ID: u_int = 0x2000;

/// Maximum packet size
const AM79C971_MAX_PKT_SIZE: usize = 2048;

/// Send up to 16 packets in a TX ring scan pass
const AM79C971_TXRING_PASS_COUNT: usize = 16;

/// CSR0: Controller Status and Control Register
const AM79C971_CSR0_ERR: m_uint32_t = 0x00008000; // Error (BABL,CERR,MISS,MERR)
const AM79C971_CSR0_BABL: m_uint32_t = 0x00004000; // Transmitter Timeout Error
const AM79C971_CSR0_CERR: m_uint32_t = 0x00002000; // Collision Error
const AM79C971_CSR0_MISS: m_uint32_t = 0x00001000; // Missed Frame
const AM79C971_CSR0_MERR: m_uint32_t = 0x00000800; // Memory Error
const AM79C971_CSR0_RINT: m_uint32_t = 0x00000400; // Receive Interrupt
const AM79C971_CSR0_TINT: m_uint32_t = 0x00000200; // Transmit Interrupt
const AM79C971_CSR0_IDON: m_uint32_t = 0x00000100; // Initialization Done
const AM79C971_CSR0_INTR: m_uint32_t = 0x00000080; // Interrupt Flag
const AM79C971_CSR0_IENA: m_uint32_t = 0x00000040; // Interrupt Enable
const AM79C971_CSR0_RXON: m_uint32_t = 0x00000020; // Receive On
const AM79C971_CSR0_TXON: m_uint32_t = 0x00000010; // Transmit On
const AM79C971_CSR0_TDMD: m_uint32_t = 0x00000008; // Transmit Demand
const AM79C971_CSR0_STOP: m_uint32_t = 0x00000004; // Stop
const AM79C971_CSR0_STRT: m_uint32_t = 0x00000002; // Start
const AM79C971_CSR0_INIT: m_uint32_t = 0x00000001; // Initialization

/// CSR3: Interrupt Masks and Deferral Control
const AM79C971_CSR3_BABLM: m_uint32_t = 0x00004000; // Transmit. Timeout Int. Mask
const AM79C971_CSR3_CERRM: m_uint32_t = 0x00002000; // Collision Error Int. Mask
const AM79C971_CSR3_MISSM: m_uint32_t = 0x00001000; // Missed Frame Interrupt Mask
const AM79C971_CSR3_MERRM: m_uint32_t = 0x00000800; // Memory Error Interrupt Mask
const AM79C971_CSR3_RINTM: m_uint32_t = 0x00000400; // Receive Interrupt Mask
const AM79C971_CSR3_TINTM: m_uint32_t = 0x00000200; // Transmit Interrupt Mask
const AM79C971_CSR3_IDONM: m_uint32_t = 0x00000100; // Initialization Done Mask
const AM79C971_CSR3_BSWP: m_uint32_t = 0x00000004; // Byte Swap
const AM79C971_CSR3_IM_MASK: m_uint32_t = 0x00007F00; // Interrupt Masks for CSR3

/// CSR5: Extended Control and Interrupt 1
const AM79C971_CSR5_TOKINTD: m_uint32_t = 0x00008000; // Receive Interrupt Mask
const AM79C971_CSR5_SPND: m_uint32_t = 0x00000001; // Suspend

/// CSR15: Mode
const AM79C971_CSR15_PROM: m_uint32_t = 0x00008000; // Promiscous Mode
const AM79C971_CSR15_DRCVBC: m_uint32_t = 0x00004000; // Disable Receive Broadcast
const AM79C971_CSR15_DRCVPA: m_uint32_t = 0x00002000; // Disable Receive PHY address
const AM79C971_CSR15_DTX: m_uint32_t = 0x00000002; // Disable Transmit
const AM79C971_CSR15_DRX: m_uint32_t = 0x00000001; // Disable Receive

/// AMD 79C971 Initialization block length
const AM79C971_INIT_BLOCK_LEN: usize = 0x1c;

/// RX descriptors
const AM79C971_RMD1_OWN: m_uint32_t = 0x80000000; // OWN=1: owned by Am79c971
const AM79C971_RMD1_ERR: m_uint32_t = 0x40000000; // Error
const AM79C971_RMD1_FRAM: m_uint32_t = 0x20000000; // Framing Error
const AM79C971_RMD1_OFLO: m_uint32_t = 0x10000000; // Overflow Error
const AM79C971_RMD1_CRC: m_uint32_t = 0x08000000; // Invalid CRC
const AM79C971_RMD1_BUFF: m_uint32_t = 0x08000000; // Buffer Error (chaining)
const AM79C971_RMD1_STP: m_uint32_t = 0x02000000; // Start of Packet
const AM79C971_RMD1_ENP: m_uint32_t = 0x01000000; // End of Packet
const AM79C971_RMD1_BPE: m_uint32_t = 0x00800000; // Bus Parity Error
const AM79C971_RMD1_PAM: m_uint32_t = 0x00400000; // Physical Address Match
const AM79C971_RMD1_LAFM: m_uint32_t = 0x00200000; // Logical Addr. Filter Match
const AM79C971_RMD1_BAM: m_uint32_t = 0x00100000; // Broadcast Address Match
const AM79C971_RMD1_LEN: m_uint32_t = 0x00000FFF; // Buffer Length

const AM79C971_RMD2_LEN: m_uint32_t = 0x00000FFF; // Received byte count

/// TX descriptors
const AM79C971_TMD1_OWN: m_uint32_t = 0x80000000; // OWN=1: owned by Am79c971
const AM79C971_TMD1_ERR: m_uint32_t = 0x40000000; // Error
const AM79C971_TMD1_ADD_FCS: m_uint32_t = 0x20000000; // FCS generation
const AM79C971_TMD1_STP: m_uint32_t = 0x02000000; // Start of Packet
const AM79C971_TMD1_ENP: m_uint32_t = 0x01000000; // End of Packet
const AM79C971_TMD1_LEN: m_uint32_t = 0x00000FFF; // Buffer Length

/// RX Descriptor
/// cbindgen:no-export
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rx_desc {
    pub rmd: [m_uint32_t; 4],
}

/// TX Descriptor
/// cbindgen:no-export
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct tx_desc {
    pub tmd: [m_uint32_t; 4],
}

/// AMD 79C971 Data
/// cbindgen:no-export
#[repr(C)]
#[derive(Copy, Clone)]
pub struct am79c971_data {
    pub name: *mut c_char,

    /// Lock
    pub lock: libc::pthread_mutex_t,

    /// Interface type (10baseT or 100baseTX)
    pub type_: c_int,

    /// RX/TX clearing count
    pub rx_tx_clear_count: c_int,

    /// Current RAP (Register Address Pointer) value
    pub rap: m_uint8_t,

    /// CSR and BCR registers
    pub csr: [m_uint32_t; 256],
    pub bcr: [m_uint32_t; 256],

    /// RX/TX rings start addresses
    pub rx_start: m_uint32_t,
    pub tx_start: m_uint32_t,

    /// RX/TX number of descriptors (log2)
    pub rx_l2len: m_uint32_t,
    pub tx_l2len: m_uint32_t,

    /// RX/TX number of descriptors
    pub rx_len: m_uint32_t,
    pub tx_len: m_uint32_t,

    /// RX/TX ring positions
    pub rx_pos: m_uint32_t,
    pub tx_pos: m_uint32_t,

    /// MII registers
    pub mii_regs: [[m_uint16_t; 32]; 32],

    /// Physical (MAC) address
    pub mac_addr: n_eth_addr_t,

    /// Device information
    pub dev: *mut vdevice,

    /// PCI device information
    pub pci_dev: *mut pci_device,

    /// Virtual machine
    pub vm: *mut vm_instance_t,

    /// NetIO descriptor
    pub nio: *mut netio_desc_t,

    /// TX ring scanner task id
    pub tx_tid: ptask_id_t,
}

/// Log an am79c971 message
macro_rules! AM79C971_LOG {
    ($d:expr, $($tt:tt)*) => {
        let d: *mut am79c971_data = $d;
        vm_log!((*d).vm, (*d).name, $($tt)*);
    };
}

/// Lock/Unlock primitives
unsafe fn AM79C971_LOCK(d: *mut am79c971_data) {
    libc::pthread_mutex_lock(addr_of_mut!((*d).lock));
}
unsafe fn AM79C971_UNLOCK(d: *mut am79c971_data) {
    libc::pthread_mutex_unlock(addr_of_mut!((*d).lock));
}

#[rustfmt::skip]
static mut mii_reg_values: [m_uint16_t; 32] = [
    0x1000, 0x782D, 0x0013, 0x78E2, 0x01E1, 0xC9E1, 0x000F, 0x2001,
    0x0000, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0x0104, 0x4780, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x00C8, 0x0000, 0xFFFF, 0x0000, 0x0000, 0x0000,
];
#[cfg(if_0)]
#[rustfmt::skip]
static mut mii_reg_values: [m_uint16_t; 32] = [
    0x1000, 0x782D, 0x0013, 0x78e2, 0x01E1, 0xC9E1, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0001, 0x8060,
    0x8023, 0x0820, 0x0000, 0x3800, 0xA3B9, 0x0000, 0x0000, 0x0000,
];

/// Read a MII register
unsafe fn mii_reg_read(d: *mut am79c971_data, phy: u_int, reg: u_int) -> m_uint16_t {
    if (phy >= 32) || (reg >= 32) {
        return 0;
    }

    (*d).mii_regs[phy as usize][reg as usize]
}

/// Write a MII register
unsafe fn mii_reg_write(d: *mut am79c971_data, phy: u_int, reg: u_int, value: m_uint16_t) {
    if (phy < 32) && (reg < 32) {
        (*d).mii_regs[phy as usize][reg as usize] = value;
    }
}

/// Check if a packet must be delivered to the emulated chip
#[inline]
unsafe fn am79c971_handle_mac_addr(d: *mut am79c971_data, pkt: *mut m_uint8_t) -> c_int {
    let hdr: *mut n_eth_hdr_t = pkt.cast::<_>();

    // Accept systematically frames if we are running in promiscuous mode
    if ((*d).csr[15] & AM79C971_CSR15_PROM) != 0 {
        return TRUE;
    }

    // Accept systematically all multicast frames
    if eth_addr_is_mcast(addr_of_mut!((*hdr).daddr)) != 0 {
        return TRUE;
    }

    // Accept frames directly for us, discard others
    if libc::memcmp(addr_of!((*d).mac_addr).cast::<_>(), addr_of!((*hdr).daddr).cast::<_>(), N_ETH_ALEN) != 0 {
        return TRUE;
    }

    FALSE
}

/// Update the Interrupt Flag bit of csr0
unsafe fn am79c971_update_irq_status(d: *mut am79c971_data) {
    // Bits set in CR3 disable the specified interrupts
    let mask: m_uint32_t = AM79C971_CSR3_IM_MASK & !((*d).csr[3] & AM79C971_CSR3_IM_MASK);

    if ((*d).csr[0] & mask) != 0 {
        (*d).csr[0] |= AM79C971_CSR0_INTR;
    } else {
        (*d).csr[0] &= !AM79C971_CSR0_INTR;
    }

    if ((*d).csr[0] & (AM79C971_CSR0_INTR | AM79C971_CSR0_IENA)) == (AM79C971_CSR0_INTR | AM79C971_CSR0_IENA) {
        pci_dev_trigger_irq((*d).vm, (*d).pci_dev);
    } else {
        pci_dev_clear_irq((*d).vm, (*d).pci_dev);
    }
}

/// Update RX/TX ON bits of csr0
unsafe fn am79c971_update_rx_tx_on_bits(d: *mut am79c971_data) {
    // Set RX ON if DRX in csr15 is cleared, and set TX on if DTX
    // in csr15 is cleared. The START bit must be set.
    (*d).csr[0] &= !(AM79C971_CSR0_RXON | AM79C971_CSR0_TXON);

    if ((*d).csr[0] & AM79C971_CSR0_STRT) != 0 {
        if ((*d).csr[15] & AM79C971_CSR15_DRX) == 0 {
            (*d).csr[0] |= AM79C971_CSR0_RXON;
        }

        if ((*d).csr[15] & AM79C971_CSR15_DTX) == 0 {
            (*d).csr[0] |= AM79C971_CSR0_TXON;
        }
    }
}

/// Update RX/TX descriptor lengths
unsafe fn am79c971_update_rx_tx_len(d: *mut am79c971_data) {
    (*d).rx_len = 1 << (*d).rx_l2len;
    (*d).tx_len = 1 << (*d).tx_l2len;

    // Normalize ring sizes
    if (*d).rx_len > 512 {
        (*d).rx_len = 512;
    }
    if (*d).tx_len > 512 {
        (*d).tx_len = 512;
    }
}

/// Fetch the initialization block from memory
unsafe fn am79c971_fetch_init_block(d: *mut am79c971_data) -> c_int {
    let mut ib: [m_uint32_t; AM79C971_INIT_BLOCK_LEN] = [0; AM79C971_INIT_BLOCK_LEN];
    let mut ib_tmp: m_uint32_t;

    // The init block address is contained in csr1 (low) and csr2 (high)
    let ib_addr: m_uint32_t = ((*d).csr[2] << 16) | (*d).csr[1];

    if ib_addr == 0 {
        AM79C971_LOG!(d, cstr!("trying to fetch init block at address 0...\n"));
        return -1;
    }

    AM79C971_LOG!(d, cstr!("fetching init block at address 0x%8.8x\n"), ib_addr);
    physmem_copy_from_vm((*d).vm, ib.as_c_void_mut(), ib_addr as m_uint64_t, size_of::<[m_uint32_t; AM79C971_INIT_BLOCK_LEN]>());

    // Extract RX/TX ring addresses
    (*d).rx_start = vmtoh32(ib[5]);
    (*d).tx_start = vmtoh32(ib[6]);

    // Set csr15 from mode field
    ib_tmp = vmtoh32(ib[0]);
    (*d).csr[15] = ib_tmp & 0xffff;

    // Extract RX/TX ring sizes
    (*d).rx_l2len = (ib_tmp >> 20) & 0x0F;
    (*d).tx_l2len = (ib_tmp >> 28) & 0x0F;
    am79c971_update_rx_tx_len(d);

    AM79C971_LOG!(d, cstr!("rx_ring = 0x%8.8x (%u), tx_ring = 0x%8.8x (%u)\n"), (*d).rx_start, (*d).rx_len, (*d).tx_start, (*d).tx_len);

    // Get the physical MAC address
    ib_tmp = vmtoh32(ib[1]);
    (*d).csr[12] = ib_tmp & 0xFFFF;
    (*d).csr[13] = ib_tmp >> 16;

    (*d).mac_addr.eth_addr_byte[3] = ((ib_tmp >> 24) & 0xFF) as m_uint8_t;
    (*d).mac_addr.eth_addr_byte[2] = ((ib_tmp >> 16) & 0xFF) as m_uint8_t;
    (*d).mac_addr.eth_addr_byte[1] = ((ib_tmp >> 8) & 0xFF) as m_uint8_t;
    (*d).mac_addr.eth_addr_byte[0] = (ib_tmp & 0xFF) as m_uint8_t;

    ib_tmp = vmtoh32(ib[2]);
    (*d).csr[14] = ib_tmp & 0xFFFF;
    (*d).mac_addr.eth_addr_byte[5] = ((ib_tmp >> 8) & 0xFF) as m_uint8_t;
    (*d).mac_addr.eth_addr_byte[4] = (ib_tmp & 0xFF) as m_uint8_t;

    // Mark the initialization as done is csr0.
    (*d).csr[0] |= AM79C971_CSR0_IDON;

    // Update RX/TX ON bits of csr0 since csr15 has been modified
    am79c971_update_rx_tx_on_bits(d);
    AM79C971_LOG!(d, cstr!("CSR0 = 0x%4.4x\n"), (*d).csr[0]);
    0
}

/// RDP (Register Data Port) access
unsafe fn am79c971_rdp_access(cpu: *mut cpu_gen_t, d: *mut am79c971_data, op_type: u_int, data: *mut m_uint64_t) {
    let mut mask: m_uint32_t;

    if DEBUG_CSR_REGS != 0 {
        if op_type == MTS_READ {
            cpu_log!(cpu, (*d).name, cstr!("read access to CSR %d\n"), (*d).rap);
        } else {
            cpu_log!(cpu, (*d).name, cstr!("write access to CSR %d, value=0x%x\n"), (*d).rap, *data);
        }
    }

    match (*d).rap {
        0 => 'block: {
            // CSR0: Controller Status and Control Register
            if op_type == MTS_READ {
                if false {
                    AM79C971_LOG!(d, cstr!("reading CSR0 (val=0x%4.4x)\n"), (*d).csr[0]);
                }
                *data = (*d).csr[0] as m_uint64_t;
            } else {
                // The STOP bit clears other bits.
                // It has precedence over INIT and START bits.
                if (*data & AM79C971_CSR0_STOP as m_uint64_t) != 0 {
                    if false {
                        AM79C971_LOG!(d, cstr!("stopping interface!\n"));
                    }
                    (*d).csr[0] = AM79C971_CSR0_STOP;
                    (*d).tx_pos = 0;
                    (*d).rx_pos = 0;
                    am79c971_update_irq_status(d);
                    break 'block;
                }

                /* These bits are cleared when set to 1 */
                mask = AM79C971_CSR0_BABL | AM79C971_CSR0_CERR;
                mask |= AM79C971_CSR0_MISS | AM79C971_CSR0_MERR;
                mask |= AM79C971_CSR0_IDON;

                (*d).rx_tx_clear_count += 1;
                if (*d).rx_tx_clear_count == 3 {
                    mask |= AM79C971_CSR0_RINT | AM79C971_CSR0_TINT;
                    (*d).rx_tx_clear_count = 0;
                }

                (*d).csr[0] &= !(*data & mask as m_uint64_t) as m_uint32_t;

                // Save the Interrupt Enable bit
                (*d).csr[0] |= (*data & AM79C971_CSR0_IENA as m_uint64_t) as m_uint32_t;

                // If INIT bit is set, fetch the initialization block
                if (*data & AM79C971_CSR0_INIT as m_uint64_t) != 0 {
                    (*d).csr[0] |= AM79C971_CSR0_INIT;
                    (*d).csr[0] &= !AM79C971_CSR0_STOP;
                    am79c971_fetch_init_block(d);
                }

                // If STRT bit is set, clear the stop bit
                if (*data & AM79C971_CSR0_STRT as m_uint64_t) != 0 {
                    if false {
                        AM79C971_LOG!(d, cstr!("enabling interface!\n"));
                    }
                    (*d).csr[0] |= AM79C971_CSR0_STRT;
                    (*d).csr[0] &= !AM79C971_CSR0_STOP;
                    am79c971_update_rx_tx_on_bits(d);
                }

                // Update IRQ status
                am79c971_update_irq_status(d);
            }
        }

        6 => {
            // CSR6: RX/TX Descriptor Table Length
            if op_type == MTS_WRITE {
                (*d).rx_l2len = ((*data >> 8) & 0x0F) as m_uint32_t;
                (*d).tx_l2len = ((*data >> 12) & 0x0F) as m_uint32_t;
                am79c971_update_rx_tx_len(d);
            } else {
                *data = (((*d).tx_l2len << 12) | ((*d).rx_l2len << 8)) as m_uint64_t;
            }
        }

        15 => {
            // CSR15: Mode
            if op_type == MTS_WRITE {
                (*d).csr[15] = *data as m_uint32_t;
                am79c971_update_rx_tx_on_bits(d);
            } else {
                *data = (*d).csr[15] as m_uint64_t;
            }
        }

        88 => {
            // CSR88: Chip ID Register Lower (VER=0, PARTID=0x2623, MANFID=1, ONE=1)
            if op_type == MTS_READ {
                match (*d).type_ {
                    AM79C971_TYPE_100BASE_TX => {
                        *data = 0x02623003;
                    }
                    AM79C971_TYPE_10BASE_T => {
                        *data = 0x02621003; // Am79C970A, "AMD Presidio", "AmdP2"
                    }
                    _ => {
                        *data = 0;
                    }
                }
            }
        }

        _ => {
            if op_type == MTS_READ {
                *data = (*d).csr[(*d).rap as usize] as m_uint64_t;
            } else {
                (*d).csr[(*d).rap as usize] = *data as m_uint32_t;
            }

            if DEBUG_UNKNOWN != 0 {
                if op_type == MTS_READ {
                    cpu_log!(cpu, (*d).name, cstr!("read access to unknown CSR %d\n"), (*d).rap);
                } else {
                    cpu_log!(cpu, (*d).name, cstr!("write access to unknown CSR %d, value=0x%x\n"), (*d).rap, *data);
                }
            }
        }
    }
}

/// BDP (BCR Data Port) access
unsafe fn am79c971_bdp_access(cpu: *mut cpu_gen_t, d: *mut am79c971_data, op_type: u_int, data: *mut m_uint64_t) {
    let mii_phy: u_int;
    let mii_reg: u_int;

    if DEBUG_BCR_REGS != 0 {
        if op_type == MTS_READ {
            cpu_log!(cpu, (*d).name, cstr!("read access to BCR %d\n"), (*d).rap);
        } else {
            cpu_log!(cpu, (*d).name, cstr!("write access to BCR %d, value=0x%x\n"), (*d).rap, *data);
        }
    }

    match (*d).rap {
        9 => {
            if op_type == MTS_READ {
                *data = 1;
            }
        }

        // BCR32: MII Control and Status Register
        #[cfg(if_0)]
        32 => {}

        34 => {
            // BCR34: MII Management Data Register
            mii_phy = ((*d).bcr[33] >> 5) & 0x1F;
            mii_reg = (*d).bcr[33] & 0x1F;

            if op_type == MTS_READ {
                *data = mii_reg_read(d, mii_phy, mii_reg) as m_uint64_t;
            } else if false {
                mii_reg_write(d, mii_phy, mii_reg, *data as m_uint16_t);
            }
        }

        _ => {
            if op_type == MTS_READ {
                *data = (*d).bcr[(*d).rap as usize] as m_uint64_t;
            } else {
                (*d).bcr[(*d).rap as usize] = *data as m_uint32_t;
            }

            if DEBUG_UNKNOWN != 0 {
                if op_type == MTS_READ {
                    cpu_log!(cpu, (*d).name, cstr!("read access to unknown BCR %d\n"), (*d).rap);
                } else {
                    cpu_log!(cpu, (*d).name, cstr!("write access to unknown BCR %d, value=0x%x\n"), (*d).rap, *data);
                }
            }
        }
    }
}

/// dev_am79c971_access()
unsafe extern "C" fn dev_am79c971_access(cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let d: *mut am79c971_data = (*dev).priv_data.cast::<_>();

    if op_type == MTS_READ {
        *data = 0;
    }

    if DEBUG_ACCESS != 0 {
        if op_type == MTS_READ {
            cpu_log!(cpu, (*d).name, cstr!("read  access to offset=0x%x, pc=0x%llx, size=%u\n"), offset, cpu_get_pc(cpu), op_size);
        } else {
            cpu_log!(cpu, (*d).name, cstr!("write access to offset=0x%x, pc=0x%llx, val=0x%llx, size=%u\n"), offset, cpu_get_pc(cpu), *data, op_size);
        }
    }

    AM79C971_LOCK(d);

    match offset {
        // RAP (Register Address Pointer) (DWIO=0)
        #[cfg(if_0)]
        0x12 => {}

        0x14 => {
            // RAP (Register Address Pointer) (DWIO=1)
            if op_type == MTS_WRITE {
                (*d).rap = (*data & 0xFF) as m_uint8_t;
            } else {
                *data = (*d).rap as m_uint64_t;
            }
        }

        0x10 => {
            // RDP (Register Data Port)
            am79c971_rdp_access(cpu, d, op_type, data);
        }

        // BDP (BCR Data Port) (DWIO=0)
        #[cfg(if_0)]
        0x16 => {}

        0x1c => {
            // BDP (BCR Data Port) (DWIO=1)
            am79c971_bdp_access(cpu, d, op_type, data);
        }
        _ => {}
    }

    AM79C971_UNLOCK(d);
    null_mut()
}

/// Read a RX descriptor
unsafe fn rxdesc_read(d: *mut am79c971_data, rxd_addr: m_uint32_t, rxd: *mut rx_desc) -> c_int {
    let mut buf: [m_uint32_t; 4] = [0; 4];

    // Get the software style
    let sw_style: m_uint8_t = (*d).bcr[20] as m_uint8_t;

    // Read the descriptor from VM physical RAM
    physmem_copy_from_vm((*d).vm, buf.as_c_void_mut(), rxd_addr as m_uint64_t, size_of::<rx_desc>());

    match sw_style {
        2 => {
            (*rxd).rmd[0] = vmtoh32(buf[0]); // rb addr
            (*rxd).rmd[1] = vmtoh32(buf[1]); // own flag, ...
            (*rxd).rmd[2] = vmtoh32(buf[2]); // rfrtag, mcnt, ...
            (*rxd).rmd[3] = vmtoh32(buf[3]); // user
        }

        3 => {
            (*rxd).rmd[0] = vmtoh32(buf[2]); // rb addr
            (*rxd).rmd[1] = vmtoh32(buf[1]); // own flag, ...
            (*rxd).rmd[2] = vmtoh32(buf[0]); // rfrtag, mcnt, ...
            (*rxd).rmd[3] = vmtoh32(buf[3]); // user
        }

        _ => {
            AM79C971_LOG!(d, cstr!("invalid software style %u!\n"), sw_style);
            return -1;
        }
    }

    0
}

/// Set the address of the next RX descriptor
#[inline]
unsafe fn rxdesc_set_next(d: *mut am79c971_data) {
    (*d).rx_pos += 1;

    if (*d).rx_pos == (*d).rx_len {
        (*d).rx_pos = 0;
    }
}

/// Compute the address of the current RX descriptor
#[inline]
unsafe fn rxdesc_get_current(d: *mut am79c971_data) -> m_uint32_t {
    (*d).rx_start + ((*d).rx_pos * size_of::<rx_desc>() as m_uint32_t)
}

/// Put a packet in buffer of a descriptor
unsafe extern "C" fn rxdesc_put_pkt(d: *mut am79c971_data, rxd: *mut rx_desc, pkt: *mut *mut u_char, pkt_len: *mut ssize_t) {
    let mut len: ssize_t;

    // Compute the data length to copy
    len = (!(((*rxd).rmd[1] & AM79C971_RMD1_LEN) - 1)) as ssize_t;
    len &= AM79C971_RMD1_LEN as ssize_t;
    let cp_len: ssize_t = min(len, *pkt_len);

    // Copy packet data to the VM physical RAM
    if DEBUG_RECEIVE != 0 {
        AM79C971_LOG!(d, cstr!("am79c971_handle_rxring: storing %u bytes at 0x%8.8x\n"), cp_len, (*rxd).rmd[0]);
    }
    physmem_copy_to_vm((*d).vm, (*pkt).cast::<_>(), (*rxd).rmd[0] as m_uint64_t, cp_len as size_t);

    *pkt = (*pkt).offset(cp_len);
    *pkt_len -= cp_len;
}

/// Put a packet in the RX ring.
unsafe fn am79c971_receive_pkt(d: *mut am79c971_data, pkt: *mut u_char, mut pkt_len: ssize_t) -> c_int {
    let mut rx_current: m_uint32_t;
    let mut rx_next: m_uint32_t;
    let mut rxdn_rmd1: m_uint32_t;
    let mut rxd0: rx_desc = zeroed::<_>();
    let mut rxdn: rx_desc = zeroed::<_>();
    let mut rxdc: *mut rx_desc;
    let mut tot_len: ssize_t = pkt_len;
    let mut pkt_ptr: *mut u_char = pkt;
    let sw_style: m_uint8_t;

    /* Truncate the packet if it is too big */
    pkt_len = min(pkt_len, AM79C971_MAX_PKT_SIZE as ssize_t);

    /* Copy the current rxring descriptor */
    let rx_start: m_uint32_t = rxdesc_get_current(d);
    rx_current = rx_start;
    rxdesc_read(d, rx_start, addr_of_mut!(rxd0));

    // We must have the first descriptor...
    if (rxd0.rmd[1] & AM79C971_RMD1_OWN) == 0 {
        return FALSE;
    }

    rxdc = addr_of_mut!(rxd0);
    for i in 0.. {
        if DEBUG_RECEIVE != 0 {
            AM79C971_LOG!(d, cstr!("am79c971_handle_rxring: i=%d, addr=0x%8.8x: rmd[0]=0x%x, rmd[1]=0x%x, rmd[2]=0x%x, rmd[3]=0x%x\n"), i, rx_current, (*rxdc).rmd[0], (*rxdc).rmd[1], (*rxdc).rmd[2], (*rxdc).rmd[3]);
        }
        // Put data into the descriptor buffer
        rxdesc_put_pkt(d, rxdc, addr_of_mut!(pkt_ptr), addr_of_mut!(tot_len));

        // Go to the next descriptor
        rxdesc_set_next(d);

        // If this is not the first descriptor, clear the OWN bit
        if i != 0 {
            (*rxdc).rmd[1] &= !AM79C971_RMD1_OWN;
        }

        // If we have finished, mark the descriptor as end of packet
        if tot_len == 0 {
            (*rxdc).rmd[1] |= AM79C971_RMD1_ENP;
            physmem_copy_u32_to_vm((*d).vm, (rx_current + 4) as m_uint64_t, (*rxdc).rmd[1]);

            // Get the software style
            sw_style = (*d).bcr[20] as m_uint8_t;

            // Update the message byte count field
            (*rxdc).rmd[2] &= !AM79C971_RMD2_LEN;
            (*rxdc).rmd[2] |= (pkt_len + 4) as m_uint32_t;

            match sw_style {
                2 => {
                    physmem_copy_u32_to_vm((*d).vm, (rx_current + 8) as m_uint64_t, (*rxdc).rmd[2]);
                }
                3 => {
                    physmem_copy_u32_to_vm((*d).vm, rx_current as m_uint64_t, (*rxdc).rmd[2]);
                }
                _ => {
                    AM79C971_LOG!(d, cstr!("invalid software style %u!\n"), sw_style);
                }
            }

            break;
        }

        // Try to acquire the next descriptor
        rx_next = rxdesc_get_current(d);
        rxdn_rmd1 = physmem_copy_u32_from_vm((*d).vm, (rx_next + 4) as m_uint64_t);

        if (rxdn_rmd1 & AM79C971_RMD1_OWN) == 0 {
            (*rxdc).rmd[1] |= AM79C971_RMD1_ERR | AM79C971_RMD1_BUFF;
            (*rxdc).rmd[1] |= AM79C971_RMD1_ENP;
            physmem_copy_u32_to_vm((*d).vm, (rx_current + 4) as m_uint64_t, (*rxdc).rmd[1]);
            break;
        }

        // Update rmd1 to store change of OWN bit
        physmem_copy_u32_to_vm((*d).vm, (rx_current + 4) as m_uint64_t, (*rxdc).rmd[1]);

        // Read the next descriptor from VM physical RAM
        rxdesc_read(d, rx_next, addr_of_mut!(rxdn));
        rxdc = addr_of_mut!(rxdn);
        rx_current = rx_next;
    }

    // Update the first RX descriptor
    rxd0.rmd[1] &= !AM79C971_RMD1_OWN;
    rxd0.rmd[1] |= AM79C971_RMD1_STP;
    physmem_copy_u32_to_vm((*d).vm, (rx_start + 4) as m_uint64_t, rxd0.rmd[1]);

    (*d).csr[0] |= AM79C971_CSR0_RINT;
    am79c971_update_irq_status(d);
    TRUE
}

/// Handle the RX ring
unsafe extern "C" fn am79c971_handle_rxring(_nio: *mut netio_desc_t, pkt: *mut u_char, pkt_len: ssize_t, d: *mut c_void, _: *mut c_void) -> c_int {
    let d: *mut am79c971_data = d.cast::<_>();

    // Don't start receive if the RX ring address has not been set
    // and if RX ON is not set.
    if ((*d).rx_start == 0) || ((*d).csr[0] & AM79C971_CSR0_RXON) == 0 {
        return FALSE;
    }

    if DEBUG_RECEIVE != 0 {
        AM79C971_LOG!(d, cstr!("receiving a packet of %d bytes\n"), pkt_len);
        mem_dump((*(*d).vm).log_fd, pkt, pkt_len as u_int);
    }

    AM79C971_LOCK(d);

    // Receive only multicast/broadcast trafic + unicast traffic
    // for this virtual machine.
    if am79c971_handle_mac_addr(d, pkt) != 0 {
        am79c971_receive_pkt(d, pkt, pkt_len);
    }

    AM79C971_UNLOCK(d);
    TRUE
}

/// Read a TX descriptor
unsafe fn txdesc_read(d: *mut am79c971_data, txd_addr: m_uint32_t, txd: *mut tx_desc) -> c_int {
    let mut buf: [m_uint32_t; 4] = [0; 4];

    // Get the software style
    let sw_style: m_uint8_t = (*d).bcr[20] as m_uint8_t;

    // Read the descriptor from VM physical RAM
    physmem_copy_from_vm((*d).vm, buf.as_c_void_mut(), txd_addr as m_uint64_t, size_of::<tx_desc>());

    match sw_style {
        2 => {
            (*txd).tmd[0] = vmtoh32(buf[0]); // tb addr
            (*txd).tmd[1] = vmtoh32(buf[1]); // own flag, ...
            (*txd).tmd[2] = vmtoh32(buf[2]); // buff, uflo, ...
            (*txd).tmd[3] = vmtoh32(buf[3]); // user
        }

        3 => {
            (*txd).tmd[0] = vmtoh32(buf[2]); // tb addr
            (*txd).tmd[1] = vmtoh32(buf[1]); // own flag, ...
            (*txd).tmd[2] = vmtoh32(buf[0]); // buff, uflo, ...
            (*txd).tmd[3] = vmtoh32(buf[3]); // user
        }

        _ => {
            AM79C971_LOG!(d, cstr!("invalid software style %u!\n"), sw_style);
            return -1;
        }
    }

    0
}

/// Set the address of the next TX descriptor
#[inline]
unsafe fn txdesc_set_next(d: *mut am79c971_data) {
    (*d).tx_pos += 1;

    if (*d).tx_pos == (*d).tx_len {
        (*d).tx_pos = 0;
    }
}

/// Compute the address of the current TX descriptor
#[inline]
unsafe fn txdesc_get_current(d: *mut am79c971_data) -> m_uint32_t {
    (*d).tx_start + ((*d).tx_pos * size_of::<tx_desc>() as m_uint32_t)
}

/// Handle the TX ring (single packet)
unsafe fn am79c971_handle_txring_single(d: *mut am79c971_data) -> c_int {
    let mut pkt: [u_char; AM79C971_MAX_PKT_SIZE] = [0; AM79C971_MAX_PKT_SIZE];
    let mut pkt_ptr: *mut u_char;
    let mut txd0: tx_desc = zeroed::<_>();
    let mut ctxd: tx_desc = zeroed::<_>();
    let mut ntxd: tx_desc = zeroed::<_>();
    let mut ptxd: *mut tx_desc;
    let mut tx_current: m_uint32_t;
    let mut clen: m_uint32_t;
    let mut tot_len: m_uint32_t;

    if ((*d).tx_start == 0) || ((*d).csr[0] & AM79C971_CSR0_TXON) == 0 {
        return FALSE;
    }

    // Check if the NIO can transmit
    if netio_can_transmit((*d).nio) == 0 {
        return FALSE;
    }

    // Copy the current txring descriptor
    let tx_start: m_uint32_t = txdesc_get_current(d);
    tx_current = tx_start;
    ptxd = addr_of_mut!(txd0);
    txdesc_read(d, tx_start, ptxd);

    // If we don't own the first descriptor, we cannot transmit
    if ((*ptxd).tmd[1] & AM79C971_TMD1_OWN) == 0 {
        return FALSE;
    }

    if DEBUG_TRANSMIT != 0 {
        AM79C971_LOG!(d, cstr!("am79c971_handle_txring: 1st desc: tmd[0]=0x%x, tmd[1]=0x%x, tmd[2]=0x%x, tmd[3]=0x%x\n"), (*ptxd).tmd[0], (*ptxd).tmd[1], (*ptxd).tmd[2], (*ptxd).tmd[3]);
    }

    // Empty packet for now
    pkt_ptr = pkt.as_c_mut();
    tot_len = 0;

    loop {
        if DEBUG_TRANSMIT != 0 {
            AM79C971_LOG!(d, cstr!("am79c971_handle_txring: loop: tmd[0]=0x%x, tmd[1]=0x%x, tmd[2]=0x%x, tmd[3]=0x%x\n"), (*ptxd).tmd[0], (*ptxd).tmd[1], (*ptxd).tmd[2], (*ptxd).tmd[3]);
        }
        // Copy packet data
        clen = !(((*ptxd).tmd[1] & AM79C971_TMD1_LEN) - 1);
        clen &= AM79C971_TMD1_LEN;

        physmem_copy_from_vm((*d).vm, pkt_ptr.cast::<_>(), (*ptxd).tmd[0] as m_uint64_t, clen as size_t);

        pkt_ptr = pkt_ptr.add(clen as usize);
        tot_len += clen;

        // Clear the OWN bit if this is not the first descriptor
        if ((*ptxd).tmd[1] & AM79C971_TMD1_STP) == 0 {
            (*ptxd).tmd[1] &= !AM79C971_TMD1_OWN;
            physmem_copy_u32_to_vm((*d).vm, (tx_current + 4) as m_uint64_t, (*ptxd).tmd[1]);
        }

        // Set the next descriptor
        txdesc_set_next(d);

        // Stop now if end of packet has been reached
        if ((*ptxd).tmd[1] & AM79C971_TMD1_ENP) != 0 {
            break;
        }

        // Read the next descriptor and try to acquire it
        tx_current = txdesc_get_current(d);
        txdesc_read(d, tx_current, addr_of_mut!(ntxd));

        if (ntxd.tmd[1] & AM79C971_TMD1_OWN) == 0 {
            AM79C971_LOG!(d, cstr!("am79c971_handle_txring: UNDERFLOW!\n"));
            return FALSE;
        }

        libc::memcpy(addr_of_mut!(ctxd).cast::<_>(), addr_of!(ntxd).cast::<_>(), size_of::<tx_desc>());
        ptxd = addr_of_mut!(ctxd);
    }

    if tot_len != 0 {
        if DEBUG_TRANSMIT != 0 {
            AM79C971_LOG!(d, cstr!("sending packet of %u bytes\n"), tot_len);
            mem_dump((*(*d).vm).log_fd, pkt.as_c_mut(), tot_len);
        }
        // rewrite ISL header if required
        cisco_isl_rewrite(pkt.as_c_mut(), tot_len);

        // send it on wire
        netio_send((*d).nio, pkt.as_c_void_mut(), tot_len as size_t);
    }

    // Clear the OWN flag of the first descriptor
    txd0.tmd[1] &= !AM79C971_TMD1_OWN;
    physmem_copy_u32_to_vm((*d).vm, (tx_start + 4) as m_uint64_t, txd0.tmd[1]);

    // Generate TX interrupt
    (*d).csr[0] |= AM79C971_CSR0_TINT;
    am79c971_update_irq_status(d);
    TRUE
}

/// Handle the TX ring
unsafe extern "C" fn am79c971_handle_txring(d: *mut c_void, _: *mut c_void) -> c_int {
    let d: *mut am79c971_data = d.cast::<_>();
    AM79C971_LOCK(d);

    for _ in 0..AM79C971_TXRING_PASS_COUNT {
        if am79c971_handle_txring_single(d) == 0 {
            break;
        }
    }

    netio_clear_bw_stat((*d).nio);
    AM79C971_UNLOCK(d);
    TRUE
}

/// pci_am79c971_read()
///
/// Read a PCI register.
unsafe extern "C" fn pci_am79c971_read(_cpu: *mut cpu_gen_t, dev: *mut pci_device, reg: c_int) -> m_uint32_t {
    let d: *mut am79c971_data = (*dev).priv_data.cast::<_>();

    if DEBUG_PCI_REGS != 0 {
        AM79C971_LOG!(d, cstr!("read PCI register 0x%x\n"), reg);
    }

    match reg {
        0x00 => (AM79C971_PCI_PRODUCT_ID << 16) | AM79C971_PCI_VENDOR_ID,

        // Status, Command
        #[cfg(if_0)]
        0x04 => 0,

        // Base-Class, Sub-Class, Programming IF, Revision ID (read-only, should be 0x02000021?)
        0x08 => 0x02000002,

        // Reserved, Header Type, Latency Timer, Reserved
        #[cfg(if_0)]
        0x0C => 0,

        // I/O Base Address
        0x10 | PCI_REG_BAR1 => (*(*d).dev).phys_addr as m_uint32_t,

        // Subsystem ID Subsystem Vendor ID
        #[cfg(if_0)]
        0x2C => 0,

        // Expansion ROM Base Address
        #[cfg(if_0)]
        0x30 => 0,

        // MAX_LAT, MIN_GNT, Interrupt Pin, Interrupt Line
        #[cfg(if_0)]
        0x3C => 0,

        _ => 0,
    }
}

/// pci_am79c971_write()
///
/// Write a PCI register.
unsafe extern "C" fn pci_am79c971_write(cpu: *mut cpu_gen_t, dev: *mut pci_device, reg: c_int, value: m_uint32_t) {
    let d: *mut am79c971_data = (*dev).priv_data.cast::<_>();

    if DEBUG_PCI_REGS != 0 {
        AM79C971_LOG!(d, cstr!("write PCI register 0x%x, value 0x%x\n"), reg, value);
    }

    #[allow(clippy::single_match)]
    match reg {
        PCI_REG_BAR1 => {
            vm_map_device((*cpu).vm, (*d).dev, value as m_uint64_t);
            AM79C971_LOG!(d, cstr!("registers are mapped at 0x%x\n"), value);
        }
        _ => {}
    }
}

/// dev_am79c971_init()
///
/// Generic AMD Am79c971 initialization code.
#[no_mangle]
pub unsafe extern "C" fn dev_am79c971_init(vm: *mut vm_instance_t, name: *mut c_char, interface_type: c_int, pci_bus: *mut pci_bus, pci_device: c_int, irq: c_int) -> *mut am79c971_data {
    // Allocate the private data structure for AM79C971
    let d: *mut am79c971_data = libc::malloc(size_of::<am79c971_data>()).cast::<_>();
    if d.is_null() {
        libc::fprintf(c_stderr(), cstr!("%s (AM79C971): out of memory\n"), name);
        return null_mut();
    }

    libc::memset(d.cast::<_>(), 0, size_of::<am79c971_data>());
    libc::memcpy((*d).mii_regs[0].as_c_void_mut(), mii_reg_values.as_c_void(), size_of::<[[m_uint16_t; 32]; 32]>());
    libc::pthread_mutex_init(addr_of_mut!((*d).lock), null_mut());

    // Add as PCI device
    let pci_dev: *mut pci_device = pci_dev_add(pci_bus, name, AM79C971_PCI_VENDOR_ID, AM79C971_PCI_PRODUCT_ID, pci_device, 0, irq, d.cast::<_>(), None, Some(pci_am79c971_read), Some(pci_am79c971_write));

    if pci_dev.is_null() {
        libc::fprintf(c_stderr(), cstr!("%s (AM79C971): unable to create PCI device.\n"), name);
        libc::free(d.cast::<_>());
        return null_mut();
    }

    // Create the device itself
    let dev: *mut vdevice = dev_create(name);
    if dev.is_null() {
        libc::fprintf(c_stderr(), cstr!("%s (AM79C971): unable to create device.\n"), name);
        pci_dev_remove(pci_dev);
        libc::free(d.cast::<_>());
        return null_mut();
    }

    (*d).name = name;
    (*d).vm = vm;
    (*d).type_ = interface_type;
    (*d).pci_dev = pci_dev;
    (*d).dev = dev;

    (*dev).phys_addr = 0;
    (*dev).phys_len = 0x4000;
    (*dev).handler = Some(dev_am79c971_access);
    (*dev).priv_data = d.cast::<_>();
    d
}

/// Remove an AMD Am79c971 device
#[no_mangle]
pub unsafe extern "C" fn dev_am79c971_remove(d: *mut am79c971_data) {
    if !d.is_null() {
        pci_dev_remove((*d).pci_dev);
        vm_unbind_device((*d).vm, (*d).dev);
        cpu_group_rebuild_mts((*(*d).vm).cpu_group);
        libc::free((*d).dev.cast::<_>());
        libc::free(d.cast::<_>());
    }
}

/// Bind a NIO to an AMD Am79c971 device
#[no_mangle]
pub unsafe extern "C" fn dev_am79c971_set_nio(d: *mut am79c971_data, nio: *mut netio_desc_t) -> c_int {
    // check that a NIO is not already bound
    if !(*d).nio.is_null() {
        return -1;
    }

    (*d).nio = nio;
    (*d).tx_tid = ptask_add(Some(am79c971_handle_txring), d.cast::<_>(), null_mut());
    netio_rxl_add(nio, Some(am79c971_handle_rxring), d.cast::<_>(), null_mut());
    0
}

/// Unbind a NIO from an AMD Am79c971 device
#[no_mangle]
pub unsafe extern "C" fn dev_am79c971_unset_nio(d: *mut am79c971_data) {
    if !(*d).nio.is_null() {
        ptask_remove((*d).tx_tid);
        netio_rxl_remove((*d).nio);
        (*d).nio = null_mut();
    }
}
