//! Arch-gate for the Sprite Inspector v2 numeric caps (Sprite_projeto
//! §11.2.1 / §10.11 / ADR-0069..0071). Any bump here requires an
//! ADR-0070-amendment-N (or -0071 for tint channels) + an impact review.
//!
//! The field counts are enforced at COMPILE TIME by exhaustive struct
//! destructuring: adding or removing a field breaks the pattern (a
//! non-exhaustive / unknown-field error), forcing the list AND the
//! asserted count to move in lockstep. The returned literal is only the
//! number the assertion reports.

use ph2d_render::{RenderInstance, Sprite};

/// Exactly the 20 v4 `Sprite` fields (5 v3 + 14 new + 1 `version`).
/// The destructure is the real gate; the `20` is for the message.
fn sprite_struct_field_count() -> usize {
    let Sprite {
        version: _,
        source: _,
        size: _,
        tint: _,
        anchor: _,
        premultiplied: _,
        self_tint: _,
        per_corner_tint: _,
        tint_fill: _,
        opacity: _,
        flip_x: _,
        flip_y: _,
        centered: _,
        offset: _,
        hframes: _,
        vframes: _,
        frame: _,
        region_enabled: _,
        region_rect: _,
        region_filter_clip: _,
    } = Sprite::atlas(0, [1.0, 1.0], [1.0; 4]);
    20
}

/// Exactly the 16 v4 `RenderInstance` fields (11 GPU vertex attrs —
/// per_corner_tint spans 4, + uv_xform from amendment-6 — + 5 CPU-only:
/// texture_id / z_order / sampling / clip_group / clip_meta).
/// Destructure enforces it; `16` is for the message.
fn render_instance_field_count() -> usize {
    use bytemuck::Zeroable;
    let RenderInstance {
        world_pos: _,
        size: _,
        atlas_uv: _,
        tint: _,
        basis: _,
        premultiplied: _,
        anchor: _,
        per_corner_tint: _,
        opacity: _,
        flip_uv: _,
        texture_id: _,
        z_order: _,
        sampling: _,
        uv_xform: _,
        clip_group: _,
        clip_meta: _,
    } = RenderInstance::zeroed();
    16
}

#[test]
fn sprite_struct_field_count_capped() {
    assert_eq!(
        sprite_struct_field_count(),
        20,
        "Sprite v4 struct field count must be exactly 20 (FROZEN by ADR-0070). \
         A 21st field = re-think (a new ECS Component is usually the right path, \
         anatomia §1.6 / §10.11)."
    );
}

#[test]
fn render_instance_field_count_capped() {
    assert_eq!(
        render_instance_field_count(),
        16,
        "RenderInstance v4 field count must be exactly 16 (amendment-7 added CPU-only \
         `clip_group: u32` + `clip_meta: u32` for ClipChildren stencil grouping; \
         amendment-5 added CPU-only `sampling: u32`; amendment-6 added GPU \
         `uv_xform: [f32;4]`). flip_uv is a packed flags word (amendment-3/-6: bits3-4 = \
         repeat) — pack new per-instance GPU bools into its reserved bits before adding a \
         vertex attribute. New CPU-only metadata goes in the tail like clip_group/clip_meta."
    );
}

#[test]
fn render_instance_pod_size_capped() {
    assert_eq!(
        std::mem::size_of::<RenderInstance>(),
        184,
        "RenderInstance ABI is 184 bytes (amendment-7: +CPU-only `clip_group` + `clip_meta`, \
         GPU vertex layout unchanged at 164 B / 12 attrs; amendment-6: +GPU `uv_xform`; \
         amendment-5: +CPU-only `sampling`). A bump requires ADR-0070-amendment-N + a \
         re-bench of sprites_upload_144b_vs_72b."
    );
}

#[test]
fn sprite_schema_version_v4() {
    assert_eq!(Sprite::VERSION, 4, "Sprite schema version is v4 this wave.");
}

/// The 4 canonical multiplicative tint channels (anatomia §4.1/§4.10):
/// tint (cascades), self_tint (local), per_corner_tint (vertex gradient),
/// opacity (final multiplier). Touching each field by name makes a rename
/// or removal a COMPILE error — so this is a real gate on the channel set,
/// not a vacuous `assert_eq!(4, 4)`. Bump = ADR-0071-amendment-N.
fn tint_channel_count() -> usize {
    let s = Sprite::atlas(0, [1.0, 1.0], [1.0; 4]);
    let _tint = s.tint;
    let _self_tint = s.self_tint;
    let _per_corner_tint = s.per_corner_tint;
    let _opacity = s.opacity;
    4
}

#[test]
fn tint_channel_count_capped() {
    assert_eq!(
        tint_channel_count(),
        4,
        "the 4 multiplicative tint channels (tint/self_tint/per_corner_tint/opacity) \
         are FROZEN — not 3 (missing self_tint), not 5. Bump = ADR-0071-amendment-N."
    );
}
