//! Extract IOS configuration from a NVRAM file (standalone tool)

use dynamips_c::_private::*;
use dynamips_c::dev_c1700::*;
use dynamips_c::dev_c2600::*;
use dynamips_c::dev_c2691::*;
use dynamips_c::dev_c3600::*;
use dynamips_c::dev_c3725::*;
use dynamips_c::dev_c3745::*;
use dynamips_c::dev_c6msfc1::*;
use dynamips_c::dev_c7200::*;
use dynamips_c::dynamips_common::*;
use dynamips_c::fs_nvram::*;
use std::env::args;

struct NvramFormat {
    name: *mut c_char,
    rom_res_0x200: *mut c_char,
    offset: size_t,
    size: size_t,
    addr: m_uint32_t,
    format: u_int,
}
impl NvramFormat {
    pub const fn new(name: *mut c_char, rom_res_0x200: *mut c_char, offset: size_t, size: size_t, addr: m_uint32_t, format: u_int) -> Self {
        Self { name, rom_res_0x200, offset, size, addr, format }
    }
}
unsafe impl std::marker::Sync for NvramFormat {}

static NVRAM_FORMATS: [NvramFormat; 11] = [
    NvramFormat::new(cstr!("c1700"), cstr!("C1700"), C1700_NVRAM_ROM_RES_SIZE, 0, 0, FS_NVRAM_FORMAT_DEFAULT),
    NvramFormat::new(cstr!("c2600"), cstr!("C2600"), C2600_NVRAM_ROM_RES_SIZE * 4, 0, 0, FS_NVRAM_FORMAT_SCALE_4),
    NvramFormat::new(cstr!("c3600"), cstr!("3600"), C3600_NVRAM_ROM_RES_SIZE, 0, 0, FS_NVRAM_FORMAT_DEFAULT),
    NvramFormat::new(cstr!("c7200"), cstr!("7200"), C7200_NVRAM_ROM_RES_SIZE, 0, C7200_NVRAM_ADDR as m_uint32_t + C7200_NVRAM_ROM_RES_SIZE as m_uint32_t, FS_NVRAM_FORMAT_ABSOLUTE),
    NvramFormat::new(cstr!("c7200-npe-g2"), cstr!("7200"), C7200_NVRAM_ROM_RES_SIZE, 0, C7200_G2_NVRAM_ADDR as m_uint32_t + C7200_NVRAM_ROM_RES_SIZE as m_uint32_t, FS_NVRAM_FORMAT_ABSOLUTE),
    NvramFormat::new(cstr!("c6msfc1"), null_mut(), C6MSFC1_NVRAM_ROM_RES_SIZE, 0, C6MSFC1_NVRAM_ADDR as m_uint32_t + C6MSFC1_NVRAM_ROM_RES_SIZE as m_uint32_t, FS_NVRAM_FORMAT_ABSOLUTE_C6),
    NvramFormat::new(cstr!("c2691"), null_mut(), C2691_NVRAM_OFFSET, C2691_NVRAM_SIZE, 0, FS_NVRAM_FORMAT_WITH_BACKUP),
    NvramFormat::new(cstr!("c3725"), null_mut(), C3725_NVRAM_OFFSET, C3725_NVRAM_SIZE, 0, FS_NVRAM_FORMAT_WITH_BACKUP), // XXX same as c2691
    NvramFormat::new(cstr!("c3745"), null_mut(), C3745_NVRAM_OFFSET, C3745_NVRAM_SIZE, 0, FS_NVRAM_FORMAT_WITH_BACKUP),
    NvramFormat::new(cstr!("c7200-npe-g1"), null_mut(), C7200_NVRAM_ROM_RES_SIZE, 0, C7200_G1_NVRAM_ADDR as m_uint32_t + C7200_NVRAM_ROM_RES_SIZE as m_uint32_t, FS_NVRAM_FORMAT_ABSOLUTE), // XXX didn't find working image
    NvramFormat::new(null_mut(), null_mut(), 0, 0, 0, 0),
];

