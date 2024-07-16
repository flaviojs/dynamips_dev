//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! Galileo GT64010/GT64120A/GT96100A system controller.
//!
//! The DMA stuff is not complete, only "normal" transfers are working
//! (source and destination addresses incrementing).
//!
//! Also, these transfers are "instantaneous" from a CPU point-of-view: when
//! a channel is enabled, the transfer is immediately done. So, this is not
//! very realistic.

use crate::_private::*;
use crate::cpu::*;
use crate::dev_vtty::*;
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

/// Debugging flags
const DEBUG_UNKNOWN: u_int = 0;
const DEBUG_DMA: u_int = 0;
const DEBUG_SDMA: u_int = 0;
const DEBUG_MPSC: u_int = 0;
const DEBUG_MII: u_int = 0;
const DEBUG_ETH: u_int = 0;
const DEBUG_ETH_TX: u_int = 0;
const DEBUG_ETH_RX: u_int = 0;
const DEBUG_ETH_HASH: u_int = 0;

/// PCI identification
const PCI_VENDOR_GALILEO: m_uint16_t = 0x11ab; // Galileo Technology
const PCI_PRODUCT_GALILEO_GT64010: m_uint16_t = 0x0146; // GT-64010
const PCI_PRODUCT_GALILEO_GT64011: m_uint16_t = 0x4146; // GT-64011
const PCI_PRODUCT_GALILEO_GT64120: m_uint16_t = 0x4620; // GT-64120
const PCI_PRODUCT_GALILEO_GT96100: m_uint16_t = 0x9653; // GT-96100

// === Global definitions =================================================

/// Interrupt High Cause Register
const GT_IHCR_ETH0_SUM: m_uint32_t = 0x00000001;
const GT_IHCR_ETH1_SUM: m_uint32_t = 0x00000002;
const GT_IHCR_SDMA_SUM: m_uint32_t = 0x00000010;

/// Serial Cause Register
const GT_SCR_ETH0_SUM: m_uint32_t = 0x00000001;
const GT_SCR_ETH1_SUM: m_uint32_t = 0x00000002;
const GT_SCR_SDMA_SUM: m_uint32_t = 0x00000010;
const GT_SCR_SDMA0_SUM: m_uint32_t = 0x00000100;
const GT_SCR_MPSC0_SUM: m_uint32_t = 0x00000200;

// === DMA definitions ====================================================
const GT_DMA_CHANNELS: usize = 4;

const GT_DMA_FLYBY_ENABLE: m_uint32_t = 0x00000001; // FlyBy Enable
const GT_DMA_FLYBY_RDWR: m_uint32_t = 0x00000002; // SDRAM Read/Write (FlyBy)
const GT_DMA_SRC_DIR: m_uint32_t = 0x0000000c; // Source Direction
const GT_DMA_DST_DIR: m_uint32_t = 0x00000030; // Destination Direction
const GT_DMA_DATA_LIMIT: m_uint32_t = 0x000001c0; // Data Transfer Limit
const GT_DMA_CHAIN_MODE: m_uint32_t = 0x00000200; // Chained Mode
const GT_DMA_INT_MODE: m_uint32_t = 0x00000400; // Interrupt Mode
const GT_DMA_TRANS_MODE: m_uint32_t = 0x00000800; // Transfer Mode
const GT_DMA_CHAN_ENABLE: m_uint32_t = 0x00001000; // Channel Enable
const GT_DMA_FETCH_NEXT: m_uint32_t = 0x00002000; // Fetch Next Record
const GT_DMA_ACT_STATUS: m_uint32_t = 0x00004000; // DMA Activity Status
const GT_DMA_SDA: m_uint32_t = 0x00008000; // Source/Destination Alignment
const GT_DMA_MDREQ: m_uint32_t = 0x00010000; // Mask DMA Requests
const GT_DMA_CDE: m_uint32_t = 0x00020000; // Close Descriptor Enable
const GT_DMA_EOTE: m_uint32_t = 0x00040000; // End-of-Transfer (EOT) Enable
const GT_DMA_EOTIE: m_uint32_t = 0x00080000; // EOT Interrupt Enable
const GT_DMA_ABORT: m_uint32_t = 0x00100000; // Abort DMA Transfer
const GT_DMA_SLP: m_uint32_t = 0x00600000; // Override Source Address
const GT_DMA_DLP: m_uint32_t = 0x01800000; // Override Dest Address
const GT_DMA_RLP: m_uint32_t = 0x06000000; // Override Record Address
const GT_DMA_REQ_SRC: m_uint32_t = 0x10000000; // DMA Request Source

/// Galileo DMA channel
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct dma_channel {
    pub byte_count: m_uint32_t,
    pub src_addr: m_uint32_t,
    pub dst_addr: m_uint32_t,
    pub cdptr: m_uint32_t,
    pub nrptr: m_uint32_t,
    pub ctrl: m_uint32_t,
}

// === Serial DMA (SDMA) ==================================================

/// SDMA: 2 groups of 8 channels
const GT_SDMA_CHANNELS: usize = 8;
const GT_SDMA_GROUPS: usize = 2;

/// SDMA channel
/// cbindgen:no-export
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sdma_channel {
    pub id: u_int,
    pub sdc: m_uint32_t,
    pub sdcm: m_uint32_t,
    pub rx_desc: m_uint32_t,
    pub rx_buf_ptr: m_uint32_t,
    pub scrdp: m_uint32_t,
    pub tx_desc: m_uint32_t,
    pub sctdp: m_uint32_t,
    pub sftdp: m_uint32_t,
}

/// SGCR: SDMA Group Register
const GT_REG_SGC: u_int = 0x101af0;

/// SDMA cause register: 8 fields (1 for each channel) of 4 bits
const GT_SDMA_CAUSE_RXBUF0: u_int = 0x01;
const GT_SDMA_CAUSE_RXERR0: u_int = 0x02;
const GT_SDMA_CAUSE_TXBUF0: u_int = 0x04;
const GT_SDMA_CAUSE_TXEND0: u_int = 0x08;

/// SDMA channel register offsets
const GT_SDMA_SDC: u_int = 0x000900; // Configuration Register
const GT_SDMA_SDCM: u_int = 0x000908; // Command Register
const GT_SDMA_RX_DESC: u_int = 0x008900; // RX descriptor
const GT_SDMA_SCRDP: u_int = 0x008910; // Current RX descriptor
const GT_SDMA_TX_DESC: u_int = 0x00c900; // TX descriptor
const GT_SDMA_SCTDP: u_int = 0x00c910; // Current TX desc. pointer
const GT_SDMA_SFTDP: u_int = 0x00c914; // First TX desc. pointer

/// SDCR: SDMA Configuration Register
const GT_SDCR_RFT: u_int = 0x00000001; // Receive FIFO Threshold
const GT_SDCR_SFM: u_int = 0x00000002; // Single Frame Mode
const GT_SDCR_RC: u_int = 0x0000003c; // Retransmit count
const GT_SDCR_BLMR: u_int = 0x00000040; // Big/Little Endian RX mode
const GT_SDCR_BLMT: u_int = 0x00000080; // Big/Litlle Endian TX mode
const GT_SDCR_POVR: u_int = 0x00000100; // PCI override
const GT_SDCR_RIFB: u_int = 0x00000200; // RX IRQ on frame boundary
const GT_SDCR_BSZ: u_int = 0x00003000; // Burst size

/// SDCMR: SDMA Command Register
const GT_SDCMR_ERD: u_int = 0x00000080; // Enable RX DMA
const GT_SDCMR_AR: u_int = 0x00008000; // Abort Receive
const GT_SDCMR_STD: u_int = 0x00010000; // Stop TX
const GT_SDCMR_STDH: u_int = GT_SDCMR_STD; // Stop TX High
const GT_SDCMR_STDL: u_int = 0x00020000; // Stop TX Low
const GT_SDCMR_TXD: u_int = 0x00800000; // TX Demand
const GT_SDCMR_TXDH: u_int = GT_SDCMR_TXD; // Start TX High
const GT_SDCMR_TXDL: u_int = 0x01000000; // Start TX Low
const GT_SDCMR_AT: u_int = 0x80000000; // Abort Transmit

/// SDMA RX/TX descriptor
/// cbindgen:no-export
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sdma_desc {
    pub buf_size: m_uint32_t,
    pub cmd_stat: m_uint32_t,
    pub next_ptr: m_uint32_t,
    pub buf_ptr: m_uint32_t,
}

/// SDMA Descriptor Command/Status word
const GT_SDMA_CMD_O: u_int = 0x80000000; // Owner bit
const GT_SDMA_CMD_AM: u_int = 0x40000000; // Auto-mode
const GT_SDMA_CMD_EI: u_int = 0x00800000; // Enable Interrupt
const GT_SDMA_CMD_F: u_int = 0x00020000; // First buffer
const GT_SDMA_CMD_L: u_int = 0x00010000; // Last buffer

const GT_SDMA_CMD_OFFSET: usize = offset_of!(sdma_desc, cmd_stat); // Offset of the Command/Status word

// === MultiProtocol Serial Controller (MPSC) =============================

/// 8 MPSC channels
const GT_MPSC_CHANNELS: usize = 8;

/// MPSC channel
/// cbindgen:no-export
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mpsc_channel {
    pub mmcrl: m_uint32_t,
    pub mmcrh: m_uint32_t,
    pub mpcr: m_uint32_t,
    pub chr: [m_uint32_t; 10],
    pub vtty: *mut vtty_t,
    pub nio: *mut netio_desc_t,
}

const GT_MPSC_MMCRL: u_int = 0x000A00; // Main Config Register Low
const GT_MPSC_MMCRH: u_int = 0x000A04; // Main Config Register High
const GT_MPSC_MPCR: u_int = 0x000A08; // Protocol Config Register
const GT_MPSC_CHR1: u_int = 0x000A0C;
const GT_MPSC_CHR2: u_int = 0x000A10;
const GT_MPSC_CHR3: u_int = 0x000A14;
const GT_MPSC_CHR4: u_int = 0x000A18;
const GT_MPSC_CHR5: u_int = 0x000A1C;
const GT_MPSC_CHR6: u_int = 0x000A20;
const GT_MPSC_CHR7: u_int = 0x000A24;
const GT_MPSC_CHR8: u_int = 0x000A28;
const GT_MPSC_CHR9: u_int = 0x000A2C;
const GT_MPSC_CHR10: u_int = 0x000A30;

const GT_MMCRL_MODE_MASK: u_int = 0x0000007;

const GT_MPSC_MODE_HDLC: u_int = 0;
const GT_MPSC_MODE_UART: u_int = 4;
const GT_MPSC_MODE_BISYNC: u_int = 5;

// === Ethernet definitions ===============================================
const GT_ETH_PORTS: usize = 2;
const GT_MAX_PKT_SIZE: usize = 2048;

/// SMI register
const GT_SMIR_DATA_MASK: u_int = 0x0000FFFF;
const GT_SMIR_PHYAD_MASK: u_int = 0x001F0000; // PHY Device Address
const GT_SMIR_PHYAD_SHIFT: u_int = 16;
const GT_SMIR_REGAD_MASK: u_int = 0x03e00000; // PHY Device Register Address
const GT_SMIR_REGAD_SHIFT: u_int = 21;
const GT_SMIR_OPCODE_MASK: u_int = 0x04000000; // Opcode (0: write, 1: read)
const GT_SMIR_OPCODE_READ: u_int = 0x04000000;
const GT_SMIR_RVALID_FLAG: u_int = 0x08000000; // Read Valid
const GT_SMIR_BUSY_FLAG: u_int = 0x10000000; // Busy: 1=op in progress

/// PCR: Port Configuration Register
const GT_PCR_PM: u_int = 0x00000001; // Promiscuous mode
const GT_PCR_RBM: u_int = 0x00000002; // Reject broadcast mode
const GT_PCR_PBF: u_int = 0x00000004; // Pass bad frames
const GT_PCR_EN: u_int = 0x00000080; // Port Enabled/Disabled
const GT_PCR_LPBK: u_int = 0x00000300; // Loopback mode
const GT_PCR_FC: u_int = 0x00000400; // Force collision
const GT_PCR_HS: u_int = 0x00001000; // Hash size
const GT_PCR_HM: u_int = 0x00002000; // Hash mode
const GT_PCR_HDM: u_int = 0x00004000; // Hash default mode
const GT_PCR_HD: u_int = 0x00008000; // Duplex Mode
const GT_PCR_ISL: u_int = 0x70000000; // ISL enabled (0x06)
const GT_PCR_ACCS: u_int = 0x80000000; // Accelerate Slot Time

/// PCXR: Port Configuration Extend Register
const GT_PCXR_IGMP: u_int = 0x00000001; // IGMP packet capture
const GT_PCXR_SPAN: u_int = 0x00000002; // BPDU packet capture
const GT_PCXR_PAR: u_int = 0x00000004; // Partition Enable
const GT_PCXR_PRIOTX: u_int = 0x00000038; // Priority weight for TX
const GT_PCXR_PRIORX: u_int = 0x000000C0; // Priority weight for RX
const GT_PCXR_PRIORX_OV: u_int = 0x00000100; // Prio RX override
const GT_PCXR_DPLX_EN: u_int = 0x00000200; // Autoneg for Duplex
const GT_PCXR_FCTL_EN: u_int = 0x00000400; // Autoneg for 802.3x
const GT_PCXR_FLP: u_int = 0x00000800; // Force Link Pass
const GT_PCXR_FCTL: u_int = 0x00001000; // Flow Control Mode
const GT_PCXR_MFL: u_int = 0x0000C000; // Maximum Frame Length
const GT_PCXR_MIB_CLR_MODE: u_int = 0x00010000; // MIB counters clear mode
const GT_PCXR_SPEED: u_int = 0x00040000; // Port Speed
const GT_PCXR_SPEED_EN: u_int = 0x00080000; // Autoneg for Speed
const GT_PCXR_RMII_EN: u_int = 0x00100000; // RMII Enable
const GT_PCXR_DSCP_EN: u_int = 0x00200000; // DSCP decoding enable

/// PCMR: Port Command Register
const GT_PCMR_FJ: u_int = 0x00008000; // Force Jam / Flow Control

