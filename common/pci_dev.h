/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 */

#ifndef	__PCI_DEV_H__
#define	__PCI_DEV_H__

#include "rust_dynamips_c.h"

#include "utils.h"

/* Trigger a PCI device IRQ */
void pci_dev_trigger_irq(vm_instance_t *vm,struct pci_device *dev);

/* Clear a PCI device IRQ */
void pci_dev_clear_irq(vm_instance_t *vm,struct pci_device *dev);

/* PCI bus lookup */
struct pci_bus *pci_bus_lookup(struct pci_bus *pci_bus_root,int bus);

/* PCI device local lookup */
struct pci_device *pci_dev_lookup_local(struct pci_bus *pci_bus,
                                        int device,int function);

/* PCI device lookup */
struct pci_device *pci_dev_lookup(struct pci_bus *pci_bus_root,
                                  int bus,int device,int function);

/* Handle the address register access */
void pci_dev_addr_handler(cpu_gen_t *cpu,struct pci_bus *pci_bus,
                          u_int op_type,int swap,m_uint64_t *data);

/* Handle the data register access */
void pci_dev_data_handler(cpu_gen_t *cpu,struct pci_bus *pci_bus,
                          u_int op_type,int swap,m_uint64_t *data);

/* Add a PCI bridge */
struct pci_bridge *pci_bridge_add(struct pci_bus *pci_bus);

/* Remove a PCI bridge */
void pci_bridge_remove(struct pci_bridge *bridge);

/* Map secondary bus to a PCI bridge */
void pci_bridge_map_bus(struct pci_bridge *bridge,struct pci_bus *pci_bus);

/* Set PCI bridge bus info */
void pci_bridge_set_bus_info(struct pci_bridge *bridge,
                             int pri_bus,int sec_bus,int sub_bus);

/* Add a PCI device */
struct pci_device *
pci_dev_add(struct pci_bus *pci_bus,
            char *name,u_int vendor_id,u_int product_id,
            int device,int function,int irq,
            void *priv_data,pci_init_t init,
            pci_reg_read_t read_register,
            pci_reg_write_t write_register);

/* Add a basic PCI device that just returns a Vendor/Product ID */
struct pci_device *
pci_dev_add_basic(struct pci_bus *pci_bus,
                  char *name,u_int vendor_id,u_int product_id,
                  int device,int function);

/* Remove a PCI device */
void pci_dev_remove(struct pci_device *dev);

/* Remove a PCI device given its ID (bus,device,function) */
int pci_dev_remove_by_id(struct pci_bus *pci_bus,
                         int bus,int device,int function);

/* Remove a PCI device given its name */
int pci_dev_remove_by_name(struct pci_bus *pci_bus,char *name);

/* Create a PCI bus */
struct pci_bus *pci_bus_create(char *name,int bus);

/* Delete a PCI bus */
void pci_bus_remove(struct pci_bus *pci_bus);

/* Create a PCI bridge device */
struct pci_device *pci_bridge_create_dev(struct pci_bus *pci_bus,char *name,
                                         u_int vendor_id,u_int product_id,
                                         int device,int function,
                                         struct pci_bus *sec_bus,
                                         pci_reg_read_t fallback_read,
                                         pci_reg_write_t fallback_write);

/* Show PCI device list */
void pci_dev_show_list(struct pci_bus *pci_bus);

#endif
