//! build script for dynamips_c

use std::env;
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
        let mut crate_path: PathBuf = env::var_os("OUT_DIR").expect("out dir").into();
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

fn main() {
    let ac = autocfg::new();

    let libc = r#"libc = { version = "0.2", features = ["extra_traits"] }"#;
    ac.emit_dep_has_struct_field(libc, "libc::tm", "tm_gmtoff", "has_libc_tm_tm_gmtoff");
    ac.emit_dep_has_path(libc, "libc::posix_memalign", "has_libc_posix_memalign");
    ac.emit_dep_has_path(libc, "libc::memalign", "has_libc_memalign");

    autocfg::rerun_path("build.rs");
}
