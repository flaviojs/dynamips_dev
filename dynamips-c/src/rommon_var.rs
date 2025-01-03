//!
//! Cisco router simulation platform.
//! Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
//!
//! ROMMON Environment Variables.

use crate::_extra::*;
use crate::utils::*;
use libc::size_t;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ptr::null_mut;

// ROMMON variable
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rommon_var {
    pub next: *mut rommon_var,
    pub name: *mut c_char,
    pub value: *mut c_char,
}

// List of ROMMON variables
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rommon_var_list {
    pub filename: *mut c_char,
    pub var_list: *mut rommon_var,
}

const DEBUG_OPEN: c_int = 0;

// Load file containing ROMMON variables
#[function_name::named]
#[no_mangle]
pub unsafe extern "C" fn rommon_load_file(rvl: *mut rommon_var_list) -> c_int {
    let mut buffer: [c_char; 512] = [0; 512];

    if (*rvl).filename.is_null() {
        return -1;
    }

    let fd: *mut libc::FILE = libc::fopen((*rvl).filename, c"r".as_ptr());
    if fd.is_null() {
        if DEBUG_OPEN != 0 {
            libc::fprintf(c_stderr(), c"%s: unable to open file %s (%s)\n".as_ptr(), concat!(function_name!(), "\0").as_ptr(), (*rvl).filename, libc::strerror(c_errno()));
        }
        return -1;
    }

    while 0 == libc::feof(fd) {
        if !m_fgets(buffer.as_mut_ptr(), buffer.len() as c_int, fd).is_null() {
            rommon_var_add_str(rvl, buffer.as_mut_ptr());
        }
    }

    libc::fclose(fd);
    0
}

// Write a file with all ROMMON variables
#[function_name::named]
unsafe fn rommon_var_update_file(rvl: *mut rommon_var_list) -> c_int {
    let mut var: *mut rommon_var;

    if (*rvl).filename.is_null() {
        return -1;
    }

    let fd: *mut libc::FILE = libc::fopen((*rvl).filename, c"w".as_ptr());
    if fd.is_null() {
        libc::fprintf(c_stderr(), c"%s: unable to create file %s (%s)\n".as_ptr(), concat!(function_name!(), "\0").as_ptr(), (*rvl).filename, libc::strerror(c_errno()));
        return -1;
    }

    var = (*rvl).var_list;
    while !var.is_null() {
        libc::fprintf(fd, c"%s=%s\n".as_ptr(), (*var).name, if !(*var).value.is_null() { (*var).value } else { c"".as_ptr() });
        var = (*var).next;
    }

    libc::fclose(fd);
    0
}

// Find the specified variable
unsafe fn rommon_var_find(rvl: *mut rommon_var_list, name: *mut c_char) -> *mut rommon_var {
    let mut var: *mut rommon_var;

    var = (*rvl).var_list;
    while !var.is_null() {
        if 0 == libc::strcmp((*var).name, name) {
            return var;
        }
        var = (*var).next;
    }

    null_mut()
}

// Create a new variable
unsafe fn rommon_var_create(name: *mut c_char) -> *mut rommon_var {
    let var: *mut rommon_var = libc::malloc(size_of::<rommon_var>()).cast::<_>();
    if var.is_null() {
        return null_mut();
    }

    (*var).next = null_mut();
    (*var).value = null_mut();
    (*var).name = libc::strdup(name);

    if (*var).name.is_null() {
        libc::free(var.cast::<_>());
        return null_mut();
    }

    var
}

// Delete a variable
unsafe fn rommon_var_delete(var: *mut rommon_var) -> *mut rommon_var {
    let next_var: *mut rommon_var = (*var).next;
    libc::free((*var).value.cast::<_>());
    libc::free((*var).name.cast::<_>());
    libc::free(var.cast::<_>());
    next_var
}

// Set value for a variable
unsafe fn rommon_var_set(var: *mut rommon_var, value: *mut c_char) -> c_int {
    let new_value: *mut c_char = libc::strdup(value);
    if new_value.is_null() {
        return -1;
    }

    // free old value
    if !(*var).value.is_null() {
        libc::free((*var).value.cast::<_>());
    }

    (*var).value = new_value;
    0
}

// Add a new variable
#[no_mangle]
pub unsafe extern "C" fn rommon_var_add(rvl: *mut rommon_var_list, name: *mut c_char, value: *mut c_char) -> c_int {
    let mut var: *mut rommon_var;

    // if the variable already exists, overwrite it
    var = rommon_var_find(rvl, name);
    if var.is_null() {
        var = rommon_var_create(name);
        if var.is_null() {
            return -1;
        }

        if rommon_var_set(var, value) == -1 {
            rommon_var_delete(var);
            return -1;
        }

        (*var).next = (*rvl).var_list;
        (*rvl).var_list = var;
    } else {
        rommon_var_set(var, value);
    }

    // synchronize disk file
    rommon_var_update_file(rvl)
}

// Add a new variable, specified at the format: var=value.
// The string is modified.
#[no_mangle]
pub unsafe extern "C" fn rommon_var_add_str(rvl: *mut rommon_var_list, str_: *mut c_char) -> c_int {
    let eq_sym: *mut c_char = libc::strchr(str_, b'=' as c_int);

    if eq_sym.is_null() {
        return -1;
    }

    // The variable cannot be null
    if str_ == eq_sym {
        return -1;
    }

    *eq_sym = 0;
    rommon_var_add(rvl, str_, eq_sym.add(1))
}

// Get the specified variable
#[no_mangle]
pub unsafe extern "C" fn rommon_var_get(rvl: *mut rommon_var_list, name: *mut c_char, buffer: *mut c_char, len: size_t) -> c_int {
    let var: *mut rommon_var = rommon_var_find(rvl, name);
    if var.is_null() || (*var).value.is_null() {
        return -1;
    }

    libc::strncpy(buffer, (*var).value, len - 1);
    *buffer.add(len - 1) = b'\0' as c_char;
    0
}

// Clear all the variables
#[no_mangle]
pub unsafe extern "C" fn rommon_var_clear(rvl: *mut rommon_var_list) {
    let mut var: *mut rommon_var;
    let mut next_var: *mut rommon_var;

    if rvl.is_null() {
        return;
    }

    var = (*rvl).var_list;
    while !var.is_null() {
        next_var = rommon_var_delete(var);
        var = next_var;
    }
    (*rvl).var_list = null_mut();
}
