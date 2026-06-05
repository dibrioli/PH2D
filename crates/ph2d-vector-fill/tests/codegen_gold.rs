//! T6.2 DoD — 5 codegen-gold snapshots. The generated WGSL is deterministic and
//! param-free, so the exact string is stable across OSes; these golden files lock
//! it. Regenerate intentionally with `PH2D_BLESS_GOLD=1 cargo test -p
//! ph2d-vector-fill --test codegen_gold` after a deliberate codegen change, then
//! review the diff.

mod common;

use ph2d_vector_fill::wgsl_codegen::codegen;
use std::path::PathBuf;

fn golden_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(format!("{name}.wgsl"))
}

#[test]
fn codegen_matches_golden() {
    let bless = std::env::var("PH2D_BLESS_GOLD").is_ok();
    let mut failures = Vec::new();

    for (name, graph) in common::fixtures() {
        let wgsl = codegen(&graph).expect("fixture must codegen");
        let path = golden_path(name);

        if bless {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &wgsl).unwrap();
            continue;
        }

        match std::fs::read_to_string(&path) {
            Ok(golden) if golden == wgsl => {}
            Ok(_) => failures.push(format!("{name}: WGSL drifted from {}", path.display())),
            Err(e) => failures.push(format!(
                "{name}: missing golden {} ({e}); run with PH2D_BLESS_GOLD=1",
                path.display()
            )),
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
