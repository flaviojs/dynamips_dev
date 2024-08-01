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
