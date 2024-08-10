//! Build script for the dynamips_c crate.

fn compile_private_c() {
    println!("cargo::rerun-if-changed=src/_private.c");
    cc::Build::new().file("src/_private.c").compile("dynamips_c_private");
}

fn probe_and_emit_config() {
    use autocfg::emit;
    use std::process::Command;
    use std::process::Output;

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
    assert!(probe_dep_expr("libc", "{ use libc::size_t; }").status.success()); // sanity check: libc::size_t should always exist
    fn emit_dep_expr_cfg(dep: &str, expr: &str, cfg: &str) {
        println!("cargo::rustc-check-cfg=cfg({})", cfg);
        if probe_dep_expr(dep, expr).status.success() {
            emit(cfg);
        }
    }

    // cfg for code that is no longer compatible
    println!("cargo::rustc-check-cfg=cfg(if_0)");

    // cfg's for stuff that might not exist
    emit_dep_expr_cfg("libc", "{ fn f(x: libc::sigaction) { let _ = x.sa_handler; } }", "has_libc_sigaction_sa_handler");
    emit_dep_expr_cfg("libc", "{ fn f(x: libc::sockaddr_in6) { let _ = x.sin6_len; } }", "has_libc_sockaddr_in6_sin6_len");
    emit_dep_expr_cfg("libc", "{ fn f(x: libc::tm) { let _ = x.tm_gmtoff; } }", "has_libc_tm_tm_gmtoff");
    emit_dep_expr_cfg("libc", "{ use libc::B230400; }", "has_libc_B230400");
    emit_dep_expr_cfg("libc", "{ use libc::B76800; }", "has_libc_B76800");
    emit_dep_expr_cfg("libc", "{ use libc::BIOCFEEDBACK; }", "has_libc_BIOCFEEDBACK");
    emit_dep_expr_cfg("libc", "{ use libc::cfmakeraw; }", "has_libc_cfmakeraw");
    emit_dep_expr_cfg("libc", "{ use libc::CNEW_RTSCTS; }", "has_libc_CNEW_RTSCTS");
    emit_dep_expr_cfg("libc", "{ use libc::CRTSCTS; }", "has_libc_CRTSCTS");
    emit_dep_expr_cfg("libc", "{ use libc::IPV6_V6ONLY; }", "has_libc_IPV6_V6ONLY");
    emit_dep_expr_cfg("libc", "{ use libc::memalign; }", "has_libc_memalign");
    emit_dep_expr_cfg("libc", "{ use libc::posix_memalign; }", "has_libc_posix_memalign");
}

#[allow(dead_code)]
fn generate_private_pcap_rs() {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let c_path = out_path.join("_private_pcap.h");
    let rs_path = out_path.join("_private_pcap.rs");

    let c_code = "#include <pcap.h>\n";
    let same_c_code: bool = fs::read(&c_path).map(|x| x == c_code.as_bytes()).unwrap_or(false);
    if !same_c_code {
        fs::write(&c_path, c_code).expect("Failed to write _private_pcap.h.");
    }

    // TODO support which non-standard include paths?
    let rs_bindings = bindgen::builder()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .header(c_path.to_str().unwrap())
        .ctypes_prefix("libc")
        .blocklist_type("FILE")
        .blocklist_type("sockaddr")
        .blocklist_type("timeval")
        .raw_line("use libc::FILE;")
        .raw_line("#[cfg(unix)] pub use libc::{sockaddr, timeval};")
        .raw_line("#[cfg(windows)] pub use winapi::shared::ws2def::SOCKADDR as sockaddr;")
        .raw_line("#[cfg(windows)] pub use winapi::um::winsock2::timeval;")
        .allowlist_item("^pcap_.*")
        .allowlist_item("^PCAP_.*")
        .allowlist_item("^DLT_.*")
        .trust_clang_mangling(false)
        .layout_tests(false)
        .derive_copy(true)
        .derive_debug(true)
        .generate()
        .expect("Failed to generate bindings.");
    rs_bindings.write_to_file(&rs_path).expect("Failed to write _private_pcap.rs.");
}

fn main() {
    compile_private_c();
    probe_and_emit_config();
    #[cfg(feature = "ENABLE_GEN_ETH")]
    generate_private_pcap_rs();
}