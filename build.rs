use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::{env, fs};

use walkdir::WalkDir;

fn main() {
    let target =
        env::var("TARGET").expect("gnurx-sys: Environment variable 'TARGET' was not defined.");

    let out_dir = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .expect("gnurx-sys: Environment variable 'OUT_DIR' was not defined.");

    println!("cargo:root={}", out_dir.display());

    if !target.ends_with("-pc-windows-gnu") {
        return; // Nothing to build for this architecture.
    }

    let regex_header = if let Some(prefix) = target_env_var_os("GNURX_LIB_DIR_PREFIX", &target) {
        let prefix = if let Ok(prefix) = dunce::canonicalize(&prefix) {
            prefix
        } else {
            panic!(
                "gnurx-sys: Failed to canonicalize '{}'.",
                Path::new(&prefix).display()
            );
        };

        use_shared_external_lib(&prefix)
    } else {
        build_static_lib(&target, &out_dir)
    };

    generate_bindings(&out_dir, &regex_header)
}

fn use_shared_external_lib(prefix: &Path) -> PathBuf {
    println!("cargo:include={}", prefix.join("include").display());
    let regex_header = prefix.join("include").join("regex.h");

    println!("cargo:rerun-if-changed={}", regex_header.display());
    if regex_header.metadata().is_err() {
        panic!("gnurx-sys: Failed to find '{}'.", regex_header.display())
    };

    let shared_lib_dir = prefix.join("bin");
    println!("cargo:lib={}", shared_lib_dir.display());
    println!(
        "cargo:rustc-link-search=native={}",
        shared_lib_dir.display()
    );

    let shared_lib_path = shared_lib_dir.join("libgnurx-0.dll");
    println!("cargo:rerun-if-changed={}", shared_lib_path.display());
    if shared_lib_path.metadata().is_err() {
        panic!("gnurx-sys: Failed to find '{}'.", shared_lib_path.display())
    };

    println!("cargo:rustc-link-lib=dylib=libgnurx-0");
    regex_header
}

fn build_static_lib(target: &str, out_dir: &Path) -> PathBuf {
    let src_dir = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .expect("gnurx-sys: Environment variable 'CARGO_MANIFEST_DIR' was not defined.");

    let lib_src_dir = src_dir.join("libgnurx");

    for &name in &["CC", "CFLAGS", "AR", "ARFLAGS"] {
        rerun_if_env_changed(name, target);
    }

    rerun_if_dir_changed(&lib_src_dir);

    println!("cargo:lib={}", out_dir.display());

    let regex_header = out_dir.join("regex.h");
    println!("cargo:include={}", out_dir.display());

    fs::copy(lib_src_dir.join("regex.h"), &regex_header)
        .expect("gnurx-sys: Failed to copy 'regex.h' from sources to output directory.");

    if env::var_os("DOCS_RS").is_none() {
        cc::Build::new()
            .static_flag(true)
            .pic(true)
            .warnings(true)
            .extra_warnings(true)
            .flag("-mthreads")
            .include(&lib_src_dir)
            .file(lib_src_dir.join("regex.c"))
            .compile("gnurx");
    }
    regex_header
}

fn generate_bindings(out_dir: &Path, regex_header: &Path) {
    let bindings = bindgen::Builder::default()
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .derive_debug(true)
        .derive_copy(true)
        .derive_partialeq(true)
        .derive_eq(true)
        .derive_hash(true)
        .impl_debug(true)
        .impl_partialeq(true)
        .size_t_is_usize(true)
        .rustfmt_bindings(true)
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .header(regex_header.to_str().unwrap())
        .allowlist_function("reg(comp|exec|error|free)")
        .allowlist_type("reg_errcode_t")
        .allowlist_var("REG_.*")
        .opaque_type("regex_t")
        .generate()
        .expect("gnurx-sys: Failed to generate Rust bindings for 'regex.h'.");

    bindings
        .write_to_file(out_dir.join("gnurx-sys.rs"))
        .expect("gnurx-sys: Failed to write 'gnurx-sys.rs'.")
}

fn target_env_var_os(name: &str, target: &str) -> Option<OsString> {
    rerun_if_env_changed(name, target);

    let target_underscores = target.replace('-', "_");

    env::var_os(format!("{}_{}", name, target))
        .or_else(|| env::var_os(format!("{}_{}", name, target_underscores)))
        .or_else(|| env::var_os(format!("TARGET_{}", name)))
        .or_else(|| env::var_os(name.to_string()))
}

fn rerun_if_env_changed(name: &str, target: &str) {
    let target_underscores = target.replace('-', "_");

    println!("cargo:rerun-if-env-changed={}_{}", name, target);
    println!("cargo:rerun-if-env-changed={}_{}", name, target_underscores);
    println!("cargo:rerun-if-env-changed=TARGET_{}", name);
    println!("cargo:rerun-if-env-changed={}", name);
}

fn rerun_if_dir_changed(dir: &Path) {
    for file in WalkDir::new(dir).follow_links(false).same_file_system(true) {
        if let Ok(file) = file {
            println!("cargo:rerun-if-changed={}", file.path().display());
        } else {
            panic!(
                "gnurx-sys: Failed to list directory contents: {}",
                dir.display()
            );
        }
    }
}
