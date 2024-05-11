/*  
 * Cisco router simulation platform.
 * Copyright (c) 2006 Christophe Fillot.  All rights reserved.
 *
 * Cisco EEPROM manipulation functions.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/socket.h>

#include "utils.h"
#include "cisco_eeprom.h"

/* ====================================================================== */
/* NM-1E: 1 Ethernet Port Network Module EEPROM                           */
/* ====================================================================== */
static m_uint16_t eeprom_nm_1e_data[] = {
   0x0143, 0x0100, 0x0075, 0xCD81, 0x500D, 0xA201, 0x0000, 0x0000,
   0x5800, 0x0000, 0x9803, 0x2000, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
};

/* ====================================================================== */
/* NM-4E: 4 Ethernet Port Network Module EEPROM                           */
/* ====================================================================== */
static m_uint16_t eeprom_nm_4e_data[] = {
   0x0142, 0x0100, 0x0075, 0xCD81, 0x500D, 0xA201, 0x0000, 0x0000,
   0x5800, 0x0000, 0x9803, 0x2000, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
};

/* ====================================================================== */
/* NM-1FE-TX: 1 FastEthernet Port Network Module EEPROM                   */
/* ====================================================================== */
static m_uint16_t eeprom_nm_1fe_tx_data[] = {
   0x0144, 0x0100, 0x0075, 0xCD81, 0x500D, 0xA201, 0x0000, 0x0000,
   0x5800, 0x0000, 0x9803, 0x2000, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
};

/* ====================================================================== */
/* NM-16ESW: 16 FastEthernet Port Switch Network Module EEPROM            */
/* ====================================================================== */
static m_uint16_t eeprom_nm_16esw_data[] = {
   0x04FF, 0x4002, 0xA941, 0x0100, 0xC046, 0x0320, 0x003B, 0x3401,
   0x4245, 0x3080, 0x0000, 0x0000, 0x0203, 0xC18B, 0x3030, 0x3030,
   0x3030, 0x3030, 0x3030, 0x3003, 0x0081, 0x0000, 0x0000, 0x0400,
   0xCF06, 0x0013, 0x1A1D, 0x0BD1, 0x4300, 0x11FF, 0xFFFF, 0xFFFF,
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
};

/* ====================================================================== */
/* NMD-36ESW: 36 FastEthernet Port Switch Network Module EEPROM           */
/* ====================================================================== */
static m_uint16_t eeprom_nmd_36esw_data[] = {
   0x04FF, 0x4002, 0xB141, 0x0100, 0xC046, 0x0320, 0x003B, 0x3401,
   0x4245, 0x3080, 0x0000, 0x0000, 0x0203, 0xC18B, 0x3030, 0x3030,
   0x3030, 0x3030, 0x3030, 0x3003, 0x0081, 0x0000, 0x0000, 0x0400,
   0xCF06, 0x0013, 0x1A1D, 0x0BD1, 0x4300, 0x26FF, 0xFFFF, 0xFFFF,
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
};

/* ====================================================================== */
/* NM-4T: 4 Serial Network Module EEPROM                                  */
/* ====================================================================== */
static m_uint16_t eeprom_nm_4t_data[] = {
   0x0154, 0x0101, 0x009D, 0x2D64, 0x5009, 0x0A02, 0x0000, 0x0000,
   0x5800, 0x0000, 0x9811, 0x0300, 0x0005, 0xFFFF, 0xFFFF, 0xFFFF,
};

/* ====================================================================== */
/* NM-2E2W: 2 Ethernet ports with 2 WIC slots Module EEPROM               */
/* ====================================================================== */
static m_uint16_t eeprom_nm_2e2w_data[] = {
   0x011E, 0x0102, 0x009A, 0xEBB1, 0x5004, 0x9305, 0x0000, 0x0000,
   0x5000, 0x0000, 0x9808, 0x1217, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
};

