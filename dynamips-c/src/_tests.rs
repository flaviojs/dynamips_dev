//! Tests

mod dynamips_common {
    use crate::dynamips_common::*;

    #[repr(C)]
    struct S {
        field_u64: u64,
        field_u32: u32,
        field_u16: u16,
        field_u8: u8,
        arr: [u8; 5],
        inner: InnerS,
    }

    #[repr(C)]
    struct InnerS {
        field: u16,
        arr: [u8; 2],
    }

    #[test]
    fn test_common_types() {
        assert_eq!(size_of::<m_uint8_t>(), 1);
        assert_eq!(size_of::<m_int8_t>(), 1);

        assert_eq!(size_of::<m_uint16_t>(), 2);
        assert_eq!(size_of::<m_int16_t>(), 2);

        assert_eq!(size_of::<m_uint32_t>(), 4);
        assert_eq!(size_of::<m_int32_t>(), 4);

        assert_eq!(size_of::<m_uint64_t>(), 8);
        assert_eq!(size_of::<m_int64_t>(), 8);

        // must be able to store a pointer address
        assert!(size_of::<m_iptr_t>() >= size_of::<*mut u8>());
    }

    #[test]
    fn test_m_min() {
        assert_eq!(m_min!(0, 1), 0);
        assert_eq!(m_min!(1, 1), 1);
        assert_eq!(m_min!(2, 1), 1);
    }

    #[test]
    fn test_m_max() {
        assert_eq!(m_max!(0, 1), 1);
        assert_eq!(m_max!(1, 1), 1);
        assert_eq!(m_max!(2, 1), 2);
    }

    #[test]
    fn test_ptr_adjust() {
        let mut buf: [u8; 0x10] = [0; 0x10];
        let ptr: *mut u8 = buf.as_mut_ptr();
        unsafe {
            assert_eq!(PTR_ADJUST!(*mut u8, ptr, 0x00), ptr);

            *PTR_ADJUST!(*mut m_uint32_t, ptr, 0x00) = 0x12345678_u32.to_be();
            *PTR_ADJUST!(*mut m_uint16_t, ptr, 0x0c) = 0x9012_u16.to_be();
        }
        assert_eq!(buf, [0x12, 0x34, 0x56, 0x78, 0, 0, 0, 0, 0, 0, 0, 0, 0x90, 0x12, 0, 0]);
    }

    #[test]
    fn test_sizeof() {
        assert_eq!(SIZEOF!(S, field_u64), 8);
        assert_eq!(SIZEOF!(S, field_u32), 4);
        assert_eq!(SIZEOF!(S, field_u16), 2);
        assert_eq!(SIZEOF!(S, field_u8), 1);
        assert_eq!(SIZEOF!(S, arr[0]), 1);
        assert_eq!(SIZEOF!(S, arr[1]), 1);
        assert_eq!(SIZEOF!(S, inner.field), 2);
        assert_eq!(SIZEOF!(S, inner.arr[0]), 1);
        assert_eq!(SIZEOF!(S, inner.arr[1]), 1);
    }

    #[test]
    fn test_offset() {
        assert_eq!(OFFSET!(S, field_u64), 0);
        assert_eq!(OFFSET!(S, field_u32), 8);
        assert_eq!(OFFSET!(S, field_u16), 12);
        assert_eq!(OFFSET!(S, field_u8), 14);
        assert_eq!(OFFSET!(S, arr[0]), 15);
        assert_eq!(OFFSET!(S, arr[1]), 16);
        assert_eq!(OFFSET!(S, inner.field), 20);
        assert_eq!(OFFSET!(S, inner.arr[0]), 22);
        assert_eq!(OFFSET!(S, inner.arr[1]), 23);
    }
}

mod hash {
    use crate::_extra::*;
    use crate::hash::*;
    use std::ffi::c_int;
    use std::ffi::c_void;

