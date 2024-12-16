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
