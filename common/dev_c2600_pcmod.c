/*  
 * Cisco router simulation platform.
 * Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
 *
 * PC Modules NM (NM-NAM, NM-CIDS, ...) for c2600 platforms.
 */

#include "dynamips_c.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>
#include <unistd.h>
#include <time.h>
#include <errno.h>
#include <assert.h>

#include "net_io.h"
#include "ptask.h"
#include "vm.h"
#include "dev_i8255x.h"
#include "dev_c2600.h"

/* Initialize a NM-NAM in the specified slot */
static int dev_c2600_pcmod_init(vm_instance_t *vm,struct cisco_card *card)
{
 
   struct i8255x_data *data;
   u_int slot = card->slot_id;

   /* 
    * Non-XM models don't have the capability to byte-swap through their
    * PCI host bridge (required for i82559 data transfers).
    */
   if (!VM_C2600(vm)->xm_model) {
      vm_error(vm,"%s is not supported in C2600 non-XM models.\n",
               card->driver->dev_type);
      return(-1);
   }
 
   /* Set the PCI bus */
   card->pci_bus = vm->slots_pci_bus[slot];

   /* Set the EEPROM */
   cisco_card_set_eeprom(vm,card,cisco_eeprom_find_nm(card->driver->dev_type));
   c2600_set_slot_eeprom(VM_C2600(vm),slot,&card->eeprom);

   /* Create the Intel i8255x chip */
   data = dev_i8255x_init(vm,card->dev_name,0,
                          card->pci_bus,slot * 4,
                          c2600_net_irq_for_slot_port(slot,0));

   /* Store device info into the router structure */
   card->drv_info = data;
   return(0);
}

/* Remove a NM PC module from the specified slot */
static int dev_c2600_pcmod_shutdown(vm_instance_t *vm,struct cisco_card *card)
{
   struct i8255x_data *data = card->drv_info;

   /* Remove the NM EEPROM */
   cisco_card_unset_eeprom(card);
   c2600_set_slot_eeprom(VM_C2600(vm),card->slot_id,NULL);

   /* Remove the Intel i2855x chip */
   dev_i8255x_remove(data);
   return(0);
}

/* Bind a Network IO descriptor */
static int dev_c2600_pcmod_set_nio(vm_instance_t *vm,struct cisco_card *card,
                                   u_int port_id,netio_desc_t *nio)
{
   struct i8255x_data *d = card->drv_info;

   if (!d || (port_id != 0))
      return(-1);

   dev_i8255x_set_nio(d,nio);
   return(0);
}

/* Unbind a Network IO descriptor */
static int dev_c2600_pcmod_unset_nio(vm_instance_t *vm,struct cisco_card *card,
                                     u_int port_id)
{
   struct i8255x_data *d = card->drv_info;

   if (!d || (port_id != 0))
      return(-1);

   dev_i8255x_unset_nio(d);
   return(0);
}

/* NM-NAM driver */
struct cisco_card_driver dev_c2600_nm_nam_driver = {
   "NM-NAM", 0, 0,
   dev_c2600_pcmod_init, 
   dev_c2600_pcmod_shutdown, 
   NULL,
   dev_c2600_pcmod_set_nio,
   dev_c2600_pcmod_unset_nio,
   NULL,
};

/* NM-CIDS driver */
struct cisco_card_driver dev_c2600_nm_cids_driver = {
   "NM-CIDS", 0, 0,
   dev_c2600_pcmod_init, 
   dev_c2600_pcmod_shutdown, 
   NULL,
   dev_c2600_pcmod_set_nio,
   dev_c2600_pcmod_unset_nio,
   NULL,
};
