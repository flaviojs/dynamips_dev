/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * Virtual Machines.
 */

#ifndef __VM_H__
#define __VM_H__

#include "rust_dynamips_c.h"

#include <pthread.h>

#include "dynamips.h"
#include "memory.h"
#include "cpu.h"
#include "dev_vtty.h"
#include "cisco_card.h"

extern int vm_file_naming_type;

/* Initialize a VM object */
void vm_object_init(vm_obj_t *obj);

/* Add a VM object to an instance */
void vm_object_add(vm_instance_t *vm,vm_obj_t *obj);

/* Remove a VM object from an instance */
void vm_object_remove(vm_instance_t *vm,vm_obj_t *obj);

/* Find an object given its name */
vm_obj_t *vm_object_find(vm_instance_t *vm,char *name);

/* Check that a mandatory object is present */
int vm_object_check(vm_instance_t *vm,char *name);

/* Dump the object list of an instance */
void vm_object_dump(vm_instance_t *vm);

/* Get VM type */
char *vm_get_type(vm_instance_t *vm);

/* Get MAC address MSB */
u_int vm_get_mac_addr_msb(vm_instance_t *vm);

/* Generate a filename for use by the instance */
char *vm_build_filename(vm_instance_t *vm,char *name);

/* Get the amount of host virtual memory used by a VM */
size_t vm_get_vspace_size(vm_instance_t *vm);

/* Check that an instance lock file doesn't already exist */
int vm_get_lock(vm_instance_t *vm);

/* Erase lock file */
void vm_release_lock(vm_instance_t *vm,int erase);

/* Log a message */
void vm_flog(vm_instance_t *vm,char *module,char *format,va_list ap);

/* Log a message */
void vm_log(vm_instance_t *vm,char *module,char *format,...);

/* Close the log file */
int vm_close_log(vm_instance_t *vm);

/* Create the log file */
int vm_create_log(vm_instance_t *vm);

/* Reopen the log file */
int vm_reopen_log(vm_instance_t *vm);

/* Error message */
void vm_error(vm_instance_t *vm,char *format,...);

/* Shutdown hardware resources used by a VM */
int vm_hardware_shutdown(vm_instance_t *vm);

/* Free resources used by a VM */
void vm_free(vm_instance_t *vm);

/* Get an instance given a name */
vm_instance_t *vm_acquire(char *name);

/* Release a VM (decrement reference count) */
int vm_release(vm_instance_t *vm);

/* Initialize RAM */
int vm_ram_init(vm_instance_t *vm,m_uint64_t paddr);

/* Initialize VTTY */
int vm_init_vtty(vm_instance_t *vm);

/* Delete VTTY */
void vm_delete_vtty(vm_instance_t *vm);

/* Bind a device to a virtual machine */
int vm_bind_device(vm_instance_t *vm,struct vdevice *dev);

/* Unbind a device from a virtual machine */
int vm_unbind_device(vm_instance_t *vm,struct vdevice *dev);

/* Map a device at the specified physical address */
int vm_map_device(vm_instance_t *vm,struct vdevice *dev,m_uint64_t base_addr);

/* Set an IRQ for a VM */
void vm_set_irq(vm_instance_t *vm,u_int irq);

/* Clear an IRQ for a VM */
void vm_clear_irq(vm_instance_t *vm,u_int irq);

/* Suspend a VM instance */
int vm_suspend(vm_instance_t *vm);

/* Resume a VM instance */
int vm_resume(vm_instance_t *vm);

/* Stop an instance */
int vm_stop(vm_instance_t *vm);

/* Monitor an instance periodically */
void vm_monitor(vm_instance_t *vm);

/* Allocate an host page */
void *vm_alloc_host_page(vm_instance_t *vm);

/* Free an host page */
void vm_free_host_page(vm_instance_t *vm,void *ptr);

/* Get a ghost image */
int vm_ghost_image_get(char *filename,u_char **ptr,int *fd);

/* Release a ghost image */
int vm_ghost_image_release(int fd);

/* Open a VM file and map it in memory */
int vm_mmap_open_file(vm_instance_t *vm,char *name,
                      u_char **ptr,off_t *fsize);

/* Open/Create a VM file and map it in memory */
int vm_mmap_create_file(vm_instance_t *vm,char *name,size_t len,u_char **ptr);

/* Close a memory mapped file */
int vm_mmap_close_file(int fd,u_char *ptr,size_t len);

/* Save the Cisco IOS configuration from NVRAM */
int vm_ios_save_config(vm_instance_t *vm);

/* Set Cisco IOS image to use */
int vm_ios_set_image(vm_instance_t *vm,char *ios_image);

/* Unset a Cisco IOS configuration file */
void vm_ios_unset_config(vm_instance_t *vm);

/* Set Cisco IOS configuration files to use (NULL to keep existing data) */
int vm_ios_set_config(vm_instance_t *vm,const char *startup_filename,const char *private_filename);

/* Extract IOS configuration from NVRAM and write it to a file */
int vm_nvram_extract_config(vm_instance_t *vm,char *filename);

/* Read IOS configuraton from the files and push it to NVRAM (NULL to keep existing data) */
int vm_nvram_push_config(vm_instance_t *vm,const char *startup_filename,const char *private_filename);

/* Save general VM configuration into the specified file */
void vm_save_config(vm_instance_t *vm,FILE *fd);

/* Find a platform */
vm_platform_t *vm_platform_find(char *name);

/* Find a platform given its CLI name */
vm_platform_t *vm_platform_find_cli_name(char *name);

/* Register a platform */
int vm_platform_register(vm_platform_t *platform);

/* Create an instance of the specified type */
vm_instance_t *vm_create_instance(char *name,int instance_id,char *type);

/* Delete a VM instance */
int vm_delete_instance(char *name);

/* Rename a VM instance */
int vm_rename_instance(vm_instance_t *vm, char *name);

/* Initialize a VM instance */
int vm_init_instance(vm_instance_t *vm);

/* Stop a VM instance */
int vm_stop_instance(vm_instance_t *vm);

/* Delete all VM instances */
int vm_delete_all_instances(void);

/* Save all VM configs */
int vm_save_config_all(FILE *fd);

/* OIR to start a slot/subslot */
int vm_oir_start(vm_instance_t *vm,u_int slot,u_int subslot);

/* OIR to stop a slot/subslot */
int vm_oir_stop(vm_instance_t *vm,u_int slot,u_int subslot);

#endif
