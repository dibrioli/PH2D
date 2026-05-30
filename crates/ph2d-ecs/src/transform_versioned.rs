//! Versioned wrapper enum for [`crate::Transform`] postcard
//! load/save (ADR-0025-amendment-1 §2.3, mirrors ADR-0070 §2.3 for
//! `Sprite`).
//!
//! ## Why a wrapper enum (not peek-first-bytes or `#[serde(default)]`)
//!
//! `Transform` serializes its fields positionally (`translation`,
//! `rotation`, `scale`, then the v2 `skew_x`/`skew_y`). Postcard is a
//! non-self-describing format: a v1 blob ends right after `scale`, so
//! a direct `postcard::from_bytes::<Transform>(v1_bytes)` returns
//! `Error::DeserializeUnexpectedEnd` when it reaches for `skew_x` —
//! `#[serde(default)]` does NOT fire on trailing-missing fields under
//! postcard (empirically pinned for `Sprite` in
//! `ph2d-render/tests/sprite_versioned_postcard.rs`, ratified in
//! ADR-0070-amendment-2 §2; the same wire semantics apply here).
//!
//! Dispatch via this wrapper enum is the only type-safe back-compat
//! path: postcard serializes an enum as `(variant_discriminant:
//! varint, payload)`, so [`TransformVersioned::V1`] produces
//! `[0x00, ...v1 bytes...]` and [`TransformVersioned::V2`] produces
//! `[0x01, ...v2 bytes...]`.
//!
//! ## Discriminant stability contract (FROZEN)
//!
//! Postcard uses sequential varint discriminants in declaration order.
//! **Never insert a variant before `V1`** — that shifts every
//! discriminant and invalidates previously saved blobs. Append new
//! versions at the end. `tests/transform_versioned_postcard.rs` pins
//! V1's discriminant byte to `0x00` and V2's to `0x01`.
//!
//! ## Why `TransformV1` is a frozen snapshot, not an alias
//!
//! The v2 bump grows `crate::Transform` from 3 fields → 5. `TransformV1`
//! here MUST stay byte-identical to the v1 wire schema forever so
//! legacy fixtures keep loading via `V1 → migrate_v1_to_v2(...)`.
//! Treating it as a frozen mirror struct (not `type TransformV1 =
//! Transform`) is the only way to preserve that invariant across the
//! bump.
//!
//! ## Wiring status (READ before assuming this is the live path)
//!
//! [`load_transform`] / [`save_transform`] are the **canonical
//! versioned (de)serialization machinery** and the entry point any
//! on-disk asset-versioning layer MUST use. They are NOT yet on the
//! live save/load path: the [`crate::scene::ComponentRegistry`] and
//! the asset cooker currently serialize `Transform` **bare**
//! (`postcard::{to_allocvec, from_bytes}::<Transform>`), with no
//! version discriminant. This is intentional and lossless for the
//! current greenfield phase — no v1 `Transform` assets exist on disk,
//! and bare-write ↔ bare-read is self-consistent.
//!
//! Two consequences to keep in mind for the NEXT bump:
//! 1. The wrapper enum gives clean back-compat for **future** bumps
//!    (v2 → v3 …) once the registry/cooker route through it, because
//!    every saved blob will then carry a discriminant.
//! 2. It does NOT rescue a hypothetical **bare** v1 blob (3 fields, no
//!    discriminant): the wrapper would misread the first float byte as
//!    a variant tag. A bare-v1 → v2 loader would need a "try wrapper,
//!    else `migrate_v1_to_v2(postcard::from_bytes::<TransformV1>())`"
//!    fallback. Moot today (no such assets), flagged for honesty.
//!
//! `Sprite` ([`ph2d-render`] `sprite_versioned`) shares this exact
//! posture as a FROZEN contract (ADR-0070). Promoting either to the
//! live registry path is a cross-cutting foundational change (a
//! `VersionedComponent` registry hook adopted by ALL components, so
//! Transform and Sprite stay symmetric) and belongs in its own ADR —
//! not a unilateral, asymmetric wiring of one component. Tracked as a
//! foundational follow-up.
//!
//! ## Skew clamp contract (§2.5/§6)
//!
//! Skew is range-clamped to [`Transform::SKEW_LIMIT`] at the
//! **authoring boundary** ([`Transform::with_skew`] and the Inspector
//! Skew X/Y commit, W2.T2.3). Deserialized skew is trusted: it is
//! finite (enforced by the `is_finite` debug-asserts in `compose` /
//! `from_transform`) and `libm::tanf` is finite across the entire f32
//! domain (even `tanf(π/2)` is ~−2.3e7, never ±∞), so an unclamped
//! deserialized value yields a valid — if extreme — affine, never a
//! poisoned matrix. The clamp is a UX sanity bound, not a NaN gate.

use ph2d_core::Vec2;
use serde::{Deserialize, Serialize};

use crate::Transform;

/// Frozen snapshot of the v1 `Transform` schema (3 fields, before the
/// ADR-0025-amendment-1 bump that added `skew_x`/`skew_y`). Field
/// order and serde shape are a faithful mirror of the v1
/// `crate::Transform`. Adding/removing/reordering fields here is a
/// schema regression; the `fixtures_match_canonical_serialization`
/// gate cross-checks against the committed `.postcard` blobs.
///
/// `#[doc(hidden)]` because this is internal migrator machinery —
/// downstream code constructs and consumes [`TransformVersioned`].
#[doc(hidden)]
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransformV1 {
    pub translation: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

/// Versioned wrapper for [`Transform`] save/load. Variants are
/// append-only; discriminants are positional and FROZEN (see module
/// docs). `V2` carries the live [`Transform`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TransformVersioned {
    /// v1 — 3-field `(translation, rotation, scale)`. Discriminant `0x00`.
    V1(TransformV1),
    /// v2 — live `Transform` with `skew_x`/`skew_y`. Discriminant `0x01`.
    V2(Transform),
}