/// Read file data.
unsafe fn read_file(filename: *const c_char, data: *mut *mut u_char, data_len: *mut size_t) -> c_int {
    // open
    let fd: *mut libc::FILE = libc::fopen(filename, cstr!("rb"));
    if fd.is_null() {
        return -1;
    }

    // len
    libc::fseek(fd, 0, libc::SEEK_END);
    let len: c_long = libc::ftell(fd);
    libc::fseek(fd, 0, libc::SEEK_SET);
    if len < 0 || libc::ferror(fd) != 0 {
        libc::fclose(fd);
        return -1;
    }

    if !data_len.is_null() {
        *data_len = len as size_t;
    }

    // data
    if !data.is_null() {
        *data = libc::malloc(len as size_t).cast::<_>();
        if (*data).is_null() {
            libc::fclose(fd);
            return -1;
        }
        if libc::fread((*data).cast::<_>(), len as size_t, 1, fd) != 1 {
            libc::free((*data).cast::<_>());
            *data = null_mut();
            libc::fclose(fd);
            return -1;
        }
    }

    // close
    libc::fclose(fd);
    0
}

/// Write file data.
unsafe fn write_file(filename: *const c_char, data: *mut u_char, len: size_t) -> c_int {
    let fp: *mut libc::FILE = libc::fopen(filename, cstr!("wb+"));
    if fp.is_null() {
        return -1;
    }

    if libc::fwrite(data.cast::<_>(), len, 1, fp) != 1 {
        libc::fclose(fp);
        return -1;
    }

    libc::fclose(fp);
    0
}

/// Export configuration from NVRAM.
unsafe fn nvram_export_config(nvram_filename: *const c_char, startup_filename: *const c_char, private_filename: *const c_char) -> c_int {
    let mut data: *mut u_char = null_mut();
    let mut startup_config: *mut u_char = null_mut();
    let mut private_config: *mut u_char = null_mut();
    let mut data_len: size_t = 0;
    let mut startup_len: size_t = 0;
    let mut private_len: size_t = 0;
    let mut len: size_t;
    let mut ret: c_int = 0;

    // read nvram
    libc::printf(cstr!("Reading %s...\n"), nvram_filename);
    if read_file(nvram_filename, addr_of_mut!(data), addr_of_mut!(data_len)) != 0 {
        libc::perror(nvram_filename);
        ret = -1;
        // cleanup
        if !startup_config.is_null() {
            libc::free(startup_config.cast::<_>());
        }
        if !private_config.is_null() {
            libc::free(private_config.cast::<_>());
        }
        if !data.is_null() {
            libc::free(data.cast::<_>());
        }
        return ret;
    }

    // try each format
    for fmt in &NVRAM_FORMATS {
        if fmt.name.is_null() {
            libc::fprintf(c_stderr(), cstr!("NVRAM not found\n"));
            ret = -1;
            // cleanup
            if !startup_config.is_null() {
                libc::free(startup_config.cast::<_>());
            }
            if !private_config.is_null() {
                libc::free(private_config.cast::<_>());
            }
            if !data.is_null() {
                libc::free(data.cast::<_>());
            }
            return ret;
        }

        if !fmt.rom_res_0x200.is_null() {
            len = libc::strlen(fmt.rom_res_0x200);
            if data_len < 0x200 + len || libc::memcmp(data.add(0x200).cast::<_>(), fmt.rom_res_0x200.cast::<_>(), len) != 0 {
                continue; // must match
            }
        }

        if fmt.size > 0 {
            if data_len < fmt.offset + fmt.size {
                continue; // must fit
            }
            len = fmt.size;
        } else {
            if data_len < fmt.offset {
                continue; // must fit
            }
            len = data_len - fmt.offset;
        }

        let fs: *mut fs_nvram_t = fs_nvram_open(data.add(fmt.offset), len, fmt.addr, fmt.format);
        if fs.is_null() {
            continue; // filesystem not found
        }

        if fs_nvram_verify(fs, FS_NVRAM_VERIFY_ALL) != 0 || fs_nvram_read_config(fs, addr_of_mut!(startup_config), addr_of_mut!(startup_len), addr_of_mut!(private_config), addr_of_mut!(private_len)) != 0 {
            fs_nvram_close(fs);
            continue; // filesystem error
        }

        libc::printf(cstr!("Found NVRAM format %s\n"), fmt.name);
        fs_nvram_close(fs);
        break;
    }

    // write config
    if !startup_filename.is_null() {
        libc::printf(cstr!("Writing startup-config to %s...\n"), startup_filename);
        if write_file(startup_filename, startup_config, startup_len) != 0 {
            libc::perror(startup_filename);
            ret = -1;
            // cleanup
            if !startup_config.is_null() {
                libc::free(startup_config.cast::<_>());
            }
            if !private_config.is_null() {
                libc::free(private_config.cast::<_>());
            }
            if !data.is_null() {
                libc::free(data.cast::<_>());
            }
            return ret;
        }
    }
    if !private_filename.is_null() {
        libc::printf(cstr!("Writing private-config to %s...\n"), private_filename);
        if write_file(private_filename, private_config, private_len) != 0 {
            libc::perror(private_filename);
            ret = -1;
            // cleanup
            if !startup_config.is_null() {
                libc::free(startup_config.cast::<_>());
            }
            if !private_config.is_null() {
                libc::free(private_config.cast::<_>());
            }
            if !data.is_null() {
                libc::free(data.cast::<_>());
            }
            return ret;
        }
    }

    // cleanup
    if !startup_config.is_null() {
        libc::free(startup_config.cast::<_>());
    }
    if !private_config.is_null() {
        libc::free(private_config.cast::<_>());
    }
    if !data.is_null() {
        libc::free(data.cast::<_>());
    }
    ret
}

