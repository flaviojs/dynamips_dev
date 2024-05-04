//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//! Patched by Jeremy Grossmann for the GNS3 project (www.gns3.net)
//!
//! Generic Cisco 7200 routines and definitions (EEPROM,...).
//!
//! Notes on IRQs (see "show stack"):
//!
//!   - triggering IRQ 3: we get indefinitely (for each slot):
//!        "Error: Unexpected NM Interrupt received from slot: 6"
//!
//!   - triggering IRQ 4: GT64010 reg access: probably "DMA/Timer Interrupt"
//!
//!   - triggering IRQ 6: we get (probably "OIR/Error Interrupt")
//!        %ERR-1-PERR: PCI bus parity error
//!        %ERR-1-SERR: PCI bus system/parity error
//!        %ERR-1-FATAL: Fatal error interrupt, No reloading
//!        err_stat=0x0, err_enable=0x0, mgmt_event=0xFFFFFFFF
