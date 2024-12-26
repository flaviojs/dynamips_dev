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
