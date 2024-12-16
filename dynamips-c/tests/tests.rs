//! Tests that only use public symbols.

#[test]
fn test_common_types() {
    use dynamips_c::dynamips_common::*;

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
