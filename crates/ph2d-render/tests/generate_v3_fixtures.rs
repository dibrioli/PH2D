//! T0.12 — generator (bootstrap) for the 5 canonical v3 binary
//! fixtures (Sprite_projeto §10.6 + HANDOFF_sprite_inspector_v2 §5).
//!
//! ## Why an integration test (vs build.rs / examples bin)
//!
//! Fixtures must be FROZEN once committed: a future schema change
//! that drifts `SpriteV3` would silently regenerate them on every
//! build, hiding the regression. `#[ignore]` keeps `cargo test`
//! from re-stamping the bytes; humans run this exactly once after
//! deliberately editing the v3 schema (or never, if v3 stays
//! frozen — which is the W0 contract). Handoff §5 sketched a "bin
//! one-shot"; the integration-test form is materially equivalent
//! and lets the canonical fixture set live in the lib
//! ([`ph2d_render::sprite_versioned::canonical_v3_fixtures`]) so
//! both the T0.12 generator and the T0.13 verifier share a single
//! source of truth — `fixtures_match_canonical_serialization` then
//! gates drift between the source array and the committed bytes
//! in a single test.
//!
//! ## Invocation
//!
//! ```text
//! CARGO_TARGET_DIR="$PWD/target/slot-coord-sprite" \
//!   cargo test -p ph2d-render \
//!     --test generate_v3_fixtures -- --ignored --nocapture
//! ```
//!
//! After running, `git status crates/ph2d-render/tests/fixtures/`
//! should be clean. If `fixtures_match_canonical_serialization`
//! turned RED before this run, that's the signal that something
//! drifted; do NOT bypass it by regenerating.

use ph2d_render::SpriteVersioned;
use ph2d_render::sprite_versioned::canonical_v3_fixtures;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
#[ignore = "manual fixture bootstrap; run with `cargo test -p ph2d-render --test generate_v3_fixtures -- --ignored`. DO NOT pass --include-ignored in CI"]
fn write_v3_fixtures() {
    write_v3_fixtures_atomically();
}

/// Atomic write: serialize each canonical fixture to a temp file in
/// the same directory, then `std::fs::rename` (atomic on the same
/// filesystem on linux/macOS/windows-NTFS) into the canonical path.
///
/// The tmp suffix is PID-keyed so two concurrent generator runs
/// (e.g., a developer + a stray `--include-ignored` invocation)
/// cannot collide on the tmp file. The rename then races to a
/// single canonical name — last writer wins, but BOTH writers
/// produce identical bytes by the cross-OS fixture contract, so
/// the race is benign.
fn write_v3_fixtures_atomically() {
    let dir = fixtures_dir();
    std::fs::create_dir_all(&dir).expect("create fixtures dir");
    let pid = std::process::id();

    for (name, sprite) in canonical_v3_fixtures() {
        let bytes = postcard::to_allocvec(&SpriteVersioned::V3(sprite))
            .expect("postcard serialize v3 fixture");
        let final_path = dir.join(name);
        let tmp_path = dir.join(format!("{name}.tmp.{pid}"));
        std::fs::write(&tmp_path, &bytes).unwrap_or_else(|e| panic!("write tmp {tmp_path:?}: {e}"));
        std::fs::rename(&tmp_path, &final_path)
            .unwrap_or_else(|e| panic!("rename {tmp_path:?} -> {final_path:?}: {e}"));
        println!("wrote {} ({} bytes)", name, bytes.len());
    }
}

/// Non-ignored sanity: every canonical fixture must be present on
/// disk (committed). Catches accidental deletion in PR review.
#[test]
fn v3_fixtures_present() {
    let dir = fixtures_dir();
    for (name, _) in canonical_v3_fixtures() {
        let path = dir.join(name);
        assert!(
            path.exists(),
            "missing fixture {path:?} — run `cargo test -p ph2d-render --test generate_v3_fixtures -- --ignored` to bootstrap"
        );
    }
}
