use super::*;

#[test]
fn render_instance_is_pod_compatible() {
    let inst = RenderInstance {
        world_pos: [1.0, 2.0],
        size: [10.0, 10.0],
        atlas_uv: [0.0, 0.0, 0.25, 0.25],
        tint: [1.0, 1.0, 1.0, 1.0],
        basis: RenderInstance::IDENTITY_BASIS,
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: 0,
        texture_id: RenderInstance::ATLAS_TEXTURE_ID,
        z_order: 0,
        sampling: RenderInstance::SAMPLING_DEFAULT,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
    };
    let bytes: &[u8] = bytemuck::bytes_of(&inst);
    assert_eq!(bytes.len(), std::mem::size_of::<RenderInstance>());
    // GPU fields = 76 bytes (world_pos 8 + size 8 + atlas_uv 16 +
    // tint 16 + basis 16 + premultiplied 4 + anchor 8).
    // + per_corner_tint [[f32;4];4] (64) + opacity f32 (4) +
    // flip_uv u32 (4) + uv_xform [f32;4] (16) = +88 → 164 GPU bytes.
    // + texture_id u32 (4) + z_order u32 (4) + sampling u32 (4)
    // CPU-only = 176 bytes. ADR-0070-amendment-7 adds the CPU-only
    // `clip_group: u32` + `clip_meta: u32` (+8 → 184 B); the GPU
    // vertex layout (164 B / 12 attrs) is unchanged.
    assert_eq!(bytes.len(), 184);
}

#[test]
fn vertex_attributes_cover_full_stride() {
    // uv_xform is the last (12th) vertex attribute, at location 15
    // (ADR-0070-amendment-6). Confirm the attribute array's last
    // offset+size lands inside the Pod stride so the vertex layout
    // doesn't read past the instance buffer.
    let attrs = RenderInstance::VERTEX_ATTRIBUTES;
    let last = attrs.last().expect("at least one attribute");
    assert_eq!(last.shader_location, 15, "uv_xform is @location(15)");
    // Float32x4 == 16 bytes.
    let end = last.offset + 16;
    assert!(
        end <= std::mem::size_of::<RenderInstance>() as u64,
        "attr end {end} must fit in stride {}",
        std::mem::size_of::<RenderInstance>()
    );
}

#[test]
fn vertex_attr_offsets_match_struct() {
    // REGRESSION GUARD. `wgpu::vertex_attr_array!` derives each
    // attribute's byte offset from the running sum of the FORMATS
    // listed — it knows nothing about the Rust struct. So every
    // GPU-read field MUST sit contiguously, in attribute order,
    // before any CPU-only field (`texture_id`, `z_order`). If a future
    // edit interleaves a non-attribute field (as the original
    // `premultiplied` placement did — it sat after `texture_id`, so
    // `@location(7)` silently read `texture_id`'s bytes), this test
    // fails instead of shipping a sampler that reads the wrong word.
    //
    // v4 (Sprite Inspector v2) appended per_corner_tint (4 attrs,
    // @location 9..12), opacity (@13), flip_uv (@14) — still all
    // before the CPU-only tail. 11 attrs total.
    use std::mem::offset_of;
    let expect = [
        (2u32, offset_of!(RenderInstance, world_pos) as u64),
        (3, offset_of!(RenderInstance, size) as u64),
        (4, offset_of!(RenderInstance, atlas_uv) as u64),
        (5, offset_of!(RenderInstance, tint) as u64),
        (6, offset_of!(RenderInstance, basis) as u64),
        (7, offset_of!(RenderInstance, premultiplied) as u64),
        (8, offset_of!(RenderInstance, anchor) as u64),
        // per_corner_tint is one [[f32;4];4] field but FOUR vertex
        // attrs; the macro lays them at +0/+16/+32/+48 from the field
        // base, so locations 9..12 map to the field offset plus the
        // per-corner stride. offset_of! gives the field base for @9;
        // @10..12 follow contiguously (checked by the macro's own sum
        // matching, asserted below via the running offsets).
        (9, offset_of!(RenderInstance, per_corner_tint) as u64),
        (10, offset_of!(RenderInstance, per_corner_tint) as u64 + 16),
        (11, offset_of!(RenderInstance, per_corner_tint) as u64 + 32),
        (12, offset_of!(RenderInstance, per_corner_tint) as u64 + 48),
        (13, offset_of!(RenderInstance, opacity) as u64),
        (14, offset_of!(RenderInstance, flip_uv) as u64),
        (15, offset_of!(RenderInstance, uv_xform) as u64),
    ];
    let attrs = RenderInstance::VERTEX_ATTRIBUTES;
    assert_eq!(attrs.len(), expect.len(), "attribute count drifted");
    for (attr, (loc, off)) in attrs.iter().zip(expect) {
        assert_eq!(attr.shader_location, loc, "location order drifted");
        assert_eq!(
            attr.offset, off,
            "@location({loc}) macro offset {} != struct field offset {off} \
             — a non-attribute field is interleaved with GPU fields",
            attr.offset
        );
    }
}