    #[test]
    fn test_hash_table_create() {
        extern "C" fn x_hash(x: *mut c_void) -> u_int {
            x as usize as _
        }
        extern "C" fn x_equal(x1: *mut c_void, x2: *mut c_void) -> c_int {
            (x1 == x2) as _
        }
        const HASH_SIZE: c_int = 16;

        unsafe {
            let ht = hash_table_create(Some(x_hash), Some(x_equal), HASH_SIZE);
            assert!(!ht.is_null());
            hash_table_delete(ht);

            let ht = hash_string_create!(HASH_SIZE);
            assert!(!ht.is_null());
            hash_table_delete(ht);

            let ht = hash_int_create!(HASH_SIZE);
            assert!(!ht.is_null());
            hash_table_delete(ht);

            let ht = hash_u64_create!(HASH_SIZE);
            assert!(!ht.is_null());
            hash_table_delete(ht);

            let ht = hash_ptr_create!(HASH_SIZE);
            assert!(!ht.is_null());
            hash_table_delete(ht);
        }
    }

    #[test]
    fn test_macro_hash_table_foreach() {
        unsafe {
            const HASH_SIZE: c_int = 16;
            let ht = hash_int_create!(HASH_SIZE);
            assert!(!ht.is_null());
            for i in 0..64 {
                hash_table_insert(ht, i as _, i as _);
            }

            let mut found: u64 = 0;
            HASH_TABLE_FOREACH!(i, ht, hn, {
                assert!((*hn).key == (*hn).value);
                found |= 1 << ((*hn).key as usize);
            });
            assert_eq!(found, !0_u64);

            hash_table_delete(ht);
        }
    }
}

mod net {
    use crate::_extra::*;
    use crate::dynamips_common::*;
    use crate::net::*;
    use crate::utils::*;
    use std::ffi::c_int;
    use std::ffi::c_uint;

    // Partial checksum test
    #[test]
    fn test_ip_cksum_partial() {
        unsafe {
            const N_BUF: usize = 4;
            let mut buffer: [[m_uint8_t; 512]; N_BUF] = [[0; 512]; N_BUF];
            let mut psum: [m_uint16_t; N_BUF] = [0; N_BUF];
            let mut tmp: m_uint32_t;
            let mut sum: m_uint32_t;
            let gsum: m_uint32_t;

            for i in 0..N_BUF {
                m_randomize_block(buffer[i].as_mut_ptr(), size_of_val(&buffer[i]));
                if false {
                    mem_dump(c_stdout(), buffer[i].as_mut_ptr(), size_of_val(&buffer[i]) as u_int);
                }

                sum = ip_cksum_partial(buffer[i].as_mut_ptr(), size_of_val(&buffer[i]) as c_int);

                while (sum >> 16) != 0 {
                    sum = (sum & 0xFFFF) + (sum >> 16);
                }

                psum[i] = (!sum) as m_uint16_t;
            }

            // partial sums + accumulator
            tmp = 0;
            for i in 0..N_BUF {
                if false {
                    libc::printf(c"psum[%d] = 0x%4.4x\n".as_ptr(), i, psum[i] as c_uint);
                }
                tmp += !psum[i] as m_uint16_t as m_uint32_t;
            }

            // global sum
            sum = ip_cksum_partial(buffer.as_mut_ptr().cast::<m_uint8_t>(), size_of_val(&buffer) as c_int);

            while (sum >> 16) != 0 {
                sum = (sum & 0xFFFF) + (sum >> 16);
            }

            gsum = sum;

            // accumulator
            while (tmp >> 16) != 0 {
                tmp = (tmp & 0xFFFF) + (tmp >> 16);
            }

            if false {
                libc::printf(c"gsum = 0x%4.4x, tmp = 0x%4.4x : %s\n".as_ptr(), gsum, tmp, if gsum == tmp { c"OK".as_ptr() } else { c"FAILURE".as_ptr() });
            }

            assert_eq!(tmp, gsum);
        }
    }
}

mod parser {
    use crate::parser::*;
    use std::ffi::c_char;
    use std::ffi::c_int;
    use std::mem::zeroed;
    use std::ptr::addr_of_mut;
    use std::ptr::null_mut;

