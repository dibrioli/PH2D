//! Versioned `Transform` postcard contract (ADR-0025-amendment-1 §2.3
//! + §2.4 + §2.5). Mirrors `ph2d-render/tests/sprite_versioned_postcard.rs`.
//!
//! Five contracts:
//!
//! 1. **PRIMARY — wrapper enum dispatch + migrator.** `load_transform`
//!    decodes the 3 committed v1 fixtures and migrates them to v2 with
//!    benign skew defaults (`0.0`), preserving translation/rotation/scale.
//! 2. **EMPIRICAL — `#[serde(default)]` is dead under postcard.** A raw
//!    v1 blob (3 fields) fails to deserialize directly as the v2
//!    `Transform` (5 fields) with `DeserializeUnexpectedEnd`; the
//!    wrapper enum is the SOLE back-compat path.
//! 3. **Cross-OS bit-identical fixture bytes (HR-5).**
//!    `fixtures_match_canonical_serialization` re-serializes the
//!    canonical inputs and asserts byte-for-byte equality with the
//!    committed `.postcard` blobs.
//! 4. **Discriminant + wire-length pins.** V1 = `0x00`, V2 = `0x01`
//!    (append-only enum); v1 envelope length pinned.
//! 5. **Skew compose determinism golden.**
//!    `blake3(postcard(skew_compose_cases()))` equals the committed
//!    `transform_skew_compose.expected` — the cross-OS proof that
//!    `libm::tanf`/`sincosf` in `compose` are platform-independent.

use ph2d_ecs::transform_versioned::{canonical_v1_fixtures, skew_compose_cases};
use ph2d_ecs::{Transform, TransformV1, TransformVersioned, load_transform};
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn read_fixture(name: &str) -> Vec<u8> {
    let path = fixtures_dir().join(name);
    std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "fixture {path:?} missing ({e}) — run `cargo test -p ph2d-ecs --test generate_transform_v1_fixtures -- --ignored --nocapture` to bootstrap"
        )
    })
}

// ---------------------------------------------------------------------------
// 1. PRIMARY — wrapper enum dispatch + migrator
// ---------------------------------------------------------------------------

#[test]
fn load_transform_migrates_v1_fixtures_with_benign_skew() {
    for (name, v1) in canonical_v1_fixtures() {
        let bytes = read_fixture(name);
        let migrated = load_transform(&bytes)
            .unwrap_or_else(|e| panic!("load_transform({name}) failed: {e:?}"));
        assert_eq!(migrated.translation, v1.translation, "{name} translation");
        assert_eq!(migrated.rotation, v1.rotation, "{name} rotation");
        assert_eq!(migrated.scale, v1.scale, "{name} scale");
        assert_eq!(migrated.skew_x, 0.0, "{name} skew_x must default to 0");
        assert_eq!(migrated.skew_y, 0.0, "{name} skew_y must default to 0");
    }
}

#[test]
fn migrate_v1_to_v2_round_trip_through_v2_save() {
    // v1 fixture → migrate → save (V2) → load → identical Transform.
    let bytes = read_fixture("transform_v1_full.postcard");
    let migrated = load_transform(&bytes).unwrap();
    let resaved = ph2d_ecs::save_transform(&migrated);
    let reloaded = load_transform(&resaved).unwrap();
    assert_eq!(migrated, reloaded);
    // Re-saved blob is a V2 envelope (discriminant 0x01).
    assert_eq!(resaved[0], 0x01, "save_transform must emit a V2 envelope");
}

// ---------------------------------------------------------------------------
// 2. EMPIRICAL — raw v1 bytes do NOT deserialize as v2 Transform
// ---------------------------------------------------------------------------