/* ====================================================================== */
/* NM-2W: 2 WIC slots Module EEPROM                                       */
/* ====================================================================== */
static m_uint16_t eeprom_nm_2w_data[] = {
   0x04FF, 0x4000, 0xD641, 0x0100, 0xC046, 0x0320, 0x0012, 0xBF01,
   0x4247, 0x3080, 0x0000, 0x0000, 0x0205, 0xC18B, 0x4A41, 0x4430,
   0x3730, 0x3330, 0x375A, 0x3203, 0x0081, 0x0000, 0x0000, 0x0400,
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
};

/* ====================================================================== */
/* NM-1A-OC3MM: 1 ATM OC3 port Module EEPROM                              */
/* ====================================================================== */
static m_uint16_t eeprom_nm_1a_oc3mm_data[] = {
   0x019A, 0x0100, 0x015B, 0x41D9, 0x500E, 0x7402, 0x0000, 0x0000,
   0x7800, 0x0000, 0x0011, 0x2117, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
};

/* ====================================================================== */
/* NM-NAM: Network Analysis Module EEPROM                                 */
/* ====================================================================== */
static m_uint16_t eeprom_nm_nam_data[] = {
   0x04FF, 0x4004, 0x6A41, 0x0100, 0xC046, 0x0320, 0x004F, 0x9E01,
   0x4241, 0x3080, 0x0000, 0x0000, 0x0202, 0xC18B, 0x4A41, 0x4230,
   0x3630, 0x3630, 0x3543, 0x3403, 0x0081, 0x0000, 0x0000, 0x0400,
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
};

/* ====================================================================== */
/* NM-CIDS: Network Analysis Module EEPROM                                */
/* ====================================================================== */
static m_uint16_t eeprom_nm_cids_data[] = {
   0x04FF, 0x4004, 0x2541, 0x0100, 0xC046, 0x0320, 0x004F, 0x9E01,
   0x4241, 0x3080, 0x0000, 0x0000, 0x0202, 0xC18B, 0x4A41, 0x4230,
   0x3630, 0x3630, 0x3543, 0x3403, 0x0081, 0x0000, 0x0000, 0x0400,
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
   0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 
};

/* ====================================================================== */
/* NM EEPROMs                                                             */
/* ====================================================================== */

static const struct cisco_eeprom eeprom_nm_array[] = {
   { "NM-1E", eeprom_nm_1e_data, sizeof(eeprom_nm_1e_data)/2 },
   { "NM-4E", eeprom_nm_4e_data, sizeof(eeprom_nm_4e_data)/2 },
   { "NM-1FE-TX", eeprom_nm_1fe_tx_data, sizeof(eeprom_nm_1fe_tx_data)/2 },
   { "NM-16ESW", eeprom_nm_16esw_data, sizeof(eeprom_nm_16esw_data)/2 },
   { "NMD-36ESW", eeprom_nmd_36esw_data, sizeof(eeprom_nmd_36esw_data)/2 },
   { "NM-4T", eeprom_nm_4t_data, sizeof(eeprom_nm_4t_data)/2 },
   { "NM-2E2W", eeprom_nm_2e2w_data, sizeof(eeprom_nm_2e2w_data)/2 },
   { "NM-2W", eeprom_nm_2w_data, sizeof(eeprom_nm_2w_data)/2 },
   { "NM-1A-OC3MM", eeprom_nm_1a_oc3mm_data, 
     sizeof(eeprom_nm_1a_oc3mm_data)/2 },
   { "NM-NAM", eeprom_nm_nam_data, sizeof(eeprom_nm_nam_data)/2 },
   { "NM-CIDS", eeprom_nm_cids_data, sizeof(eeprom_nm_cids_data)/2 },
   { NULL, NULL, 0 },
};

/* Find a NM EEPROM */
const struct cisco_eeprom *cisco_eeprom_find_nm(char *name)
{
   return(cisco_eeprom_find(eeprom_nm_array,name));
}

/* ====================================================================== */
/* WIC EEPROMs                                                            */
/* ====================================================================== */

/* ====================================================================== */
/* C6k EEPROMs                                                            */
/* ====================================================================== */

/* ====================================================================== */
/* Utility functions                                                      */
/* ====================================================================== */