/// PSR: Port Status Register
const GT_PSR_SPEED: u_int = 0x00000001; // Speed: 10/100 Mb/s (100=>1)
const GT_PSR_DUPLEX: u_int = 0x00000002; // Duplex (1: full)
const GT_PSR_FCTL: u_int = 0x00000004; // Flow Control Mode
const GT_PSR_LINK: u_int = 0x00000008; // Link Up/Down
const GT_PSR_PAUSE: u_int = 0x00000010; // Flow-control disabled state
const GT_PSR_TXLOW: u_int = 0x00000020; // TX Low priority status
const GT_PSR_TXHIGH: u_int = 0x00000040; // TX High priority status
const GT_PSR_TXINP: u_int = 0x00000080; // TX in Progress

/// ICR: Interrupt Cause Register
const GT_ICR_RXBUF: u_int = 0x00000001; // RX Buffer returned to host
const GT_ICR_TXBUFH: u_int = 0x00000004; // TX Buffer High
const GT_ICR_TXBUFL: u_int = 0x00000008; // TX Buffer Low
const GT_ICR_TXENDH: u_int = 0x00000040; // TX End High
const GT_ICR_TXENDL: u_int = 0x00000080; // TX End Low
const GT_ICR_RXERR: u_int = 0x00000100; // RX Error
const GT_ICR_TXERRH: u_int = 0x00000400; // TX Error High
const GT_ICR_TXERRL: u_int = 0x00000800; // TX Error Low
const GT_ICR_RXOVR: u_int = 0x00001000; // RX Overrun
const GT_ICR_TXUDR: u_int = 0x00002000; // TX Underrun
const GT_ICR_RXBUFQ0: u_int = 0x00010000; // RX Buffer in Prio Queue 0
const GT_ICR_RXBUFQ1: u_int = 0x00020000; // RX Buffer in Prio Queue 1
const GT_ICR_RXBUFQ2: u_int = 0x00040000; // RX Buffer in Prio Queue 2
const GT_ICR_RXBUFQ3: u_int = 0x00080000; // RX Buffer in Prio Queue 3
const GT_ICR_RXERRQ0: u_int = 0x00010000; // RX Error in Prio Queue 0
const GT_ICR_RXERRQ1: u_int = 0x00020000; // RX Error in Prio Queue 1
const GT_ICR_RXERRQ2: u_int = 0x00040000; // RX Error in Prio Queue 2
const GT_ICR_RXERRQ3: u_int = 0x00080000; // RX Error in Prio Queue 3
const GT_ICR_MII_STC: u_int = 0x10000000; // MII PHY Status Change
const GT_ICR_SMI_DONE: u_int = 0x20000000; // SMI Command Done
const GT_ICR_INT_SUM: u_int = 0x80000000; // Ethernet Interrupt Summary
const GT_ICR_MASK: u_int = 0x7FFFFFFF;

/// Ethernet hash entry
const GT_HTE_VALID: u_int = 0x00000001; // Valid entry
const GT_HTE_SKIP: u_int = 0x00000002; // Skip entry in a chain
const GT_HTE_RD: u_int = 0x00000004; // 0: Discard, 1: Receive
const GT_HTE_ADDR_MASK: u64 = 0x7fffffffffff8;

const GT_HTE_HOPNUM: u_int = 12; // Hash Table Hop Number

// TODO enum
const GT_HTLOOKUP_MISS: u_int = 0;
const GT_HTLOOKUP_MATCH: u_int = 1;
const GT_HTLOOKUP_HOP_EXCEEDED: u_int = 2;

/// TX Descriptor
const GT_TXDESC_OWN: u_int = 0x80000000; // Ownership
const GT_TXDESC_AM: u_int = 0x40000000; // Auto-mode
const GT_TXDESC_EI: u_int = 0x00800000; // Enable Interrupt
const GT_TXDESC_GC: u_int = 0x00400000; // Generate CRC
const GT_TXDESC_P: u_int = 0x00040000; // Padding
const GT_TXDESC_F: u_int = 0x00020000; // First buffer of packet
const GT_TXDESC_L: u_int = 0x00010000; // Last buffer of packet
const GT_TXDESC_ES: u_int = 0x00008000; // Error Summary
const GT_TXDESC_RC: u_int = 0x00003c00; // Retransmit Count
const GT_TXDESC_COL: u_int = 0x00000200; // Collision
const GT_TXDESC_RL: u_int = 0x00000100; // Retransmit Limit Error
const GT_TXDESC_UR: u_int = 0x00000040; // Underrun Error
const GT_TXDESC_LC: u_int = 0x00000020; // Late Collision Error

const GT_TXDESC_BC_MASK: u_int = 0xFFFF0000; // Number of bytes to transmit
const GT_TXDESC_BC_SHIFT: u_int = 16;

/// RX Descriptor
const GT_RXDESC_OWN: u_int = 0x80000000; // Ownership
const GT_RXDESC_AM: u_int = 0x40000000; // Auto-mode
const GT_RXDESC_EI: u_int = 0x00800000; // Enable Interrupt
const GT_RXDESC_F: u_int = 0x00020000; // First buffer of packet
const GT_RXDESC_L: u_int = 0x00010000; // Last buffer of packet
const GT_RXDESC_ES: u_int = 0x00008000; // Error Summary
const GT_RXDESC_IGMP: u_int = 0x00004000; // IGMP packet detected
const GT_RXDESC_HE: u_int = 0x00002000; // Hash Table Expired
const GT_RXDESC_M: u_int = 0x00001000; // Dst MAC Miss in Hash Table
const GT_RXDESC_FT: u_int = 0x00000800; // Frame Type (802.3/Ethernet)
const GT_RXDESC_SF: u_int = 0x00000100; // Short Frame Error
const GT_RXDESC_MFL: u_int = 0x00000080; // Maximum Frame Length Error
const GT_RXDESC_OR: u_int = 0x00000040; // Overrun Error
const GT_RXDESC_COL: u_int = 0x00000010; // Collision
const GT_RXDESC_CE: u_int = 0x00000001; // CRC Error

const GT_RXDESC_BC_MASK: u_int = 0x0000FFFF; // Byte count
const GT_RXDESC_BS_MASK: u_int = 0xFFFF0000; // Buffer size
const GT_RXDESC_BS_SHIFT: u_int = 16;

/// Galileo Ethernet port
/// cbindgen:no-export
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct eth_port {
    pub nio: *mut netio_desc_t,

    /// First and Current RX descriptors (4 queues)
    pub rx_start: [m_uint32_t; 4],
    pub rx_current: [m_uint32_t; 4],

    /// Current TX descriptors (2 queues)
    pub tx_current: [m_uint32_t; 2],

    /// Port registers
    pub pcr: m_uint32_t,
    pub pcxr: m_uint32_t,
    pub pcmr: m_uint32_t,
    pub psr: m_uint32_t,

    /// SDMA registers
    pub sdcr: m_uint32_t,
    pub sdcmr: m_uint32_t,

    /// Interrupt registers
    pub icr: m_uint32_t,
    pub imr: m_uint32_t,

    /// Hash Table pointer
    pub ht_addr: m_uint32_t,

    /// Ethernet MIB counters
    pub rx_bytes: m_uint32_t,
    pub tx_bytes: m_uint32_t,
    pub rx_frames: m_uint32_t,
    pub tx_frames: m_uint32_t,
}

// ========================================================================

/// Galileo GT64xxx/GT96xxx system controller
/// cbindgen:no-export
#[repr(C)]
#[derive(Copy, Clone)]
pub struct gt_data {
    pub name: *mut c_char,
    pub vm_obj: vm_obj_t,
    pub dev: vdevice,
    pub pci_dev: *mut pci_device,
    pub vm: *mut vm_instance_t,
    pub lock: libc::pthread_mutex_t,

    pub bus: [*mut pci_bus; 2],
    pub dma: [dma_channel; GT_DMA_CHANNELS],

    /// Interrupts (common)
    pub int_cause_reg: m_uint32_t,
    pub int_high_cause_reg: m_uint32_t,
    pub int_mask_reg: m_uint32_t,

    /// Interrupts (GT96100)
    pub int0_main_mask_reg: m_uint32_t,
    pub int0_high_mask_reg: m_uint32_t,
    pub int1_main_mask_reg: m_uint32_t,
    pub int1_high_mask_reg: m_uint32_t,
    pub ser_cause_reg: m_uint32_t,
    pub serint0_mask_reg: m_uint32_t,
    pub serint1_mask_reg: m_uint32_t,
    pub int0_irq: u_int,
    pub int1_irq: u_int,
    pub serint0_irq: u_int,
    pub serint1_irq: u_int,

    /// SDMA - Serial DMA (GT96100)
    pub sgcr: m_uint32_t,
    pub sdma_cause_reg: m_uint32_t,
    pub sdma_mask_reg: m_uint32_t,
    pub sdma: [[sdma_channel; GT_SDMA_CHANNELS]; GT_SDMA_GROUPS],

    /// MPSC - MultiProtocol Serial Controller (GT96100)
    pub mpsc: [mpsc_channel; GT_MPSC_CHANNELS],

    /// Ethernet ports (GT96100)
    pub eth_irq: u_int,
    pub eth_tx_tid: ptask_id_t,
    pub eth_ports: [eth_port; GT_ETH_PORTS],
    pub smi_reg: m_uint32_t,
    pub mii_regs: [[m_uint16_t; 32]; 32],

    /// IRQ status update
    pub gt_update_irq_status: Option<unsafe extern "C" fn(gt_data: *mut gt_data)>,
}

unsafe fn GT_LOCK(d: *mut gt_data) {
    libc::pthread_mutex_lock(addr_of_mut!((*d).lock));
}
unsafe fn GT_UNLOCK(d: *mut gt_data) {
    libc::pthread_mutex_unlock(addr_of_mut!((*d).lock));
}

/// Log a GT message
macro_rules! GT_LOG {
    ($d:ident, $($tt:tt)*) => {
        let d: *mut gt_data = $d;
        vm_log!((*d).vm, (*d).name, $($tt)*);
    }
}

/// Update the interrupt status
unsafe extern "C" fn gt64k_update_irq_status(gt_data: *mut gt_data) {
    if !(*gt_data).pci_dev.is_null() {
        if ((*gt_data).int_cause_reg & (*gt_data).int_mask_reg) != 0 {
            pci_dev_trigger_irq((*gt_data).vm, (*gt_data).pci_dev);
        } else {
            pci_dev_clear_irq((*gt_data).vm, (*gt_data).pci_dev);
        }
    }
}

/// Fetch a DMA record (chained mode)
unsafe fn gt_dma_fetch_rec(vm: *mut vm_instance_t, channel: *mut dma_channel) {
    if DEBUG_DMA != 0 {
        vm_log!(vm, cstr!("GT_DMA"), cstr!("fetching record at address 0x%x\n"), (*channel).nrptr);
    }

    // fetch the record from RAM
    let ptr: m_uint32_t = (*channel).nrptr;
    (*channel).byte_count = swap32(physmem_copy_u32_from_vm(vm, ptr as m_uint64_t));
    (*channel).src_addr = swap32(physmem_copy_u32_from_vm(vm, (ptr + 0x04) as m_uint64_t));
    (*channel).dst_addr = swap32(physmem_copy_u32_from_vm(vm, (ptr + 0x08) as m_uint64_t));
    (*channel).nrptr = swap32(physmem_copy_u32_from_vm(vm, (ptr + 0x0c) as m_uint64_t));

    // clear the "fetch next record bit"
    (*channel).ctrl &= !GT_DMA_FETCH_NEXT;
}

/// Handle control register of a DMA channel
unsafe fn gt_dma_handle_ctrl(gt_data: *mut gt_data, chan_id: c_int) {
    let channel: *mut dma_channel = addr_of_mut!((*gt_data).dma[chan_id as usize]);
    let vm: *mut vm_instance_t = (*gt_data).vm;

    if ((*channel).ctrl & GT_DMA_FETCH_NEXT) != 0 {
        if (*channel).nrptr == 0 {
            vm_log!(vm, cstr!("GT_DMA"), cstr!("trying to load a NULL DMA record...\n"));
            return;
        }

        gt_dma_fetch_rec(vm, channel);
    }

    if ((*channel).ctrl & GT_DMA_CHAN_ENABLE) != 0 {
        loop {
            let mut done = true;

            if DEBUG_DMA != 0 {
                vm_log!(vm, cstr!("GT_DMA"), cstr!("starting transfer from 0x%x to 0x%x (size=%u bytes)\n"), (*channel).src_addr, (*channel).dst_addr, (*channel).byte_count & 0xFFFF);
            }
            physmem_dma_transfer(vm, (*channel).src_addr as m_uint64_t, (*channel).dst_addr as m_uint64_t, ((*channel).byte_count & 0xFFFF) as size_t);

            // chained mode
            if ((*channel).ctrl & GT_DMA_CHAIN_MODE) != 0 && (*channel).nrptr != 0 {
                gt_dma_fetch_rec(vm, channel);
                done = false;
            }
            if done {
                break;
            }
        }

        if DEBUG_DMA != 0 {
            vm_log!(vm, cstr!("GT_DMA"), cstr!("finished transfer.\n"));
        }
        // Trigger DMA interrupt
        (*gt_data).int_cause_reg |= 1 << (4 + chan_id);
        (*gt_data).gt_update_irq_status.unwrap()(gt_data);
    }
}