    #[test]
    fn test_parser() {
        unsafe {
            // Parser tests
            let parser_test_str: [*mut c_char; 8] = [
                c"c7200 show_hardware R1".as_ptr().cast_mut(),
                c"c7200 show_hardware \"R1\"".as_ptr().cast_mut(),
                c"   c7200    show_hardware   \"R1\"    ".as_ptr().cast_mut(),
                c"\"c7200\" \"show_hardware\" \"R1\"".as_ptr().cast_mut(),
                c"hypervisor set_working_dir \"C:\\Program Files\\Dynamips Test\"".as_ptr().cast_mut(),
                c"hypervisor # This is a comment set_working_dir \"C:\\Program Files\"".as_ptr().cast_mut(),
                c"\"c7200\" \"show_hardware\" \"R1".as_ptr().cast_mut(), // FIXME does not check if this test produces an error
                null_mut(),
            ];
            let mut ctx: parser_context_t = zeroed();
            let mut i: c_int;
            let mut res: c_int;

            i = 0;
            while !parser_test_str[i as usize].is_null() {
                parser_context_init(addr_of_mut!(ctx));

                res = parser_scan_buffer(addr_of_mut!(ctx), parser_test_str[i as usize], libc::strlen(parser_test_str[i as usize]) + 1);

                libc::printf(c"\n%d: Test string: [%s] => res=%d, state=%d\n".as_ptr(), i, parser_test_str[i as usize], res, ctx.state);

                if (res != 0) && (ctx.error == 0) {
                    if !ctx.tok_head.is_null() {
                        libc::printf(c"Tokens: ".as_ptr());
                        parser_dump_tokens(addr_of_mut!(ctx));
                        libc::printf(c"\n".as_ptr());
                    }
                }

                parser_context_free(addr_of_mut!(ctx));
                i += 1;
            }
        }
    }
}

mod utils {
    use crate::utils::*;
    use std::ffi::c_char;
    use std::ffi::c_int;
    use std::ffi::CStr;
    use std::ptr::addr_of_mut;
    use std::ptr::null_mut;

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct S {
        pub field: u8,
        pub list_next: *mut S,
        pub list_pprev: *mut *mut S,
    }
    impl Default for S {
        fn default() -> Self {
            Self { field: 0, list_next: null_mut(), list_pprev: null_mut() }
        }
    }

    fn _test_m_list<const N: usize, F: FnOnce([*mut S; N])>(f: F) {
        let mut arr: [S; N] = [S::default(); N];
        let p: [*mut S; N] = arr.each_mut().map(|r| addr_of_mut!(*r));
        f(p)
    }

    #[test]
    fn test_m_list_add() {
        _test_m_list(|[p1, p2, p3]| unsafe {
            let mut root: *mut S = null_mut();
            M_LIST_ADD!(p1, root, list);
            M_LIST_ADD!(p2, root, list);
            M_LIST_ADD!(p3, root, list);
            assert!(root == p3);
            assert!((*p3).list_next == p2);
            assert!((*p2).list_next == p1);
            assert!((*p1).list_next == null_mut());
            assert!((*p1).list_pprev == addr_of_mut!((*p2).list_next));
            assert!((*p2).list_pprev == addr_of_mut!((*p3).list_next));
            assert!((*p3).list_pprev == addr_of_mut!(root));
        });
    }

    #[test]
    fn test_m_list_remove_0() {
        _test_m_list(|[p1]| unsafe {
            M_LIST_REMOVE!(p1, list);
            assert!((*p1).list_next == null_mut());
            assert!((*p1).list_pprev == null_mut());
        });
    }

    #[test]
    fn test_m_list_remove_1() {
        _test_m_list(|[p1]| unsafe {
            let mut root: *mut S = null_mut();
            M_LIST_ADD!(p1, root, list);
            M_LIST_REMOVE!(p1, list);
            assert!(root == null_mut());
            assert!((*p1).list_next == null_mut());
            assert!((*p1).list_pprev == null_mut());
        });
    }

