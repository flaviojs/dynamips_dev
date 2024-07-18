//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//! Patched by Jeremy Grossmann for the GNS3 project (www.gns3.net)
//!
//! Many thanks to Nicolas Szalay for his patch
//! for the command line parsing and virtual machine
//! settings (RAM, ROM, NVRAM, ...)

use crate::_private::*;
use crate::cisco_card::*;
use crate::cpu::*;
use crate::crc::*;
use crate::dev_vtty::*;
use crate::dynamips_common::*;
#[cfg(feature = "ENABLE_GEN_ETH")]
use crate::gen_eth::*;
use crate::net_io::*;
use crate::net_io_bridge::*;
use crate::net_io_filter::*;
use crate::plugin::*;
#[cfg(feature = "USE_PROFILER")]
use crate::profiler::*;
use crate::ptask::*;
use crate::registry::*;
use crate::timer::*;
use crate::utils::*;
use crate::vm::*;
use std::env::consts::ARCH;
use std::env::consts::OS;

/// Debugging flags
pub const DEBUG_BLOCK_SCAN: c_int = 0;
pub const DEBUG_BLOCK_COMPILE: c_int = 0;
pub const DEBUG_BLOCK_PATCH: c_int = 0;
pub const DEBUG_BLOCK_CHUNK: c_int = 0;
#[cfg(not(feature = "USE_DEBUG_BLOCK_TIMESTAMP"))]
pub const DEBUG_BLOCK_TIMESTAMP: c_int = 0; // block timestamping (little overhead)
#[cfg(feature = "USE_DEBUG_BLOCK_TIMESTAMP")]
pub const DEBUG_BLOCK_TIMESTAMP: c_int = 1;
#[cfg(not(feature = "USE_DEBUG_SYM_TREE"))]
pub const DEBUG_SYM_TREE: c_int = 0; // use symbol tree (slow)
#[cfg(feature = "USE_DEBUG_SYM_TREE")]
pub const DEBUG_SYM_TREE: c_int = 1;
pub const DEBUG_MTS_MAP_DEV: c_int = 0;
pub const DEBUG_MTS_MAP_VIRT: c_int = 1;
pub const DEBUG_MTS_ACC_U: c_int = 1; // undefined memory
pub const DEBUG_MTS_ACC_T: c_int = 1; // tlb exception
pub const DEBUG_MTS_ACC_AE: c_int = 1; // address error exception
pub const DEBUG_MTS_DEV: c_int = 0; // debugging for device access
pub const DEBUG_MTS_STATS: c_int = 1; // MTS cache performance
pub const DEBUG_INSN_PERF_CNT: c_int = 0; // Instruction performance counter
pub const DEBUG_BLOCK_PERF_CNT: c_int = 0; // Block performance counter
pub const DEBUG_DEV_PERF_CNT: c_int = 1; // Device performance counter
pub const DEBUG_TLB_ACTIVITY: c_int = 0;
pub const DEBUG_SYSCALL: c_int = 0;
pub const DEBUG_CACHE: c_int = 0;
pub const DEBUG_JR0: c_int = 0; // Debug register jumps to 0

/// Feature flags
pub const MEMLOG_ENABLE: c_int = 0; // Memlogger (fast memop must be off)
pub const BREAKPOINT_ENABLE: c_int = 1; // Virtual Breakpoints
pub const NJM_STATS_ENABLE: c_int = 1; // Non-JIT mode stats (little overhead)

/// Symbol
#[repr(C)]
#[derive(Debug)]
pub struct symbol {
    pub addr: m_uint64_t,
    pub name: [c_char; 0], // XXX length determined by the C string NUL terminator
}

/// ROM identification tag
pub const ROM_ID: m_uint32_t = 0x1e94b3df;

/// Command Line long options
pub const OPT_DISK0_SIZE: c_int = 0x100;
pub const OPT_DISK1_SIZE: c_int = 0x101;
pub const OPT_EXEC_AREA: c_int = 0x102;
pub const OPT_IDLE_PC: c_int = 0x103;
pub const OPT_TIMER_ITV: c_int = 0x104;
pub const OPT_VM_DEBUG: c_int = 0x105;
pub const OPT_IOMEM_SIZE: c_int = 0x106;
pub const OPT_SPARSE_MEM: c_int = 0x107;
pub const OPT_NOCTRL: c_int = 0x120;
pub const OPT_NOTELMSG: c_int = 0x121;
pub const OPT_FILEPID: c_int = 0x122;
pub const OPT_STARTUP_CONFIG_FILE: c_int = 0x140;
pub const OPT_PRIVATE_CONFIG_FILE: c_int = 0x141;
pub const OPT_CONSOLE_BINDING_ADDR: c_int = 0x150;

/// Default name for logfile
const LOGFILE_DEFAULT_NAME: *mut c_char = cstr!("dynamips_log.txt");

/// Operating system name
#[no_mangle]
pub static mut os_name: *const c_char = cstr!(OS);

/// Software version
#[no_mangle]
pub static mut sw_version: *const c_char = cstr!(env!("CARGO_PKG_VERSION"), "-", ARCH);

/// Software version tag
#[no_mangle]
pub static mut sw_version_tag: *const c_char = cstr!("2023010200");

/// Hypervisor
static mut hypervisor_mode: c_int = 0;
static mut hypervisor_tcp_port: c_int = 0;
static mut hypervisor_ip_address: *mut c_char = null_mut();

/// Log file
static mut log_file_name: *mut c_char = null_mut();

/// VM flags
static mut vm_save_state: Volatile<c_int> = Volatile(0);

/// Default platform
static mut default_platform: *mut c_char = cstr!("7200");