/// Handle a DMA channel
unsafe fn gt_dma_access(_cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, _op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> c_int {
    let gt_data: *mut gt_data = (*dev).priv_data.cast::<_>();

    let dma = &mut (*gt_data).dma;
    let DMA_REG = |reg: &mut m_uint32_t| {
        if op_type == MTS_WRITE {
            *reg = *data as m_uint32_t;
        } else {
            *data = *reg as m_uint64_t;
        }
    };

    match offset {
        // DMA Source Address
        0x810 => {
            DMA_REG(&mut dma[0].src_addr);
            1
        }
        0x814 => {
            DMA_REG(&mut dma[1].src_addr);
            1
        }
        0x818 => {
            DMA_REG(&mut dma[2].src_addr);
            1
        }
        0x81c => {
            DMA_REG(&mut dma[3].src_addr);
            1
        }

        // DMA Destination Address
        0x820 => {
            DMA_REG(&mut dma[0].dst_addr);
            1
        }
        0x824 => {
            DMA_REG(&mut dma[1].dst_addr);
            1
        }
        0x828 => {
            DMA_REG(&mut dma[1].dst_addr);
            1
        }
        0x82c => {
            DMA_REG(&mut dma[1].dst_addr);
            1
        }

        // DMA Next Record Pointer
        0x830 => {
            (*gt_data).dma[0].cdptr = *data as m_uint32_t;
            DMA_REG(&mut dma[0].nrptr);
            1
        }

        0x834 => {
            (*gt_data).dma[1].cdptr = *data as m_uint32_t;
            DMA_REG(&mut dma[1].nrptr);
            1
        }

        0x838 => {
            (*gt_data).dma[2].cdptr = *data as m_uint32_t;
            DMA_REG(&mut dma[2].nrptr);
            1
        }

        0x83c => {
            (*gt_data).dma[3].cdptr = *data as m_uint32_t;
            DMA_REG(&mut dma[3].nrptr);
            1
        }

        // DMA Channel Control
        0x840 => {
            DMA_REG(&mut dma[0].ctrl);
            if op_type == MTS_WRITE {
                gt_dma_handle_ctrl(gt_data, 0);
            }
            1
        }

        0x844 => {
            DMA_REG(&mut dma[1].ctrl);
            if op_type == MTS_WRITE {
                gt_dma_handle_ctrl(gt_data, 1);
            }
            1
        }

        0x848 => {
            DMA_REG(&mut dma[2].ctrl);
            if op_type == MTS_WRITE {
                gt_dma_handle_ctrl(gt_data, 2);
            }
            1
        }

        0x84c => {
            DMA_REG(&mut dma[3].ctrl);
            if op_type == MTS_WRITE {
                gt_dma_handle_ctrl(gt_data, 3);
            }
            1
        }

        _ => 0,
    }
}

/// dev_gt64010_access()
#[no_mangle]
pub unsafe extern "C" fn dev_gt64010_access(cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let gt_data: *mut gt_data = (*dev).priv_data.cast::<_>();

    if op_type == MTS_READ {
        *data = 0;
    } else {
        *data = swap32(*data as m_uint32_t) as m_uint64_t;
    }

    if gt_dma_access(cpu, dev, offset, op_size, op_type, data) != 0 {
        if op_type == MTS_READ {
            *data = swap32(*data as m_uint32_t) as m_uint64_t;
        }
        return null_mut();
    }

    match offset {
        // ===== DRAM Settings (completely faked, 128 Mb) =====
        0x008 => {
            // ras10_low
            if op_type == MTS_READ {
                *data = 0x000;
            }
        }
        0x010 => {
            // ras10_high
            if op_type == MTS_READ {
                *data = 0x7F;
            }
        }
        0x018 => {
            // ras32_low
            if op_type == MTS_READ {
                *data = 0x080;
            }
        }
        0x020 => {
            // ras32_high
            if op_type == MTS_READ {
                *data = 0x7F;
            }
        }
        0x400 => {
            // ras0_low
            if op_type == MTS_READ {
                *data = 0x00;
            }
        }
        0x404 => {
            // ras0_high
            if op_type == MTS_READ {
                *data = 0xFF;
            }
        }
        0x408 => {
            // ras1_low
            if op_type == MTS_READ {
                *data = 0x7F;
            }
        }
        0x40c => {
            // ras1_high
            if op_type == MTS_READ {
                *data = 0x00;
            }
        }
        0x410 => {
            // ras2_low
            if op_type == MTS_READ {
                *data = 0x00;
            }
        }
        0x414 => {
            // ras2_high
            if op_type == MTS_READ {
                *data = 0xFF;
            }
        }
        0x418 => {
            // ras3_low
            if op_type == MTS_READ {
                *data = 0x7F;
            }
        }
        0x41c => {
            // ras3_high
            if op_type == MTS_READ {
                *data = 0x00;
            }
        }
        0xc08 => {
            // pci0_cs10
            if op_type == MTS_READ {
                *data = 0xFFF;
            }
        }
        0xc0c => {
            // pci0_cs32
            if op_type == MTS_READ {
                *data = 0xFFF;
            }
        }

        0xc00 => {
            // pci_cmd
            if op_type == MTS_READ {
                *data = 0x00008001;
            }
        }

        // ===== Interrupt Cause Register =====
        0xc18 => {
            if op_type == MTS_READ {
                *data = (*gt_data).int_cause_reg as m_uint64_t;
            } else {
                (*gt_data).int_cause_reg &= *data as m_uint32_t;
                gt64k_update_irq_status(gt_data);
            }
        }

        // ===== Interrupt Mask Register =====
        0xc1c => {
            if op_type == MTS_READ {
                *data = (*gt_data).int_mask_reg as m_uint64_t;
            } else {
                (*gt_data).int_mask_reg = *data as m_uint32_t;
                gt64k_update_irq_status(gt_data);
            }
        }

        // ===== PCI Configuration =====
        PCI_BUS_ADDR => {
            // pci configuration address (0xcf8)
            pci_dev_addr_handler(cpu, (*gt_data).bus[0], op_type, FALSE, data);
        }

        PCI_BUS_DATA => {
            // pci data address (0xcfc)
            pci_dev_data_handler(cpu, (*gt_data).bus[0], op_type, FALSE, data);
        }

        _ => {
            if DEBUG_UNKNOWN != 0 {
                if op_type == MTS_READ {
                    cpu_log!(cpu, cstr!("GT64010"), cstr!("read from unknown addr 0x%x, pc=0x%llx\n"), offset, cpu_get_pc(cpu));
                } else {
                    cpu_log!(cpu, cstr!("GT64010"), cstr!("write to unknown addr 0x%x, value=0x%llx, pc=0x%llx\n"), offset, *data, cpu_get_pc(cpu));
                }
            }
        }
    }

    if op_type == MTS_READ {
        *data = swap32(*data as m_uint32_t) as m_uint64_t;
    }
    null_mut()
}

// dev_gt64120_access()
#[no_mangle]
pub unsafe extern "C" fn dev_gt64120_access(cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let gt_data: *mut gt_data = (*dev).priv_data.cast::<_>();

    if op_type == MTS_READ {
        *data = 0;
    } else {
        *data = swap32(*data as m_uint32_t) as m_uint64_t;
    }

    if gt_dma_access(cpu, dev, offset, op_size, op_type, data) != 0 {
        if op_type == MTS_READ {
            *data = swap32(*data as m_uint32_t) as m_uint64_t;
        }
        return null_mut();
    }

    match offset {
        0x008 => {
            // ras10_low
            if op_type == MTS_READ {
                *data = 0x000;
            }
        }
        0x010 => {
            // ras10_high
            if op_type == MTS_READ {
                *data = 0x7F;
            }
        }
        0x018 => {
            // ras32_low
            if op_type == MTS_READ {
                *data = 0x100;
            }
        }
        0x020 => {
            // ras32_high
            if op_type == MTS_READ {
                *data = 0x7F;
            }
        }
        0x400 => {
            // ras0_low
            if op_type == MTS_READ {
                *data = 0x00;
            }
        }
        0x404 => {
            // ras0_high
            if op_type == MTS_READ {
                *data = 0xFF;
            }
        }
        0x408 => {
            // ras1_low
            if op_type == MTS_READ {
                *data = 0x7F;
            }
        }
        0x40c => {
            // ras1_high
            if op_type == MTS_READ {
                *data = 0x00;
            }
        }
        0x410 => {
            // ras2_low
            if op_type == MTS_READ {
                *data = 0x00;
            }
        }
        0x414 => {
            // ras2_high
            if op_type == MTS_READ {
                *data = 0xFF;
            }
        }
        0x418 => {
            // ras3_low
            if op_type == MTS_READ {
                *data = 0x7F;
            }
        }
        0x41c => {
            // ras3_high
            if op_type == MTS_READ {
                *data = 0x00;
            }
        }
        0xc08 => {
            // pci0_cs10
            if op_type == MTS_READ {
                *data = 0xFFF;
            }
        }
        0xc0c => {
            // pci0_cs32
            if op_type == MTS_READ {
                *data = 0xFFF;
            }
        }

        0xc00 => {
            // pci_cmd
            if op_type == MTS_READ {
                *data = 0x00008001;
            }
        }

        // ===== Interrupt Cause Register =====
        0xc18 => {
            if op_type == MTS_READ {
                *data = (*gt_data).int_cause_reg as m_uint64_t;
            } else {
                (*gt_data).int_cause_reg &= *data as m_uint32_t;
                gt64k_update_irq_status(gt_data);
            }
        }

        // ===== Interrupt Mask Register =====
        0xc1c => {
            if op_type == MTS_READ {
                *data = (*gt_data).int_mask_reg as m_uint64_t;
            } else {
                (*gt_data).int_mask_reg = *data as m_uint32_t;
                gt64k_update_irq_status(gt_data);
            }
        }

        // ===== PCI Bus 1 =====
        0xcf0 => {
            pci_dev_addr_handler(cpu, (*gt_data).bus[1], op_type, FALSE, data);
        }

        0xcf4 => {
            pci_dev_data_handler(cpu, (*gt_data).bus[1], op_type, FALSE, data);
        }

        // ===== PCI Bus 0 =====
        PCI_BUS_ADDR => {
            // pci configuration address (0xcf8)
            pci_dev_addr_handler(cpu, (*gt_data).bus[0], op_type, FALSE, data);
        }

        PCI_BUS_DATA => {
            // pci data address (0xcfc)
            pci_dev_data_handler(cpu, (*gt_data).bus[0], op_type, FALSE, data);
        }

        _ => {
            if DEBUG_UNKNOWN != 0 {
                if op_type == MTS_READ {
                    cpu_log!(cpu, cstr!("GT64120"), cstr!("read from unknown addr 0x%x, pc=0x%llx\n"), offset, cpu_get_pc(cpu));
                } else {
                    cpu_log!(cpu, cstr!("GT64120"), cstr!("write to unknown addr 0x%x, value=0x%llx, pc=0x%llx\n"), offset, *data, cpu_get_pc(cpu));
                }
            }
        }
    }

    if op_type == MTS_READ {
        *data = swap32(*data as m_uint32_t) as m_uint64_t;
    }
    null_mut()
}

// ========================================================================
// GT96k Interrupts
// ========================================================================

unsafe extern "C" fn gt96k_update_irq_status(d: *mut gt_data) {
    // Interrupt0* active ?
    if ((*d).int_cause_reg & (*d).int0_main_mask_reg) != 0 || ((*d).int_high_cause_reg & (*d).int0_high_mask_reg) != 0 {
        (*d).int_cause_reg |= 1 << 30;
        vm_set_irq((*d).vm, (*d).int0_irq);
    } else {
        (*d).int_cause_reg &= !(1 << 30);
        vm_clear_irq((*d).vm, (*d).int0_irq);
    }

    // Interrupt1* active ?
    if ((*d).int_cause_reg & (*d).int1_main_mask_reg) != 0 || ((*d).int_high_cause_reg & (*d).int1_high_mask_reg) != 0 {
        (*d).int_cause_reg |= 1 << 31;
        vm_set_irq((*d).vm, (*d).int1_irq);
    } else {
        (*d).int_cause_reg &= !(1 << 31);
        vm_clear_irq((*d).vm, (*d).int1_irq);
    }

    // SerInt0* active ?
    if ((*d).ser_cause_reg & (*d).serint0_mask_reg) != 0 {
        vm_set_irq((*d).vm, (*d).serint0_irq);
    } else {
        vm_clear_irq((*d).vm, (*d).serint0_irq);
    }

    // SerInt1* active ?
    if ((*d).ser_cause_reg & (*d).serint1_mask_reg) != 0 {
        vm_set_irq((*d).vm, (*d).serint1_irq);
    } else {
        vm_clear_irq((*d).vm, (*d).serint1_irq);
    }
}

// ========================================================================
// SDMA (Serial DMA)
// ========================================================================

/// Update SDMA interrupt status
unsafe fn gt_sdma_update_int_status(d: *mut gt_data) {
    // Update general SDMA status
    if ((*d).sdma_cause_reg & (*d).sdma_mask_reg) != 0 {
        (*d).ser_cause_reg |= GT_SCR_SDMA_SUM;
        (*d).int_high_cause_reg |= GT_IHCR_SDMA_SUM;
    } else {
        (*d).ser_cause_reg &= !GT_SCR_SDMA_SUM;
        (*d).int_high_cause_reg &= !GT_IHCR_SDMA_SUM;
    }

    gt96k_update_irq_status(d);
}

/// Update SDMA interrupt status for the specified channel
unsafe fn gt_sdma_update_channel_int_status(d: *mut gt_data, chan_id: u_int) {
    // Get the status of the specified SDMA channel
    let ch_st: m_uint32_t = (*d).sdma_cause_reg & (0x0000000F << (chan_id << 2));

    if ch_st != 0 {
        (*d).ser_cause_reg |= GT_SCR_SDMA0_SUM << (chan_id << 1);
    } else {
        (*d).ser_cause_reg &= !(GT_SCR_SDMA0_SUM << (chan_id << 1));
    }

    gt_sdma_update_int_status(d);
}

/// Set SDMA cause register for a channel
#[inline]
unsafe fn gt_sdma_set_cause(d: *mut gt_data, chan_id: u_int, value: u_int) {
    (*d).sdma_cause_reg |= value << (chan_id << 2);
}

/// Read a SDMA descriptor from memory
unsafe fn gt_sdma_desc_read(d: *mut gt_data, addr: m_uint32_t, desc: *mut sdma_desc) {
    physmem_copy_from_vm((*d).vm, desc.cast::<_>(), addr as m_uint64_t, size_of::<sdma_desc>());

    // byte-swapping
    (*desc).buf_size = vmtoh32((*desc).buf_size);
    (*desc).cmd_stat = vmtoh32((*desc).cmd_stat);
    (*desc).next_ptr = vmtoh32((*desc).next_ptr);
    (*desc).buf_ptr = vmtoh32((*desc).buf_ptr);
}

/// Write a SDMA descriptor to memory
unsafe fn gt_sdma_desc_write(d: *mut gt_data, addr: m_uint32_t, desc: *mut sdma_desc) {
    let mut tmp: sdma_desc = zeroed::<_>();

    // byte-swapping
    tmp.cmd_stat = vmtoh32((*desc).cmd_stat);
    tmp.buf_size = vmtoh32((*desc).buf_size);
    tmp.next_ptr = vmtoh32((*desc).next_ptr);
    tmp.buf_ptr = vmtoh32((*desc).buf_ptr);

    physmem_copy_to_vm((*d).vm, addr_of_mut!(tmp).cast::<_>(), addr as m_uint64_t, size_of::<sdma_desc>())
}

/// Send contents of a SDMA buffer
unsafe fn gt_sdma_send_buffer(d: *mut gt_data, chan_id: u_int, buffer: *mut u_char, len: m_uint32_t) {
    let channel: *mut mpsc_channel = addr_of_mut!((*d).mpsc[chan_id as usize]);
    let mode: u_int = (*channel).mmcrl & GT_MMCRL_MODE_MASK;

    match mode {
        GT_MPSC_MODE_HDLC => {
            if !(*channel).nio.is_null() {
                netio_send((*channel).nio, buffer.cast::<_>(), len as size_t);
            }
        }

        GT_MPSC_MODE_UART => {
            if !(*channel).vtty.is_null() {
                vtty_put_buffer((*channel).vtty, buffer.cast::<_>(), len as size_t);
            }
        }
        _ => {}
    }
}

/// Start TX DMA process
unsafe fn gt_sdma_tx_start(d: *mut gt_data, chan: *mut sdma_channel) -> c_int {
    let mut pkt: [u_char; GT_MAX_PKT_SIZE] = [0; GT_MAX_PKT_SIZE];
    let mut pkt_ptr: *mut u_char;
    let mut txd0: sdma_desc = zeroed::<_>();
    let mut ctxd: sdma_desc = zeroed::<_>();
    let mut ptxd: *mut sdma_desc;
    let mut tx_current: m_uint32_t;
    let mut len: m_uint32_t;
    let mut tot_len: m_uint32_t;
    let mut abort: c_int = FALSE;

    let tx_start: m_uint32_t = (*chan).sctdp;
    tx_current = tx_start;

    if tx_start == 0 {
        return FALSE;
    }

    ptxd = addr_of_mut!(txd0);
    gt_sdma_desc_read(d, tx_start, ptxd);

    // If we don't own the first descriptor, we cannot transmit
    if (txd0.cmd_stat & GT_TXDESC_OWN) == 0 {
        return FALSE;
    }

    // Empty packet for now
    pkt_ptr = pkt.as_c_mut();
    tot_len = 0;

    loop {
        // Copy packet data to the buffer
        len = ((*ptxd).buf_size & GT_TXDESC_BC_MASK) >> GT_TXDESC_BC_SHIFT;

        physmem_copy_from_vm((*d).vm, pkt_ptr.cast::<_>(), (*ptxd).buf_ptr as m_uint64_t, len as size_t);
        pkt_ptr = pkt_ptr.add(len as usize);
        tot_len += len;

        // Clear the OWN bit if this is not the first descriptor
        if ((*ptxd).cmd_stat & GT_TXDESC_F) == 0 {
            (*ptxd).cmd_stat &= !GT_TXDESC_OWN;
            physmem_copy_u32_to_vm((*d).vm, (tx_current + GT_SDMA_CMD_OFFSET as m_uint32_t) as m_uint64_t, (*ptxd).cmd_stat);
        }

        tx_current = (*ptxd).next_ptr;

        // Last descriptor or no more desc available ?
        if ((*ptxd).cmd_stat & GT_TXDESC_L) != 0 {
            break;
        }

        if tx_current == 0 {
            abort = TRUE;
            break;
        }

        // Fetch the next descriptor
        gt_sdma_desc_read(d, tx_current, addr_of_mut!(ctxd));
        ptxd = addr_of_mut!(ctxd);
    }

    if (tot_len != 0) && abort == 0 {
        if DEBUG_SDMA != 0 {
            GT_LOG!(d, cstr!("SDMA%u: sending packet of %u bytes\n"), tot_len);
            mem_dump((*(*d).vm).log_fd, pkt.as_c_mut(), tot_len);
        }
        // send it on wire
        gt_sdma_send_buffer(d, (*chan).id, pkt.as_c_mut(), tot_len);

        // Signal that a TX buffer has been transmitted
        gt_sdma_set_cause(d, (*chan).id, GT_SDMA_CAUSE_TXBUF0);
    }

    // Clear the OWN flag of the first descriptor
    txd0.cmd_stat &= !GT_TXDESC_OWN;
    physmem_copy_u32_to_vm((*d).vm, (tx_start + GT_SDMA_CMD_OFFSET as m_uint32_t) as m_uint64_t, txd0.cmd_stat);

    (*chan).sctdp = tx_current;

    if abort != 0 || tx_current == 0 {
        gt_sdma_set_cause(d, (*chan).id, GT_SDMA_CAUSE_TXEND0);
        (*chan).sdcm &= !GT_SDCMR_TXD;
    }

    // Update interrupt status
    gt_sdma_update_channel_int_status(d, (*chan).id);
    TRUE
}

/// Put a packet in buffer of a descriptor
unsafe fn gt_sdma_rxdesc_put_pkt(d: *mut gt_data, rxd: *mut sdma_desc, pkt: *mut *mut u_char, pkt_len: *mut ssize_t) {
    let len: ssize_t = (((*rxd).buf_size & GT_RXDESC_BS_MASK) >> GT_RXDESC_BS_SHIFT) as ssize_t;

    // compute the data length to copy
    let cp_len: ssize_t = min(len, *pkt_len);

    // copy packet data to the VM physical RAM
    physmem_copy_to_vm((*d).vm, (*pkt).cast::<_>(), (*rxd).buf_ptr as m_uint64_t, cp_len as usize);

    // set the byte count in descriptor
    (*rxd).buf_size |= cp_len as m_uint32_t;

    *pkt = (*pkt).offset(cp_len);
    *pkt_len -= cp_len;
}

/// Put a packet into SDMA buffers
unsafe fn gt_sdma_handle_rxqueue(d: *mut gt_data, channel: *mut sdma_channel, pkt: *mut u_char, mut pkt_len: ssize_t) -> c_int {
    let mut rx_current: m_uint32_t;
    let mut rxd0: sdma_desc = zeroed::<_>();
    let mut rxdn: sdma_desc = zeroed::<_>();
    let mut rxdc: *mut sdma_desc;
    let mut tot_len: ssize_t;
    let mut pkt_ptr: *mut u_char = pkt;

    // Truncate the packet if it is too big
    pkt_len = min(pkt_len, GT_MAX_PKT_SIZE as ssize_t);
    tot_len = pkt_len;

    // Copy the first RX descriptor
    let rx_start: m_uint32_t = (*channel).scrdp;
    rx_current = rx_start;
    if rx_start == 0 {
        gt_sdma_set_cause(d, (*channel).id, GT_SDMA_CAUSE_RXERR0);
        gt_sdma_update_channel_int_status(d, (*channel).id);
        return FALSE;
    }

    // Load the first RX descriptor
    gt_sdma_desc_read(d, rx_start, addr_of_mut!(rxd0));

    if DEBUG_SDMA != 0 {
        GT_LOG!(d, cstr!("SDMA channel %u: reading desc at 0x%8.8x [buf_size=0x%8.8x,cmd_stat=0x%8.8x,next_ptr=0x%8.8x,buf_ptr=0x%8.8x]\n"), (*channel).id, rx_start, rxd0.buf_size, rxd0.cmd_stat, rxd0.next_ptr, rxd0.buf_ptr);
    }

    rxdc = addr_of_mut!(rxd0);
    for i in 0.. {
        if tot_len <= 0 {
            break;
        }
        // We must own the descriptor
        if ((*rxdc).cmd_stat & GT_RXDESC_OWN) == 0 {
            gt_sdma_set_cause(d, (*channel).id, GT_SDMA_CAUSE_RXERR0);
            gt_sdma_update_channel_int_status(d, (*channel).id);
            return FALSE;
        }

        // Put data into the descriptor buffer
        gt_sdma_rxdesc_put_pkt(d, rxdc, addr_of_mut!(pkt_ptr), addr_of_mut!(tot_len));

        // Clear the OWN bit
        (*rxdc).cmd_stat &= !GT_RXDESC_OWN;

        // We have finished if the complete packet has been stored
        if tot_len == 0 {
            (*rxdc).cmd_stat |= GT_RXDESC_L;
            (*rxdc).buf_size += 2; // Add 2 bytes for CRC
        }

        // Update the descriptor in host memory (but not the 1st)
        if i != 0 {
            gt_sdma_desc_write(d, rx_current, rxdc);
        }

        // Get address of the next descriptor
        rx_current = (*rxdc).next_ptr;

        if tot_len == 0 {
            break;
        }

        if rx_current == 0 {
            gt_sdma_set_cause(d, (*channel).id, GT_SDMA_CAUSE_RXERR0);
            gt_sdma_update_channel_int_status(d, (*channel).id);
            return FALSE;
        }

        // Read the next descriptor from VM physical RAM
        gt_sdma_desc_read(d, rx_current, addr_of_mut!(rxdn));
        rxdc = addr_of_mut!(rxdn);
    }

    // Update the RX pointers
    (*channel).scrdp = rx_current;

    // Update the first RX descriptor
    rxd0.cmd_stat |= GT_RXDESC_F;
    gt_sdma_desc_write(d, rx_start, addr_of_mut!(rxd0));

    // Indicate that we have a frame ready
    gt_sdma_set_cause(d, (*channel).id, GT_SDMA_CAUSE_RXBUF0);
    gt_sdma_update_channel_int_status(d, (*channel).id);
    TRUE
}

/// Handle RX packet for a SDMA channel
unsafe extern "C" fn gt_sdma_handle_rx_pkt(_nio: *mut netio_desc_t, pkt: *mut u_char, pkt_len: ssize_t, d: *mut c_void, arg: *mut c_void) -> c_int {
    let d: *mut gt_data = d.cast::<_>();
    let chan_id: u_int = arg as u_long as u_int;

    GT_LOCK(d);

    // Find the SDMA group associated to the MPSC channel for receiving
    let group_id: u_int = ((*d).sgcr >> chan_id) & 0x01;
    let channel: *mut sdma_channel = addr_of_mut!((*d).sdma[group_id as usize][chan_id as usize]);

    gt_sdma_handle_rxqueue(d, channel, pkt, pkt_len);
    GT_UNLOCK(d);
    TRUE
}

/// Handle a SDMA channel
unsafe fn gt_sdma_access(cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, _op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> c_int {
    let gt_data: *mut gt_data = (*dev).priv_data.cast::<_>();

    if (offset & 0x000F00) != 0x000900 {
        return FALSE;
    }

    // Decode group, channel and register
    let group: u_int = (offset >> 20) & 0x0F;
    let chan_id: u_int = (offset >> 16) & 0x0F;
    let reg: u_int = offset & 0xFFFF;

    if (group >= GT_SDMA_GROUPS as u_int) || (chan_id >= GT_SDMA_CHANNELS as u_int) {
        cpu_log!(cpu, cstr!("GT96100"), cstr!("invalid SDMA register 0x%8.8x\n"), offset);
        return TRUE;
    }

    let channel: *mut sdma_channel = addr_of_mut!((*gt_data).sdma[group as usize][chan_id as usize]);

    if false {
        libc::printf(cstr!("SDMA: access to reg 0x%6.6x (group=%u, channel=%u)\n"), offset, group, chan_id);
    }

    match reg {
        // Configuration Register
        GT_SDMA_SDC => {}

        // Command Register
        GT_SDMA_SDCM => {
            if op_type == MTS_WRITE {
                (*channel).sdcm = *data as m_uint32_t;

                if ((*channel).sdcm & GT_SDCMR_TXD) != 0 {
                    if DEBUG_SDMA != 0 {
                        cpu_log!(cpu, cstr!("GT96100-SDMA"), cstr!("starting TX transfer (%u/%u)\n"), group, chan_id);
                    }
                    while gt_sdma_tx_start(gt_data, channel) != 0 {}
                }
            } else {
                *data = 0xFF; //0xFFFFFFFF;
            }
        }

        // Current RX descriptor
        GT_SDMA_SCRDP => {
            if op_type == MTS_READ {
                *data = (*channel).scrdp as m_uint64_t;
            } else {
                (*channel).scrdp = *data as m_uint32_t;
            }
        }

        // Current TX desc. pointer
        GT_SDMA_SCTDP => {
            if op_type == MTS_READ {
                *data = (*channel).sctdp as m_uint64_t;
            } else {
                (*channel).sctdp = *data as m_uint32_t;
            }
        }

        // First TX desc. pointer
        GT_SDMA_SFTDP => {
            if op_type == MTS_READ {
                *data = (*channel).sftdp as m_uint64_t;
            } else {
                (*channel).sftdp = *data as m_uint32_t;
            }
        }

        _ => {
            // unknown/unmanaged register
            return FALSE;
        }
    }

    TRUE
}

// ========================================================================
// MPSC (MultiProtocol Serial Controller)
// ========================================================================

/// Handle a MPSC channel
unsafe fn gt_mpsc_access(_cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, _op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> c_int {
    let gt_data: *mut gt_data = (*dev).priv_data.cast::<_>();
    let reg2: u_int;

    if (offset & 0x000F00) != 0x000A00 {
        return FALSE;
    }

    // Decode channel ID and register
    let chan_id: u_int = offset >> 15;
    let reg: u_int = offset & 0xFFF;

    if chan_id >= GT_MPSC_CHANNELS as u_int {
        return FALSE;
    }

    let channel: *mut mpsc_channel = addr_of_mut!((*gt_data).mpsc[chan_id as usize]);

    match reg {
      // Main Config Register Low
      GT_MPSC_MMCRL => {
         if op_type == MTS_READ {
            *data = (*channel).mmcrl as m_uint64_t;
         } else {
            if DEBUG_MPSC != 0 {
                GT_LOG!(gt_data, cstr!("MPSC channel %u set in mode %llu\n"), chan_id, *data & 0x07);
            }
            (*channel).mmcrl = *data as m_uint32_t;
         }
        }

      // Main Config Register High
      GT_MPSC_MMCRH => {
         if op_type == MTS_READ {
            *data = (*channel).mmcrh as m_uint64_t;
         } else {
            (*channel).mmcrh = *data as m_uint32_t;
         }
      }

      // Protocol Config Register
      GT_MPSC_MPCR => {
         if op_type == MTS_READ {
            *data = (*channel).mpcr as m_uint64_t;
         } else {
            (*channel).mpcr = *data as m_uint32_t;
         }
      }

      // Channel registers
      GT_MPSC_CHR1 | GT_MPSC_CHR2 | GT_MPSC_CHR3 | GT_MPSC_CHR4 | GT_MPSC_CHR5 | GT_MPSC_CHR6 | GT_MPSC_CHR7 | GT_MPSC_CHR8 | GT_MPSC_CHR9 /*| GT_MPSC_CHR10*/ => {
         reg2 = (reg - GT_MPSC_CHR1) >> 2;
         if op_type == MTS_READ {
            *data = (*channel).chr[reg2 as usize] as m_uint64_t;
         } else {
            (*channel).chr[reg2 as usize] = *data as m_uint32_t;
         }
      }

      GT_MPSC_CHR10 => {
         if op_type == MTS_READ {
            *data = ((*channel).chr[9] | 0x20) as m_uint64_t;
         } else {
            (*channel).chr[9] = *data as m_uint32_t;
         }
      }

      _ => {
         // unknown/unmanaged register
         return FALSE;
      }
   }

    TRUE
}

/// Set NIO for a MPSC channel
#[no_mangle]
pub unsafe extern "C" fn dev_gt96100_mpsc_set_nio(d: *mut gt_data, chan_id: u_int, nio: *mut netio_desc_t) -> c_int {
    if chan_id >= GT_MPSC_CHANNELS as u_int {
        return -1;
    }

    let channel: *mut mpsc_channel = addr_of_mut!((*d).mpsc[chan_id as usize]);

    if !(*channel).nio.is_null() {
        return -1;
    }

    (*channel).nio = nio;
    netio_rxl_add(nio, Some(gt_sdma_handle_rx_pkt), d.cast::<_>(), chan_id as u_long as *mut c_void);
    0
}

/// Unset NIO for a MPSC channel
#[no_mangle]
pub unsafe extern "C" fn dev_gt96100_mpsc_unset_nio(d: *mut gt_data, chan_id: u_int) -> c_int {
    if chan_id >= GT_MPSC_CHANNELS as u_int {
        return -1;
    }

    if d.is_null() {
        return 0;
    }

    let channel: *mut mpsc_channel = addr_of_mut!((*d).mpsc[chan_id as usize]);

    if !(*channel).nio.is_null() {
        netio_rxl_remove((*channel).nio);
        (*channel).nio = null_mut();
    }

    0
}

/// Set a VTTY for a MPSC channel
#[no_mangle]
pub unsafe extern "C" fn dev_gt96100_mpsc_set_vtty(d: *mut gt_data, chan_id: u_int, vtty: *mut vtty_t) -> c_int {
    if chan_id >= GT_MPSC_CHANNELS as u_int {
        return -1;
    }

    let channel: *mut mpsc_channel = addr_of_mut!((*d).mpsc[chan_id as usize]);

    if !(*channel).vtty.is_null() {
        return -1;
    }

    (*channel).vtty = vtty;
    0
}

/// Unset a VTTY for a MPSC channel
pub unsafe extern "C" fn dev_gt96100_mpsc_unset_vtty(d: *mut gt_data, chan_id: u_int) -> c_int {
    if chan_id >= GT_MPSC_CHANNELS as u_int {
        return -1;
    }

    let channel: *mut mpsc_channel = addr_of_mut!((*d).mpsc[chan_id as usize]);

    if !(*channel).vtty.is_null() {
        (*channel).vtty = null_mut();
    }

    0
}

// ========================================================================
// Ethernet
// ========================================================================

/// Trigger/clear Ethernet interrupt if one or both port have pending events
unsafe fn gt_eth_set_int_status(d: *mut gt_data) {
    // Compute Ether0 summary
    if ((*d).eth_ports[0].icr & GT_ICR_INT_SUM) != 0 {
        (*d).ser_cause_reg |= GT_SCR_ETH0_SUM;
        (*d).int_high_cause_reg |= GT_IHCR_ETH0_SUM;
    } else {
        (*d).ser_cause_reg &= !GT_SCR_ETH0_SUM;
        (*d).int_high_cause_reg &= !GT_IHCR_ETH0_SUM;
    }

    // Compute Ether1 summary
    if ((*d).eth_ports[1].icr & GT_ICR_INT_SUM) != 0 {
        (*d).ser_cause_reg |= GT_SCR_ETH1_SUM;
        (*d).int_high_cause_reg |= GT_IHCR_ETH1_SUM;
    } else {
        (*d).ser_cause_reg &= !GT_SCR_ETH1_SUM;
        (*d).int_high_cause_reg &= !GT_IHCR_ETH1_SUM;
    }

    gt96k_update_irq_status(d);
}

/// Update the Ethernet port interrupt status
unsafe fn gt_eth_update_int_status(d: *mut gt_data, port: *mut eth_port) {
    if ((*port).icr & (*port).imr & GT_ICR_MASK) != 0 {
        (*port).icr |= GT_ICR_INT_SUM;
    } else {
        (*port).icr &= !GT_ICR_INT_SUM;
    }

    gt_eth_set_int_status(d);
}

/// Read a MII register
unsafe fn gt_mii_read(d: *mut gt_data) -> m_uint32_t {
    let mut res: m_uint32_t = 0;

    let port: m_uint8_t = (((*d).smi_reg & GT_SMIR_PHYAD_MASK) >> GT_SMIR_PHYAD_SHIFT) as m_uint8_t;
    let reg: m_uint8_t = (((*d).smi_reg & GT_SMIR_REGAD_MASK) >> GT_SMIR_REGAD_SHIFT) as m_uint8_t;

    if DEBUG_MII != 1 {
        GT_LOG!(d, cstr!("MII: port 0x%4.4x, reg 0x%2.2x: reading.\n"), port, reg);
    }

    if (port < GT_ETH_PORTS as m_uint8_t) && (reg < 32) {
        res = (*d).mii_regs[port as usize][reg as usize] as m_uint32_t;

        match reg {
            0x00 => {
                res &= !0x8200; // clear reset bit and autoneg restart
            }
            0x01 => {
                #[cfg(if_0)]
                {
                    if !(*d).ports[port].nio.is_null() && bcm5600_mii_port_status(d, port) {
                        (*d).mii_output = 0x782C;
                    } else {
                        (*d).mii_output = 0;
                    }
                    res = 0x782c;
                }
            }
            0x02 => {
                res = 0x40;
            }
            0x03 => {
                res = 0x61d4;
            }
            0x04 => {
                res = 0x1E1;
            }
            0x05 => {
                res = 0x41E1;
            }
            _ => {
                res = 0;
            }
        }
    }

    // Mark the data as ready
    res |= GT_SMIR_RVALID_FLAG;

    res
}

/// Write a MII register
unsafe fn gt_mii_write(d: *mut gt_data) {
    let isolation: m_uint16_t;

    let port: m_uint8_t = (((*d).smi_reg & GT_SMIR_PHYAD_MASK) >> GT_SMIR_PHYAD_SHIFT) as m_uint8_t;
    let reg: m_uint8_t = (((*d).smi_reg & GT_SMIR_REGAD_MASK) >> GT_SMIR_REGAD_SHIFT) as m_uint8_t;

    if (port < GT_ETH_PORTS as m_uint8_t) && (reg < 32) {
        if DEBUG_MII != 0 {
            GT_LOG!(d, cstr!("MII: port 0x%4.4x, reg 0x%2.2x: writing 0x%4.4x\n"), port, reg, (*d).smi_reg & GT_SMIR_DATA_MASK);
        }

        // Check if PHY isolation status is changing
        if reg == 0 {
            isolation = ((*d).smi_reg as m_uint16_t ^ (*d).mii_regs[port as usize][reg as usize]) & 0x400;

            if isolation != 0 {
                if DEBUG_MII != 0 {
                    GT_LOG!(d, cstr!("MII: port 0x%4.4x: generating IRQ\n"), port);
                }
                (*d).eth_ports[port as usize].icr |= GT_ICR_MII_STC;
                gt_eth_update_int_status(d, addr_of_mut!((*d).eth_ports[port as usize]));
            }
        }

        (*d).mii_regs[port as usize][reg as usize] = ((*d).smi_reg & GT_SMIR_DATA_MASK) as m_uint16_t;
    }
}

/// Handle registers of Ethernet ports
#[allow(clippy::manual_range_contains)]
unsafe fn gt_eth_access(cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, _op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> c_int {
    let d: *mut gt_data = (*dev).priv_data.cast::<_>();
    let mut port: *mut eth_port = null_mut();
    let queue: u_int;
    let access: *mut c_char = if op_type == MTS_READ { cstr!("read") } else { cstr!("write") };

    if (offset < 0x80000) || (offset >= 0x90000) {
        return FALSE;
    }

    // Determine the Ethernet port
    #[allow(clippy::manual_range_contains)]
    if (offset >= 0x84800) && (offset < 0x88800) {
        port = addr_of_mut!((*d).eth_ports[0]);
    } else if (offset >= 0x88800) && (offset < 0x8c800) {
        port = addr_of_mut!((*d).eth_ports[1]);
    }

    match offset {
        // SMI register
        0x80810 => {
            if DEBUG_ETH != 0 {
                cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("SMI register\n"));
            }
            if op_type == MTS_WRITE {
                (*d).smi_reg = *data as m_uint32_t;

                if ((*d).smi_reg & GT_SMIR_OPCODE_READ) == 0 {
                    gt_mii_write(d);
                }
            } else {
                *data = 0;

                if ((*d).smi_reg & GT_SMIR_OPCODE_READ) != 0 {
                    *data = gt_mii_read(d) as m_uint64_t;
                }
            }
        }

        // ICR: Interrupt Cause Register
        0x84850 | 0x88850 => {
            if DEBUG_ETH != 0 {
                cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("ICR: Interrupt Cause Register [%s]\n"), access);
            }
            if op_type == MTS_READ {
                *data = (*port).icr as m_uint64_t;
            } else {
                (*port).icr &= *data as m_uint32_t;
                gt_eth_update_int_status(d, port);
            }
        }

        // IMR: Interrupt Mask Register
        0x84858 | 0x88858 => {
            if DEBUG_ETH != 0 {
                cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("IMR: Interrupt Mask Register [%s]\n"), access);
            }
            if op_type == MTS_READ {
                *data = (*port).imr as m_uint64_t;
            } else {
                (*port).imr = *data as m_uint32_t;
                gt_eth_update_int_status(d, port);
            }
        }

        // PCR: Port Configuration Register
        0x84800 | 0x88800 => {
            if DEBUG_ETH != 0 {
                cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("PCR: Port Configuration Register [%s]\n"), access);
            }
            if op_type == MTS_READ {
                *data = (*port).pcr as m_uint64_t;
            } else {
                (*port).pcr = *data as m_uint32_t;
            }
        }

        // PCXR: Port Configuration Extend Register
        0x84808 | 0x88808 => {
            if DEBUG_ETH != 0 {
                cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("PCXR: Port Configuration Extend Register [%s]\n"), access);
            }
            if op_type == MTS_READ {
                *data = (*port).pcxr as m_uint64_t;
                *data |= GT_PCXR_SPEED as m_uint64_t;
            } else {
                (*port).pcxr = *data as m_uint32_t;
            }
        }

        // PCMR: Port Command Register
        0x84810 | 0x88810 => {
            if DEBUG_ETH != 0 {
                cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("PCMR: Port Command Register [%s]\n"), access);
            }
            if op_type == MTS_READ {
                *data = (*port).pcmr as m_uint64_t;
            } else {
                (*port).pcmr = *data as m_uint32_t;
            }
        }

        // Port Status Register
        0x84818 | 0x88818 => {
            if DEBUG_ETH != 0 {
                cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("Port Status Register [%s]\n"), access);
            }
            if op_type == MTS_READ {
                *data = 0x0F;
            }
        }

        // First RX descriptor
        0x84880 | 0x88880 | 0x84884 | 0x88884 | 0x84888 | 0x88888 | 0x8488C | 0x8888C => {
            if DEBUG_ETH != 0 {
                cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("First RX descriptor [%s]\n"), access);
            }
            queue = (offset >> 2) & 0x03;
            if op_type == MTS_READ {
                *data = (*port).rx_start[queue as usize] as m_uint64_t;
            } else {
                (*port).rx_start[queue as usize] = *data as m_uint32_t;
            }
        }

        // Current RX descriptor
        0x848A0 | 0x888A0 | 0x848A4 | 0x888A4 | 0x848A8 | 0x888A8 | 0x848AC | 0x888AC => {
            if DEBUG_ETH != 0 {
                cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("Current RX descriptor [%s]\n"), access);
            }
            queue = (offset >> 2) & 0x03;
            if op_type == MTS_READ {
                *data = (*port).rx_current[queue as usize] as m_uint64_t;
            } else {
                (*port).rx_current[queue as usize] = *data as m_uint32_t;
            }
        }

        // Current TX descriptor
        0x848E0 | 0x888E0 | 0x848E4 | 0x888E4 => {
            if DEBUG_ETH != 0 {
                cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("Current RX descriptor [%s]\n"), access);
            }
            queue = (offset >> 2) & 0x01;
            if op_type == MTS_READ {
                *data = (*port).tx_current[queue as usize] as m_uint64_t;
            } else {
                (*port).tx_current[queue as usize] = *data as m_uint32_t;
            }
        }

        // Hash Table Pointer
        0x84828 | 0x88828 => {
            if DEBUG_ETH != 0 {
                cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("Hash Table Pointer [%s]\n"), access);
            }
            if op_type == MTS_READ {
                *data = (*port).ht_addr as m_uint64_t;
            } else {
                (*port).ht_addr = *data as m_uint32_t;
            }
        }

        // SDCR: SDMA Configuration Register
        0x84840 | 0x88840 => {
            if DEBUG_ETH != 0 {
                cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("SDCR: SDMA Configuration Register [%s]\n"), access);
            }
            if op_type == MTS_READ {
                *data = (*port).sdcr as m_uint64_t;
            } else {
                (*port).sdcr = *data as m_uint32_t;
            }
        }

        // SDCMR: SDMA Command Register
        0x84848 | 0x88848 => {
            if DEBUG_ETH != 0 {
                cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("SDCMR: SDMA Command Register [%s]\n"), access);
            }
            if op_type == MTS_WRITE {
                // Start RX DMA
                if (*data & GT_SDCMR_ERD as m_uint64_t) != 0 {
                    (*port).sdcmr |= GT_SDCMR_ERD;
                    (*port).sdcmr &= !GT_SDCMR_AR;
                }

                // Abort RX DMA
                if (*data & GT_SDCMR_AR as m_uint64_t) != 0 {
                    (*port).sdcmr &= !GT_SDCMR_ERD;
                }

                // Start TX High
                if (*data & GT_SDCMR_TXDH as m_uint64_t) != 0 {
                    (*port).sdcmr |= GT_SDCMR_TXDH;
                    (*port).sdcmr &= !GT_SDCMR_STDH;
                }

                // Start TX Low
                if (*data & GT_SDCMR_TXDL as m_uint64_t) != 0 {
                    (*port).sdcmr |= GT_SDCMR_TXDL;
                    (*port).sdcmr &= !GT_SDCMR_STDL;
                }

                // Stop TX High
                if (*data & GT_SDCMR_STDH as m_uint64_t) != 0 {
                    (*port).sdcmr &= !GT_SDCMR_TXDH;
                    (*port).sdcmr |= GT_SDCMR_STDH;
                }

                // Stop TX Low
                if (*data & GT_SDCMR_STDL as m_uint64_t) != 0 {
                    (*port).sdcmr &= !GT_SDCMR_TXDL;
                    (*port).sdcmr |= GT_SDCMR_STDL;
                }
            } else {
                *data = (*port).sdcmr as m_uint64_t;
            }
        }

        // Ethernet MIB Counters
        0x85800 | 0x89800 => {
            if DEBUG_ETH != 0 {
                cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("Ethernet MIB Counters - Bytes Received [%s]\n"), access);
            }
            if op_type == MTS_READ {
                *data = (*port).rx_bytes as m_uint64_t;
                (*port).rx_bytes = 0;
            }
        }

        0x85804 | 0x89804 => {
            if DEBUG_ETH != 0 {
                cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("Ethernet MIB Counters - Bytes Sent [%s]\n"), access);
            }
            if op_type == MTS_READ {
                *data = (*port).tx_bytes as m_uint64_t;
                (*port).tx_bytes = 0;
            }
        }

        0x85808 | 0x89808 => {
            if DEBUG_ETH != 0 {
                cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("Ethernet MIB Counters - Frames Received [%s]\n"), access);
            }
            if op_type == MTS_READ {
                *data = (*port).rx_frames as m_uint64_t;
                (*port).rx_frames = 0;
            }
        }

        0x8580C | 0x8980C => {
            if DEBUG_ETH != 0 {
                cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("Ethernet MIB Counters - Frames Sent [%s]\n"), access);
            }
            if op_type == MTS_READ {
                *data = (*port).tx_frames as m_uint64_t;
                (*port).tx_frames = 0;
            }
        }

        _ => {
            if DEBUG_UNKNOWN != 0 {
                if op_type == MTS_READ {
                    cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("read access to unknown register 0x%x, pc=0x%llx\n"), offset, cpu_get_pc(cpu));
                } else {
                    cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("write access to unknown register 0x%x, value=0x%llx, pc=0x%llx\n"), offset, *data, cpu_get_pc(cpu));
                }
            }
        }
    }

    if DEBUG_ETH != 0 {
        cpu_log!(cpu, cstr!("GT96100/ETH"), cstr!("DONE register 0x%x, value=0x%llx, pc=0x%llx [%s]\n"), offset, *data, cpu_get_pc(cpu), access);
    }

    TRUE
}