    #[test]
    fn test_m_list_remove_2_1() {
        _test_m_list(|[p1, p2]| unsafe {
            let mut root: *mut S = null_mut();
            M_LIST_ADD!(p1, root, list);
            M_LIST_ADD!(p2, root, list);
            M_LIST_REMOVE!(p1, list);
            assert!(root == p2);
            assert!((*p2).list_next == null_mut());
            assert!((*p1).list_next == null_mut());
            assert!((*p1).list_pprev == null_mut());
            assert!((*p2).list_pprev == addr_of_mut!(root));
        });
    }

    #[test]
    fn test_m_list_remove_2_2() {
        _test_m_list(|[p1, p2]| unsafe {
            let mut root: *mut S = null_mut();
            M_LIST_ADD!(p1, root, list);
            M_LIST_ADD!(p2, root, list);
            M_LIST_REMOVE!(p2, list);
            assert!(root == p1);
            assert!((*p2).list_next == null_mut());
            assert!((*p1).list_next == null_mut());
            assert!((*p1).list_pprev == addr_of_mut!(root));
            assert!((*p2).list_pprev == null_mut());
        });
    }

    #[test]
    fn test_m_list_remove_3_1() {
        _test_m_list(|[p1, p2, p3]| unsafe {
            let mut root: *mut S = null_mut();
            M_LIST_ADD!(p1, root, list);
            M_LIST_ADD!(p2, root, list);
            M_LIST_ADD!(p3, root, list);
            M_LIST_REMOVE!(p1, list);
            assert!(root == p3);
            assert!((*p3).list_next == p2);
            assert!((*p2).list_next == null_mut());
            assert!((*p1).list_next == null_mut());
            assert!((*p1).list_pprev == null_mut());
            assert!((*p2).list_pprev == addr_of_mut!((*p3).list_next));
            assert!((*p3).list_pprev == addr_of_mut!(root));
        });
    }

    #[test]
    fn test_m_list_remove_3_2() {
        _test_m_list(|[p1, p2, p3]| unsafe {
            let mut root: *mut S = null_mut();
            M_LIST_ADD!(p1, root, list);
            M_LIST_ADD!(p2, root, list);
            M_LIST_ADD!(p3, root, list);
            M_LIST_REMOVE!(p2, list);
            assert!(root == p3);
            assert!((*p3).list_next == p1);
            assert!((*p2).list_next == null_mut());
            assert!((*p1).list_next == null_mut());
            assert!((*p1).list_pprev == addr_of_mut!((*p3).list_next));
            assert!((*p2).list_pprev == null_mut());
            assert!((*p3).list_pprev == addr_of_mut!(root));
        });
    }

    #[test]
    fn test_m_list_remove_3_3() {
        _test_m_list(|[p1, p2, p3]| unsafe {
            let mut root: *mut S = null_mut();
            M_LIST_ADD!(p1, root, list);
            M_LIST_ADD!(p2, root, list);
            M_LIST_ADD!(p3, root, list);
            M_LIST_REMOVE!(p3, list);
            assert!(root == p2);
            assert!((*p3).list_next == null_mut());
            assert!((*p2).list_next == p1);
            assert!((*p1).list_next == null_mut());
            assert!((*p1).list_pprev == addr_of_mut!((*p2).list_next));
            assert!((*p2).list_pprev == addr_of_mut!(root));
            assert!((*p3).list_pprev == null_mut());
        });
    }

