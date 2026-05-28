//! v3 → v4 migrator gate (Sprite_projeto §10.6).
//!
//! W0 created this file as a `#[ignore]`d stub reserving the canonical
//! path the spec names ("crates/ph2d-render/tests/migrate_sprite_v3_to_v4.rs").
//! **W1.T1.6 lands the real suite here:** the migrator
//! ([`Sprite::migrate_v3_to_v4`]) and the canonical load entry point
//! ([`ph2d_render::load_sprite`], ADR-0070-amendment-2 §4) now exist, so
//! the assertions below exercise the actual back-compat path against the
//! 5 frozen v3 fixtures generated in W0.T0.12 — NOT a tautology, because
//! the fixtures were serialized by the v3 `SpriteV3` schema before the
//! `Sprite` struct was bumped to v4.
//!
//! ## Why these assertions matter (the bug class they guard)
//!
//! `#[serde(default)]` on the 14 new v4 fields is documentary-only under
//! postcard (positional, non-self-describing — T0.13 empirically pinned
//! that postcard REJECTS trailing-missing fields). The wrapper-enum
//! dispatch + migrator is therefore the SOLE working back-compat path.
//! If a future edit deletes the migrator and leans on serde defaults,
//! `deserialize_v3_postcard_loads_as_v4_with_defaults` fails loudly.
//! If a future edit makes `region_filter_clip` an unconditional default
//! (the `#[serde(default)]` helper returns the Atlas value `true` for
//! everyone), `..._individual_..._region_filter_clip_false` fails.

use ph2d_render::sprite_versioned::{SpriteV3, SpriteVersioned, canonical_v3_fixtures};
use ph2d_render::{LoadError, Sprite, SpriteSource, load_sprite};
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn read_fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixtures_dir().join(name)).unwrap_or_else(|e| {
        panic!(
            "fixture {name} missing ({e}) — run `cargo test -p ph2d-render --test generate_v3_fixtures -- --ignored --nocapture` to bootstrap"
        )
    })
}

/// W0 carry-over: every committed fixture is a `V3` envelope, so the
/// wrapper dispatch the migrator consumes stays honest. Complements the
/// migration assertions below (this checks the discriminant; those check
/// the transform output).
#[test]
fn v3_fixtures_dispatch_through_wrapper() {
    for (name, _canonical) in canonical_v3_fixtures() {
        let bytes = read_fixture(name);
        let versioned: SpriteVersioned = postcard::from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("postcard wrapper-enum dispatch on {name} failed: {e}"));
        match versioned {
            SpriteVersioned::V3(_) => {} // expected — committed fixtures are v3 envelopes
            SpriteVersioned::V4(_) => {
                panic!("v3 fixture {name} dispatched as V4 — discriminant/fixture regression")
            }
        }
    }
}

