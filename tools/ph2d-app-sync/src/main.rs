#![forbid(unsafe_code)]
//! Regenera os blocos do `ph2d-app-registry-init` de uma varredura de `crates/ph2d-app-*`.
//!
//! ```text
//! cargo run -p ph2d-app-sync
//! ```

use std::path::PathBuf;

use ph2d_app_sync::{
    RS_BEGIN, RS_END, TOML_DEFAULT_BEGIN, TOML_DEFAULT_END, TOML_DEPS_BEGIN, TOML_DEPS_END,
    TOML_FEATURES_BEGIN, TOML_FEATURES_END, render_default, render_deps, render_features,
    render_rs, rewrite, scan_app_crates,
};

fn main() {
    let root: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("tools/<x>/ tem dois pais")
        .to_path_buf();
    let crates_dir = root.join("crates");
    let found = scan_app_crates(&crates_dir);
    println!("famílias encontradas: {}", found.join(", "));

    let reg = crates_dir.join("ph2d-app-registry-init");

    let lib = reg.join("src/lib.rs");
    let src = std::fs::read_to_string(&lib).expect("lib.rs do registo");
    let out = rewrite(&src, RS_BEGIN, RS_END, &render_rs(&found)).expect("marcadores no lib.rs");
    std::fs::write(&lib, out).expect("escrita do lib.rs");

    let toml = reg.join("Cargo.toml");
    let mut src = std::fs::read_to_string(&toml).expect("Cargo.toml do registo");
    for (b, e, body) in [
        (TOML_DEPS_BEGIN, TOML_DEPS_END, render_deps(&found)),
        (TOML_DEFAULT_BEGIN, TOML_DEFAULT_END, render_default(&found)),
        (TOML_FEATURES_BEGIN, TOML_FEATURES_END, render_features(&found)),
    ] {
        src = rewrite(&src, b, e, &body).unwrap_or_else(|| panic!("marcadores {b} no Cargo.toml"));
    }
    std::fs::write(&toml, src).expect("escrita do Cargo.toml");
    println!("ok: 4 blocos regenerados");
}