    #[test]
    fn test_dyn_sprintf() {
        macro_rules! _assert_eq {
            ($code:expr, $expected:expr) => {{
                let p: *mut c_char = $code;
                if p.is_null() {
                    panic!("{} is null", stringify!($code));
                } else {
                    assert_eq!(CStr::from_ptr(p), $expected);
                    libc::free(p.cast::<_>());
                }
            }};
        }
        unsafe {
            _assert_eq!(dyn_sprintf!("no args"), c"no args");
            _assert_eq!(dyn_sprintf!("%d int", 1), c"1 int");
            _assert_eq!(dyn_sprintf!("%d %d int", 1, 2), c"1 2 int");
            _assert_eq!(dyn_sprintf!("%d %d %d int", 1, 2, 3), c"1 2 3 int");
            _assert_eq!(dyn_sprintf!("%s", "str"), c"str");
            _assert_eq!(dyn_sprintf!("%s", c"CStr"), c"CStr");
            _assert_eq!(dyn_sprintf!("%s", c"CString".to_owned()), c"CString");
            _assert_eq!(dyn_sprintf!("%s", c"c_str_ptr".as_ptr()), c"c_str_ptr");
        }
    }

    #[test]
    fn test_m_log() {
        let mut buffer: [c_char; 100] = [0; 100];
        unsafe {
            crate::utils::log_file = libc::fmemopen(buffer.as_mut_ptr().cast::<_>(), buffer.len(), c"w".as_ptr());

            m_log!(c"module".as_ptr(), c"no args\n".as_ptr());
            assert!(CStr::from_ptr(buffer.as_ptr()).to_str().expect("str").ends_with("module: no args\n"));

            m_log!(c"x".as_ptr(), c"%d %s args\n".as_ptr(), 1, c"two".as_ptr());
            assert!(CStr::from_ptr(buffer.as_ptr()).to_str().expect("str").ends_with("x: 1 two args\n"));

            libc::fclose(crate::utils::log_file);
            crate::utils::log_file = null_mut();
        }
    }

    #[test]
    fn test_fd_printf() {
        unsafe {
            let mut fds: [c_int; 2] = [-1; 2];
            assert_eq!(libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()), 0);

            let mut buffer: [c_char; 100] = [0; 100];
            assert_eq!(fd_printf!(fds[0], 0, "no args\n"), 8);
            assert_eq!(libc::read(fds[1], buffer.as_mut_ptr().cast::<_>(), buffer.len()), 8);
            assert!(CStr::from_ptr(buffer.as_ptr()).to_str().expect("str").ends_with("no args\n"));

            let mut buffer: [c_char; 100] = [0; 100];
            assert_eq!(fd_printf!(fds[0], 0, "%d %s args\n", 1, c"two".as_ptr()), 11);
            assert_eq!(libc::read(fds[1], buffer.as_mut_ptr().cast::<_>(), buffer.len()), 11);
            assert!(CStr::from_ptr(buffer.as_ptr()).to_str().expect("str").ends_with("1 two args\n"));

            libc::close(fds[0]);
            libc::close(fds[1]);
        }
    }
}

mod x86_codegen {
    use crate::_extra::*;
    use crate::x86_codegen::*;
    use std::ffi::c_uchar;