// dev_gt96100_access()
#[no_mangle]
pub unsafe extern "C" fn dev_gt96100_access(cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let gt_data: *mut gt_data = (*dev).priv_data.cast::<_>();

    GT_LOCK(gt_data);

    if op_type == MTS_READ {
        *data = 0;
    } else if op_size == 4 {
        *data = swap32(*data as m_uint32_t) as m_uint64_t;
    }

    if false {
        // DEBUG
        if offset != 0x101a80 {
            if op_type == MTS_READ {
                cpu_log!(cpu, cstr!("GT96100"), cstr!("READ OFFSET 0x%6.6x\n"), offset);
            } else {
                cpu_log!(cpu, cstr!("GT96100"), cstr!("WRITE OFFSET 0x%6.6x, DATA=0x%8.8llx\n"), offset, *data);
            }
        }
    }

    // DMA registers
    if gt_dma_access(cpu, dev, offset, op_size, op_type, data) != 0 {
        GT_UNLOCK(gt_data);
        if (op_type == MTS_READ) && (op_size == 4) {
            *data = swap32(*data as m_uint32_t) as m_uint64_t;
        }
        return null_mut();
    }

    // Serial DMA channel registers
    if gt_sdma_access(cpu, dev, offset, op_size, op_type, data) != 0 {
        GT_UNLOCK(gt_data);
        if (op_type == MTS_READ) && (op_size == 4) {
            *data = swap32(*data as m_uint32_t) as m_uint64_t;
        }
        return null_mut();
    }

    // MPSC registers
    if gt_mpsc_access(cpu, dev, offset, op_size, op_type, data) != 0 {
        GT_UNLOCK(gt_data);
        if (op_type == MTS_READ) && (op_size == 4) {
            *data = swap32(*data as m_uint32_t) as m_uint64_t;
        }
        return null_mut();
    }

    // Ethernet registers
    if gt_eth_access(cpu, dev, offset, op_size, op_type, data) != 0 {
        GT_UNLOCK(gt_data);
        if (op_type == MTS_READ) && (op_size == 4) {
            *data = swap32(*data as m_uint32_t) as m_uint64_t;
        }
        return null_mut();
    }

    match offset {
        // Watchdog configuration register
        0x101a80 => {}

        // Watchdog value register
        0x101a84 => {}

        0x008 => {
            // ras10_low
            if op_type == MTS_READ {
                *data = 0x000;
            }
        }
        0x010 => {
            // ras10_high
            if op_type == MTS_READ {
                *data = 0x7F;
            }
        }
        0x018 => {
            // ras32_low
            if op_type == MTS_READ {
                *data = 0x100;
            }
        }
        0x020 => {
            // ras32_high
            if op_type == MTS_READ {
                *data = 0x7F;
            }
        }
        0x400 => {
            // ras0_low
            if op_type == MTS_READ {
                *data = 0x00;
            }
        }
        0x404 => {
            // ras0_high
            if op_type == MTS_READ {
                *data = 0xFF;
            }
        }
        0x408 => {
            // ras1_low
            if op_type == MTS_READ {
                *data = 0x7F;
            }
        }
        0x40c => {
            // ras1_high
            if op_type == MTS_READ {
                *data = 0x00;
            }
        }
        0x410 => {
            // ras2_low
            if op_type == MTS_READ {
                *data = 0x00;
            }
        }
        0x414 => {
            // ras2_high
            if op_type == MTS_READ {
                *data = 0xFF;
            }
        }
        0x418 => {
            // ras3_low
            if op_type == MTS_READ {
                *data = 0x7F;
            }
        }
        0x41c => {
            // ras3_high
            if op_type == MTS_READ {
                *data = 0x00;
            }
        }
        0xc08 => {
            // pci0_cs10
            if op_type == MTS_READ {
                *data = 0xFFF;
            }
        }
        0xc0c => {
            // pci0_cs32
            if op_type == MTS_READ {
                *data = 0xFFF;
            }
        }

        0xc00 => {
            // pci_cmd
            if op_type == MTS_READ {
                *data = 0x00008001;
            }
        }

        // ===== Interrupt Main Cause Register =====
        0xc18 => {
            if op_type == MTS_READ {
                *data = (*gt_data).int_cause_reg as m_uint64_t;
            } else {
                // Don't touch bit 0, 30 and 31 which are read-only
                (*gt_data).int_cause_reg &= *data as m_uint32_t | 0xC0000001;
                gt96k_update_irq_status(gt_data);
            }
        }

        // ===== Interrupt High Cause Register =====
        0xc98 => {
            if op_type == MTS_READ {
                *data = (*gt_data).int_high_cause_reg as m_uint64_t;
            }
        }

        // ===== Interrupt0 Main Mask Register =====
        0xc1c => {
            if op_type == MTS_READ {
                *data = (*gt_data).int0_main_mask_reg as m_uint64_t;
            } else {
                (*gt_data).int0_main_mask_reg = *data as m_uint32_t;
                gt96k_update_irq_status(gt_data);
            }
        }

        // ===== Interrupt0 High Mask Register =====
        0xc9c => {
            if op_type == MTS_READ {
                *data = (*gt_data).int0_high_mask_reg as m_uint64_t;
            } else {
                (*gt_data).int0_high_mask_reg = *data as m_uint32_t;
                gt96k_update_irq_status(gt_data);
            }
        }

        // ===== Interrupt1 Main Mask Register =====
        0xc24 => {
            if op_type == MTS_READ {
                *data = (*gt_data).int1_main_mask_reg as m_uint64_t;
            } else {
                (*gt_data).int1_main_mask_reg = *data as m_uint32_t;
                gt96k_update_irq_status(gt_data);
            }
        }

        // ===== Interrupt1 High Mask Register =====
        0xca4 => {
            if op_type == MTS_READ {
                *data = (*gt_data).int1_high_mask_reg as m_uint64_t;
            } else {
                (*gt_data).int1_high_mask_reg = *data as m_uint32_t;
                gt96k_update_irq_status(gt_data);
            }
        }

        // ===== Serial Cause Register (read-only) =====
        0x103a00 => {
            if op_type == MTS_READ {
                *data = (*gt_data).ser_cause_reg as m_uint64_t;
            }
        }

        // ===== SerInt0 Mask Register =====
        0x103a80 => {
            if op_type == MTS_READ {
                *data = (*gt_data).serint0_mask_reg as m_uint64_t;
            } else {
                (*gt_data).serint0_mask_reg = *data as m_uint32_t;
                gt96k_update_irq_status(gt_data);
            }
        }

        // ===== SerInt1 Mask Register =====
        0x103a88 => {
            if op_type == MTS_READ {
                *data = (*gt_data).serint1_mask_reg as m_uint64_t;
            } else {
                (*gt_data).serint1_mask_reg = *data as m_uint32_t;
                gt96k_update_irq_status(gt_data);
            }
        }

        // ===== SDMA cause register =====
        0x103a10 => {
            if op_type == MTS_READ {
                *data = (*gt_data).sdma_cause_reg as m_uint64_t;
            } else {
                (*gt_data).sdma_cause_reg &= *data as m_uint32_t;
                gt_sdma_update_int_status(gt_data);
            }
        }

        0x103a13 => {
            if op_type == MTS_WRITE {
                if false {
                    libc::printf(cstr!("Writing 0x103a13, *data = 0x%8.8llx, sdma_cause_reg=0x%8.8x\n"), *data, (*gt_data).sdma_cause_reg);
                }

                (*gt_data).sdma_cause_reg = 0;
                gt_sdma_update_channel_int_status(gt_data, 6);
                gt_sdma_update_channel_int_status(gt_data, 7);
            }
        }

        // ==== SDMA mask register
        0x103a90 => {
            if op_type == MTS_READ {
                *data = (*gt_data).sdma_mask_reg as m_uint64_t;
            } else {
                (*gt_data).sdma_mask_reg = *data as m_uint32_t;
                gt_sdma_update_int_status(gt_data);
            }
        }

        0x103a38 | 0x103a3c | 0x100A48 => {
            if op_type == MTS_READ {
                #[allow(clippy::collapsible_if)]
                if false {
                    *data = 0xFFFFFFFF;
                }
            }
        }

        // CIU Arbiter Configuration Register
        0x101ac0 => {
            if op_type == MTS_READ {
                *data = 0x80000000;
            }
        }

        // SGCR - SDMA Global Configuration Register
        GT_REG_SGC => {
            if op_type == MTS_READ {
                *data = (*gt_data).sgcr as m_uint64_t;
            } else {
                (*gt_data).sgcr = *data as m_uint32_t;
            }
        }

        // ===== PCI Bus 1 =====
        0xcf0 => {
            pci_dev_addr_handler(cpu, (*gt_data).bus[1], op_type, FALSE, data);
        }

        0xcf4 => {
            pci_dev_data_handler(cpu, (*gt_data).bus[1], op_type, FALSE, data);
        }

        // ===== PCI Bus 0 =====
        PCI_BUS_ADDR => {
            // pci configuration address (0xcf8)
            pci_dev_addr_handler(cpu, (*gt_data).bus[0], op_type, FALSE, data);
        }

        PCI_BUS_DATA => {
            // pci data address (0xcfc)
            pci_dev_data_handler(cpu, (*gt_data).bus[0], op_type, FALSE, data);
        }

        _ => {
            if DEBUG_UNKNOWN != 0 {
                if op_type == MTS_READ {
                    cpu_log!(cpu, cstr!("GT96100"), cstr!("read from unknown addr 0x%x, pc=0x%llx\n"), offset, cpu_get_pc(cpu));
                } else {
                    cpu_log!(cpu, cstr!("GT96100"), cstr!("write to unknown addr 0x%x, value=0x%llx, pc=0x%llx\n"), offset, *data, cpu_get_pc(cpu));
                }
            }
        }
    }

    GT_UNLOCK(gt_data);
    if (op_type == MTS_READ) && (op_size == 4) {
        *data = swap32(*data as m_uint32_t) as m_uint64_t;
    }
    null_mut()
}