#[test]
fn deserialize_v3_postcard_loads_as_v4_with_defaults() {
    let bytes = read_fixture("sprite_v3_atlas.postcard");
    let sprite = load_sprite(&bytes).expect("v3 atlas fixture loads");

    // Forwarded v3 fields.
    assert_eq!(sprite.version, 4);
    assert_eq!(sprite.source, SpriteSource::Atlas { key: 0 });
    assert_eq!(sprite.size, [10.0, 10.0]);
    assert_eq!(sprite.tint, [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(sprite.anchor, [0.0, 0.0]);

    // New v4 fields → benign identity defaults.
    assert_eq!(sprite.self_tint, [1.0; 4]);
    assert_eq!(sprite.per_corner_tint, [[1.0; 4]; 4]);
    assert!(!sprite.tint_fill);
    assert_eq!(sprite.opacity, 1.0);
    assert!(!sprite.flip_x);
    assert!(!sprite.flip_y);
    assert!(sprite.centered);
    assert_eq!(sprite.offset, [0.0, 0.0]);
    assert_eq!(sprite.hframes, 1);
    assert_eq!(sprite.vframes, 1);
    assert_eq!(sprite.frame, 0);
    assert!(!sprite.region_enabled);
    assert_eq!(sprite.region_rect, [0.0; 4]);
    assert!(sprite.region_filter_clip); // Atlas → true via migrator
}

#[test]
fn deserialize_v3_individual_sprite_loads_with_region_filter_clip_false() {
    let bytes = read_fixture("sprite_v3_individual.postcard");
    let sprite = load_sprite(&bytes).expect("v3 individual fixture loads");
    assert_eq!(sprite.version, 4);
    assert_eq!(sprite.source, SpriteSource::Individual { texture_id: 42 });
    assert_eq!(sprite.tint, [1.0, 1.0, 1.0, 0.5]);
    assert!(!sprite.region_filter_clip); // Individual → false via migrator
}

/// Drive ALL 5 frozen fixtures through `load_sprite`, asserting the
/// forwarded fields survive verbatim and the conditional
/// `region_filter_clip` branch matches the source variant. Covers the
/// atlas / atlas_with_anchor / individual / premultiplied / max_size
/// coverage matrix (anatomia / Lens D D25).
#[test]
fn all_v3_fixtures_migrate_preserving_fields_and_region_branch() {
    for (name, v3) in canonical_v3_fixtures() {
        let bytes = read_fixture(name);
        let sprite = load_sprite(&bytes).unwrap_or_else(|e| panic!("{name}: {e}"));

        assert_eq!(sprite.version, 4, "{name}: version");
        assert_eq!(sprite.source, v3.source, "{name}: source");
        assert_eq!(sprite.size, v3.size, "{name}: size");
        assert_eq!(sprite.tint, v3.tint, "{name}: tint");
        assert_eq!(sprite.anchor, v3.anchor, "{name}: anchor");

        // `premultiplied` is `#[serde(skip)]` — never on the wire, so a
        // wire-loaded sprite is ALWAYS `false`, even for the
        // `sprite_v3_premultiplied` fixture whose in-memory canonical
        // carries `true`. The runtime flag is rebuilt from texture-store
        // context at the extract boundary, not from the blob.
        assert!(
            !sprite.premultiplied,
            "{name}: wire-loaded premultiplied must be false (serde skip)"
        );

        let expect_clip = matches!(v3.source, SpriteSource::Atlas { .. });
        assert_eq!(
            sprite.region_filter_clip, expect_clip,
            "{name}: region_filter_clip must follow Atlas={expect_clip}"
        );

        // v4 identity defaults hold for every migrated sprite.
        assert_eq!(sprite.self_tint, [1.0; 4], "{name}: self_tint");
        assert_eq!(sprite.per_corner_tint, [[1.0; 4]; 4], "{name}: per_corner");
        assert!(!sprite.tint_fill, "{name}: tint_fill");
        assert_eq!(sprite.opacity, 1.0, "{name}: opacity");
        assert!(sprite.centered, "{name}: centered");
        assert_eq!(sprite.hframes, 1, "{name}: hframes");
        assert_eq!(sprite.vframes, 1, "{name}: vframes");
        assert!(!sprite.region_enabled, "{name}: region_enabled");
    }
}

/// The PURE migrator (no wire round-trip) must NOT drop a `premultiplied`
/// flag a caller set in memory — `#[serde(skip)]` only erases it on the
/// wire. This pins the migrator-as-value-transform contract distinct
/// from the wire-load behavior asserted above.
#[test]
fn migrate_v3_to_v4_preserves_in_memory_premultiplied_flag() {
    let v3 = SpriteV3 {
        source: SpriteSource::Individual { texture_id: 1 },
        size: [64.0, 64.0],
        tint: [1.0, 1.0, 1.0, 1.0],
        anchor: [0.0, 0.0],
        premultiplied: true,
    };
    let sprite = Sprite::migrate_v3_to_v4(v3);
    assert!(
        sprite.premultiplied,
        "pure migrator preserves an in-memory premultiplied flag"
    );
    assert!(!sprite.region_filter_clip, "Individual → false");
    assert_eq!(sprite.version, 4);
}

#[test]
fn deserialize_v4_round_trip() {
    let mut original = Sprite::atlas(3, [20.0, 40.0], [0.9, 0.8, 0.7, 0.6]);
    original.per_corner_tint = [
        [1.0, 0.0, 0.0, 1.0], // TL red
        [0.0, 1.0, 0.0, 1.0], // TR green
        [0.0, 0.0, 1.0, 1.0], // BL blue
        [1.0, 1.0, 0.0, 1.0], // BR yellow
    ];
    original.self_tint = [0.5, 0.5, 0.5, 1.0];
    original.tint_fill = true;
    original.opacity = 0.5;
    original.flip_x = true;
    original.flip_y = true;
    original.centered = false;
    original.offset = [3.0, -4.0];
    original.hframes = 4;
    original.vframes = 2;
    original.frame = 5;
    original.region_enabled = true;
    original.region_rect = [1.0, 2.0, 8.0, 8.0];

    let bytes = postcard::to_allocvec(&SpriteVersioned::V4(original)).expect("v4 serializes");
    let restored = load_sprite(&bytes).expect("v4 round-trips");
    assert_eq!(original, restored);
}

/// A `V4` envelope is returned VERBATIM — `load_sprite` must not re-run
/// the migrator's `region_filter_clip` branch on it. Guards against a
/// future edit that "normalizes" on every load and would clobber a
/// deliberately non-default value the author saved.
#[test]
fn load_sprite_v4_envelope_returns_without_re_migrating() {
    let mut s = Sprite::individual(5, [10.0, 10.0], [1.0; 4]);
    s.region_filter_clip = true; // deliberately non-default for Individual
    let bytes = postcard::to_allocvec(&SpriteVersioned::V4(s)).expect("v4 serializes");
    let loaded = load_sprite(&bytes).expect("v4 loads");
    assert!(
        loaded.region_filter_clip,
        "V4 path must not re-run the Individual→false migrator branch"
    );
    assert_eq!(loaded, s);
}

/// Bytes that are not a valid `SpriteVersioned` envelope (truncated, or
/// a pre-wrapper v2-era blob) surface as `LoadError::Deserialize` — never
/// a panic, never silent corruption (ADR-0070-amendment-2 §4). The error
/// preserves the postcard cause via `source()`.
#[test]
fn load_sprite_rejects_garbage_bytes_as_error_not_panic() {
    use std::error::Error;
    // A multi-byte varint discriminant far beyond the 2 declared variants.
    let garbage = [0xFFu8, 0xFF, 0xFF, 0xFF, 0xFF];
    let err = load_sprite(&garbage).expect_err("garbage must not load");
    assert!(matches!(err, LoadError::Deserialize(_)));
    assert!(!err.to_string().is_empty(), "Display is non-empty");
    assert!(
        err.source().is_some(),
        "postcard cause preserved in source()"
    );
}

#[test]
fn load_sprite_rejects_empty_bytes() {
    assert!(matches!(load_sprite(&[]), Err(LoadError::Deserialize(_))));
}

/// W1.T1.6 trailing-byte cap. Raw `postcard::from_bytes` silently accepts
/// a valid V3 prefix followed by hostile padding (pinned by the W0 test
/// `versioned_load_silently_ignores_trailing_bytes_after_valid_v3`).
/// `load_sprite` is the defense layer that REJECTS it via `take_from_bytes`
/// + a leftover check — a single-sprite blob must be exactly one envelope.
#[test]
fn load_sprite_rejects_trailing_bytes_after_valid_envelope() {
    let canonical = read_fixture("sprite_v3_atlas.postcard");
    let mut padded = canonical.clone();
    padded.extend_from_slice(&[0xAAu8; 100]); // 100 hostile trailing bytes
    let err = load_sprite(&padded).expect_err("trailing bytes must be rejected");
    match err {
        LoadError::TrailingBytes { consumed, total } => {
            assert_eq!(total, padded.len(), "total = full input length");
            assert_eq!(consumed, canonical.len(), "consumed = exactly the envelope");
            assert_eq!(total - consumed, 100, "100 hostile trailing bytes detected");
        }
        other => panic!("expected TrailingBytes, got {other:?}"),
    }
    // And the exact (unpadded) blob still loads cleanly — the cap rejects
    // only the surplus, never a well-formed envelope.
    assert!(
        load_sprite(&canonical).is_ok(),
        "exact envelope still loads"
    );
}