/// Binding address (NULL means any or 0.0.0.0)
#[no_mangle]
pub static mut binding_addr: *mut c_char = null_mut();

/// Console (vtty tcp) binding address (NULL means any or 0.0.0.0)
#[no_mangle]
pub static mut console_binding_addr: *mut c_char = null_mut();

/// Generic signal handler
unsafe extern "C" fn signal_gen_handler(sig: c_int) {
    match sig {
        libc::SIGHUP => {
            // For future use
        }

        libc::SIGQUIT => {
            // save VM context
            vm_save_state.set(TRUE);
        }

        // Handle SIGPIPE by ignoring it
        libc::SIGPIPE => {
            libc::fprintf(c_stderr(), cstr!("Error: unwanted SIGPIPE.\n"));
        }

        libc::SIGINT => {
            // CTRL+C has been pressed
            if hypervisor_mode != 0 {
                hypervisor_stopsig();
            } else {
                // In theory, this shouldn't happen thanks to VTTY settings
                let vm: *mut vm_instance_t = vm_acquire(cstr!("default"));
                if !vm.is_null() {
                    // Only forward ctrl-c if user has requested local terminal
                    if (*vm).vtty_con_type == VTTY_TYPE_TERM {
                        vtty_store_ctrlc((*vm).vtty_con);
                    } else {
                        vm_stop(vm);
                    }
                    vm_release(vm);
                } else {
                    libc::fprintf(c_stderr(), cstr!("Error: Cannot acquire instance handle.\n"));
                }
            }
        }

        _ => {
            libc::fprintf(c_stderr(), cstr!("Unhandled signal %d\n"), sig);
        }
    }
}

/// Setups signals
unsafe fn setup_signals() {
    let mut act: libc::sigaction = zeroed::<_>();

    libc::memset(addr_of_mut!(act).cast::<_>(), 0, size_of::<libc::sigaction>());
    #[cfg(has_libc_sigaction_sa_handler)]
    {
        act.sa_handler = signal_gen_handler as _;
    }
    #[cfg(not(has_libc_sigaction_sa_handler))]
    {
        // XXX assume sa_handler was aliased to sa_sigaction or is in a union with sa_sigaction
        act.sa_sigaction = signal_gen_handler as _;
    }
    act.sa_flags = libc::SA_RESTART;
    libc::sigaction(libc::SIGHUP, addr_of_mut!(act), null_mut());
    libc::sigaction(libc::SIGQUIT, addr_of_mut!(act), null_mut());
    libc::sigaction(libc::SIGINT, addr_of_mut!(act), null_mut());
    libc::sigaction(libc::SIGPIPE, addr_of_mut!(act), null_mut());
}

/// Create general log file
unsafe fn create_log_file() {
    // Set the default value of the log file name
    if log_file_name.is_null() {
        log_file_name = libc::strdup(LOGFILE_DEFAULT_NAME);
        if log_file_name.is_null() {
            libc::fprintf(c_stderr(), cstr!("Unable to set log file name.\n"));
            libc::exit(libc::EXIT_FAILURE);
        }
    }

    log_file = libc::fopen(log_file_name, cstr!("w"));
    if log_file.is_null() {
        libc::fprintf(c_stderr(), cstr!("Unable to create log file (%s).\n"), libc::strerror(c_errno()));
        libc::exit(libc::EXIT_FAILURE);
    }
}

/// Close general log file
#[no_mangle]
pub unsafe extern "C" fn close_log_file() {
    if !log_file.is_null() {
        libc::fclose(log_file);
    }
    libc::free(log_file_name.cast::<_>());

    log_file = null_mut();
    log_file_name = null_mut();
}