#[test]
fn raw_v1_payload_cannot_deserialize_as_v2_transform() {
    // A bare v1 Transform (3 fields, NO wrapper) is what an old direct
    // `postcard::to_allocvec(&transform_v1)` produced. Deserializing it
    // as the v2 `Transform` must FAIL (postcard rejects trailing-
    // missing skew_x) — proving the wrapper enum is mandatory and
    // `#[serde(default)]` is documentary-only under postcard.
    let v1 = TransformV1 {
        translation: ph2d_core::Vec2::new(1.0, 2.0),
        rotation: 0.5,
        scale: ph2d_core::Vec2::new(1.0, 1.0),
    };
    let bare = postcard::to_allocvec(&v1).unwrap();
    let result: Result<Transform, _> = postcard::from_bytes(&bare);
    let err = result.expect_err(
        "raw v1 bytes deserialized as v2 Transform — if this now SUCCEEDS, \
         postcard added trailing-default support and ADR-0025-amendment-1 §2.3 \
         must be re-evaluated",
    );
    match err {
        postcard::Error::DeserializeUnexpectedEnd => {}
        other => panic!("expected DeserializeUnexpectedEnd, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 3. Cross-OS bit-identical fixture bytes (HR-5 gate)
// ---------------------------------------------------------------------------

#[test]
fn fixtures_match_canonical_serialization() {
    for (name, v1) in canonical_v1_fixtures() {
        let committed = read_fixture(name);
        let generated = postcard::to_allocvec(&TransformVersioned::V1(v1))
            .unwrap_or_else(|e| panic!("serialize canonical {name}: {e}"));
        assert_eq!(
            generated, committed,
            "fixture {name} drift — committed bytes ≠ canonical TransformV1. \
             Either TransformV1 field shape changed without regenerating, postcard \
             wire drifted, or the blob was hand-edited. Resolve before relying on it."
        );
    }
}

// ---------------------------------------------------------------------------
// 4. Discriminant + wire-length pins
// ---------------------------------------------------------------------------

#[test]
fn v1_fixtures_start_with_zero_discriminant() {
    for (name, _) in canonical_v1_fixtures() {
        let bytes = read_fixture(name);
        assert!(!bytes.is_empty(), "{name} empty — generation regression");
        assert_eq!(
            bytes[0], 0x00,
            "{name} first byte = 0x{:02x}; TransformVersioned::V1 discriminant must be 0x00 — \
             never insert a variant before V1",
            bytes[0]
        );
    }
}

#[test]
fn transform_versioned_v2_serializes_with_one_discriminant() {
    let bytes = postcard::to_allocvec(&TransformVersioned::V2(Transform::IDENTITY)).unwrap();
    assert_eq!(
        bytes[0], 0x01,
        "TransformVersioned::V2 discriminant must be 0x01 (appended after V1=0x00); got 0x{:02x}",
        bytes[0]
    );
}

#[test]
fn v1_translation_fixture_wire_length_pin() {
    // 1 (V1 discriminant) + 8 (translation 2×f32) + 4 (rotation f32) +
    // 8 (scale 2×f32) = 21 bytes. Pins the v1 wire shape.
    let bytes = read_fixture("transform_v1_translation.postcard");
    assert_eq!(
        bytes.len(),
        21,
        "canonical v1 envelope must be 21 bytes; drift = wire-format regression"
    );
}

#[test]
fn versioned_round_trip_preserves_v2_with_skew() {
    let original = Transform {
        translation: ph2d_core::Vec2::new(42.0, 17.0),
        rotation: 0.9,
        scale: ph2d_core::Vec2::new(2.0, 0.5),
        skew_x: 0.3,
        skew_y: -0.2,
    };
    let bytes = ph2d_ecs::save_transform(&original);
    let restored = load_transform(&bytes).unwrap();
    assert_eq!(
        restored, original,
        "v2 round trip must preserve every field"
    );
}

// ---------------------------------------------------------------------------
// 5. Skew compose determinism golden (cross-OS bit-identical via libm)
// ---------------------------------------------------------------------------

#[test]
fn transform_compose_skew_byte_identical_cross_os() {
    let bytes = postcard::to_allocvec(&skew_compose_cases()).unwrap();
    let hash = blake3::hash(&bytes);
    let expected = std::fs::read_to_string(fixtures_dir().join("transform_skew_compose.expected"))
        .expect("transform_skew_compose.expected missing — run the ignored generator");
    let expected = expected.trim();
    assert_eq!(
        hash.to_hex().as_str(),
        expected,
        "Transform compose-with-skew is not cross-OS bit-identical. Either the libm \
         migration is incomplete (a `f32::tan`/`sin_cos` slipped into the skew path), \
         the compose math changed, or the golden fixture is stale. DO NOT regenerate \
         without a cross-OS re-verify + ADR amendment (HR-5 contract)."
    );
}
