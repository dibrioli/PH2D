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

/// Exactly the 13 v4 `RenderInstance` fields (10 GPU vertex attrs —
/// counting per_corner_tint as one field that spans 4 attrs — + 3
/// CPU-only: texture_id / z_order / sampling). Destructure enforces it;
/// `13` is for the message.
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
    } = RenderInstance::zeroed();
    13
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
        13,
        "RenderInstance v4 field count must be exactly 13 (ADR-0070-amendment-5 added \
         the CPU-only `sampling: u32` for per-node TextureFilter/Repeat draw grouping). \
         flip_uv is a packed flags word (ADR-0070-amendment-3) — pack new per-instance \
         GPU bools into its reserved bits instead of adding a vertex attribute."
    );
}

#[test]
fn render_instance_pod_size_capped() {
    assert_eq!(
        std::mem::size_of::<RenderInstance>(),
        160,
        "RenderInstance ABI is 160 bytes (ADR-0070-amendment-5: +CPU-only `sampling: u32`; \
         GPU vertex layout unchanged at 148 B). amendment-4: `rotation: f32` → \
         `basis: [f32; 4]` so the shader renders skew as a true parallelogram instead \
         of decomposing to a lossy rotation scalar). Was 144 at v4 freeze."
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