    #[test]
    fn test_macro_code() {
        // macro code is only checked when the macro is used
        unsafe {
            let mut buf: [c_uchar; 16] = [0; 16];
            let _ = X86_IS_SCRATCH!(X86_EAX);
            let _ = X86_IS_CALLEE!(X86_EAX);
            let _ = X86_IS_BYTE_REG!(X86_EAX);
            let _ = x86_modrm_mod!(0);
            let _ = x86_modrm_reg!(0);
            let _ = x86_modrm_rm!(0);
            let mut inst = buf.as_mut_ptr();
            x86_address_byte!(&mut inst, 0, 0, 0);
            let mut inst = buf.as_mut_ptr();
            x86_imm_emit32!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_imm_emit16!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_imm_emit8!(&mut inst, 0);
            let _ = x86_is_imm8!(0);
            let _ = x86_is_imm16!(0);
            let mut inst = buf.as_mut_ptr();
            x86_reg_emit!(&mut inst, X86_EAX, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_reg8_emit!(&mut inst, X86_EAX, X86_EAX, false, false);
            let mut inst = buf.as_mut_ptr();
            x86_regp_emit!(&mut inst, X86_EAX, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_mem_emit!(&mut inst, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_membase_emit!(&mut inst, X86_EAX, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_memindex_emit!(&mut inst, X86_EAX, X86_EAX, 0, X86_EAX, 0);
            buf[0] = 0xeb; // jump8
            x86_patch!(buf.as_mut_ptr(), buf.as_mut_ptr());
            let mut inst = buf.as_mut_ptr();
            x86_breakpoint!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_clc!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_cld!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_stosb!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_stosl!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_stosd!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_movsb!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_movsl!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_movsd!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_prefix!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_bswap!(&mut inst, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_rdtsc!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_cmpxchg_reg_reg!(&mut inst, X86_EAX, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_cmpxchg_mem_reg!(&mut inst, 0, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_cmpxchg_membase_reg!(&mut inst, X86_EAX, 0, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_xchg_reg_reg!(&mut inst, X86_EAX, X86_EAX, 4);
            let mut inst = buf.as_mut_ptr();
            x86_xchg_mem_reg!(&mut inst, 0, X86_EAX, 4);
            let mut inst = buf.as_mut_ptr();
            x86_xchg_membase_reg!(&mut inst, X86_EAX, 0, X86_EAX, 4);
            let mut inst = buf.as_mut_ptr();
            x86_xadd_reg_reg!(&mut inst, X86_EAX, X86_EAX, 4);
            let mut inst = buf.as_mut_ptr();
            x86_xadd_mem_reg!(&mut inst, 0, X86_EAX, 4);
            let mut inst = buf.as_mut_ptr();
            x86_xadd_membase_reg!(&mut inst, X86_EAX, 0, X86_EAX, 4);
            let mut inst = buf.as_mut_ptr();
            x86_inc_mem!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_inc_membase!(&mut inst, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_inc_reg!(&mut inst, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_dec_mem!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_dec_membase!(&mut inst, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_dec_reg!(&mut inst, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_not_mem!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_not_membase!(&mut inst, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_not_reg!(&mut inst, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_neg_mem!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_neg_reg!(&mut inst, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_nop!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_alu_reg_imm!(&mut inst, 0, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_alu_mem_imm!(&mut inst, 0, 0, 0);
            let mut inst = buf.as_mut_ptr();
            x86_alu_membase_imm!(&mut inst, 0, X86_EAX, 0, 0);
            let mut inst = buf.as_mut_ptr();
            x86_alu_membase8_imm!(&mut inst, 0, X86_EAX, 0, 0);
            let mut inst = buf.as_mut_ptr();
            x86_alu_mem_reg!(&mut inst, 0, 0, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_alu_membase_reg!(&mut inst, 0, X86_EAX, 0, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_alu_reg_reg!(&mut inst, 0, X86_EAX, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_alu_reg8_reg8!(&mut inst, 0, X86_EAX, X86_EAX, false, false);
            let mut inst = buf.as_mut_ptr();
            x86_alu_reg_mem!(&mut inst, 0, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_alu_reg_membase!(&mut inst, 0, X86_EAX, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_test_reg_imm!(&mut inst, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_test_mem_imm!(&mut inst, 0, 0);
            let mut inst = buf.as_mut_ptr();
            x86_test_membase_imm!(&mut inst, X86_EAX, 0, 0);
            let mut inst = buf.as_mut_ptr();
            x86_test_reg_reg!(&mut inst, X86_EAX, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_test_mem_reg!(&mut inst, 0, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_test_membase_reg!(&mut inst, X86_EAX, 0, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_shift_reg_imm!(&mut inst, 0, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_shift_mem_imm!(&mut inst, 0, 0, 0);
            let mut inst = buf.as_mut_ptr();
            x86_shift_membase_imm!(&mut inst, 0, X86_EAX, 0, 0);
            let mut inst = buf.as_mut_ptr();
            x86_shift_reg!(&mut inst, 0, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_shift_mem!(&mut inst, 0, 0);
            let mut inst = buf.as_mut_ptr();
            x86_shift_membase!(&mut inst, 0, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_shrd_reg!(&mut inst, X86_EAX, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_shrd_reg_imm!(&mut inst, 0, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_shld_reg!(&mut inst, X86_EAX, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_shld_reg_imm!(&mut inst, X86_EAX, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_mul_reg!(&mut inst, X86_EAX, false);
            let mut inst = buf.as_mut_ptr();
            x86_mul_mem!(&mut inst, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_mul_membase!(&mut inst, X86_EAX, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_imul_reg_reg!(&mut inst, X86_EAX, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_imul_reg_mem!(&mut inst, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_imul_reg_membase!(&mut inst, X86_EAX, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_imul_reg_reg_imm!(&mut inst, X86_EAX, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_imul_reg_mem_imm!(&mut inst, X86_EAX, 0, 0);
            let mut inst = buf.as_mut_ptr();
            x86_imul_reg_membase_imm!(&mut inst, X86_EAX, X86_EAX, 0, 0);
            let mut inst = buf.as_mut_ptr();
            x86_div_reg!(&mut inst, X86_EAX, false);
            let mut inst = buf.as_mut_ptr();
            x86_div_mem!(&mut inst, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_div_membase!(&mut inst, X86_EAX, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_mov_mem_reg!(&mut inst, 0, X86_EAX, 4);
            let mut inst = buf.as_mut_ptr();
            x86_mov_regp_reg!(&mut inst, X86_EAX, X86_EAX, 4);
            let mut inst = buf.as_mut_ptr();
            x86_mov_membase_reg!(&mut inst, X86_EAX, 0, X86_EAX, 4);
            let mut inst = buf.as_mut_ptr();
            x86_mov_memindex_reg!(&mut inst, X86_EAX, 0, X86_EAX, 0, X86_EAX, 4);
            let mut inst = buf.as_mut_ptr();
            x86_mov_reg_reg!(&mut inst, X86_EAX, X86_EAX, 4);
            let mut inst = buf.as_mut_ptr();
            x86_mov_reg_membase!(&mut inst, X86_EAX, X86_EAX, 0, 4);
            let mut inst = buf.as_mut_ptr();
            x86_mov_reg_memindex!(&mut inst, X86_EAX, X86_EAX, 0, X86_EAX, 0, 4);
            let mut inst = buf.as_mut_ptr();
            x86_clear_reg!(&mut inst, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_mov_reg_imm!(&mut inst, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_mov_mem_imm!(&mut inst, 0, 0, 4);
            let mut inst = buf.as_mut_ptr();
            x86_mov_membase_imm!(&mut inst, X86_EAX, 0, 0, 4);
            let mut inst = buf.as_mut_ptr();
            x86_mov_memindex_imm!(&mut inst, X86_EAX, 0, X86_EAX, 0, 0, 4);
            let mut inst = buf.as_mut_ptr();
            x86_lea_mem!(&mut inst, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_lea_membase!(&mut inst, X86_EAX, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_lea_memindex!(&mut inst, X86_EAX, X86_EAX, 0, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_widen_reg!(&mut inst, X86_EAX, X86_EAX, false, false);
            let mut inst = buf.as_mut_ptr();
            x86_widen_mem!(&mut inst, X86_EAX, 0, false, false);
            let mut inst = buf.as_mut_ptr();
            x86_widen_membase!(&mut inst, X86_EAX, X86_EAX, 0, false, false);
            let mut inst = buf.as_mut_ptr();
            x86_widen_memindex!(&mut inst, X86_EAX, X86_EAX, 0, X86_EAX, 0, false, false);
            let mut inst = buf.as_mut_ptr();
            x86_lahf!(&mut inst);
            //let mut inst = buf.as_mut_ptr();
            //x86_sahf!(&mut inst); // FIXME overridden later
            let mut inst = buf.as_mut_ptr();
            x86_xchg_ah_al!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_cdq!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_wait!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_fp_op_mem!(&mut inst, 0, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_fp_op_membase!(&mut inst, 0, X86_EAX, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_fp_op!(&mut inst, 0, 0);
            let mut inst = buf.as_mut_ptr();
            x86_fp_op_reg!(&mut inst, 0, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_fp_int_op_membase!(&mut inst, 0, X86_EAX, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_fstp!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_fcompp!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_fucompp!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_fnstsw!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_fnstcw!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_fnstcw_membase!(&mut inst, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_fldcw!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_fldcw_membase!(&mut inst, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_fchs!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_frem!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_fxch!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_fcomi!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_fcomip!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_fucomip!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_fld!(&mut inst, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_fld_membase!(&mut inst, X86_EAX, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_fld80_mem!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_fld80_membase!(&mut inst, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_fild!(&mut inst, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_fild_membase!(&mut inst, X86_EAX, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_fld_reg!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_fldz!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_fld1!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_fldpi!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_fst!(&mut inst, 0, false, false);
            let mut inst = buf.as_mut_ptr();
            x86_fst_membase!(&mut inst, X86_EAX, 0, false, false);
            let mut inst = buf.as_mut_ptr();
            x86_fst80_mem!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_fst80_membase!(&mut inst, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_fist_pop!(&mut inst, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_fist_pop_membase!(&mut inst, X86_EAX, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_fstsw!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_fist_membase!(&mut inst, X86_EAX, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_push_reg!(&mut inst, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_push_regp!(&mut inst, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_push_mem!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_push_membase!(&mut inst, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_push_memindex!(&mut inst, X86_EAX, 0, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_push_imm_template!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_pop_reg!(&mut inst, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_pop_mem!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_pop_membase!(&mut inst, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_pushad!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_pushfd!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_popad!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_popfd!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_loop!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_loope!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_loopne!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_jump32!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_jump8!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_jump_reg!(&mut inst, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_jump_mem!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_jump_membase!(&mut inst, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_jump_code!(&mut inst, buf.as_mut_ptr());
            let mut inst = buf.as_mut_ptr();
            x86_jump_disp!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_branch8!(&mut inst, X86_CC_Z, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_branch32!(&mut inst, X86_CC_Z, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_branch!(&mut inst, X86_CC_Z, buf.as_mut_ptr(), false);
            let mut inst = buf.as_mut_ptr();
            x86_branch_disp!(&mut inst, X86_CC_Z, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_set_reg!(&mut inst, X86_CC_Z, X86_EAX, false);
            let mut inst = buf.as_mut_ptr();
            x86_set_mem!(&mut inst, X86_CC_Z, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_set_membase!(&mut inst, X86_CC_Z, X86_EAX, 0, false);
            let mut inst = buf.as_mut_ptr();
            x86_call_imm!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_call_reg!(&mut inst, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_call_mem!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_call_membase!(&mut inst, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_call_code!(&mut inst, buf.as_mut_ptr());
            let mut inst = buf.as_mut_ptr();
            x86_ret!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_ret_imm!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_cmov_reg!(&mut inst, X86_CC_Z, false, X86_EAX, X86_EAX);
            let mut inst = buf.as_mut_ptr();
            x86_cmov_mem!(&mut inst, X86_CC_Z, false, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_cmov_membase!(&mut inst, X86_CC_Z, false, X86_EAX, X86_EAX, 0);
            let mut inst = buf.as_mut_ptr();
            x86_enter!(&mut inst, 0);
            let mut inst = buf.as_mut_ptr();
            x86_leave!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_sahf!(&mut inst); // FIXME overrides earlier version
            let mut inst = buf.as_mut_ptr();
            x86_fsin!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_fcos!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_fabs!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_ftst!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_fxam!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_fpatan!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_fprem!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_fprem1!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_frndint!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_fsqrt!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_fptan!(&mut inst);
            let mut inst = buf.as_mut_ptr();
            x86_padding!(&mut inst, 1);
            let mut inst = buf.as_mut_ptr();
            x86_prolog!(&mut inst, 0, 0);
            let mut inst = buf.as_mut_ptr();
            x86_epilog!(&mut inst, 0);
        }
    }
}