/// Handle a TX queue (single packet)
unsafe extern "C" fn gt_eth_handle_port_txqueue(d: *mut gt_data, port: *mut eth_port, queue: c_int) -> c_int {
    let mut pkt: [u_char; GT_MAX_PKT_SIZE] = [0; GT_MAX_PKT_SIZE];
    let mut pkt_ptr: *mut u_char;
    let mut ctxd: sdma_desc = zeroed::<_>();
    let mut tx_current: m_uint32_t;
    let mut len: m_uint32_t;
    let mut tot_len: m_uint32_t;
    let mut abort: c_int = FALSE;

    // Check if this TX queue is active
    if (queue == 0) && ((*port).sdcmr & GT_SDCMR_STDL) != 0 {
        return FALSE;
    }

    if (queue == 1) && ((*port).sdcmr & GT_SDCMR_STDH) != 0 {
        return FALSE;
    }

    // Copy the current txring descriptor
    tx_current = (*port).tx_current[queue as usize];

    if tx_current == 0 {
        return FALSE;
    }

    gt_sdma_desc_read(d, tx_current, addr_of_mut!(ctxd));

    // If we don't own the first descriptor, we cannot transmit
    if (ctxd.cmd_stat & GT_TXDESC_OWN) == 0 {
        if queue == 0 {
            (*port).icr |= GT_ICR_TXENDL;
            (*port).sdcmr |= GT_SDCMR_STDL;
            (*port).sdcmr &= !GT_SDCMR_TXDL;
        } else {
            (*port).icr |= GT_ICR_TXENDH;
            (*port).sdcmr |= GT_SDCMR_STDH;
            (*port).sdcmr &= !GT_SDCMR_TXDH;
        }

        gt_eth_update_int_status(d, port);
        return FALSE;
    }

    // Empty packet for now
    pkt_ptr = pkt.as_c_mut();
    tot_len = 0;

    loop {
        if DEBUG_ETH_TX != 0 {
            GT_LOG!(d, cstr!("gt_eth_handle_txqueue: loop: tx_current=0x%08x, cmd_stat=0x%08x, buf_size=0x%08x, next_ptr=0x%08x, buf_ptr=0x%08x\n"), tx_current, ctxd.cmd_stat, ctxd.buf_size, ctxd.next_ptr, ctxd.buf_ptr);
        }

        if (ctxd.cmd_stat & GT_TXDESC_OWN) == 0 {
            if DEBUG_ETH_TX != 0 {
                GT_LOG!(d, cstr!("gt_eth_handle_txqueue: descriptor not owned!\n"));
            }
            abort = TRUE;
            break;
        }

        // Copy packet data to the buffer
        len = (ctxd.buf_size & GT_TXDESC_BC_MASK) >> GT_TXDESC_BC_SHIFT;

        physmem_copy_from_vm((*d).vm, pkt_ptr.cast::<_>(), ctxd.buf_ptr as m_uint64_t, len as usize);
        pkt_ptr = pkt_ptr.add(len as usize);
        tot_len += len;

        // Clear the OWN bit if this is not the last descriptor
        if (ctxd.cmd_stat & GT_TXDESC_L) == 0 {
            ctxd.cmd_stat &= !GT_TXDESC_OWN;
            physmem_copy_u32_to_vm((*d).vm, (tx_current + GT_SDMA_CMD_OFFSET as m_uint32_t) as m_uint64_t, ctxd.cmd_stat);
        }

        // Last descriptor or no more desc available ?
        if (ctxd.cmd_stat & GT_TXDESC_L) != 0 {
            break;
        }

        if (ctxd.next_ptr) == 0 {
            abort = TRUE;
            break;
        }

        // Fetch the next descriptor
        tx_current = ctxd.next_ptr;
        gt_sdma_desc_read(d, tx_current, addr_of_mut!(ctxd));
    }

    if (tot_len != 0) && abort == 0 {
        if DEBUG_ETH_TX != 0 {
            GT_LOG!(d, cstr!("Ethernet: sending packet of %u bytes\n"), tot_len);
            mem_dump((*(*d).vm).log_fd, pkt.as_c_mut(), tot_len);
        }
        // rewrite ISL header if required
        cisco_isl_rewrite(pkt.as_c_mut(), tot_len);

        // send it on wire
        netio_send((*port).nio, pkt.as_c_void_mut(), tot_len as usize);

        // Update MIB counters
        (*port).tx_bytes += tot_len;
        (*port).tx_frames += 1;
    }

    // Clear the OWN flag of the last descriptor
    ctxd.cmd_stat &= !GT_TXDESC_OWN;
    physmem_copy_u32_to_vm((*d).vm, (tx_current + GT_SDMA_CMD_OFFSET as m_uint32_t) as m_uint64_t, ctxd.cmd_stat);

    tx_current = ctxd.next_ptr;
    (*port).tx_current[queue as usize] = tx_current;

    // Notify host about transmitted packet
    if queue == 0 {
        (*port).icr |= GT_ICR_TXBUFL;
    } else {
        (*port).icr |= GT_ICR_TXBUFH;
    }

    if abort == 1 {
        // TX underrun
        (*port).icr |= GT_ICR_TXUDR;

        if queue == 0 {
            (*port).icr |= GT_ICR_TXERRL;
        } else {
            (*port).icr |= GT_ICR_TXERRH;
        }
    } else {
        // End of queue has been reached
        if tx_current == 0 {
            if queue == 0 {
                (*port).icr |= GT_ICR_TXENDL;
            } else {
                (*port).icr |= GT_ICR_TXENDH;
            }
        }
    }

    // Update the interrupt status
    gt_eth_update_int_status(d, port);
    TRUE
}

