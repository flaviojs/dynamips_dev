//! 10/100 Mbps Ethernet PHY.
//!
//! Based on:
//!  Intel® LXT970A
//!  Dual-Speed Fast Ethernet Transceiver
//!  Order Number: 249099-001
//!  January 2001
//! Based on:
//!  IEEE Std 802.3-2008, Section 2

use crate::dynamips_common::*;

/*
Abreviations:
 RW=x - Read/Write with default value x
 RO=x - Read Only with default value x
 [xx] - default value determined by pins xx
 SC   - Self Clearing
 LH   - Latching High (remains High until read, and then returns to Low)
 LL   - Latching Low (remains Low until read, and then returns to High)
 IGN  - IGNored on certain conditions
 RTOR - Related To Other Register

Pin states used for the default values:
 MF0 = VMF2 or VMF3 (Enabled)
 MF1 = VMF1 or VMF4 (Disabled)
 MF2 = VMF2 or VMF3 (Enabled)
 MF3 = VMF2 or VMF3 (Enabled)
 MF4 = VMF2 or VMF3 (Enabled)
 CFG0 = High
 CFG1 = High
 TRSTE = Low
 FDE = High
 MDDIS = Low, RESET = High, PWERDWN = Low (2.4.1.2, MDIO Control)
*/

/*
Standard MII Registers
*/

pub const LX970A_CR: usize = 0x00; // Table 45.  Control Register (Address 0)
pub const LX970A_CR_RESET: m_uint16_t = 0x8000; // Reset (RW=0,SC)
pub const LX970A_CR_LOOP: m_uint16_t = 0x4000; // Loopback (RW=0)
pub const LX970A_CR_SPEEDSELECT: m_uint16_t = 0x2000; // Speed Selection (RW=1[CFG0],IGN)
pub const LX970A_CR_ANENABLE: m_uint16_t = 0x1000; // Auto-Negotiation Enable (RW=1[MF0])
pub const LX970A_CR_POWERDOWN: m_uint16_t = 0x0800; // Power Down (RW=0?)
pub const LX970A_CR_ISOLATE: m_uint16_t = 0x0400; // Isolate (RW=0[TRSTE])
pub const LX970A_CR_ANRESTART: m_uint16_t = 0x0200; // Restart Auto-Negotiation (RW=1[CFG0],SC)
pub const LX970A_CR_DUPLEXMODE: m_uint16_t = 0x0100; // Duplex Mode (RW=1[FDE],IGN)
pub const LX970A_CR_COLLISIONTEST: m_uint16_t = 0x0080; // Collision Test (RW=0,IGN)
pub const LX970A_CR_TTM: m_uint16_t = 0x0070; // Transceiver Test Mode (RO=0)
pub const LX970A_CR_MSENABLE: m_uint16_t = 0x0008; // Master-Slave Enable (RO=0)
pub const LX970A_CR_MSVALUE: m_uint16_t = 0x0004; // Master-Slave Value (RO=0)
pub const LX970A_CR_RESERVED: m_uint16_t = 0x0003; // Reserved (RW=0)
pub const LX970A_CR_RO_MASK: m_uint16_t = 0x007C;
pub const LX970A_CR_RW_MASK: m_uint16_t = 0xFF83;
pub const LX970A_CR_DEFAULT: m_uint16_t = 0x3300;