fn main() {
    unsafe {
        libc::printf(cstr!("Cisco NVRAM configuration export.\n"));
        libc::printf(cstr!("Copyright (c) 2013 Flávio J. Saraiva.\n\n"));

        let argv: Vec<CString> = args().map(|x| CString::new(x).unwrap()).collect();
        let argc = argv.len();
        if !(3..=4).contains(&argc) {
            libc::fprintf(c_stderr(), cstr!("Usage: %s nvram_file config_file [private_file]\n"), argv[0].as_c());
            libc::fprintf(c_stderr(), cstr!("\n"));
            libc::fprintf(c_stderr(), cstr!("This tools extracts 'startup-config' and 'private-config' from NVRAM.\n"));
            libc::fprintf(c_stderr(), cstr!("  nvram_file   - file that contains the NVRAM data\n"));
            libc::fprintf(c_stderr(), cstr!("                 (on some platforms, NVRAM is simulated inside the ROM)\n"));
            libc::fprintf(c_stderr(), cstr!("  config_file  - file for 'startup-config'\n"));
            libc::fprintf(c_stderr(), cstr!("  private_file - file for 'private-config' (optional)\n"));
            libc::fprintf(c_stderr(), cstr!("\n"));
            libc::fprintf(c_stderr(), cstr!("Supports:"));
            for fmt in &NVRAM_FORMATS {
                if fmt.name.is_null() {
                    break;
                }
                libc::fprintf(c_stderr(), cstr!(" %s"), fmt.name);
            }
            libc::fprintf(c_stderr(), cstr!("\n"));
            libc::exit(libc::EXIT_FAILURE);
        }

        let nvram_filename: *const c_char = argv[1].as_c();
        let startup_filename: *const c_char = argv[2].as_c();
        let mut private_filename: *const c_char = null_mut();
        if argc > 3 {
            private_filename = argv[3].as_c();
        }

        if nvram_export_config(nvram_filename, startup_filename, private_filename) != 0 {
            libc::exit(libc::EXIT_FAILURE);
        }

        libc::printf(cstr!("Done\n"));
        libc::exit(0);
    }
}