/// Handle TX ring of the specified port
unsafe fn gt_eth_handle_port_txqueues(d: *mut gt_data, port: u_int) {
    gt_eth_handle_port_txqueue(d, addr_of_mut!((*d).eth_ports[port as usize]), 0); // TX Low
    gt_eth_handle_port_txqueue(d, addr_of_mut!((*d).eth_ports[port as usize]), 1);
    // TX High
}

/// Handle all TX rings of all Ethernet ports
unsafe extern "C" fn gt_eth_handle_txqueues(d: *mut c_void, _: *mut c_void) -> c_int {
    let d: *mut gt_data = d.cast::<_>();
    GT_LOCK(d);

    for i in 0..GT_ETH_PORTS as u_int {
        gt_eth_handle_port_txqueues(d, i);
    }

    GT_UNLOCK(d);
    TRUE
}

/// Inverse a nibble
#[rustfmt::skip]
static mut inv_nibble: [c_int; 16] = [
    0x0, 0x8, 0x4, 0xC, 0x2, 0xA, 0x6, 0xE, 
    0x1, 0x9, 0x5, 0xD, 0x3, 0xB, 0x7, 0xF,
];

/// Inverse a 9-bit value
#[inline]
unsafe fn gt_hash_inv_9bit(val: u_int) -> u_int {
    let mut res: u_int;

    res = (inv_nibble[(val & 0x0F) as usize] << 5) as u_int;
    res |= (inv_nibble[((val & 0xF0) >> 4) as usize] << 1) as u_int;
    res |= (val & 0x100) >> 8;
    res
}

