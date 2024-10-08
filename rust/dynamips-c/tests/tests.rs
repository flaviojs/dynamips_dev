//! Tests of the dynamips_c crate.
#![allow(non_snake_case)]

#[test]
fn test_str0() {
    use dynamips_c::_private::*;
    let want = "test\0";
    let have = str0!("test");
    assert_eq!(want, have);
}

#[test]
fn test_cstr() {
    use dynamips_c::_private::*;
    let want = "test\0".as_ptr().cast::<c_char>().cast_mut();
    let have = cstr!("test");
    assert_eq!(unsafe { libc::strcmp(want, have) }, 0);
}

#[test]
fn test_common_type_sizes() {
    use dynamips_c::dynamips_common::*;
    assert_eq!(size_of::<m_uint8_t>(), 1);
    assert_eq!(size_of::<m_int8_t>(), 1);

    assert_eq!(size_of::<m_uint16_t>(), 2);
    assert_eq!(size_of::<m_int16_t>(), 2);

    assert_eq!(size_of::<m_uint32_t>(), 4);
    assert_eq!(size_of::<m_int32_t>(), 4);

    assert_eq!(size_of::<m_uint64_t>(), 8);
    assert_eq!(size_of::<m_int64_t>(), 8);
}

#[test]
fn test_PTR_ADJUST() {
    use dynamips_c::dynamips_common::*;
    let buf: [u8; 256] = [0; 256];

    let p: *mut u32 = unsafe { PTR_ADJUST!(*mut u32, buf.as_ptr(), 4) };
    assert_eq!(p as usize, buf.as_ptr() as usize + 4);

    let p: *mut u32 = unsafe { PTR_ADJUST!(*mut u32, buf.as_ptr(), 32) };
    assert_eq!(p as usize, buf.as_ptr() as usize + 32);
}

#[test]
fn test_SIZEOF() {
    use dynamips_c::dynamips_common::*;
    #[repr(C)]
    struct S {
        size1: u8,
        size2: u16,
        size4: u32,
        size8: u64,
    }
    assert_eq!(SIZEOF!(S, size1), 1);
    assert_eq!(SIZEOF!(S, size2), 2);
    assert_eq!(SIZEOF!(S, size4), 4);
    assert_eq!(SIZEOF!(S, size8), 8);
}

#[test]
fn test_OFFSET() {
    use dynamips_c::dynamips_common::*;
    #[repr(C)]
    struct S {
        offset0: u8,
        offset2: u16,
        offset4: u32,
        offset8: u64,
    }
    assert_eq!(OFFSET!(S, offset0), 0);
    assert_eq!(OFFSET!(S, offset2), 2);
    assert_eq!(OFFSET!(S, offset4), 4);
    assert_eq!(OFFSET!(S, offset8), 8);
}

#[cfg(feature = "USE_UNSTABLE")]
#[test]
fn test_M_LIST() {
    use dynamips_c::_private::*;
    use dynamips_c::utils::*;
    unsafe {
        #[derive(Debug, Copy, Clone, PartialEq)]
        struct S {
            linked_list_next: *mut S,
            linked_list_pprev: *mut *mut S,
        }
        impl Default for S {
            fn default() -> S {
                S { linked_list_next: null_mut(), linked_list_pprev: null_mut() }
            }
        }
        let mut head: *mut S = null_mut();
        let mut s1_data: S = S::default();
        let mut s2_data: S = S::default();
        let mut s3_data: S = S::default();
        let s1: *mut S = addr_of_mut!(s1_data);
        let s2: *mut S = addr_of_mut!(s2_data);
        let s3: *mut S = addr_of_mut!(s3_data);

        M_LIST_ADD!(s1, head, linked_list);
        M_LIST_ADD!(s2, head, linked_list);
        M_LIST_ADD!(s3, head, linked_list);
        assert_eq!(head, s3);
        assert_eq!(s3_data.linked_list_next, s2);
        assert_eq!(s2_data.linked_list_next, s1);
        assert_eq!(s1_data.linked_list_next, null_mut());
        assert_eq!(s1_data.linked_list_pprev, addr_of_mut!(s2_data.linked_list_next));
        assert_eq!(s2_data.linked_list_pprev, addr_of_mut!(s3_data.linked_list_next));
        assert_eq!(s3_data.linked_list_pprev, addr_of_mut!(head));

        M_LIST_REMOVE!(s2, linked_list); // remove middle
        M_LIST_REMOVE!(s3, linked_list); // remove first
        M_LIST_REMOVE!(s1, linked_list); // remove last
        assert_eq!(head, null_mut());
        assert_eq!(s1_data, S::default());
        assert_eq!(s2_data, S::default());
        assert_eq!(s3_data, S::default());
    }
}

