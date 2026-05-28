//! Versioned wrapper enum for [`crate::sprite::Sprite`] postcard
//! load/save (ADR-0070 §2.3, Sprite_projeto §10.3).
//!
//! ## Why a wrapper enum (not peek-first-bytes)
//!
//! The current v3 `Sprite` struct does NOT serialize a `version` u32
//! as its first field — postcard writes `source: SpriteSource` first
//! (a `#[repr(Rust)]` enum varint). A naive
//! `postcard::from_bytes::<u32>(&bytes[0..4])` peek returns the source
//! discriminant, not a schema version. Dispatch via wrapper enum is
//! the only type-safe path: postcard serializes an enum as
//! `(variant_discriminant: varint, payload: T)`, so
//! `SpriteVersioned::V3(...)` produces `[0x00, ...v3 bytes...]` and
//! `SpriteVersioned::V4(...)` will produce `[0x01, ...v4 bytes...]`
//! once W1 adds the V4 variant.
//!
//! ## Discriminant stability contract (FROZEN by W0)
//!
//! Postcard uses sequential varint discriminants in declaration order.
//! **Never insert a variant before `V3`** — that would shift the
//! discriminant and invalidate every previously saved postcard blob.
//! Append new versions at the end. The `sprite_versioned_postcard`
//! integration test pins V3's discriminant byte to `0x00`.
//!
//! ## Empirical pivot — wrapper enum is the SOLE back-compat path
//!
//! T0.13 (`tests/sprite_versioned_postcard.rs`) empirically validated
//! that postcard **REJECTS** trailing-missing fields with
//! `Error::DeserializeUnexpectedEnd`; `#[serde(default)]` on trailing
//! fields does not fire because postcard is a non-self-describing
//! positional format. This falsifies the
//! `Sprite_projeto/10_schema_versionamento.md §10.4`
//! "defesa-em-profundidade" claim — under postcard, `#[serde(default)]`
//! is documentary/aspirational only (load-bearing under hypothetical
//! self-describing formats like JSON debug bridges). The wrapper
//! enum dispatch here is the SOLE working back-compat path. The v3
//! → v4 migrator (W1.T1.6) is therefore mandatory.
//! See `tests/sprite_versioned_postcard.rs::postcard_rejects_trailing_serde_default_on_short_payload`
//! for the empirical pin and
//! `docs/architecture/decisions/0070-amendment-2.md` for the ratified
//! spec correction.
//!
//! ## Why `SpriteV3` is a frozen snapshot, not an alias for `Sprite`
//!
//! W1 mutates `crate::sprite::Sprite` from 5 fields → 20 fields (v4).
//! `SpriteV3` here MUST stay byte-identical to the W0 v3 schema
//! forever, so legacy fixtures keep loading correctly via the
//! `V3 → migrate_v3_to_v4(...)` path in W1. Treating it as a frozen
//! mirror struct (instead of `type SpriteV3 = Sprite`) is the only
//! way to preserve that invariant across schema bumps. Lens C drift
//! that aliases the two will reintroduce the tautology this whole
//! task exists to prevent.

use crate::sprite::{Sprite, SpriteSource};

/// Frozen snapshot of the v3 `Sprite` schema (W0 baseline; before
/// the W1 bump to v4). Field order and serde attributes are a
/// **faithful mirror** of [`crate::sprite::Sprite`] at v3 commit
/// time (see `crates/ph2d-render/src/sprite.rs:62`). Adding,
/// removing, or attribute-asymmetrising fields here is a schema
/// regression; the `spritev3_struct_wire_matches_live_sprite_v3`
/// test cross-checks `SpriteV3` against the live `Sprite` v3 via
/// postcard round-trip to catch drift in the W0 → W1 window.
///
/// `premultiplied` matches v3 semantics: `#[serde(skip)]`, so it is
/// never on the wire — it is a runtime hint only. The W1 migrator
/// (T1.6) rebuilds the runtime flag from `SpriteSource::Individual`
/// **plus** [`crate::individual::IndividualTextureStore`] context
/// (only BG-Removal Apply'd individuals are premultiplied; the
/// naive `matches!(source, Individual)` over-triggers).
///
/// `anchor` keeps `#[serde(default)]` as a faithful documentary
/// mirror of the live `Sprite::anchor` attribute set, per
/// ADR-0070-amendment-2 §3. Under postcard the attribute is
/// empirically dead (T0.13 pin); under a hypothetical future
/// self-describing format swap (JSON debug bridge) it would
/// activate. Removing it here would create an asymmetric mirror
/// (live carries the attribute, frozen doesn't) which contradicts
/// the "wire-faithful snapshot" semantic.
///
/// `#[doc(hidden)]` because `SpriteV3` is internal migrator
/// machinery; downstream code constructs and consumes
/// [`SpriteVersioned`] instead.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SpriteV3 {
    pub source: SpriteSource,
    pub size: [f32; 2],
    pub tint: [f32; 4],
    #[serde(default)]
    pub anchor: [f32; 2],
    #[serde(skip)]
    pub premultiplied: bool,
}

