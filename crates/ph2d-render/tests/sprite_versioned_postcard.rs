//! T0.13 — empirical postcard validation (Sprite_projeto §10.3 +
//! §10.4 + ADR-0070-amendment-2 + HANDOFF_sprite_inspector_v2 §5).
//!
//! Five independent contracts the W1 schema bump will rely on:
//!
//! 1. **PRIMARY — wrapper enum dispatch.** `SpriteVersioned::V3(...)`
//!    serializes as `[0x00, ...v3 bytes...]` (postcard varint
//!    discriminant). Loads of the 5 frozen v3 fixtures dispatch
//!    correctly via `postcard::from_bytes::<SpriteVersioned>`.
//!
//! 2. **EMPIRICAL FALSIFICATION — `#[serde(default)]` is dead under
//!    postcard.** ADR-0070-amendment-2 §2 ratified the finding;
//!    postcard returns `Error::DeserializeUnexpectedEnd` on
//!    trailing-missing fields. The wrapper enum is therefore the
//!    SOLE back-compat path; the W1 migrator (T1.6) is mandatory.
//!
//! 3. **Cross-OS bit-identical fixture bytes.** The
//!    `fixtures_match_canonical_serialization` test re-runs the
//!    canonical serialization in-process and asserts byte-for-byte
//!    equality with committed `.postcard` blobs. Single gate
//!    covering SpriteV3 field drift + postcard version drift +
//!    accidental fixture regeneration + cross-OS f32 / varint
//!    encoding drift — the HR-5 contract.
//!
//! 4. **Discriminant byte pin + total wire length pin.** V3 stays
//!    at discriminant `0x00` (append-only enum) + total envelope =
//!    35 bytes for the canonical V3 shape.
//!
//! 5. **Fuzzing surface — hostile bytes don't panic / OOM.**
//!    Empty / truncated / unknown-discriminant / max-varint /
//!    NaN-payload / oversized-trailing inputs each pin the loader's
//!    behavior so W1.T1.6's migrator inherits a tested error
//!    surface.

use ph2d_render::sprite_versioned::{SpriteV3, canonical_v3_fixtures};
use ph2d_render::{Sprite, SpriteSource, SpriteVersioned};
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn read_fixture(name: &str) -> Vec<u8> {
    let path = fixtures_dir().join(name);
    std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "fixture {path:?} missing ({e}) — run `cargo test -p ph2d-render --test generate_v3_fixtures -- --ignored --nocapture` to bootstrap"
        )
    })
}

const CANONICAL_FIXTURES: &[&str] = &[
    "sprite_v3_atlas.postcard",
    "sprite_v3_atlas_with_anchor.postcard",
    "sprite_v3_individual.postcard",
    "sprite_v3_premultiplied.postcard",
    "sprite_v3_max_size.postcard",
];

/// Canonical V3 envelope length (1 discriminant + 1 source variant +
/// 1 u32-varint payload + 8 size + 16 tint + 8 anchor + 0 skipped
/// premultiplied = 35). Pinned at module scope because it appears in
/// both the per-fixture length test and the oversized-trailing test.
const CANONICAL_V3_WIRE_LEN: usize = 35;

/// Match helper — exhaustive today (single V3 variant); grows a `V4
/// => panic!()` arm when W1 lands.
fn expect_v3(versioned: SpriteVersioned) -> SpriteV3 {
    match versioned {
        SpriteVersioned::V3(sprite) => sprite,
    }
}

// ---------------------------------------------------------------------------
// 1. PRIMARY — wrapper enum dispatch
// ---------------------------------------------------------------------------

#[test]
fn versioned_dispatch_loads_v3_via_wrapper_enum() {
    for &name in CANONICAL_FIXTURES {
        let bytes = read_fixture(name);
        let versioned: SpriteVersioned = postcard::from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("postcard deserialize {name}: {e}"));
        let _ = expect_v3(versioned);
    }
}

