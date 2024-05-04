//! Tests of the dynamips_c crate.

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