/// Display the command line use
unsafe fn show_usage(vm: *mut vm_instance_t, _argc: c_int, argv: *mut *mut c_char) {
    libc::printf(cstr!("Usage: %s [options] <ios_image>\n\n"), *argv.add(0));

    libc::printf(
        cstr!(
            "Available options:\n",
            "  -H [<ip_address>:]<tcp_port> : Run in hypervisor mode\n\n",
            "  -P <platform>      : Platform to emulate (7200, 3600, 2691, 3725, 3745, 2600 or 1700) (default: 7200)\n\n",
            "  -l <log_file>      : Set logging file (default is %s)\n",
            "  -j                 : Disable the JIT compiler, very slow\n",
            "  --idle-pc <pc>     : Set the idle PC (default: disabled)\n",
            "  --timer-itv <val>  : Timer IRQ interval check (default: %u)\n",
            "\n",
            "  -i <instance>      : Set instance ID\n",
            "  -r <ram_size>      : Set the virtual RAM size (default: %u Mb)\n",
            "  -o <rom_size>      : Set the virtual ROM size (default: %u Mb)\n",
            "  -n <nvram_size>    : Set the NVRAM size (default: %d Kb)\n",
            "  -c <conf_reg>      : Set the configuration register (default: 0x%04x)\n",
            "  -m <mac_addr>      : Set the MAC address of the chassis\n",
            "                       (default: automatically generated)\n",
            "  -C, --startup-config <file> : Import IOS configuration file into NVRAM\n",
            "  --private-config <file> : Import IOS configuration file into NVRAM\n",
            "  -X                 : Do not use a file to simulate RAM (faster)\n",
            "  -G <ghost_file>    : Use a ghost file to simulate RAM\n",
            "  -g <ghost_file>    : Generate a ghost RAM file\n",
            "  --sparse-mem       : Use sparse memory\n",
            "  -R <rom_file>      : Load an alternate ROM (default: embedded)\n",
            "  -k <clock_div>     : Set the clock divisor (default: %d)\n",
            "\n",
            "  -T <port>          : Console is on TCP <port>\n",
            "  -U <si_desc>       : Console in on serial interface <si_desc>\n",
            "                       (default is on the terminal)\n",
            "\n",
            "  -A <port>          : AUX is on TCP <port>\n",
            "  -B <si_desc>       : AUX is on serial interface <si_desc>\n",
            "                       (default is no AUX port)\n",
            "\n",
            "  --disk0 <size>     : Set PCMCIA ATA disk0: size (default: %u Mb)\n",
            "  --disk1 <size>     : Set PCMCIA ATA disk1: size (default: %u Mb)\n",
            "\n",
            "  --noctrl           : Disable ctrl+] monitor console\n",
            "  --notelnetmsg      : Disable message when using tcp console/aux\n",
            "  --filepid filename : Store dynamips pid in a file\n",
            "  --console-binding-addr: binding address for tcp console/aux\n",
            "\n"
        ),
        LOGFILE_DEFAULT_NAME,
        VM_TIMER_IRQ_CHECK_ITV,
        (*vm).ram_size,
        (*vm).rom_size,
        (*vm).nvram_size,
        (*vm).conf_reg_setup,
        (*vm).clock_divisor,
        (*vm).pcmcia_disk_size[0],
        (*vm).pcmcia_disk_size[1],
    );

    if (*(*vm).platform).cli_show_options.is_some() {
        (*(*vm).platform).cli_show_options.unwrap()(vm);
    }

    libc::printf(cstr!("\n"));
    if DEBUG_SYM_TREE != 0 {
        libc::printf(cstr!("  -S <sym_file>      : Load a symbol file\n"));
    }
    libc::printf(cstr!(
        "  -a <cfg_file>      : Virtual ATM switch configuration file\n",
        "  -f <cfg_file>      : Virtual Frame-Relay switch configuration file\n",
        "  -E <cfg_file>      : Virtual Ethernet switch configuration file\n",
        "  -b <cfg_file>      : Virtual bridge configuration file\n",
        "  -e                 : Show network device list of the host machine\n",
        "\n"
    ));

    #[rustfmt::skip]
    libc::printf(cstr!(
        "<si_desc> format:\n",
        "   \"device{:baudrate{:databits{:parity{:stopbits{:hwflow}}}}}}\"\n",
        "\n"
    ));

    match (*vm).slots_type {
        CISCO_CARD_TYPE_PA => {
            #[rustfmt::skip]
            libc::printf(cstr!(
                "<pa_desc> format:\n",
                "   \"slot:sub_slot:pa_driver\"\n",
                "\n"
            ));

            #[rustfmt::skip]
            libc::printf(cstr!(
                "<pa_nio> format:\n",
                "   \"slot:port:netio_type{:netio_parameters}\"\n",
                "\n"
            ));
        }

        CISCO_CARD_TYPE_NM => {
            #[rustfmt::skip]
            libc::printf(cstr!(
                "<nm_desc> format:\n",
                "   \"slot:sub_slot:nm_driver\"\n",
                "\n"
            ));

            #[rustfmt::skip]
            libc::printf(cstr!(
                "<nm_nio> format:\n",
                "   \"slot:port:netio_type{:netio_parameters}\"\n",
                "\n"
            ));
        }

        CISCO_CARD_TYPE_WIC => {
            #[rustfmt::skip]
            libc::printf(cstr!(
                "<wic_desc> format:\n",
                "   \"slot:wic_driver\"\n",
                "\n"
            ));

            #[rustfmt::skip]
            libc::printf(cstr!(
                "<wic_nio> format:\n",
                "   \"slot:port:netio_type{:netio_parameters}\"\n",
                "\n"
            ));
        }

        _ => {}
    }

    if (*(*vm).platform).show_spec_drivers.is_some() {
        (*(*vm).platform).show_spec_drivers.unwrap()();
    }

    // Show possible slot drivers
    vm_slot_show_drivers(vm);

    // Show the possible NETIO types
    netio_show_types();
}

/// Find an option in the command line
unsafe fn cli_find_option(argc: c_int, argv: *mut *mut c_char, opt: *mut c_char) -> *mut c_char {
    for i in 1..argc {
        if libc::strncmp(*argv.offset(i as isize), opt, 2) == 0 {
            if *(*argv.offset(i as isize)).add(2) != 0 {
                return (*argv.offset(i as isize)).add(2);
            } else {
                #[allow(clippy::collapsible_else_if)]
                if !(*argv.offset(i as isize + 1)).is_null() {
                    return *argv.offset(i as isize + 1);
                } else {
                    libc::fprintf(c_stderr(), cstr!("Error: option '%s': no argument specified.\n"), opt);
                    libc::exit(libc::EXIT_FAILURE);
                }
            }
        }
    }

    null_mut()
}

/// Load plugins
unsafe fn cli_load_plugins(argc: c_int, argv: *mut *mut c_char) {
    let mut str_: *mut c_char;

    for i in 1..argc {
        if libc::strncmp(*argv.offset(i as isize), cstr!("-L"), 2) == 0 {
            if *(*argv.offset(i as isize)).add(2) != 0 {
                str_ = (*argv.offset(i as isize)).add(2);
            } else {
                #[allow(clippy::collapsible_else_if)]
                if !(*argv.offset(i as isize + 1)).is_null() {
                    str_ = *argv.offset(i as isize + 1);
                } else {
                    libc::fprintf(c_stderr(), cstr!("Plugin error: no argument specified.\n"));
                    libc::exit(libc::EXIT_FAILURE);
                }
            }

            if plugin_load(str_).is_null() {
                libc::fprintf(c_stderr(), cstr!("Unable to load plugin '%s'!\n"), str_);
            }
        }
    }
}

