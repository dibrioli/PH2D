//! W0 stub for the v3 → v4 migrator gate (Sprite_projeto §10.6).
//!
//! Spec §10.6 ("Fixtures de teste v3 → v4") names this file as
//! "(a criar em W0.T0.12, NÃO W1)" — the FILE lives in W0; the
//! actual `migrate_v3_to_v4` impl + the per-fixture assertions land
//! in **W1.T1.6** (after `Sprite` becomes the 20-field v4 schema and
//! `crate::sprite_migrator::migrate_v3_to_v4` exists).
//!
//! In W0 the migrator function does NOT exist yet. The stub:
//!  1. asserts the 5 frozen v3 fixtures are present (delegating to
//!     T0.13's existing pin) — keeps PR review honest that nothing
//!     deleted the inputs the W1 migrator will read.
//!  2. asserts the wrapper enum still dispatches the fixtures as V3
//!     — the gate stays green from W0 onward.
//!
//! When W1.T1.6 lands, replace the `migrator_unimplemented_until_w1`
//! body with the per-fixture assertions from spec §10.6:
//!
//! ```ignore
//! #[test]
//! fn deserialize_v3_postcard_loads_as_v4_with_defaults() { ... }
//! #[test]
//! fn deserialize_v3_individual_sprite_loads_with_region_filter_clip_false() { ... }
//! #[test]
//! fn deserialize_v4_round_trip() { ... }
//! ```

use ph2d_render::SpriteVersioned;
use ph2d_render::sprite_versioned::canonical_v3_fixtures;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn w0_stub_v3_fixtures_dispatch_through_wrapper_for_w1_migrator() {
    // Cross-check the wrapper dispatch lives for the migrator to
    // consume in W1. When W1.T1.6 lands, this becomes the per-
    // fixture assertion suite from spec §10.6.
    for (name, _canonical_sprite) in canonical_v3_fixtures() {
        let path = fixtures_dir().join(name);
        let bytes = std::fs::read(&path).unwrap_or_else(|e| {
            panic!("fixture {path:?} missing ({e}) — bootstrap via generate_v3_fixtures --ignored")
        });
        let versioned: SpriteVersioned =
            postcard::from_bytes(&bytes).unwrap_or_else(|e| panic!("dispatch {name}: {e}"));
        match versioned {
            SpriteVersioned::V3(_) => {} // expected
        }
    }
}

#[test]
#[ignore = "W1.T1.6 will implement migrate_v3_to_v4 and convert this test from #[ignore] \
            to the per-fixture assertion suite spec §10.6 describes (region_filter_clip \
            branch on Atlas vs Individual, opacity = 1.0 default, self_tint = white, etc.)"]
fn migrator_unimplemented_until_w1() {
    // The function `migrate_v3_to_v4` does not exist in W0. This
    // `#[ignore]`d stub keeps the spec §10.6 file name reserved
    // ("crates/ph2d-render/tests/migrate_sprite_v3_to_v4.rs (a
    // criar em W0.T0.12, NÃO W1)") so W1.T1.6 lands at the
    // canonical path. Removing this attribute and implementing the
    // body is the explicit W1.T1.6 acceptance criterion.
    unimplemented!("migrate_v3_to_v4 lands in W1.T1.6");
}