#[test]
fn test_N_ETH_HLEN() {
    use dynamips_c::net::*;
    // cbindgen 0.27.0 does not support size_of
    assert_eq!(N_ETH_HLEN, size_of::<n_eth_hdr_t>());
}

#[test]
fn test_N_ISL_HDR_SIZE() {
    use dynamips_c::net::*;
    // cbindgen 0.27.0 does not support size_of
    assert_eq!(N_ISL_HDR_SIZE, size_of::<n_eth_llc_hdr_t>() + size_of::<n_eth_isl_hdr_t>());
}

#[test]
fn test_memblock_roundtrip() {
    use dynamips_c::_private::*;
    use dynamips_c::mempool::*;
    unsafe {
        let mut memory = Box::new(zeroed::<memblock_t>());
        let block: *mut memblock_t = addr_of_mut!(*memory);

        // memblock to addr (mp_alloc_inline)
        let addr: *mut c_void = (*block).data.as_c_void_mut();

        // addr to memblock (mp_realloc)
        let roundtrip_block: *mut memblock_t = addr.cast::<memblock_t>().sub(1);

        assert!(block == roundtrip_block);
    }
}

#[test]
fn test_HASH_TABLE_FOREACH() {
    use dynamips_c::_private::*;
    use dynamips_c::hash::*;
    unsafe {
        let mut i: c_int;
        let mut hn: *mut hash_node_t;

        let ht: *mut hash_table_t = hash_u64_create(11);
        let mut id: u64 = 1234;
        hash_table_insert(ht, addr_of_mut!(id).cast::<_>(), null_mut());
        HASH_TABLE_FOREACH!(i, ht, hn, {
            println!("{:?} {:?}", i, hn);
        });
        hash_table_delete(ht);
    }
}

#[test]
fn test_parser() {
    use dynamips_c::_private::*;
    use dynamips_c::parser::*;
    unsafe {
        // Parser tests
        struct Test {
            buf: *mut c_char,
            error: c_int,
        }
        impl Test {
            fn new(buf: *mut c_char, error: c_int) -> Self {
                Self { buf, error }
            }
        }
        let parser_test: [Test; 8] = [
            Test::new(cstr!("c7200 show_hardware R1"), 0),
            Test::new(cstr!("c7200 show_hardware \"R1\""), 0),
            Test::new(cstr!("   c7200    show_hardware   \"R1\"    "), 0),
            Test::new(cstr!("\"c7200\" \"show_hardware\" \"R1\""), 0),
            Test::new(cstr!("hypervisor set_working_dir \"C:\\Program Files\\Dynamips Test\""), 0),
            Test::new(cstr!("hypervisor # This is a comment set_working_dir \"C:\\Program Files\""), 0),
            Test::new(cstr!("\"c7200\" \"show_hardware\" \"R1"), PARSER_ERROR_UNEXP_EOL),
            Test::new(null_mut(), 0),
        ];

        let mut ctx: parser_context_t = zeroed::<_>();
        let mut i = 0;
        while !parser_test[i].buf.is_null() {
            parser_context_init(addr_of_mut!(ctx));

            let res: c_int = parser_scan_buffer(addr_of_mut!(ctx), parser_test[i].buf, libc::strlen(parser_test[i].buf) + 1);

            libc::printf(cstr!("\n%d: Test string: [%s] => res=%d, state=%d\n"), i as c_int, parser_test[i].buf, res, ctx.state);

            if res != 0 && ctx.error == 0 {
                if !ctx.tok_head.is_null() {
                    libc::printf(cstr!("Tokens: "));
                    parser_dump_tokens(addr_of_mut!(ctx));
                    libc::printf(cstr!("\n"));
                }
            }

            assert_eq!(res, 1);
            assert_eq!(ctx.state, PARSER_STATE_DONE);
            assert_eq!(ctx.error, parser_test[i].error);

            parser_context_free(addr_of_mut!(ctx));
            i += 1;
        }
    }
}

#[test]
pub fn test_MIPS_INSN_PER_PAGE() {
    use dynamips_c::mips64::*;
    use dynamips_c::utils::*;

    // cbindgen 0.26.0 does not support size_of
    assert_eq!(size_of::<mips_insn_t>(), 4);
    assert_eq!(MIPS_INSN_PER_PAGE, MIPS_MIN_PAGE_SIZE / size_of::<mips_insn_t>());
}

#[test]
fn test_PPC32_INSN_PER_PAGE() {
    use dynamips_c::ppc32::*;
    use dynamips_c::utils::*;

    // cbindgen 0.26.0 does not support size_of
    assert_eq!(size_of::<ppc_insn_t>(), 4);
    assert_eq!(PPC32_INSN_PER_PAGE, PPC32_MIN_PAGE_SIZE / size_of::<ppc_insn_t>());
}