/// Determine the platform (Cisco 3600, 7200). Default is Cisco 7200
unsafe fn cli_get_platform_type(argc: c_int, argv: *mut *mut c_char) -> *mut vm_platform_t {
    let mut str_: *mut c_char;

    str_ = cli_find_option(argc, argv, cstr!("-P"));
    if str_.is_null() {
        str_ = default_platform;
    }

    let platform: *mut vm_platform_t = vm_platform_find_cli_name(str_);
    if platform.is_null() {
        libc::fprintf(c_stderr(), cstr!("Invalid platform type '%s'\n"), str_);
    }

    platform
}

static mut cmd_line_lopts: [libc::option; 14] = [
    libc::option { name: cstr!("disk0"), has_arg: 1, flag: null_mut(), val: OPT_DISK0_SIZE },
    libc::option { name: cstr!("disk1"), has_arg: 1, flag: null_mut(), val: OPT_DISK1_SIZE },
    libc::option { name: cstr!("idle-pc"), has_arg: 1, flag: null_mut(), val: OPT_IDLE_PC },
    libc::option { name: cstr!("timer-itv"), has_arg: 1, flag: null_mut(), val: OPT_TIMER_ITV },
    libc::option { name: cstr!("vm-debug"), has_arg: 1, flag: null_mut(), val: OPT_VM_DEBUG },
    libc::option { name: cstr!("iomem-size"), has_arg: 1, flag: null_mut(), val: OPT_IOMEM_SIZE },
    libc::option { name: cstr!("sparse-mem"), has_arg: 0, flag: null_mut(), val: OPT_SPARSE_MEM },
    libc::option { name: cstr!("noctrl"), has_arg: 0, flag: null_mut(), val: OPT_NOCTRL },
    libc::option { name: cstr!("notelnetmsg"), has_arg: 0, flag: null_mut(), val: OPT_NOTELMSG },
    libc::option { name: cstr!("filepid"), has_arg: 1, flag: null_mut(), val: OPT_FILEPID },
    libc::option { name: cstr!("startup-config"), has_arg: 1, flag: null_mut(), val: OPT_STARTUP_CONFIG_FILE },
    libc::option { name: cstr!("private-config"), has_arg: 1, flag: null_mut(), val: OPT_PRIVATE_CONFIG_FILE },
    libc::option { name: cstr!("console-binding-addr"), has_arg: 1, flag: null_mut(), val: OPT_CONSOLE_BINDING_ADDR },
    libc::option { name: null_mut(), has_arg: 0, flag: null_mut(), val: 0 },
];

/// Create a router instance
unsafe fn cli_create_instance(name: *mut c_char, platform_name: *mut c_char, instance_id: c_int) -> *mut vm_instance_t {
    let vm: *mut vm_instance_t = vm_create_instance(name, instance_id, platform_name);

    if vm.is_null() {
        libc::fprintf(c_stderr(), cstr!("%s: unable to create instance %s!\n"), platform_name, name);
        return null_mut();
    }

    vm
}