#[test]
fn versioned_v3_atlas_round_trip_preserves_fields() {
    let bytes = read_fixture("sprite_v3_atlas.postcard");
    let sprite = expect_v3(postcard::from_bytes(&bytes).unwrap());
    assert_eq!(sprite.source, SpriteSource::Atlas { key: 0 });
    assert_eq!(sprite.size, [10.0, 10.0]);
    assert_eq!(sprite.tint, [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(sprite.anchor, [0.0, 0.0]);
    // `premultiplied` is `#[serde(skip)]` — always deserialises to
    // its `bool::default()` (`false`), regardless of in-memory value
    // at serialization time. Pinning this matches v3 wire semantics.
    assert!(!sprite.premultiplied);
}

#[test]
fn versioned_v3_individual_round_trip_preserves_texture_id() {
    let bytes = read_fixture("sprite_v3_individual.postcard");
    let sprite = expect_v3(postcard::from_bytes(&bytes).unwrap());
    assert_eq!(sprite.source, SpriteSource::Individual { texture_id: 42 });
    assert_eq!(sprite.size, [128.0, 256.0]);
    assert_eq!(sprite.tint, [1.0, 1.0, 1.0, 0.5]);
}

#[test]
fn versioned_v3_atlas_with_anchor_preserves_anchor() {
    let bytes = read_fixture("sprite_v3_atlas_with_anchor.postcard");
    let sprite = expect_v3(postcard::from_bytes(&bytes).unwrap());
    assert_eq!(sprite.source, SpriteSource::Atlas { key: 7 });
    assert_eq!(sprite.anchor, [-3.0, 4.5]);
}

#[test]
fn versioned_v3_premultiplied_round_trips_with_skip_default_false() {
    // In-memory `premultiplied = true` set at generation but never
    // crossed the wire (`#[serde(skip)]`). The W1 migrator (T1.6)
    // rebuilds the runtime flag from `SpriteSource::Individual` +
    // `IndividualTextureStore` context (only BG-Removal Apply'd
    // textures are premultiplied; a naive
    // `matches!(source, Individual)` over-triggers).
    let bytes = read_fixture("sprite_v3_premultiplied.postcard");
    let sprite = expect_v3(postcard::from_bytes(&bytes).unwrap());
    assert_eq!(sprite.source, SpriteSource::Individual { texture_id: 1 });
    assert!(!sprite.premultiplied);
}

#[test]
fn versioned_v3_max_size_carries_extremal_values() {
    let bytes = read_fixture("sprite_v3_max_size.postcard");
    let sprite = expect_v3(postcard::from_bytes(&bytes).unwrap());
    assert_eq!(sprite.size, [16384.0, 16384.0]);
    assert_eq!(sprite.anchor, [8192.0, -8192.0]);
    for v in sprite
        .size
        .iter()
        .chain(sprite.anchor.iter())
        .chain(sprite.tint.iter())
    {
        assert!(v.is_finite(), "max-size fixture leaked non-finite {v}");
    }
}

// ---------------------------------------------------------------------------
// 2. Empirical pin — `#[serde(default)]` on trailing-missing is DEAD
// ---------------------------------------------------------------------------

/// `ProbeV1` ↔ `ProbeV2` is the smallest test that pins postcard's
/// behavior on trailing-missing fields.
///
/// ## Empirical finding (T0.13, 2026-05-28)
///
/// Postcard rejects trailing-missing fields with
/// `Error::DeserializeUnexpectedEnd`. Spec §10.4 hybrid claim is
/// SUPERSEDED by ADR-0070-amendment-2 §3; wrapper enum is the SOLE
/// back-compat path.
#[derive(serde::Serialize)]
struct ProbeV1 {
    a: u32,
    b: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
struct ProbeV2 {
    a: u32,
    b: u32,
    #[serde(default = "probe_default_c")]
    c: u32,
    #[serde(default)]
    d: u32,
}

fn probe_default_c() -> u32 {
    42
}

#[test]
fn postcard_rejects_trailing_serde_default_on_short_payload() {
    let v1 = ProbeV1 { a: 1, b: 2 };
    let bytes = postcard::to_allocvec(&v1).expect("serialize v1");
    let result: Result<ProbeV2, _> = postcard::from_bytes(&bytes);
    let err = result.expect_err(
        "postcard's behavior on trailing-missing fields is the W1 contract's pivot — \
         if this now SUCCEEDS, re-evaluate ADR-0070-amendment-2 §3",
    );
    // Explicit match (not `matches!()`) is the contract: a postcard
    // rename or new variant MUST land in the `other =>` arm so a
    // human can audit semantic equivalence before this pin is
    // updated. Silent `matches!()` would let a renamed variant
    // slip past as a green test.
    match err {
        postcard::Error::DeserializeUnexpectedEnd => {}
        other => panic!("expected DeserializeUnexpectedEnd, got {other:?}"),
    }
}

#[test]
fn postcard_round_trip_succeeds_on_complete_payload() {
    let original = ProbeV2 {
        a: 7,
        b: 11,
        c: 13,
        d: 17,
    };
    let bytes = postcard::to_allocvec(&original).unwrap();
    let restored: ProbeV2 = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(restored, original);
}

// ---------------------------------------------------------------------------
// 3. Cross-OS bit-identical fixture bytes (HR-5 gate)
// ---------------------------------------------------------------------------

#[test]
fn fixtures_match_canonical_serialization() {
    // The committed `.postcard` blobs MUST equal byte-for-byte what
    // `postcard::to_allocvec(&SpriteVersioned::V3(canonical))`
    // produces at test time. Single gate covering SpriteV3 field
    // drift, postcard version drift (despite the `=1.1.3` pin),
    // accidental fixture regeneration, and cross-OS f32 / varint
    // encoding drift — the HR-5 contract.
    for (name, sprite) in canonical_v3_fixtures() {
        let committed = read_fixture(name);
        let generated = postcard::to_allocvec(&SpriteVersioned::V3(sprite))
            .unwrap_or_else(|e| panic!("serialize canonical {name}: {e}"));
        assert_eq!(
            generated, committed,
            "fixture {name} drift — committed bytes do not match the canonical \
             SpriteV3 input. Either: (a) SpriteV3 field shape changed without \
             regenerating fixtures, (b) postcard wire encoding drifted (check \
             the `=1.1.3` exact-pin in crates/ph2d-render/Cargo.toml), (c) the \
             fixture was hand-edited, or (d) committed bytes were regenerated \
             on an OS / arch with non-bit-identical f32 / varint encoding — \
             should be impossible per HR-5; if it occurred, file an \
             ADR amendment first. Resolve before W1 schema bump."
        );
    }
}

#[test]
fn spritev3_struct_wire_matches_live_sprite_v3() {
    // Drift gate: live `Sprite` (v3 today; v4 in W1) and the frozen
    // `SpriteV3` mirror MUST share byte-identical wire format while
    // v3 is still the active schema. Serialize a `Sprite` directly,
    // deserialize as `SpriteV3`, assert equality. Catches incidental
    // edits to `Sprite` field order / type / serde attributes that
    // desync from the frozen baseline.
    let live = Sprite {
        source: SpriteSource::Atlas { key: 7 },
        size: [32.0, 48.0],
        tint: [0.5, 0.75, 0.25, 1.0],
        anchor: [-3.0, 4.5],
        premultiplied: false,
    };
    let bytes = postcard::to_allocvec(&live).expect("serialize live v3 Sprite");
    let frozen: SpriteV3 =
        postcard::from_bytes(&bytes).expect("SpriteV3 wire format must match live Sprite v3 in W0");
    assert_eq!(frozen.source, live.source);
    assert_eq!(frozen.size, live.size);
    assert_eq!(frozen.tint, live.tint);
    assert_eq!(frozen.anchor, live.anchor);
    assert!(!frozen.premultiplied);
}

#[test]
fn spritev3_mirror_anchor_missing_blob_rejects_symmetric_with_live() {
    // Lens E (R2) asymmetry pin: both `Sprite::anchor` and
    // `SpriteV3::anchor` carry `#[serde(default)]`; both
    // `Sprite::premultiplied` and `SpriteV3::premultiplied` carry
    // `#[serde(skip)]`. We can't grep attributes from a test, but
    // we CAN assert that a hypothetical anchor-missing blob behaves
    // identically against both structs (both reject, because
    // postcard ignores `#[serde(default)]`). Empirically validates
    // the mirror invariant ADR-0070-amendment-2 §3 calls out.
    let bytes_without_anchor: Vec<u8> = {
        // Hand-craft a v2-ish blob: source(Atlas, key=0) + size +
        // tint, but NO anchor. Postcard layout is positional; this
        // is exactly what a pre-anchor cooked Sprite would look like
        // on disk.
        let mut v: Vec<u8> = Vec::new();
        v.push(0x00); // SpriteSource::Atlas variant
        v.push(0x00); // key = 0
        v.extend_from_slice(&1.0f32.to_le_bytes());
        v.extend_from_slice(&1.0f32.to_le_bytes()); // size
        v.extend_from_slice(&1.0f32.to_le_bytes());
        v.extend_from_slice(&1.0f32.to_le_bytes());
        v.extend_from_slice(&1.0f32.to_le_bytes());
        v.extend_from_slice(&1.0f32.to_le_bytes()); // tint
        v
    };
    let live_result: Result<Sprite, _> = postcard::from_bytes(&bytes_without_anchor);
    let frozen_result: Result<SpriteV3, _> = postcard::from_bytes(&bytes_without_anchor);
    // Both must agree: either both succeed (postcard supports
    // serde(default)) or both fail (postcard rejects trailing-
    // missing). Today: BOTH fail. ADR-0070-amendment-2 §2 confirms.
    assert_eq!(
        live_result.is_err(),
        frozen_result.is_err(),
        "asymmetric attribute set — live={:?} frozen={:?}",
        live_result.map(|_| "ok"),
        frozen_result.map(|_| "ok"),
    );
}

// ---------------------------------------------------------------------------
// 4. Discriminant byte + total wire length pins
// ---------------------------------------------------------------------------

#[test]
fn v3_fixtures_start_with_zero_discriminant_byte() {
    for &name in CANONICAL_FIXTURES {
        let bytes = read_fixture(name);
        assert!(
            !bytes.is_empty(),
            "{name} is empty — fixture generation regression"
        );
        assert_eq!(
            bytes[0], 0x00,
            "{name} first byte = 0x{:02x}; SpriteVersioned::V3 discriminant must be 0x00 — \
             never insert a variant before V3 in src/sprite_versioned.rs",
            bytes[0]
        );
    }
}

#[test]
fn sprite_versioned_v3_serializes_with_zero_discriminant() {
    let probe = SpriteV3 {
        source: SpriteSource::Atlas { key: 0 },
        size: [1.0, 1.0],
        tint: [1.0, 1.0, 1.0, 1.0],
        anchor: [0.0, 0.0],
        premultiplied: false,
    };
    let bytes = postcard::to_allocvec(&SpriteVersioned::V3(probe)).unwrap();
    assert_eq!(bytes[0], 0x00);
}

#[test]
fn versioned_wrapper_total_wire_length_pin() {
    // Pin the FULL wire format, not just the discriminant byte.
    // 1 (V3 discriminant) + 1 (Atlas variant tag) + 1 (key=0 varint) +
    // 8 (size: 2× f32 LE) + 16 (tint: 4× f32 LE) + 8 (anchor: 2× f32 LE) +
    // 0 (premultiplied: #[serde(skip)]) = 35.
    let bytes = read_fixture("sprite_v3_atlas.postcard");
    assert_eq!(
        bytes.len(),
        CANONICAL_V3_WIRE_LEN,
        "canonical atlas-zero V3 envelope must be {CANONICAL_V3_WIRE_LEN} bytes; \
         drift indicates a wire-format regression"
    );
    // All 5 canonical fixtures share the wire shape (varint widths
    // are 1 byte for the texture_id / key values used; all < 128).
    for &name in CANONICAL_FIXTURES {
        let bytes = read_fixture(name);
        assert_eq!(
            bytes.len(),
            CANONICAL_V3_WIRE_LEN,
            "{name} wire length drifted from {CANONICAL_V3_WIRE_LEN}"
        );
    }
}

#[test]
fn versioned_wrapper_round_trip_preserves_v3() {
    let original = SpriteVersioned::V3(SpriteV3 {
        source: SpriteSource::Atlas { key: 1234 },
        size: [42.0, 17.0],
        tint: [0.125, 0.25, 0.5, 1.0],
        anchor: [1.5, -2.5],
        premultiplied: false,
    });
    let bytes = postcard::to_allocvec(&original)
        .expect("round-trip ser of canonical V3 envelope must always succeed");
    let restored: SpriteVersioned = postcard::from_bytes(&bytes)
        .expect("round-trip de of self-serialized bytes must always succeed");
    assert_eq!(restored, original);
}

// ---------------------------------------------------------------------------
// 5. Fuzzing surface — hostile bytes must not panic / OOM
// ---------------------------------------------------------------------------

#[test]
fn versioned_v3_load_permits_nan_size_for_w1_reject_gate_to_refuse() {
    // W0 stance: postcard load is byte-mechanical; accepts f32 NaN.
    // W1 NaN/Inf reject gate (`sprite_tint_finite_rejects_nan_and_inf`,
    // spec §10.10) is the layer that refuses. Pin the W0 behavior so
    // a future "reject in deserializer" change doesn't silently
    // swallow W1's reject-gate job.
    let mut bytes: Vec<u8> = Vec::new();
    bytes.push(0x00); // V3 discriminant
    bytes.push(0x00); // SpriteSource::Atlas variant
    bytes.push(0x00); // key = 0 (single-byte varint)
    bytes.extend_from_slice(&f32::NAN.to_le_bytes()); // size.x = NaN
    bytes.extend_from_slice(&1.0f32.to_le_bytes()); // size.y = 1.0
    bytes.extend_from_slice(&[0u8; 16]); // tint = [0,0,0,0]
    bytes.extend_from_slice(&[0u8; 8]); // anchor = [0,0]
    let versioned: SpriteVersioned = postcard::from_bytes(&bytes)
        .expect("hostile-bytes load: W0 postcard accepts NaN; W1 reject gate refuses");
    let sprite = expect_v3(versioned);
    assert!(sprite.size[0].is_nan());
    assert_eq!(sprite.size[1], 1.0);
}

#[test]
fn versioned_load_rejects_empty_input() {
    let result: Result<SpriteVersioned, _> = postcard::from_bytes(&[]);
    let err = result.expect_err("empty input must fail cleanly, not return Ok with garbage");
    match err {
        postcard::Error::DeserializeUnexpectedEnd => {}
        other => panic!("expected DeserializeUnexpectedEnd, got {other:?}"),
    }
}

#[test]
fn versioned_load_rejects_truncated_v3_payload() {
    // 1-byte payload: just the V3 discriminant, no source variant.
    let result: Result<SpriteVersioned, _> = postcard::from_bytes(&[0x00]);
    let err = result.expect_err("truncated payload must fail, not return Ok with default-zeroed");
    match err {
        postcard::Error::DeserializeUnexpectedEnd => {}
        other => panic!("expected DeserializeUnexpectedEnd, got {other:?}"),
    }
}

#[test]
fn versioned_load_rejects_unknown_wrapper_discriminant() {
    // 0xFF discriminant: no V255 variant exists. Postcard varints
    // are LEB128; a single 0xFF byte means "more bytes follow"
    // (high bit set) and is also a corrupt enum tag here. Pin the
    // resulting error so a future variant addition (V0xFE+) is a
    // deliberate amendment, not a silent reinterpretation of old
    // corrupt blobs.
    let mut bytes = vec![0xFFu8, 0x01]; // 0x7F (127) varint
    bytes.extend_from_slice(&[0u8; 32]); // padding
    let result: Result<SpriteVersioned, _> = postcard::from_bytes(&bytes);
    assert!(
        result.is_err(),
        "0xFF discriminant must fail — no V127 variant exists, got Ok"
    );
}

#[test]
fn versioned_load_accepts_max_varint_atlas_key() {
    // `SpriteSource::Atlas { key: u32::MAX }` — 5-byte varint width
    // path. Pins that the loader accepts the widest legal key and
    // that the result round-trips. Bandwidth-sensitive: the
    // fixture wire length grows by 4 bytes (35 → 39).
    let probe = SpriteVersioned::V3(SpriteV3 {
        source: SpriteSource::Atlas { key: u32::MAX },
        size: [1.0, 1.0],
        tint: [1.0, 1.0, 1.0, 1.0],
        anchor: [0.0, 0.0],
        premultiplied: false,
    });
    let bytes = postcard::to_allocvec(&probe).unwrap();
    assert_eq!(
        bytes.len(),
        CANONICAL_V3_WIRE_LEN + 4,
        "u32::MAX is 5-byte LEB128 varint (vs 1-byte for small keys); wire = 39 expected"
    );
    let restored: SpriteVersioned = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(restored, probe);
}

#[test]
fn postcard_exact_version_pin_enforced_in_cargo_toml() {
    // HR-5 cross-OS bit-identical contract for v3 postcard fixtures
    // depends on postcard's wire format being frozen. Cargo's caret
    // semver default (`postcard = "1.1.3"`) would let `cargo update`
    // drift to 1.1.4 / 1.2 silently — and `fixtures_match_canonical_serialization`
    // would only catch the drift if the WIRE format actually changed
    // (not if the `=` prefix were removed pre-emptively). This gate
    // pins the EXACT-pin syntax itself, so any agent removing the
    // `=` lands here before they can land it elsewhere.
    let cargo_toml = include_str!("../Cargo.toml");
    assert!(
        cargo_toml.contains(r#"postcard = { version = "=1.1.3""#),
        "postcard exact-pin (`=1.1.3`) lost from crates/ph2d-render/Cargo.toml — \
         HR-5 wire format contract (ADR-0070-amendment-2 §1) requires the exact-pin \
         syntax with the leading `=`. Any bump = regenerate fixtures + cross-OS re-verify \
         + ADR amendment."
    );
}

#[test]
fn versioned_load_silently_ignores_trailing_bytes_after_valid_v3() {
    // Defense-in-depth pin: postcard ≤1.1.3 does NOT cap input
    // length — `from_bytes` consumes a valid V3 prefix and returns
    // `Ok`, silently leaving trailing bytes unparsed. This is the
    // motivation for the per-blob size cap that W1.T1.6 SHOULD
    // enforce at the loader entry point (e.g., `if bytes.len() !=
    // CANONICAL_V3_WIRE_LEN { return Err(...) }`). The W0 stance is
    // to permit the silent-trailing behavior; the W1 cap is the
    // defense layer.
    let canonical = read_fixture("sprite_v3_atlas.postcard");
    let mut padded = canonical.clone();
    padded.extend_from_slice(&[0xAAu8; 100]); // 100 hostile trailing bytes
    let result: Result<SpriteVersioned, _> = postcard::from_bytes(&padded);
    assert!(
        result.is_ok(),
        "W0 stance: postcard accepts valid V3 prefix + trailing bytes silently; W1 \
         migrator MUST add a length cap at the loader entry. Got: {result:?}"
    );
    // Trailing bytes did not corrupt the parsed sprite.
    let sprite = expect_v3(result.unwrap());
    assert_eq!(sprite.source, SpriteSource::Atlas { key: 0 });
}
