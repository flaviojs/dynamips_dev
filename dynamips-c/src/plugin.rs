//! Cisco router simulation platform.
//! Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
//!
//! Plugins.
//! Plugin management.

use crate::_extra::*;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_void;
use std::mem::transmute;
use std::ptr::null_mut;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct plugin {
    pub filename: *mut c_char,
    pub dl_handle: *mut c_void,
    pub next: *mut plugin,
}

pub type plugin_init_t = Option<unsafe extern "C" fn() -> c_int>;

// Plugin list
static mut plugin_list: *mut plugin = null_mut();

// Find a symbol address
#[no_mangle]
pub unsafe extern "C" fn plugin_find_symbol(plugin: *mut plugin, symbol: *mut c_char) -> *mut c_void {
    if !plugin.is_null() {
        libc::dlsym((*plugin).dl_handle, symbol)
    } else {
        null_mut()
    }
}

// Initialize a plugin
#[no_mangle]
pub unsafe extern "C" fn plugin_init(plugin: *mut plugin) -> c_int {
    let init: plugin_init_t = transmute(plugin_find_symbol(plugin, c"init".as_ptr().cast_mut()));
    if init.is_none() {
        return -1;
    }

    init.unwrap()()
}

// Load a plugin
#[no_mangle]
pub unsafe extern "C" fn plugin_load(filename: *mut c_char) -> *mut plugin {
    let p: *mut plugin = libc::malloc(size_of::<plugin>()).cast::<_>();
    if p.is_null() {
        return null_mut();
    }

    libc::memset(p.cast::<_>(), 0, size_of::<plugin>());

    (*p).filename = libc::strdup(filename);
    if (*p).filename.is_null() {
        libc::free(p.cast::<_>());
        return null_mut();
    }

    (*p).dl_handle = libc::dlopen(filename, libc::RTLD_LAZY);
    if (*p).dl_handle.is_null() {
        libc::fprintf(c_stderr(), c"plugin_load(\"%s\"): %s\n".as_ptr(), filename, libc::dlerror());
        libc::free((*p).filename.cast::<_>());
        libc::free(p.cast::<_>());
        return null_mut();
    }

    if plugin_init(p) == -1 {
        libc::dlclose((*p).dl_handle);
        libc::free((*p).filename.cast::<_>());
        libc::free(p.cast::<_>());
        return null_mut();
    }

    (*p).next = plugin_list;
    plugin_list = p;
    p
}