/// Parse the command line
unsafe fn parse_std_cmd_line(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let options_list: *mut c_char = cstr!("r:o:n:c:m:l:C:i:jt:p:s:k:T:U:A:B:a:f:E:b:S:R:M:eXP:N:G:g:L:I:");
    let mut instance_id: c_int;
    let mut option: c_int;
    let mut str_: *mut c_char;
    let mut pid_file: *mut libc::FILE; // For saving the pid if requested

    // Get the instance ID
    instance_id = 0;

    // Use the old VM file naming type
    vm_file_naming_type = 1;

    cli_load_plugins(argc, argv);

    str_ = cli_find_option(argc, argv, cstr!("-i"));
    if !str_.is_null() {
        instance_id = libc::atoi(str_);
        libc::printf(cstr!("Instance ID set to %d.\n"), instance_id);
    }

    str_ = cli_find_option(argc, argv, cstr!("-N"));
    if !str_.is_null() {
        vm_file_naming_type = libc::atoi(str_);
    }

    // Get the platform type
    let platform: *mut vm_platform_t = cli_get_platform_type(argc, argv);
    if platform.is_null() {
        libc::exit(libc::EXIT_FAILURE);
        #[allow(unreachable_code)]
        {
            return -1;
        }
    }

    // Create the default instance
    let vm: *mut vm_instance_t = cli_create_instance(cstr!("default"), (*platform).name, instance_id);
    if vm.is_null() {
        libc::exit(libc::EXIT_FAILURE);
        #[allow(unreachable_code)]
        {
            return -1;
        }
    }

    c_opterr_set(0);

    vtty_set_ctrlhandler(1); // By default allow ctrl ]
    vtty_set_telnetmsg(1); // By default allow telnet message

    loop {
        option = libc::getopt_long(argc, argv, options_list, cmd_line_lopts.as_c(), null_mut());
        if option == -1 {
            break;
        }
        match option {
            // Instance ID (already managed)
            x if x == b'i' as c_int => {}

            // Platform (already managed)
            x if x == b'P' as c_int => {}

            // RAM size
            x if x == b'r' as c_int => {
                (*vm).ram_size = libc::strtol(c_optarg(), null_mut(), 10) as u_int;
                libc::printf(cstr!("Virtual RAM size set to %d MB.\n"), (*vm).ram_size);
            }

            // ROM size
            x if x == b'o' as c_int => {
                (*vm).rom_size = libc::strtol(c_optarg(), null_mut(), 10) as u_int;
                libc::printf(cstr!("Virtual ROM size set to %d MB.\n"), (*vm).rom_size);
            }

            // NVRAM size
            x if x == b'n' as c_int => {
                (*vm).nvram_size = libc::strtol(c_optarg(), null_mut(), 10) as u_int;
                libc::printf(cstr!("NVRAM size set to %d KB.\n"), (*vm).nvram_size);
            }

            // PCMCIA disk0 size
            OPT_DISK0_SIZE => {
                (*vm).pcmcia_disk_size[0] = libc::atoi(c_optarg()) as u_int;
                libc::printf(cstr!("PCMCIA ATA disk0 size set to %u MB.\n"), (*vm).pcmcia_disk_size[0]);
            }

            // PCMCIA disk1 size
            OPT_DISK1_SIZE => {
                (*vm).pcmcia_disk_size[1] = libc::atoi(c_optarg()) as u_int;
                libc::printf(cstr!("PCMCIA ATA disk1 size set to %u MB.\n"), (*vm).pcmcia_disk_size[1]);
            }

            OPT_NOCTRL => {
                vtty_set_ctrlhandler(0); // Ignore ctrl ]
                libc::printf(cstr!("Block ctrl+] access to monitor console.\n"));
            }

            // Config Register
            x if x == b'c' as c_int => {
                (*vm).conf_reg_setup = libc::strtol(c_optarg(), null_mut(), 0) as u_int;
                libc::printf(cstr!("Config. Register set to 0x%x.\n"), (*vm).conf_reg_setup);
            }

            // IOS startup configuration file
            x if x == b'C' as c_int || x == OPT_STARTUP_CONFIG_FILE => {
                vm_ios_set_config(vm, c_optarg(), (*vm).ios_private_config);
            }

            // IOS private configuration file
            OPT_PRIVATE_CONFIG_FILE => {
                vm_ios_set_config(vm, (*vm).ios_startup_config, c_optarg());
            }

            // Global console (vtty tcp) binding address
            OPT_CONSOLE_BINDING_ADDR => {
                if !console_binding_addr.is_null() {
                    libc::free(console_binding_addr.cast::<_>());
                }
                console_binding_addr = libc::strdup(c_optarg());
                libc::printf(cstr!("Console binding address set to %s\n"), console_binding_addr);
            }

            // Use physical memory to emulate RAM (no-mapped file)
            x if x == b'X' as c_int => {
                (*vm).ram_mmap = 0;
            }

            // Use a ghost file to simulate RAM
            x if x == b'G' as c_int => {
                libc::free((*vm).ghost_ram_filename.cast::<_>());
                (*vm).ghost_ram_filename = libc::strdup(c_optarg());
                (*vm).ghost_status = VM_GHOST_RAM_USE;
            }

            // Generate a ghost RAM image
            x if x == b'g' as c_int => {
                libc::free((*vm).ghost_ram_filename.cast::<_>());
                (*vm).ghost_ram_filename = libc::strdup(c_optarg());
                (*vm).ghost_status = VM_GHOST_RAM_GENERATE;
            }

            // Use sparse memory
            OPT_SPARSE_MEM => {
                (*vm).sparse_mem = TRUE;
            }

            // Alternate ROM
            x if x == b'R' as c_int => {
                libc::free((*vm).rom_filename.cast::<_>());
                (*vm).rom_filename = libc::strdup(c_optarg());
            }

            OPT_NOTELMSG => {
                vtty_set_telnetmsg(0); // disable telnet greeting
                libc::printf(cstr!("Prevent telnet message on AUX/CONSOLE connecte.\n"));
            }

            OPT_FILEPID => {
                pid_file = libc::fopen(c_optarg(), cstr!("w"));
                if !pid_file.is_null() {
                    libc::fprintf(pid_file, cstr!("%d"), libc::getpid());
                    libc::fclose(pid_file);
                } else {
                    libc::printf(cstr!("Unable to save to %s.\n"), c_optarg());
                }
            }

            // Idle PC
            OPT_IDLE_PC => {
                (*vm).idle_pc = libc::strtoull(c_optarg(), null_mut(), 0);
                libc::printf(cstr!("Idle PC set to 0x%llx.\n"), (*vm).idle_pc);
            }

            // Timer IRQ check interval
            OPT_TIMER_ITV => {
                (*vm).timer_irq_check_itv = libc::atoi(c_optarg()) as u_int;
            }

            // Clock divisor
            x if x == b'k' as c_int => {
                (*vm).clock_divisor = libc::atoi(c_optarg()) as u_int;

                if (*vm).clock_divisor == 0 {
                    libc::fprintf(c_stderr(), cstr!("Invalid Clock Divisor specified!\n"));
                    if !vm.is_null() {
                        vm_release(vm);
                        vm_delete_instance(cstr!("default"));
                    }
                    libc::exit(libc::EXIT_FAILURE);
                    #[allow(unreachable_code)]
                    {
                        return -1;
                    }
                }

                libc::printf(cstr!("Using a clock divisor of %d.\n"), (*vm).clock_divisor);
            }

            // Disable JIT
            x if x == b'j' as c_int => {
                (*vm).jit_use = FALSE;
            }

            // VM debug level
            OPT_VM_DEBUG => {
                (*vm).debug_level = libc::atoi(c_optarg());
            }

            // Log file
            x if x == b'l' as c_int => {
                log_file_name = libc::realloc(log_file_name.cast::<_>(), libc::strlen(c_optarg()) + 1).cast::<_>();
                if log_file_name.is_null() {
                    libc::fprintf(c_stderr(), cstr!("Unable to set log file name.\n"));
                    if !vm.is_null() {
                        vm_release(vm);
                        vm_delete_instance(cstr!("default"));
                    }
                    libc::exit(libc::EXIT_FAILURE);
                    #[allow(unreachable_code)]
                    {
                        return -1;
                    }
                }
                libc::strcpy(log_file_name, c_optarg());
                libc::printf(cstr!("Log file: writing to %s\n"), log_file_name);
            }

            // Symbol file
            #[cfg(feature = "USE_DEBUG_SYM_TREE")]
            x if x == b'S' as c_int => {
                (*vm).sym_filename = libc::strdup(c_optarg());
            }

            // TCP server for Console Port
            x if x == b'T' as c_int => {
                (*vm).vtty_con_type = VTTY_TYPE_TCP;
                (*vm).vtty_con_tcp_port = libc::atoi(c_optarg());
            }

            // Serial interface for Console port
            x if x == b'U' as c_int => {
                (*vm).vtty_con_type = VTTY_TYPE_SERIAL;
                if vtty_parse_serial_option(addr_of_mut!((*vm).vtty_con_serial_option), c_optarg()) != 0 {
                    libc::fprintf(c_stderr(), cstr!("Invalid Console serial interface descriptor!\n"));
                    if !vm.is_null() {
                        vm_release(vm);
                        vm_delete_instance(cstr!("default"));
                    }
                    libc::exit(libc::EXIT_FAILURE);
                    #[allow(unreachable_code)]
                    {
                        return -1;
                    }
                }
            }

            // TCP server for AUX Port
            x if x == b'A' as c_int => {
                (*vm).vtty_aux_type = VTTY_TYPE_TCP;
                (*vm).vtty_aux_tcp_port = libc::atoi(c_optarg());
            }

            // Serial interface for AUX port
            x if x == b'B' as c_int => {
                (*vm).vtty_aux_type = VTTY_TYPE_SERIAL;
                if vtty_parse_serial_option(addr_of_mut!((*vm).vtty_aux_serial_option), c_optarg()) != 0 {
                    libc::fprintf(c_stderr(), cstr!("Invalid AUX serial interface descriptor!\n"));
                    if !vm.is_null() {
                        vm_release(vm);
                        vm_delete_instance(cstr!("default"));
                    }
                    libc::exit(libc::EXIT_FAILURE);
                    #[allow(unreachable_code)]
                    {
                        return -1;
                    }
                }
            }

            // Port settings
            x if x == b'p' as c_int => {
                vm_slot_cmd_create(vm, c_optarg());
            }

            // NIO settings
            x if x == b's' as c_int => {
                vm_slot_cmd_add_nio(vm, c_optarg());
            }

            // Virtual ATM switch
            x if x == b'a' as c_int => {
                if atmsw_start(c_optarg()) == -1 {
                    if !vm.is_null() {
                        vm_release(vm);
                        vm_delete_instance(cstr!("default"));
                    }
                    libc::exit(libc::EXIT_FAILURE);
                    #[allow(unreachable_code)]
                    {
                        return -1;
                    }
                }
            }

            // Virtual ATM bridge
            x if x == b'M' as c_int => {
                if atm_bridge_start(c_optarg()) == -1 {
                    if !vm.is_null() {
                        vm_release(vm);
                        vm_delete_instance(cstr!("default"));
                    }
                    libc::exit(libc::EXIT_FAILURE);
                    #[allow(unreachable_code)]
                    {
                        return -1;
                    }
                }
            }

            // Virtual Frame-Relay switch
            x if x == b'f' as c_int => {
                if frsw_start(c_optarg()) == -1 {
                    if !vm.is_null() {
                        vm_release(vm);
                        vm_delete_instance(cstr!("default"));
                    }
                    libc::exit(libc::EXIT_FAILURE);
                    #[allow(unreachable_code)]
                    {
                        return -1;
                    }
                }
            }

            // Virtual Ethernet switch
            x if x == b'E' as c_int => {
                if ethsw_start(c_optarg()) == -1 {
                    if !vm.is_null() {
                        vm_release(vm);
                        vm_delete_instance(cstr!("default"));
                    }
                    libc::exit(libc::EXIT_FAILURE);
                    #[allow(unreachable_code)]
                    {
                        return -1;
                    }
                }
            }

            // Virtual bridge
            x if x == b'b' as c_int => {
                if netio_bridge_start(c_optarg()) == -1 {
                    if !vm.is_null() {
                        vm_release(vm);
                        vm_delete_instance(cstr!("default"));
                    }
                    libc::exit(libc::EXIT_FAILURE);
                    #[allow(unreachable_code)]
                    {
                        return -1;
                    }
                }
            }

            // Ethernet device list
            #[cfg(feature = "ENABLE_GEN_ETH")]
            x if x == b'e' as c_int => {
                gen_eth_show_dev_list();
                if !vm.is_null() {
                    vm_release(vm);
                    vm_delete_instance(cstr!("default"));
                }
                libc::exit(libc::EXIT_SUCCESS);
                #[allow(unreachable_code)]
                {
                    return -1;
                }
            }

            // Load plugin (already handled)
            x if x == b'L' as c_int => {}

            // Oops !
            x if x == b'?' as c_int => {
                show_usage(vm, argc, argv);
                if !vm.is_null() {
                    vm_release(vm);
                    vm_delete_instance(cstr!("default"));
                }
                libc::exit(libc::EXIT_FAILURE);
                #[allow(unreachable_code)]
                {
                    return -1;
                }
            }

            // Parse options specific to the platform
            _ => {
                if (*(*vm).platform).cli_parse_options.is_some() {
                    // If you get an option wrong, say which option is was
                    // Wont be pretty for a long option, but it will at least help
                    if (*(*vm).platform).cli_parse_options.unwrap()(vm, option) == -1 {
                        libc::printf(cstr!("Flag not recognised: -%c\n"), option as c_char as c_int);
                        if !vm.is_null() {
                            vm_release(vm);
                            vm_delete_instance(cstr!("default"));
                        }
                        libc::exit(libc::EXIT_FAILURE);
                        #[allow(unreachable_code)]
                        {
                            return -1;
                        }
                    }
                }
            }
        }
    }

    // Last argument, this is the IOS filename
    if c_optind() == (argc - 1) {
        // setting IOS image file
        vm_ios_set_image(vm, *argv.offset(c_optind() as isize));
        libc::printf(cstr!("IOS image file: %s\n\n"), (*vm).ios_image);
    } else {
        // IOS missing
        libc::fprintf(c_stderr(), cstr!("Please specify an IOS image filename\n"));
        show_usage(vm, argc, argv);
        if !vm.is_null() {
            vm_release(vm);
            vm_delete_instance(cstr!("default"));
        }
        libc::exit(libc::EXIT_FAILURE);
        #[allow(unreachable_code)]
        {
            return -1;
        }
    }

    vm_release(vm);
    0
}