pub const LX970A_SR: usize = 0x01; // Table 46.  Status Register (Address 1)
pub const LX970A_SR_100T4: m_uint16_t = 0x8000; // 100BASE-T4 (RO=0)
pub const LX970A_SR_100TX_FD: m_uint16_t = 0x4000; // 100BASE-X full-duplex (RO=1)
pub const LX970A_SR_100TX_HD: m_uint16_t = 0x2000; // 100BASE-X hald-duplex (RO=1)
pub const LX970A_SR_10T_FD: m_uint16_t = 0x1000; // 10 Mb/s full-duplex (RO=1)
pub const LX970A_SR_10T_HD: m_uint16_t = 0x0800; // 10 Mb/s half-duplex (RO=1)
pub const LX970A_SR_100T2_FD: m_uint16_t = 0x0400; // 100BASE-T2 full-duplex (RO=0)
pub const LX970A_SR_100T2_HD: m_uint16_t = 0x0200; // 100BASE-T2 half-duplex (RO=0)
pub const LX970A_SR_RESERVED: m_uint16_t = 0x0100; // Reserved (RO=0)
pub const LX970A_SR_MSCFGFAULT: m_uint16_t = 0x0080; // Master-Slave Configuration Fault (RO=0)
pub const LX970A_SR_MFPS: m_uint16_t = 0x0040; // MF Preamble Suppression (RO=0)
pub const LX970A_SR_ANCOMPLETE: m_uint16_t = 0x0020; // Auto-Neg. Complete (RO=0)
pub const LX970A_SR_REMOTEFAULT: m_uint16_t = 0x0010; // Remote Fault (RO=0,LH)
pub const LX970A_SR_ANABILITY: m_uint16_t = 0x0008; // Auto-Neg. Ability (RO=1)
pub const LX970A_SR_LINKSTATUS: m_uint16_t = 0x0004; // Link Status (RO=0,LL)
pub const LX970A_SR_JABBERDETECT: m_uint16_t = 0x0002; // Jabber Detect (10BASE-T Only) (RO=0,LH)
pub const LX970A_SR_EXTCAPABILITY: m_uint16_t = 0x0001; // Extended Capability (RO=1)
pub const LX970A_SR_RO_MASK: m_uint16_t = 0xFFFF;
pub const LX970A_SR_RW_MASK: m_uint16_t = 0x0000;
pub const LX970A_SR_DEFAULT: m_uint16_t = 0x7809;

/*
PHY ID number:
 Intel has OUI=00207Bh and ROUI=DE0400h (OUI with bits reversed)
 (ROUI << 10) & FFFFFFFFh = 78100000h
*/

pub const LX970A_PIR1: usize = 0x02; // Table 47.  PHY Identification Register 1 (Address 2)
pub const LX970A_PIR1_PIN: m_uint16_t = 0xFFFF; // PHY ID Number (RO=7810h)
pub const LX970A_PIR1_RO_MASK: m_uint16_t = 0xFFFF;
pub const LX970A_PIR1_RW_MASK: m_uint16_t = 0x0000;
pub const LX970A_PIR1_DEFAULT: m_uint16_t = 0x7810;

pub const LX970A_PIR2: usize = 0x03; // Table 48.  PHY Identification Register 2 (Address 3)
pub const LX970A_PIR2_PIN: m_uint16_t = 0xFC00; // PHY ID number (RO=000000b)
pub const LX970A_PIR2_MANMODELNUM: m_uint16_t = 0x03F0; // Manufacturer’s model number (RO=000000b)
pub const LX970A_PIR2_MANREVNUM: m_uint16_t = 0x000F; // Manufacturer’s revision number (RO=0011b)
pub const LX970A_PIR2_RO_MASK: m_uint16_t = 0xFFFF;
pub const LX970A_PIR2_RW_MASK: m_uint16_t = 0x0000;
pub const LX970A_PIR2_DEFAULT: m_uint16_t = 0x0003;

