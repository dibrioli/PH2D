//! Generator (bootstrap) for the canonical v1 binary `Transform`
//! fixtures + the skew-compose determinism golden (ADR-0025-amendment-1
//! §2.5 + §2.4). Mirrors `ph2d-render/tests/generate_v3_fixtures.rs`.
//!
//! ## Why an integration test (vs build.rs / examples bin)
//!
//! Fixtures must be FROZEN once committed: a future schema change that
//! drifts `TransformV1` would silently regenerate them on every build,
//! hiding the regression. `#[ignore]` keeps `cargo test` from
//! re-stamping the bytes; humans run this exactly once after
//! deliberately editing the v1 schema (or never, if v1 stays frozen).
//!
//! ## Invocation
//!
//! ```text
//! CARGO_TARGET_DIR=… cargo test -p ph2d-ecs \
//!   --test generate_transform_v1_fixtures -- --ignored --nocapture
//! ```
//!
//! After running, `git status crates/ph2d-ecs/tests/fixtures/` should
//! be clean. If `fixtures_match_canonical_serialization` turned RED
//! before this run, that's the signal that something drifted; do NOT
//! bypass it by regenerating.

use ph2d_ecs::TransformVersioned;
use ph2d_ecs::transform_versioned::{canonical_v1_fixtures, skew_compose_cases};
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
#[ignore = "manual fixture bootstrap; run with `--ignored --nocapture`. DO NOT pass --include-ignored in CI"]
fn write_transform_v1_fixtures() {
    let dir = fixtures_dir();
    std::fs::create_dir_all(&dir).expect("create fixtures dir");
    let pid = std::process::id();

    // 1. Binary v1 fixtures (wrapped as TransformVersioned::V1).
    for (name, v1) in canonical_v1_fixtures() {
        let bytes = postcard::to_allocvec(&TransformVersioned::V1(v1))
            .expect("postcard serialize v1 fixture");
        let final_path = dir.join(name);
        let tmp_path = dir.join(format!("{name}.tmp.{pid}"));
        std::fs::write(&tmp_path, &bytes).unwrap_or_else(|e| panic!("write tmp {tmp_path:?}: {e}"));
        std::fs::rename(&tmp_path, &final_path)
            .unwrap_or_else(|e| panic!("rename {tmp_path:?} -> {final_path:?}: {e}"));
        println!("wrote {} ({} bytes)", name, bytes.len());
    }

    // 2. Skew-compose determinism golden (single text file, identical
    //    on every OS because libm is cross-platform bit-identical).
    let bytes = postcard::to_allocvec(&skew_compose_cases()).expect("serialize skew cases");
    let hash = blake3::hash(&bytes);
    let final_path = dir.join("transform_skew_compose.expected");
    let tmp_path = dir.join(format!("transform_skew_compose.expected.tmp.{pid}"));
    std::fs::write(&tmp_path, hash.to_hex().as_bytes()).expect("write golden tmp");
    std::fs::rename(&tmp_path, &final_path).expect("rename golden");
    println!("wrote transform_skew_compose.expected ({})", hash.to_hex());
}

/// Non-ignored sanity: every committed fixture must be present on disk.
#[test]
fn transform_v1_fixtures_present() {
    let dir = fixtures_dir();
    for (name, _) in canonical_v1_fixtures() {
        assert!(
            dir.join(name).exists(),
            "missing committed fixture {name}; run the ignored generator to bootstrap"
        );
    }
    assert!(
        dir.join("transform_skew_compose.expected").exists(),
        "missing transform_skew_compose.expected golden"
    );
}