/// Run in hypervisor mode with a config file if the "-H" option
/// is present in command line.
unsafe fn run_hypervisor(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let options_list: *mut c_char = cstr!("H:l:hN:L:");
    let mut option: c_int;
    let mut index: *mut c_char;
    let mut len: size_t;
    let mut pid_file: *mut libc::FILE; // For saving the pid if requested

    vtty_set_ctrlhandler(1); // By default allow ctrl ]
    vtty_set_telnetmsg(1); // By default allow telnet message

    for i in 1..argc {
        if libc::strcmp(*argv.add(i as usize), cstr!("-H")) == 0 {
            hypervisor_mode = 1;
            break;
        }
    }

    // standard mode with one instance
    if hypervisor_mode == 0 {
        return FALSE;
    }

    cli_load_plugins(argc, argv);

    c_opterr_set(0);

    // New long options are sometimes appropriate for hypervisor mode
    loop {
        option = libc::getopt_long(argc, argv, options_list, cmd_line_lopts.as_c(), null_mut());
        if option == -1 {
            break;
        }
        const OPT_H: c_int = b'H' as c_int;
        const OPT_l: c_int = b'l' as c_int;
        const OPT_N: c_int = b'N' as c_int;
        const OPT_L: c_int = b'L' as c_int;
        const OPT_QUESTION_MARK: c_int = b'?' as c_int;
        match option {
            // Hypervisor TCP port
            x if x == b'H' as c_int => {
                index = libc::strrchr(c_optarg(), b':' as c_int);

                if index.is_null() {
                    hypervisor_tcp_port = libc::atoi(c_optarg());
                } else {
                    len = index as size_t - c_optarg() as size_t;
                    hypervisor_ip_address = libc::realloc(hypervisor_ip_address.cast::<_>(), len + 1).cast::<_>();

                    if hypervisor_ip_address.is_null() {
                        libc::fprintf(c_stderr(), cstr!("Unable to set hypervisor IP address!\n"));
                        libc::exit(libc::EXIT_FAILURE);
                    }

                    libc::memcpy(hypervisor_ip_address.cast::<_>(), c_optarg().cast::<_>(), len);
                    *hypervisor_ip_address.add(len) = 0;
                    hypervisor_tcp_port = libc::atoi(index.add(1));
                }
            }

            // Log file
            x if x == b'l' as c_int => {
                log_file_name = libc::realloc(log_file_name.cast::<_>(), libc::strlen(c_optarg()) + 1).cast::<_>();
                if log_file_name.is_null() {
                    libc::fprintf(c_stderr(), cstr!("Unable to set log file name!\n"));
                    libc::exit(libc::EXIT_FAILURE);
                }
                libc::strcpy(log_file_name, c_optarg());
                libc::printf(cstr!("Log file: writing to %s\n"), log_file_name);
            }

            // VM file naming type
            x if x == b'N' as c_int => {
                vm_file_naming_type = libc::atoi(c_optarg());
            }

            // Load plugin (already handled)
            x if x == b'L' as c_int => {}

            OPT_NOCTRL => {
                vtty_set_ctrlhandler(0); // Ignore ctrl ]
                libc::printf(cstr!("Block ctrl+] access to monitor console.\n"));
            }

            OPT_NOTELMSG => {
                vtty_set_telnetmsg(0); // disable telnet greeting
                libc::printf(cstr!("Prevent telnet message on AUX/CONSOLE connect.\n"));
            }

            OPT_FILEPID => {
                pid_file = libc::fopen(c_optarg(), cstr!("w"));
                if !pid_file.is_null() {
                    libc::fprintf(pid_file, cstr!("%d"), libc::getpid());
                    libc::fclose(pid_file);
                } else {
                    libc::printf(cstr!("Unable to save to %s.\n"), c_optarg());
                }
            }

            // Global console (vtty tcp) binding address
            OPT_CONSOLE_BINDING_ADDR => {
                if !console_binding_addr.is_null() {
                    libc::free(console_binding_addr.cast::<_>());
                }
                console_binding_addr = libc::strdup(c_optarg());
                libc::printf(cstr!("Console binding address set to %s\n"), console_binding_addr);
            }

            // Oops !
            x if x == b'?' as c_int => {
                //show_usage(argc,argv,VM_TYPE_C7200);
                libc::exit(libc::EXIT_FAILURE);
            }

            _ => {}
        }
    }

    TRUE
}