pub const LX970A_ANAR: usize = 0x04; // Table 49.  Auto Negotiation Advertisement Register (Address 4)
pub const LX970A_ANAR_NEXTPAGE: m_uint16_t = 0x8000; // Next Page (RO=0)
pub const LX970A_ANAR_RESERVED_1: m_uint16_t = 0x4000; // Reserved (RO=0)
pub const LX970A_ANAR_REMOTEFAULT: m_uint16_t = 0x2000; // Remote Fault (RW=0)
pub const LX970A_ANAR_RESERVED_2: m_uint16_t = 0x1800; // Reserved (RW=0)
pub const LX970A_ANAR_PAUSE: m_uint16_t = 0x0400; // Pause (RW=0)
pub const LX970A_ANAR_100T4: m_uint16_t = 0x0200; // 100BASE-T4 (RW=0)
pub const LX970A_ANAR_100TX_FD: m_uint16_t = 0x0100; // 100BASE-TX (RW=1[FDE,MF4])
pub const LX970A_ANAR_100TX_HD: m_uint16_t = 0x0080; // 100BASE-TX (RW=1[MF4])
pub const LX970A_ANAR_10T_FD: m_uint16_t = 0x0040; // 10BASE-T full-duplex (RW=1[FDE,CFG1])
pub const LX970A_ANAR_10T_HD: m_uint16_t = 0x0020; // 10BASE-T (RW=1[CFG1])
pub const LX970A_ANAR_SF: m_uint16_t = 0x001F; // Selector Field, S<4:0> (RW=00001b)
pub const LX970A_ANAR_RO_MASK: m_uint16_t = 0xC000;
pub const LX970A_ANAR_RW_MASK: m_uint16_t = 0x3FFF;
pub const LX970A_ANAR_DEFAULT: m_uint16_t = 0x01E1;
// auxiliary
pub const LX970A_ANAR_SF_IEEE8023: m_uint16_t = 0x0001; // <00001> = IEEE 802.3

pub const LX970A_ANLPAR: usize = 0x05; // Table 50.  Auto Negotiation Link Partner Ability Register (Address 5)
pub const LX970A_ANLPAR_NEXTPAGE: m_uint16_t = 0x8000; // Next Page (RO)
pub const LX970A_ANLPAR_ACKNOWLEDGE: m_uint16_t = 0x4000; // Acknowledge (RO)
pub const LX970A_ANLPAR_REMOTEFAULT: m_uint16_t = 0x2000; // Remote Fault (RO)
pub const LX970A_ANLPAR_RESERVED: m_uint16_t = 0x1800; // Reserved (RO)
pub const LX970A_ANLPAR_PAUSE: m_uint16_t = 0x0400; // Pause (RO)
pub const LX970A_ANLPAR_100T4: m_uint16_t = 0x0200; // 100BASE-T4 (RO)
pub const LX970A_ANLPAR_100TX_FD: m_uint16_t = 0x0100; // 100BASE-TX full-duplex (RO)
pub const LX970A_ANLPAR_100TX_HD: m_uint16_t = 0x0080; // 100BASE-TX (RO)
pub const LX970A_ANLPAR_10T_FD: m_uint16_t = 0x0040; // 10BASE-T full-duplex (RO)
pub const LX970A_ANLPAR_10T_HD: m_uint16_t = 0x0020; // 10BASE-T (RO)
pub const LX970A_ANLPAR_SF: m_uint16_t = 0x001F; // Selector Field S[4:0] (RO)
pub const LX970A_ANLPAR_RO_MASK: m_uint16_t = 0xFFFF;
pub const LX970A_ANLPAR_RW_MASK: m_uint16_t = 0x0000;
pub const LX970A_ANLPAR_DEFAULT: m_uint16_t = 0x0000;
// auxiliary
pub const LX970A_ANLPAR_SF_IEEE8023: m_uint16_t = 0x0001; // <00001> = IEEE 802.3