/// Migrate a v1 `Transform` to v2 by defaulting the new skew axes to
/// `0.0` (identity shear) — the only benign default, since v1 scenes
/// had no skew concept.
pub fn migrate_v1_to_v2(v1: TransformV1) -> Transform {
    Transform {
        translation: v1.translation,
        rotation: v1.rotation,
        scale: v1.scale,
        skew_x: 0.0,
        skew_y: 0.0,
    }
}

/// Load a `Transform` from postcard bytes via the versioned wrapper,
/// migrating v1 → v2 transparently. This is the canonical back-compat
/// entry point; never `postcard::from_bytes::<Transform>` a stored
/// blob directly (it cannot read v1).
pub fn load_transform(bytes: &[u8]) -> Result<Transform, postcard::Error> {
    let versioned: TransformVersioned = postcard::from_bytes(bytes)?;
    Ok(match versioned {
        TransformVersioned::V1(v1) => migrate_v1_to_v2(v1),
        TransformVersioned::V2(v2) => v2,
    })
}

/// Serialize a `Transform` as a versioned wrapper (`V2`). Use this for
/// all new saves so the wire is self-identifying for the next bump.
pub fn save_transform(transform: &Transform) -> Vec<u8> {
    postcard::to_allocvec(&TransformVersioned::V2(*transform))
        .expect("postcard serialize of an in-memory Transform cannot fail")
}

/// Canonical v1 fixture set — single source of truth shared by the
/// fixture generator (`tests/generate_transform_v1_fixtures.rs`) and
/// the verifier (`tests/transform_versioned_postcard.rs`). The
/// `fixtures_match_canonical_serialization` gate asserts these three
/// instances serialize byte-identically to the committed
/// `tests/fixtures/transform_v1_*.postcard` blobs.
///
/// Coverage (ADR-0025-amendment-1 §2.5 "3 fixtures v1 binárias"):
/// - **translation**: pure translation, identity rotation/scale —
///   the most common Transform shape on disk.
/// - **rotation**: non-trivial rotation (exercises the `sincosf`
///   determinism path through the migrator's round trip).
/// - **full**: translation + rotation + non-uniform scale — the
///   maximal v1 shape; pins every field's wire position.
///
/// `#[doc(hidden)]` — testing/migrator scaffolding, not a public API.
#[doc(hidden)]
pub fn canonical_v1_fixtures() -> [(&'static str, TransformV1); 3] {
    [
        (
            "transform_v1_translation.postcard",
            TransformV1 {
                translation: Vec2::new(10.0, -5.0),
                rotation: 0.0,
                scale: Vec2::new(1.0, 1.0),
            },
        ),
        (
            "transform_v1_rotation.postcard",
            TransformV1 {
                translation: Vec2::new(0.0, 0.0),
                rotation: std::f32::consts::FRAC_PI_4,
                scale: Vec2::new(1.0, 1.0),
            },
        ),
        (
            "transform_v1_full.postcard",
            TransformV1 {
                translation: Vec2::new(3.5, 7.25),
                rotation: 1.2,
                scale: Vec2::new(2.0, 0.5),
            },
        ),
    ]
}

/// Three canonical `compose` cases (rotation+skew, scale+skew-only,
/// full T·R·Sk·S) whose `postcard` + `blake3` hash is the cross-OS
/// determinism golden (ADR-0025-amendment-1 §2.4). Single source of
/// truth shared by the `generate_transform_v1_fixtures` generator and
/// the `transform_versioned_postcard` verifier — both hash THIS array
/// and compare against the committed
/// `tests/fixtures/transform_skew_compose.expected`. Because `compose`
/// routes skew through `libm::tanf`, the bytes are bit-identical on
/// every host, so the golden file is OS-independent.
///
/// `#[doc(hidden)]` — testing/determinism scaffolding, not public API.
#[doc(hidden)]
pub fn skew_compose_cases() -> [Transform; 3] {
    [
        Transform::compose(
            Transform {
                translation: Vec2::new(10.0, 20.0),
                rotation: std::f32::consts::FRAC_PI_4,
                scale: Vec2::new(2.0, 1.5),
                skew_x: 0.1,
                skew_y: 0.0,
            },
            Transform {
                translation: Vec2::new(5.0, -3.0),
                rotation: 0.2,
                scale: Vec2::new(0.5, 0.5),
                skew_x: 0.0,
                skew_y: 0.05,
            },
        ),
        Transform::compose(
            Transform {
                translation: Vec2::new(-7.0, 4.0),
                rotation: 0.0,
                scale: Vec2::new(3.0, 0.25),
                skew_x: -0.3,
                skew_y: 0.15,
            },
            Transform::IDENTITY,
        ),
        Transform::compose(
            Transform {
                translation: Vec2::new(1.25, -8.5),
                rotation: 1.2,
                scale: Vec2::new(1.0, 1.0),
                skew_x: 0.4,
                skew_y: -0.4,
            },
            Transform {
                translation: Vec2::new(2.0, 2.0),
                rotation: -0.6,
                scale: Vec2::new(1.5, 2.5),
                skew_x: 0.2,
                skew_y: 0.1,
            },
        ),
    ]
}