/// Delete all objects
#[no_mangle]
pub unsafe extern "C" fn dynamips_reset() {
    libc::printf(cstr!("Shutdown in progress...\n"));

    // Delete all virtual router instances
    vm_delete_all_instances();

    // Delete ATM and Frame-Relay switches + bridges
    netio_bridge_delete_all();
    atmsw_delete_all();
    atm_bridge_delete_all();
    frsw_delete_all();
    ethsw_delete_all();

    // Delete all NIO descriptors
    netio_delete_all();

    m_log!(cstr!("GENERAL"), cstr!("reset done.\n"));

    libc::printf(cstr!("Shutdown completed.\n"));
}

/// Default platforms
#[rustfmt::skip]
#[cfg(not(feature = "USE_UNSTABLE"))]
pub static mut platform_register: [Option<unsafe extern "C" fn() -> c_int>; 10] = [
   Some(c7200_platform_register),
   Some(c3600_platform_register),
   Some(c3725_platform_register),
   Some(c3745_platform_register),
   Some(c2691_platform_register),
   Some(c2600_platform_register),
   Some(c1700_platform_register),
   Some(c6sup1_platform_register),
   Some(c6msfc1_platform_register),
   None,
];

/// Default platforms
#[rustfmt::skip]
#[cfg(feature = "USE_UNSTABLE")]
pub static mut platform_register: [Option<unsafe extern "C" fn() -> c_int>; 11] = [
   Some(c7200_platform_register),
   Some(c3600_platform_register),
   Some(c3725_platform_register),
   Some(c3745_platform_register),
   Some(c2691_platform_register),
   Some(c2600_platform_register),
   Some(c1700_platform_register),
   Some(c6sup1_platform_register),
   Some(c6msfc1_platform_register),
   Some(ppc32_vmtest_platform_register),
   None,
];