pub const LX970A_ANE: usize = 0x06; // Table 51.  Auto Negotiation Expansion (Address 6)
pub const LX970A_ANE_RESERVED: m_uint16_t = 0xFFE0; // Reserved (RO=0)
pub const LX970A_ANE_PDETECTFAULT: m_uint16_t = 0x0010; // Parallel Detection Fault (RO=0,LH)
pub const LX970A_ANE_LPNPA: m_uint16_t = 0x0008; // Link Partner Next Page Able (RO=0)
pub const LX970A_ANE_NPA: m_uint16_t = 0x0004; // Next Page Able (RO=0)
pub const LX970A_ANE_PR: m_uint16_t = 0x0002; // Page Received (RO=0,LH)
pub const LX970A_ANE_LPANA: m_uint16_t = 0x0001; // Link Partner Auto Neg Able (RO=0)
pub const LX970A_ANE_RO_MASK: m_uint16_t = 0xFFFF;
pub const LX970A_ANE_RW_MASK: m_uint16_t = 0x0000;
pub const LX970A_ANE_DEFAULT: m_uint16_t = 0x0000;

/*
Vendor Specific MII Registers
*/

pub const LX970A_MR: usize = 0x10; // Table 52.  Mirror Register (Address 16, Hex 10)
pub const LX970A_MR_USERDEFINED: m_uint16_t = 0xFFFF; // User Defined (RW=0)
pub const LX970A_MR_RO_MASK: m_uint16_t = 0x0000;
pub const LX970A_MR_RW_MASK: m_uint16_t = 0xFFFF;
pub const LX970A_MR_DEFAULT: m_uint16_t = 0x0000;

pub const LX970A_IER: usize = 0x11; // Table 53.  Interrupt Enable Register (Address 17, Hex 11)
pub const LX970A_IER_RESERVED: m_uint16_t = 0xFFF0; // Reserved (RO=0)
pub const LX970A_IER_MIIDRVLVL: m_uint16_t = 0x0008; // MIIDRVLVL (RW=0)
pub const LX970A_IER_LNK_CRITERIA: m_uint16_t = 0x0004; // LNK CRITERIA (RW=0)
pub const LX970A_IER_INTEN: m_uint16_t = 0x0002; // INTEN (RW=0)
pub const LX970A_IER_TINT: m_uint16_t = 0x0001; // TINT (RW=0,IGN)
pub const LX970A_IER_RO_MASK: m_uint16_t = 0xFFF0;
pub const LX970A_IER_RW_MASK: m_uint16_t = 0x000F;
pub const LX970A_IER_DEFAULT: m_uint16_t = 0x0000;

pub const LX970A_ISR: usize = 0x12; // Table 54.  Interrupt Status Register (Address 18, Hex 12)
pub const LX970A_ISR_MINT: m_uint16_t = 0x8000; // MINT (RO=0?)
pub const LX970A_ISR_XTALOK: m_uint16_t = 0x4000; // XTALOK (RO=0)
pub const LX970A_ISR_RESERVED: m_uint16_t = 0x3FFF; // Reserved (RO=0)
pub const LX970A_ISR_RO_MASK: m_uint16_t = 0xFFFF;
pub const LX970A_ISR_RW_MASK: m_uint16_t = 0x0000;
pub const LX970A_ISR_DEFAULT: m_uint16_t = 0x0000;

