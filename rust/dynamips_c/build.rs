//! Build script for the dynamips_c crate.

fn main() {
    cc::Build::new().file("src/_ext.c").compile("dynamips_c_ext");
}