#[test]
fn collapsed_tint_folds_self_tint_into_tint() {
    // Identity case: default self_tint = WHITE → tint unchanged
    // (zero-regression for v3-migrated sprites).
    let s = Sprite::atlas(0, [1.0, 1.0], [0.4, 0.5, 0.6, 0.7]);
    assert_eq!(s.collapsed_tint(), [0.4, 0.5, 0.6, 0.7]);

    // Non-identity: per-component multiply of tint × self_tint.
    let mut s = Sprite::atlas(0, [1.0, 1.0], [0.5, 1.0, 0.2, 1.0]);
    s.self_tint = [0.5, 0.5, 1.0, 0.8];
    assert_eq!(s.collapsed_tint(), [0.25, 0.5, 0.2, 0.8]);

    // Each channel is independent (no cross-channel bleed).
    let mut s = Sprite::atlas(0, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]);
    s.tint = [2.0, 0.0, 0.0, 0.0];
    s.self_tint = [0.5, 7.0, 9.0, 9.0];
    assert_eq!(s.collapsed_tint(), [1.0, 0.0, 0.0, 0.0]);
}

#[test]
fn resolve_anchor_centered_default_is_unchanged() {
    // The common case (centered, no offset, no tool pivot) must be
    // bit-identical to raw `anchor` so every legacy sprite renders
    // and hit-tests exactly as before.
    let s = Sprite::atlas(0, [2.0, 3.0], [1.0; 4]);
    assert_eq!(s.resolve_anchor(100.0), [0.0, 0.0]);

    // A tool-set anchor passes through untouched when centered.
    let mut s = Sprite::atlas(0, [2.0, 3.0], [1.0; 4]);
    s.anchor = [0.25, -0.5];
    assert_eq!(s.resolve_anchor(100.0), [0.25, -0.5]);
}

#[test]
fn resolve_anchor_uncentered_puts_pivot_at_top_left() {
    // centered=false → origin at the texture top-left, so the quad
    // CENTER sits +half-width right and half-height DOWN (-y local).
    let mut s = Sprite::atlas(0, [2.0, 4.0], [1.0; 4]);
    s.centered = false;
    assert_eq!(s.resolve_anchor(100.0), [1.0, -2.0]);
}

#[test]
fn resolve_anchor_offset_is_pixels_over_ppm_with_godot_y_down() {
    // offset is intrinsic px (Godot +x right / +y down); local frame
    // is Y-up so +y offset maps to -y. 50 px @ 100 px/m = 0.5 m.
    let mut s = Sprite::atlas(0, [2.0, 2.0], [1.0; 4]);
    s.offset = [50.0, 100.0];
    assert_eq!(s.resolve_anchor(100.0), [0.5, -1.0]);
}

#[test]
fn resolve_anchor_composes_anchor_centered_and_offset_additively() {
    let mut s = Sprite::atlas(0, [2.0, 2.0], [1.0; 4]);
    s.anchor = [0.1, 0.2];
    s.centered = false; // +[1.0, -1.0]
    s.offset = [100.0, 0.0]; // +[1.0, 0.0] @ 100 ppm
    assert_eq!(s.resolve_anchor(100.0), [0.1 + 1.0 + 1.0, 0.2 - 1.0]);
}

#[test]
fn resolve_anchor_guards_nonpositive_ppm() {
    // A zero/garbage ppm must not NaN/inf the quad off-screen.
    let mut s = Sprite::atlas(0, [2.0, 2.0], [1.0; 4]);
    s.offset = [10.0, 0.0];
    let a = s.resolve_anchor(0.0);
    assert!(a[0].is_finite() && a[1].is_finite());
}