/// Register default platforms
unsafe fn register_default_platforms() {
    let mut i: c_int = 0;
    while platform_register[i as usize].is_some() {
        platform_register[i as usize].unwrap()();
        i += 1;
    }
}

/// Destroy variables generated from the standard command line
extern "C" fn destroy_cmd_line_vars() {
    unsafe {
        if !log_file_name.is_null() {
            libc::free(log_file_name.cast::<_>());
            log_file_name = null_mut();
        }
        if !hypervisor_ip_address.is_null() {
            libc::free(hypervisor_ip_address.cast::<_>());
            hypervisor_ip_address = null_mut();
        }
        if !console_binding_addr.is_null() {
            libc::free(console_binding_addr.cast::<_>());
            console_binding_addr = null_mut();
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn dynamips_main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let vm: *mut vm_instance_t;

    #[cfg(feature = "USE_PROFILER")]
    {
        libc::atexit(profiler_savestat);
    }

    if cfg!(feature = "USE_UNSTABLE") {
        libc::printf(cstr!("Cisco Router Simulation Platform (version %s/%s unstable)\n"), sw_version, os_name);
    } else {
        libc::printf(cstr!("Cisco Router Simulation Platform (version %s/%s stable)\n"), sw_version, os_name);
    }

    libc::printf(cstr!("Copyright (c) 2005-2011 Christophe Fillot.\n"));
    libc::printf(cstr!("Build date: %s %s\n\n"), cstr!(compile_time::date_str!()), cstr!(compile_time::time_str!()));

    // Register platforms
    register_default_platforms();

    // Initialize timers
    timer_init();

    // Initialize object registry
    registry_init();

    // Initialize ATM module (for HEC checksums)
    atm_init();

    // Initialize CRC functions
    crc_init();

    // Initialize NetIO code
    netio_rxl_init();

    // Initialize NetIO packet filters
    netio_filter_load_all();

    // Initialize VTTY code
    vtty_init();

    // Parse standard command line
    libc::atexit(destroy_cmd_line_vars);
    if run_hypervisor(argc, argv) == 0 {
        parse_std_cmd_line(argc, argv);
    }

    // Create general log file
    create_log_file();

    // Periodic tasks initialization
    if ptask_init(0) == -1 {
        libc::exit(libc::EXIT_FAILURE);
    }

    // Create instruction lookup tables
    mips64_jit_create_ilt();
    mips64_exec_create_ilt();
    ppc32_jit_create_ilt();
    ppc32_exec_create_ilt();

    setup_signals();

    if hypervisor_mode == 0 {
        // Initialize the default instance
        vm = vm_acquire(cstr!("default"));
        assert!(!vm.is_null());

        if vm_init_instance(vm) == -1 {
            libc::fprintf(c_stderr(), cstr!("Unable to initialize router instance.\n"));
            libc::exit(libc::EXIT_FAILURE);
        }

        if DEBUG_INSN_PERF_CNT > 0 || DEBUG_BLOCK_PERF_CNT > 0 {
            let mut counter: m_uint32_t;
            let mut prev: m_uint32_t = 0;
            let mut delta: m_uint32_t;
            #[allow(clippy::while_immutable_condition)]
            while (*vm).status == VM_STATUS_RUNNING {
                counter = cpu_get_perf_counter((*vm).boot_cpu);
                delta = counter - prev;
                prev = counter;
                libc::printf(cstr!("delta = %u\n"), delta);
                libc::sleep(1);
            }
        } else {
            // Start instance monitoring
            vm_monitor(vm);
        }

        // Free resources used by instance
        vm_release(vm);
    } else {
        hypervisor_tcp_server(hypervisor_ip_address, hypervisor_tcp_port);
    }

    dynamips_reset();
    close_log_file();
    0
}
