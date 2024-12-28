//! build script for dynamips_c

use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

trait AutoCfgExt {
    /// Probes if a raw crate can be compiled.
    fn probe_raw_crate(&self, name: &str, cargo_toml: &str, lib_rs: &str) -> Result<(), io::Error>;
    /// Emits cfg if a dependency exposes a specific struct field.
    ///  * dep - dependency name
    ///  * dep_version - optional dependency version (raw toml string or object)
    ///  * stuct_path - path to the structure
    ///  * field_path - path to the field
    fn emit_dep_has_struct_field(&self, dep: &str, struct_path: &str, field_path: &str, cfg: &str);
    /// Emits cfg if a dependency exposes a specific path.
    fn emit_dep_has_path(&self, dep: &str, path: &str, cfg: &str);
}
impl AutoCfgExt for autocfg::AutoCfg {
    fn probe_raw_crate(&self, name: &str, cargo_toml: &str, lib_rs: &str) -> Result<(), io::Error> {
        // create crate
        let mut crate_path: PathBuf = out_dir();
        crate_path.push("probe_raw_crate");
        crate_path.push(name);
        let src_path = crate_path.join("src");
        if !src_path.is_dir() {
            std::fs::create_dir_all(&src_path)?;
        }
        std::fs::write(crate_path.join("Cargo.toml"), cargo_toml)?;
        std::fs::write(src_path.join("lib.rs"), lib_rs)?;
        // cargo check
        let cargo = env::var_os("CARGO").expect("cargo");
        let mut child = Command::new(cargo).arg("check").current_dir(&crate_path).spawn()?;
        match child.wait() {
            Ok(status) if status.success() => Ok(()),
            Ok(_) => Err(io::ErrorKind::Other.into()), // failure status code
            Err(err) => Err(err),
        }
    }
    fn emit_dep_has_struct_field(&self, dep: &str, struct_path: &str, field_path: &str, cfg: &str) {
        let cargo_toml = format!(
            r#"
        [package]
        name = "{cfg}"
        edition = "2021"
        [workspace]
        [dependencies]
        {dep}
        "#
        );
        let lib_rs = format!("fn _f(x: &{struct_path}) {{ let _ = x.{field_path}; }}");
        autocfg::emit_possibility(cfg);
        if self.probe_raw_crate(cfg, &cargo_toml, &lib_rs).is_ok() {
            autocfg::emit(cfg);
        }
    }
    fn emit_dep_has_path(&self, dep: &str, path: &str, cfg: &str) {
        let cargo_toml = format!(
            r#"
        [package]
        name = "{cfg}"
        edition = "2021"
        [workspace]
        [dependencies]
        {dep}
        "#
        );
        let lib_rs = format!("pub use {path};");
        autocfg::emit_possibility(cfg);
        if self.probe_raw_crate(cfg, &cargo_toml, &lib_rs).is_ok() {
            autocfg::emit(cfg);
        }
    }
}

fn out_dir() -> PathBuf {
    env::var_os("OUT_DIR").expect("out dir").into()
}

fn main() {
    // auto config
    let ac = autocfg::new();

    let libc = r#"libc = { version = "0.2", features = ["extra_traits"] }"#;
    ac.emit_dep_has_path(libc, "libc::memalign", "has_libc_memalign");
    ac.emit_dep_has_path(libc, "libc::posix_memalign", "has_libc_posix_memalign");
    ac.emit_dep_has_path(libc, "libc::IPV6_V6ONLY", "has_libc_ipv6_v6only");
    ac.emit_dep_has_struct_field(libc, "libc::sockaddr_in6", "sin6_len", "has_libc_sockaddr_in6_sin6_len");
    ac.emit_dep_has_struct_field(libc, "libc::tm", "tm_gmtoff", "has_libc_tm_tm_gmtoff");

    autocfg::rerun_path("build.rs");

    // Extra system symbols not included in libc, update only if the bindings changed.
    let contents = r#"
    #include <arpa/inet.h>
    #include <netdb.h>
    "#;
    let mut new_data = Vec::new();
    bindgen::Builder::default()
        .header_contents("_extra_sys.h", contents)
        .blocklist_item("IPPORT_RESERVED") // defined multiple times
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("bindings")
        .write(Box::new(&mut new_data))
        .expect("new data");
    let rs_path: PathBuf = out_dir().join("_extra_sys.rs");
    let create_or_update = fs::read_to_string(&rs_path).map_or_else(|_| true, |old_data| old_data.as_bytes() != new_data);
    if create_or_update {
        fs::write(rs_path, &new_data).expect("extra sys");
    }

    // Extra C symbols.
    autocfg::rerun_path("src/_extra.c");
    cc::Build::new().static_flag(true).file("src/_extra.c").cargo_warnings(true).cargo_output(true).warnings_into_errors(true).compile("_extra_c");
}
