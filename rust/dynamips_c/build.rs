//! Build script for the dynamips_c crate.
use std::process::Command;
use std::process::Output;

fn main() {
    cc::Build::new().file("src/_ext.c").compile("dynamips_c_ext");

    // TODO how to get around rustc_private with autocfg <https://github.com/cuviper/autocfg/issues/60>
    // FIXME temporary solution: rust-script compiles for and runs in the local system? I want to test-compile for the target system or similar

    // does rust-script work? (warning: outputs to stdout/stderr)
    let status = Command::new("rust-script").arg("--version").status().expect("rust-script is required, running 'cargo install rust-script' in the console should fix this error\n");
    if !status.success() {
        panic!("rust-script: {}", status);
    }
    // auxiliary functions similiar to autocfg
    fn probe_dep_expr(dep: &str, expr: &str) -> Output {
        Command::new("rust-script").args(["--dep", dep, "--expr", expr]).output().unwrap()
    }
    fn emit_dep_expr_cfg(dep: &str, expr: &str, cfg: &str) {
        if probe_dep_expr(dep, expr).status.success() {
            autocfg::emit(cfg);
        }
    }
    // sanity check: libc::size_t should always exist
    assert!(probe_dep_expr("libc", "{ use libc::size_t; }").status.success());
    emit_dep_expr_cfg("libc", "{ fn f(x: libc::sockaddr_in6) { let _ = x.sin6_len; } }", "has_libc_sockaddr_in6_sin6_len");
    emit_dep_expr_cfg("libc", "{ fn f(x: libc::tm) { let _ = x.tm_gmtoff; } }", "has_libc_tm_tm_gmtoff");
    emit_dep_expr_cfg("libc", "{ use libc::cfmakeraw; }", "has_libc_cfmakeraw");
    emit_dep_expr_cfg("libc", "{ use libc::B76800; }", "has_libc_B76800");
    emit_dep_expr_cfg("libc", "{ use libc::B230400; }", "has_libc_B230400");
    emit_dep_expr_cfg("libc", "{ use libc::CRTSCTS; }", "has_libc_CRTSCTS");
    emit_dep_expr_cfg("libc", "{ use libc::CNEW_RTSCTS; }", "has_libc_CNEW_RTSCTS");
    emit_dep_expr_cfg("libc", "{ use libc::IPV6_V6ONLY; }", "has_libc_IPV6_V6ONLY");
}