/// Canonical W0 v3 fixture set — single source of truth shared by
/// the T0.12 generator and the T0.13 verifier. The
/// `fixtures_match_canonical_serialization` gate asserts these 5
/// instances serialize byte-identically to the committed
/// `tests/fixtures/sprite_v3_*.postcard` blobs.
///
/// `#[doc(hidden)]` because this is testing/migrator scaffolding,
/// not a public sprite-construction surface.
///
/// Coverage rationale (Lens D D25 "3 vs 5" reconciliation):
/// - **atlas / atlas_with_anchor**: Atlas source variant, default
///   and non-default anchor. Exercises the `region_filter_clip =
///   matches!(source, Atlas)` migrator branch (W1.T1.6).
/// - **individual**: Individual source variant with non-default
///   tint alpha. Exercises the `texture_id` opaque-handle round
///   trip and the non-Atlas migrator branch.
/// - **premultiplied**: Individual source with the in-memory
///   `premultiplied = true` runtime hint. Pinned because
///   `#[serde(skip)]` means the flag is absent on the wire; the
///   migrator must rebuild the runtime flag from texture-store
///   context, not from the wire.
/// - **max_size**: extremal-but-finite size + non-default anchor.
///   Exercises 16384²-grid scale, signed anchor offsets, and the
///   `versioned_v3_max_size_carries_extremal_values::is_finite()`
///   gate that precedes the W1 NaN/Inf reject.
#[doc(hidden)]
pub fn canonical_v3_fixtures() -> [(&'static str, SpriteV3); 5] {
    [
        (
            "sprite_v3_atlas.postcard",
            SpriteV3 {
                source: SpriteSource::Atlas { key: 0 },
                size: [10.0, 10.0],
                tint: [1.0, 1.0, 1.0, 1.0],
                anchor: [0.0, 0.0],
                premultiplied: false,
            },
        ),
        (
            "sprite_v3_atlas_with_anchor.postcard",
            SpriteV3 {
                source: SpriteSource::Atlas { key: 7 },
                size: [32.0, 48.0],
                tint: [0.5, 0.75, 0.25, 1.0],
                anchor: [-3.0, 4.5],
                premultiplied: false,
            },
        ),
        (
            "sprite_v3_individual.postcard",
            SpriteV3 {
                source: SpriteSource::Individual { texture_id: 42 },
                size: [128.0, 256.0],
                tint: [1.0, 1.0, 1.0, 0.5],
                anchor: [0.0, 0.0],
                premultiplied: false,
            },
        ),
        (
            "sprite_v3_premultiplied.postcard",
            SpriteV3 {
                source: SpriteSource::Individual { texture_id: 1 },
                size: [64.0, 64.0],
                tint: [1.0, 1.0, 1.0, 1.0],
                anchor: [0.0, 0.0],
                premultiplied: true,
            },
        ),
        (
            "sprite_v3_max_size.postcard",
            // 16384 is the practical max texture dimension on the
            // M-series tier (Vulkan MAX_IMAGE_DIMENSION_2D minimum
            // guarantee). Stresses extract-time `f32 * scale`
            // without going to `f32::INFINITY` (which W1's NaN/Inf
            // reject gate will refuse).
            SpriteV3 {
                source: SpriteSource::Individual { texture_id: 99 },
                size: [16384.0, 16384.0],
                tint: [0.25, 0.5, 0.75, 1.0],
                anchor: [8192.0, -8192.0],
                premultiplied: false,
            },
        ),
    ]
}

/// Versioned dispatch wrapper for postcard load/save. The variant
/// discriminant (postcard varint, sequential by declaration order)
/// IS the schema version on the wire — no separate `version: u32`
/// field is needed, and none exists.
///
/// ## When to use
///
/// - **Postcard save (cooker / scene writer):** wrap a `Sprite` v3 in
///   `SpriteVersioned::V3(...)` before calling
///   `postcard::to_allocvec` so the discriminant byte is on the wire
///   from day one. In W1+ the same pattern wraps a v4 `Sprite` in
///   `V4(...)`.
/// - **Postcard load (cooker / scene loader / migrator):** deserialize
///   to `SpriteVersioned`, then match on the variant. The W1.T1.6
///   `crate::sprite_versioned::load_sprite` helper (ADR-0070-amendment-2
///   §4) is the canonical entry point and dispatches V3 →
///   `migrate_v3_to_v4` → v4 `Sprite` internally.
/// - **Runtime / ECS / shaders:** use `Sprite` directly.
///   `SpriteVersioned` is purely a load/save boundary wrapper; it
///   should never appear in component storage or extract output.
///
/// **Two variants since W1.T1.1** — `V4` wraps the LIVE 20-field
/// [`Sprite`] schema; `V3` keeps wrapping the frozen [`SpriteV3`]
/// mirror so legacy blobs still load. The discriminant bytes are
/// `V3 = 0x00` (frozen) and `V4 = 0x01` (appended). New saves wrap a
/// `Sprite` in `V4(...)`; loads dispatch `V3 → migrate_v3_to_v4 → v4`
/// (W1.T1.6) or `V4 → Sprite` directly.
///
/// `#[non_exhaustive]` is INTENTIONALLY NOT applied: with the
/// migrator (`crate::sprite_versioned::*` consumers + the future
/// W1.T1.6 `migrate_v3_to_v4`) living in this very crate, the
/// attribute would force `_ =>` arms in every match without giving
/// any real forward-compat benefit (postcard varint discriminant is
/// the actual stability contract, pinned by the
/// `v3_fixtures_start_with_zero_discriminant_byte` and
/// `versioned_wrapper_total_wire_length_pin` tests). Appending V4 in
/// W1 stays non-breaking because the discriminant byte 0x00 is
/// frozen; existing V3 fixtures keep loading. Adding a variant
/// BEFORE V3 is forbidden — append-only.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SpriteVersioned {
    /// Legacy v3 payload — 5 fields, 4 on the wire (`premultiplied`
    /// is `#[serde(skip)]`). Discriminant byte 0x00.
    V3(SpriteV3),
    /// Current v4 payload — the live [`Sprite`] schema (20 fields, 18
    /// on the wire: `premultiplied` is `#[serde(skip)]`). Discriminant
    /// byte 0x01, appended after `V3` so the frozen 0x00 fixtures keep
    /// loading. Never reorder these variants (postcard discriminants
    /// are positional — see module docs).
    V4(Sprite),
}