// Compute hash value for Ethernet address filtering.
// Two modes are available (p.271 of the GT96100 doc).
unsafe fn gt_eth_hash_value(addr: *mut n_eth_addr_t, mode: c_int) -> u_int {
    let mut tmp: m_uint64_t;
    let mut res: u_int;

    // Swap the nibbles
    tmp = 0;
    for i in 0..N_ETH_ALEN {
        tmp <<= 8;
        tmp |= ((inv_nibble[((*addr).eth_addr_byte[i] & 0x0F) as usize]) << 4) as m_uint64_t;
        tmp |= inv_nibble[(((*addr).eth_addr_byte[i] & 0xF0) >> 4) as usize] as m_uint64_t;
    }

    if mode == 0 {
        // Fill bits 0:8
        res = ((tmp & 0x00000003) | ((tmp & 0x00007f00) >> 6)) as u_int;
        res ^= ((tmp & 0x00ff8000) >> 15) as u_int;
        res ^= ((tmp & 0x1ff000000) >> 24) as u_int;

        // Fill bits 9:14
        res |= ((tmp & 0xfc) << 7) as u_int;
    } else {
        // Fill bits 0:8
        res = gt_hash_inv_9bit(((tmp & 0x00007fc0) >> 6) as u_int);
        res ^= gt_hash_inv_9bit(((tmp & 0x00ff8000) >> 15) as u_int);
        res ^= gt_hash_inv_9bit(((tmp & 0x1ff000000) >> 24) as u_int);

        // Fill bits 9:14
        res |= ((tmp & 0x3f) << 9) as u_int;
    }

    res
}

/// Walk through the Ethernet hash table.
unsafe fn gt_eth_hash_lookup(d: *mut gt_data, port: *mut eth_port, addr: *mut n_eth_addr_t, entry: *mut m_uint64_t) -> c_int {
    let mut eth_val: m_uint64_t;
    let mut hte_addr: m_uint32_t;

    eth_val = ((*addr).eth_addr_byte[0] as m_uint64_t) << 3;
    eth_val |= ((*addr).eth_addr_byte[1] as m_uint64_t) << 11;
    eth_val |= ((*addr).eth_addr_byte[2] as m_uint64_t) << 19;
    eth_val |= ((*addr).eth_addr_byte[3] as m_uint64_t) << 27;
    eth_val |= ((*addr).eth_addr_byte[4] as m_uint64_t) << 35;
    eth_val |= ((*addr).eth_addr_byte[5] as m_uint64_t) << 43;

    // Compute hash value for Ethernet address filtering
    let hash_val: u_int = gt_eth_hash_value(addr, ((*port).pcr & GT_PCR_HM) as c_int);

    if ((*port).pcr & GT_PCR_HS) != 0 {
        // 1/2K address filtering
        hte_addr = (*port).ht_addr + ((hash_val & 0x7ff) << 3);
    } else {
        // 8K address filtering
        hte_addr = (*port).ht_addr + (hash_val << 3);
    }

    if DEBUG_ETH_HASH != 0 {
        GT_LOG!(d, cstr!("Hash Lookup for Ethernet address %2.2x:%2.2x:%2.2x:%2.2x:%2.2x:%2.2x: addr=0x%x\n"), (*addr).eth_addr_byte[0], (*addr).eth_addr_byte[1], (*addr).eth_addr_byte[2], (*addr).eth_addr_byte[3], (*addr).eth_addr_byte[4], (*addr).eth_addr_byte[5], hte_addr);
    }

    for _ in 0..GT_HTE_HOPNUM {
        *entry = (physmem_copy_u32_from_vm((*d).vm, hte_addr as m_uint64_t) as m_uint64_t) << 32;
        *entry |= physmem_copy_u32_from_vm((*d).vm, (hte_addr + 4) as m_uint64_t) as m_uint64_t;

        // Empty entry ?
        if !(*entry & GT_HTE_VALID as m_uint64_t) == 0 {
            return GT_HTLOOKUP_MISS as c_int;
        }

        // Skip flag or different Ethernet address: jump to next entry
        if (*entry & GT_HTE_SKIP as m_uint64_t) != 0 || ((*entry & GT_HTE_ADDR_MASK) != eth_val) {
            hte_addr += 8;
            continue;
        }

        // We have the good MAC address in this entry
        return GT_HTLOOKUP_MATCH as c_int;
    }

    GT_HTLOOKUP_HOP_EXCEEDED as c_int
}

// Check if a packet (given its destination address) must be handled
// at RX path.
//
// Return values:
//   - 0: Discard packet ;
//   - 1: Receive packet but set "M" bit in RX descriptor;
//   - 2: Receive packet.
//
// The documentation is not clear about the M bit in RX descriptor.
// It is described as "Miss" or "Match" depending on the section.
// However, it turns out that IOS treats the bit as "Miss" bit.
// If the bit is set, the destination MAC address has not been found
// in the hash table, and the frame may be subject to software MAC
// address filter associated by IOS with the interface. If the bit
// is clear, the destination MAC address has been found in the hash
// table and the frame will be accepted by IOS unconditionally.
// The M bit is required to correctly handle unicast frames destined
// to other MAC addresses when the interface works in promiscuous mode.
// IOS puts an interface into promiscuous mode when multicast routing
// or bridging has been configured on it.
#[inline]
unsafe fn gt_eth_handle_rx_daddr(_d: *mut gt_data, port: *mut eth_port, hash_res: u_int, hash_entry: m_uint64_t) -> c_int {
    // Hop Number exceeded
    if hash_res == GT_HTLOOKUP_HOP_EXCEEDED {
        return 1;
    }

    // Match and hash entry marked as "Receive"
    if (hash_res == GT_HTLOOKUP_MATCH) && (hash_entry & GT_HTE_RD as m_uint64_t) != 0 {
        return 2;
    }

    // Miss but hash table default mode to forward ?
    if (hash_res == GT_HTLOOKUP_MISS) && ((*port).pcr & GT_PCR_HDM) != 0 {
        return 2;
    }

    // Promiscous Mode
    if ((*port).pcr & GT_PCR_PM) != 0 {
        return 1;
    }

    // Drop packet for other cases
    0
}