pub const LX970A_CFGR: usize = 0x13; // Table 55.  Configuration Register (Address 19, Hex 13)
pub const LX970A_CFGR_RESERVED_1: m_uint16_t = 0x8000; // Reserved (RO=0)
pub const LX970A_CFGR_TXMITTEST: m_uint16_t = 0x4000; // Txmit Test (100BASE-TX) (RW=0,RTOR)
pub const LX970A_CFGR_REPEATERMODE: m_uint16_t = 0x2000; // Repeater Mode (RW=0[MF1])
pub const LX970A_CFGR_MDIOINT: m_uint16_t = 0x1000; // MDIO_INT (RW=0,IGN)
pub const LX970A_CFGR_TPLOOPBACK: m_uint16_t = 0x0800; // TP Loopback (10BASE-T) (RW=0)
pub const LX970A_CFGR_SQE: m_uint16_t = 0x0400; // SQE (10BASE-T) (RW=0)
pub const LX970A_CFGR_JABBER: m_uint16_t = 0x0200; // Jabber (10BASE-T) (RW=0)
pub const LX970A_CFGR_LINKTEST: m_uint16_t = 0x0100; // Link Test (10BASE-T) (RW=0[CFG1,MF0])
pub const LX970A_CFGR_LEDC: m_uint16_t = 0x00C0; // LEDC Programming bits (RW=0)
pub const LX970A_CFGR_ATXC: m_uint16_t = 0x0020; // Advance TX Clock (RW=0)
pub const LX970A_CFGR_5BS_4BN: m_uint16_t = 0x0010; // 5B Symbol/(100BASE-X only) 4B Nibble (RW=1[MF2])
pub const LX970A_CFGR_SCRAMBLER: m_uint16_t = 0x0008; // Scrambler (100BASE-X only) (RW=1[MF3])
pub const LX970A_CFGR_100FX: m_uint16_t = 0x0004; // 100BASE-FX (RW=1[MF4,MF0])
pub const LX970A_CFGR_RESERVED_2: m_uint16_t = 0x0002; // Reserved (RO=0)
pub const LX970A_CFGR_TD: m_uint16_t = 0x0001; // Transmit Disconnect (RW=0)
pub const LX970A_CFGR_RO_MASK: m_uint16_t = 0x8002;
pub const LX970A_CFGR_RW_MASK: m_uint16_t = 0x7FFD;
pub const LX970A_CFGR_DEFAULT: m_uint16_t = 0x0014;
// auxiliary
pub const LX970A_CFGR_LEDC_COLLISION: m_uint16_t = 0x0000; // 0 0 LEDC indicates collision
pub const LX970A_CFGR_LEDC_OFF: m_uint16_t = 0x0040; // 0 1 LEDC is off
pub const LX970A_CFGR_LEDC_ACTIVITY: m_uint16_t = 0x0080; // 1 0 LEDC indicates activity
pub const LX970A_CFGR_LEDC_ALWAYSON: m_uint16_t = 0x00C0; // 1 1 LEDC is continuously on (for diagnostic use)

pub const LX970A_CSR: usize = 0x14; // Table 56.  Chip Status Register (Address 20, Hex 14)
pub const LX970A_CSR_RESERVED_1: m_uint16_t = 0xC000; // Reserved (RO=0?)
pub const LX970A_CSR_LINK: m_uint16_t = 0x2000; // Link (RO=0,RTOR)
pub const LX970A_CSR_DUPLEXMODE: m_uint16_t = 0x1000; // Duplex Mode (RO=1[FDE],RTOR)
pub const LX970A_CSR_SPEED: m_uint16_t = 0x0800; // Speed (RO=1[CFG0],RTOR)
pub const LX970A_CSR_RESERVED_2: m_uint16_t = 0x0400; // Reserved (RO=0?)
pub const LX970A_CSR_ANCOMPLETE: m_uint16_t = 0x0200; // Auto-Negotiation Complete (RO=0,LH,RTOR)
pub const LX970A_CSR_PAGERECEIVED: m_uint16_t = 0x0100; // Page Received (RO=0,LH,RTOR)
pub const LX970A_CSR_RESERVED_3: m_uint16_t = 0x00C0; // Reserved (RO=0)
pub const LX970A_CSR_RESERVED_4: m_uint16_t = 0x0038; // Reserved (RO=0?)
pub const LX970A_CSR_LOWVOLTAGE: m_uint16_t = 0x0004; // Low-Voltage (RO=0?)
pub const LX970A_CSR_RESERVED_5: m_uint16_t = 0x0003; // Reserved (RO=0?)
pub const LX970A_CSR_RO_MASK: m_uint16_t = 0xFFFF;
pub const LX970A_CSR_RW_MASK: m_uint16_t = 0x0000;
pub const LX970A_CSR_DEFAULT: m_uint16_t = 0x1800;