#[test]
fn flip_uv_flag_bits_roundtrip() {
    // Pin the ADR-0070-amendment-3 bit layout (the Rust ENCODE side):
    // a future edit can't silently re-map flip_x/flip_y/tint_fill in
    // `pack_flip_flags`. NOTE: this does NOT mechanically guard the
    // WGSL decode literals (`& 1u`/`& 2u`/`& 4u` in shaders/sprite.wgsl)
    // — those are hand-kept in lockstep and verified by the headless
    // pipeline validation (`pipeline::tests`) + the Enio GPU smoke.
    assert_eq!(RenderInstance::FLIP_X_BIT, 1);
    assert_eq!(RenderInstance::FLIP_Y_BIT, 2);
    assert_eq!(RenderInstance::TINT_FILL_BIT, 4);

    assert_eq!(RenderInstance::pack_flip_flags(false, false, false), 0);
    assert_eq!(RenderInstance::pack_flip_flags(true, false, false), 0b001);
    assert_eq!(RenderInstance::pack_flip_flags(false, true, false), 0b010);
    assert_eq!(RenderInstance::pack_flip_flags(false, false, true), 0b100);
    assert_eq!(RenderInstance::pack_flip_flags(true, true, true), 0b111);
    // Each flag is independent — no bit bleeds into another.
    let f = RenderInstance::pack_flip_flags(true, false, true);
    assert_ne!(f & RenderInstance::FLIP_X_BIT, 0);
    assert_eq!(f & RenderInstance::FLIP_Y_BIT, 0);
    assert_ne!(f & RenderInstance::TINT_FILL_BIT, 0);
    // Reserved bits 3..31 stay zero for any input.
    assert_eq!(
        RenderInstance::pack_flip_flags(true, true, true) & !0b111u32,
        0
    );
}

#[test]
fn blend_bits_round_trip_without_clobbering_flip_repeat() {
    for tag in 0u8..=5 {
        let packed = RenderInstance::pack_blend_bits(tag);
        assert_eq!(RenderInstance::unpack_blend(packed), tag);
        // Blend bits live above the flip/tint (0-2) and repeat (3-4) bits.
        assert_eq!(packed & 0b1_1111, 0, "blend must not touch bits 0-4");
    }
    // Coexists with flip + repeat in one word, each decodes independently.
    let word = RenderInstance::pack_flip_flags(true, false, true)
        | RenderInstance::pack_repeat_bits(3)
        | RenderInstance::pack_blend_bits(4);
    assert_ne!(word & RenderInstance::FLIP_X_BIT, 0);
    assert_eq!((word >> RenderInstance::REPEAT_SHIFT) & 0b11, 3);
    assert_eq!(RenderInstance::unpack_blend(word), 4);
}

#[test]
fn sprite_constructors_default_straight_alpha() {
    assert!(!Sprite::atlas(0, [1.0, 1.0], [1.0; 4]).premultiplied);
    assert!(!Sprite::individual(1, [1.0, 1.0], [1.0; 4]).premultiplied);
}

#[test]
fn sprite_source_atlas_zero_const() {
    assert_eq!(SpriteSource::ATLAS_ZERO, SpriteSource::Atlas { key: 0 });
}

#[test]
fn sprite_atlas_constructor_round_trip() {
    let s = Sprite::atlas(7, [1.0, 2.0], [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(s.source, SpriteSource::Atlas { key: 7 });
    assert_eq!(s.size, [1.0, 2.0]);
}

#[test]
fn sprite_individual_constructor_round_trip() {
    let s = Sprite::individual(42, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(s.source, SpriteSource::Individual { texture_id: 42 });
}

#[test]
fn quad_strip_winding() {
    // Triangle strip [0,1,2] then [1,3,2] → both CCW when viewed
    // from +Z (Y-up world space). Just sanity that vertex order
    // matches what the shader expects.
    assert_eq!(QuadVertex::QUAD_STRIP.len(), 4);
}

#[test]
fn quad_strip_uv_natural_mapping() {
    // M14.4e v2: camera projection no longer Y-flips, so the UV
    // mapping is straightforward — world-up vertex (pos.y > 0)
    // samples texture top (V=0), world-down samples texture
    // bottom (V=1). X is straight (no flip).
    for v in QuadVertex::QUAD_STRIP {
        let expected_v = if v.pos[1] < 0.0 { 1.0 } else { 0.0 };
        assert_eq!(
            v.uv[1], expected_v,
            "pos {:?} expected V={expected_v} got V={}",
            v.pos, v.uv[1]
        );
        let expected_u = if v.pos[0] < 0.0 { 0.0 } else { 1.0 };
        assert_eq!(v.uv[0], expected_u);
    }
}