/// Put a packet in the specified RX queue
unsafe fn gt_eth_handle_rxqueue(d: *mut gt_data, port_id: u_int, queue: u_int, pkt: *mut u_char, mut pkt_len: ssize_t) -> c_int {
    let port: *mut eth_port = addr_of_mut!((*d).eth_ports[port_id as usize]);
    let mut rx_current: m_uint32_t;
    let mut rxd0: sdma_desc = zeroed::<_>();
    let mut rxdn: sdma_desc = zeroed::<_>();
    let mut rxdc: *mut sdma_desc;
    let mut tot_len: ssize_t = pkt_len;
    let mut pkt_ptr: *mut u_char = pkt;
    let mut hash_entry: m_uint64_t = 0;

    // Truncate the packet if it is too big
    pkt_len = min(pkt_len, GT_MAX_PKT_SIZE as ssize_t);

    // Copy the first RX descriptor
    let rx_start: m_uint32_t = (*port).rx_start[queue as usize];
    rx_current = rx_start;
    if rx_current == 0 {
        (*port).icr |= (GT_ICR_RXERRQ0 << queue) | GT_ICR_RXERR;
        gt_eth_update_int_status(d, port);
        return FALSE;
    }

    // Analyze the Ethernet header
    let hdr: *mut n_eth_dot1q_hdr_t = pkt.cast::<n_eth_dot1q_hdr_t>();

    // Hash table lookup for address filtering
    let hash_res: c_int = gt_eth_hash_lookup(d, port, addr_of_mut!((*hdr).daddr), addr_of_mut!(hash_entry));

    if DEBUG_ETH_HASH != 0 {
        GT_LOG!(d, cstr!("Hash result: %d, hash_entry=0x%llx\n"), hash_res, hash_entry);
    }

    let addr_action: c_int = gt_eth_handle_rx_daddr(d, port, hash_res as u_int, hash_entry);
    if addr_action == 0 {
        return FALSE;
    }

    // Load the first RX descriptor
    gt_sdma_desc_read(d, rx_start, addr_of_mut!(rxd0));

    if DEBUG_ETH_RX != 0 {
        GT_LOG!(d, cstr!("port %u/queue %u: reading desc at 0x%8.8x [buf_size=0x%8.8x,cmd_stat=0x%8.8x,next_ptr=0x%8.8x,buf_ptr=0x%8.8x]\n"), port_id, queue, rx_start, rxd0.buf_size, rxd0.cmd_stat, rxd0.next_ptr, rxd0.buf_ptr);
    }

    rxdc = addr_of_mut!(rxd0);
    for i in 0.. {
        if tot_len <= 0 {
            break;
        }
        // We must own the descriptor
        if ((*rxdc).cmd_stat & GT_RXDESC_OWN) == 0 {
            (*port).icr |= (GT_ICR_RXERRQ0 << queue) | GT_ICR_RXERR;
            gt_eth_update_int_status(d, port);
            return FALSE;
        }

        // Put data into the descriptor buffer
        gt_sdma_rxdesc_put_pkt(d, rxdc, addr_of_mut!(pkt_ptr), addr_of_mut!(tot_len));

        // Clear the OWN bit
        (*rxdc).cmd_stat &= !GT_RXDESC_OWN;

        // We have finished if the complete packet has been stored
        if tot_len == 0 {
            (*rxdc).cmd_stat |= GT_RXDESC_L;
            (*rxdc).buf_size += 4; // Add 4 bytes for CRC
        }

        // Update the descriptor in host memory (but not the 1st)
        if i != 0 {
            gt_sdma_desc_write(d, rx_current, rxdc);
        }

        // Get address of the next descriptor
        rx_current = (*rxdc).next_ptr;

        if tot_len == 0 {
            break;
        }

        if rx_current == 0 {
            (*port).icr |= (GT_ICR_RXERRQ0 << queue) | GT_ICR_RXERR;
            gt_eth_update_int_status(d, port);
            return FALSE;
        }

        // Read the next descriptor from VM physical RAM
        gt_sdma_desc_read(d, rx_current, addr_of_mut!(rxdn));
        rxdc = addr_of_mut!(rxdn);
    }

    // Update the RX pointers
    (*port).rx_start[queue as usize] = rx_current;
    (*port).rx_current[queue as usize] = rx_current;

    // Update the first RX descriptor
    rxd0.cmd_stat |= GT_RXDESC_F;

    if hash_res == GT_HTLOOKUP_HOP_EXCEEDED as c_int {
        rxd0.cmd_stat |= GT_RXDESC_HE;
    }

    if addr_action == 1 {
        rxd0.cmd_stat |= GT_RXDESC_M;
    }

    if ntohs((*hdr).r#type) <= N_ETH_MTU {
        // 802.3 frame
        rxd0.cmd_stat |= GT_RXDESC_FT;
    }

    gt_sdma_desc_write(d, rx_start, addr_of_mut!(rxd0));

    // Update MIB counters
    (*port).rx_bytes += pkt_len as m_uint32_t;
    (*port).rx_frames += 1;

    // Indicate that we have a frame ready
    (*port).icr |= (GT_ICR_RXBUFQ0 << queue) | GT_ICR_RXBUF;
    gt_eth_update_int_status(d, port);
    TRUE
}

/// Handle RX packet for an Ethernet port
unsafe extern "C" fn gt_eth_handle_rx_pkt(_nio: *mut netio_desc_t, pkt: *mut u_char, pkt_len: ssize_t, d: *mut c_void, arg: *mut c_void) -> c_int {
    let d: *mut gt_data = d.cast::<_>();
    let port_id: u_int = arg as u_long as u_int;

    let port: *mut eth_port = addr_of_mut!((*d).eth_ports[port_id as usize]);

    GT_LOCK(d);

    // Check if RX DMA is active
    if ((*port).sdcmr & GT_SDCMR_ERD) == 0 {
        GT_UNLOCK(d);
        return FALSE;
    }

    let queue: u_int = 0; // At this time, only put packet in queue 0
    gt_eth_handle_rxqueue(d, port_id, queue, pkt, pkt_len);
    GT_UNLOCK(d);
    TRUE
}

/// Shutdown a GT system controller
#[no_mangle]
pub unsafe extern "C" fn dev_gt_shutdown(vm: *mut vm_instance_t, d: *mut c_void) -> *mut c_void {
    let d: *mut gt_data = d.cast::<_>();
    if !d.is_null() {
        // Stop the Ethernet TX ring scanner
        ptask_remove((*d).eth_tx_tid);

        // Remove the device
        dev_remove(vm, addr_of_mut!((*d).dev));

        // Remove the PCI device
        pci_dev_remove((*d).pci_dev);

        // Free the structure itself
        libc::free(d.cast::<_>());
    }
    null_mut()
}

/// Create a new GT64010 controller
#[no_mangle]
pub unsafe extern "C" fn dev_gt64010_init(vm: *mut vm_instance_t, name: *mut c_char, paddr: m_uint64_t, len: m_uint32_t, irq: u_int) -> c_int {
    let d: *mut gt_data = libc::malloc(size_of::<gt_data>()).cast::<_>();
    if d.is_null() {
        libc::fprintf(c_stderr(), cstr!("gt64010: unable to create device data.\n"));
        return -1;
    }

    libc::memset(d.cast::<_>(), 0, size_of::<gt_data>());
    libc::pthread_mutex_init(addr_of_mut!((*d).lock), null_mut());
    (*d).vm = vm;
    (*d).bus[0] = (*vm).pci_bus[0];
    (*d).gt_update_irq_status = Some(gt64k_update_irq_status);

    vm_object_init(addr_of_mut!((*d).vm_obj));
    (*d).vm_obj.name = name;
    (*d).vm_obj.data = d.cast::<_>();
    (*d).vm_obj.shutdown = Some(dev_gt_shutdown);

    dev_init(addr_of_mut!((*d).dev));
    (*d).dev.name = name;
    (*d).dev.priv_data = d.cast::<_>();
    (*d).dev.phys_addr = paddr;
    (*d).dev.phys_len = len;
    (*d).dev.handler = Some(dev_gt64010_access);

    // Add the controller as a PCI device
    if pci_dev_lookup((*d).bus[0], 0, 0, 0).is_null() {
        (*d).pci_dev = pci_dev_add((*d).bus[0], name, PCI_VENDOR_GALILEO as u_int, PCI_PRODUCT_GALILEO_GT64010 as u_int, 0, 0, irq as c_int, d.cast::<_>(), None, None, None);

        if (*d).pci_dev.is_null() {
            libc::fprintf(c_stderr(), cstr!("gt64010: unable to create PCI device.\n"));
            return -1;
        }
    }

    // Map this device to the VM
    vm_bind_device(vm, addr_of_mut!((*d).dev));
    vm_object_add(vm, addr_of_mut!((*d).vm_obj));
    0
}

/// pci_gt64120_read()
///
/// Read a PCI register.
unsafe extern "C" fn pci_gt64120_read(_cpu: *mut cpu_gen_t, _dev: *mut pci_device, reg: c_int) -> m_uint32_t {
    match reg {
        0x08 => 0x03008005,
        _ => 0,
    }
}

/// Create a new GT64120 controller
#[no_mangle]
pub unsafe extern "C" fn dev_gt64120_init(vm: *mut vm_instance_t, name: *mut c_char, paddr: m_uint64_t, len: m_uint32_t, irq: u_int) -> c_int {
    let d: *mut gt_data = libc::malloc(size_of::<gt_data>()).cast::<_>();
    if d.is_null() {
        libc::fprintf(c_stderr(), cstr!("gt64120: unable to create device data.\n"));
        return -1;
    }

    libc::memset(d.cast::<_>(), 0, size_of::<gt_data>());
    libc::pthread_mutex_init(addr_of_mut!((*d).lock), null_mut());
    (*d).vm = vm;
    (*d).bus[0] = (*vm).pci_bus[0];
    (*d).bus[1] = (*vm).pci_bus[1];
    (*d).gt_update_irq_status = Some(gt64k_update_irq_status);

    vm_object_init(addr_of_mut!((*d).vm_obj));
    (*d).vm_obj.name = name;
    (*d).vm_obj.data = d.cast::<_>();
    (*d).vm_obj.shutdown = Some(dev_gt_shutdown);

    dev_init(addr_of_mut!((*d).dev));
    (*d).dev.name = name;
    (*d).dev.priv_data = d.cast::<_>();
    (*d).dev.phys_addr = paddr;
    (*d).dev.phys_len = len;
    (*d).dev.handler = Some(dev_gt64120_access);

    // Add the controller as a PCI device
    if pci_dev_lookup((*d).bus[0], 0, 0, 0).is_null() {
        (*d).pci_dev = pci_dev_add((*d).bus[0], name, PCI_VENDOR_GALILEO as u_int, PCI_PRODUCT_GALILEO_GT64120 as u_int, 0, 0, irq as c_int, d.cast::<_>(), None, Some(pci_gt64120_read), None);
        if (*d).pci_dev.is_null() {
            libc::fprintf(c_stderr(), cstr!("gt64120: unable to create PCI device.\n"));
            return -1;
        }
    }

    // Map this device to the VM
    vm_bind_device(vm, addr_of_mut!((*d).dev));
    vm_object_add(vm, addr_of_mut!((*d).vm_obj));
    0
}

// pci_gt96100_read()
//
// Read a PCI register.
unsafe extern "C" fn pci_gt96100_read(_cpu: *mut cpu_gen_t, _dev: *mut pci_device, reg: c_int) -> m_uint32_t {
    match reg {
        0x08 => 0x03008005,
        _ => 0,
    }
}

/// Create a new GT96100 controller
#[no_mangle]
pub unsafe extern "C" fn dev_gt96100_init(vm: *mut vm_instance_t, name: *mut c_char, paddr: m_uint64_t, len: m_uint32_t, int0_irq: u_int, int1_irq: u_int, serint0_irq: u_int, serint1_irq: u_int) -> c_int {
    let d: *mut gt_data = libc::malloc(size_of::<gt_data>()).cast::<_>();
    if d.is_null() {
        libc::fprintf(c_stderr(), cstr!("gt96100: unable to create device data.\n"));
        return -1;
    }

    libc::memset(d.cast::<_>(), 0, size_of::<gt_data>());
    libc::pthread_mutex_init(addr_of_mut!((*d).lock), null_mut());
    (*d).name = name;
    (*d).vm = vm;
    (*d).gt_update_irq_status = Some(gt96k_update_irq_status);

    for i in 0..GT_SDMA_CHANNELS as u_int {
        (*d).sdma[0][i as usize].id = i;
        (*d).sdma[1][i as usize].id = i;
    }

    // IRQ setup
    (*d).int0_irq = int0_irq;
    (*d).int1_irq = int1_irq;
    (*d).serint0_irq = serint0_irq;
    (*d).serint1_irq = serint1_irq;

    (*d).bus[0] = (*vm).pci_bus[0];
    (*d).bus[1] = (*vm).pci_bus[1];

    vm_object_init(addr_of_mut!((*d).vm_obj));
    (*d).vm_obj.name = name;
    (*d).vm_obj.data = d.cast::<_>();
    (*d).vm_obj.shutdown = Some(dev_gt_shutdown);

    dev_init(addr_of_mut!((*d).dev));
    (*d).dev.name = name;
    (*d).dev.priv_data = d.cast::<_>();
    (*d).dev.phys_addr = paddr;
    (*d).dev.phys_len = len;
    (*d).dev.handler = Some(dev_gt96100_access);

    // Add the controller as a PCI device
    if pci_dev_lookup((*d).bus[0], 0, 0, 0).is_null() {
        (*d).pci_dev = pci_dev_add((*d).bus[0], name, PCI_VENDOR_GALILEO as u_int, PCI_PRODUCT_GALILEO_GT96100 as u_int, 0, 0, -1, d.cast::<_>(), None, Some(pci_gt96100_read), None);
        if (*d).pci_dev.is_null() {
            libc::fprintf(c_stderr(), cstr!("gt96100: unable to create PCI device.\n"));
            return -1;
        }
    }

    // Start the Ethernet TX ring scanner
    (*d).eth_tx_tid = ptask_add(Some(gt_eth_handle_txqueues), d.cast::<_>(), null_mut());

    // Map this device to the VM
    vm_bind_device(vm, addr_of_mut!((*d).dev));
    vm_object_add(vm, addr_of_mut!((*d).vm_obj));
    0
}

/// Bind a NIO to GT96100 Ethernet device
#[no_mangle]
pub unsafe extern "C" fn dev_gt96100_eth_set_nio(d: *mut gt_data, port_id: u_int, nio: *mut netio_desc_t) -> c_int {
    if d.is_null() || (port_id >= GT_ETH_PORTS as u_int) {
        return -1;
    }

    let port: *mut eth_port = addr_of_mut!((*d).eth_ports[port_id as usize]);

    // check that a NIO is not already bound
    if !(*port).nio.is_null() {
        return -1;
    }

    (*port).nio = nio;
    netio_rxl_add(nio, Some(gt_eth_handle_rx_pkt), d.cast::<_>(), port_id as u_long as *mut c_void);
    0
}

/// Unbind a NIO from a GT96100 device
#[no_mangle]
pub unsafe extern "C" fn dev_gt96100_eth_unset_nio(d: *mut gt_data, port_id: u_int) -> c_int {
    if d.is_null() || (port_id >= GT_ETH_PORTS as u_int) {
        return -1;
    }

    let port: *mut eth_port = addr_of_mut!((*d).eth_ports[port_id as usize]);

    if !(*port).nio.is_null() {
        netio_rxl_remove((*port).nio);
        (*port).nio = null_mut();
    }

    0
}

/// Show debugging information
unsafe fn dev_gt96100_show_eth_info(d: *mut gt_data, port_id: u_int) {
    let port: *mut eth_port = addr_of_mut!((*d).eth_ports[port_id as usize]);

    libc::printf(cstr!("GT96100 Ethernet port %u:\n"), port_id);
    libc::printf(cstr!("  PCR  = 0x%8.8x\n"), (*port).pcr);
    libc::printf(cstr!("  PCXR = 0x%8.8x\n"), (*port).pcxr);
    libc::printf(cstr!("  PCMR = 0x%8.8x\n"), (*port).pcmr);
    libc::printf(cstr!("  PSR  = 0x%8.8x\n"), (*port).psr);
    libc::printf(cstr!("  ICR  = 0x%8.8x\n"), (*port).icr);
    libc::printf(cstr!("  IMR  = 0x%8.8x\n"), (*port).imr);

    libc::printf(cstr!("\n"));
}

/// Show debugging information
#[no_mangle]
pub unsafe extern "C" fn dev_gt96100_show_info(d: *mut gt_data) -> c_int {
    GT_LOCK(d);
    dev_gt96100_show_eth_info(d, 0);
    dev_gt96100_show_eth_info(d, 1);
    GT_UNLOCK(d);
    0
}
