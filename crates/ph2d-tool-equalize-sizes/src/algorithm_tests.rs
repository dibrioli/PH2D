//! Testes de `algorithm.rs` — irmão por tecto de LOC (`workspace_src_files_under_loc_cap`).
//!
//! Corte mecânico: o `mod tests` saiu inteiro, verbatim, do ficheiro que o continha.

use super::*;

/// A `w`×`h` canvas where every pixel is `[r,g,b,255]`.
fn solid(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
    let mut v = Vec::with_capacity((w * h * 4) as usize);
    for _ in 0..(w * h) {
        v.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
    }
    v
}

fn sprite(w: u32, h: u32, sx: f32, sy: f32) -> SpriteInput {
    SpriteInput {
        rgba: solid(w, h, [10, 20, 30]),
        width: w,
        height: h,
        scale_x: sx,
        scale_y: sy,
    }
}

// ⚠️ The EPX gates (the Scale2x oracle, the not-an-alias gate) live
// in `algorithm_epx.rs`, next to the kernel they measure.

#[test]
fn empty_input_returns_empty_output() {
    let p = EqualizeSizesParams::default();
    assert!(run_equalize_sizes(&[], &p).is_empty());
}

#[test]
fn max_of_selection_picks_largest_visual_dim() {
    let p = EqualizeSizesParams::default();
    let inputs = vec![sprite(64, 64, 1.0, 1.0), sprite(32, 32, 2.0, 2.0)];
    // Visual: 64x64 and 64x64 — both are 64; target = (64,64).
    assert_eq!(compute_global_target(&inputs, &p), Some((64, 64)));

    // Add one larger sprite → target grows to its visual size.
    let inputs = vec![
        sprite(64, 64, 1.0, 1.0),
        sprite(32, 32, 2.0, 2.0),
        sprite(100, 50, 1.5, 1.0),
    ];
    assert_eq!(compute_global_target(&inputs, &p), Some((150, 64)));
}

#[test]
fn fixed_mode_uses_typed_dims() {
    let mut p = EqualizeSizesParams::default();
    p.target_mode = TargetMode::Fixed;
    p.fixed_w = 200;
    p.fixed_h = 100;
    let inputs = vec![sprite(64, 64, 1.0, 1.0)];
    assert_eq!(compute_global_target(&inputs, &p), Some((200, 100)));
}

#[test]
fn grid_uniform_target_is_cell_minus_offset_both_axes() {
    // cell 64, offset 0 → (64, 64).
    assert_eq!(grid_uniform_target(64, 0), (64, 64));
    // cell 64, offset 8 → (56, 56) — uniform reduction, both axes.
    assert_eq!(grid_uniform_target(64, 8), (56, 56));
    // Offset capped at cell/2 silently (caller's clamp).
    assert_eq!(grid_uniform_target(32, 99), (16, 16));
    // grid 0 (degenerate) → at least (1, 1) so no zero-dim texture.
    assert_eq!(grid_uniform_target(0, 0), (1, 1));
}

#[test]
fn grid_mode_shrinks_sprites_to_cell_minus_offset() {
    let mut p = EqualizeSizesParams::default();
    p.target_mode = TargetMode::GridUnit;
    p.grid_unit = 64;
    p.grid_offset = 8;
    p.rasterize_after = true;
    let inputs = vec![
        // Big sprite (128 visual) should shrink to (56, 56).
        sprite(128, 128, 1.0, 1.0),
        // Tiny sprite (8 visual) should grow to (56, 56) too —
        // uniform target across the selection (legacy semantics).
        sprite(8, 8, 1.0, 1.0),
    ];
    let out = run_equalize_sizes(&inputs, &p);
    assert_eq!(out.len(), 2);
    assert_eq!((out[0].width, out[0].height), (56, 56));
    assert_eq!((out[1].width, out[1].height), (56, 56));
}

#[test]
fn rasterize_after_resamples_to_target_and_resets_scale() {
    let mut p = EqualizeSizesParams::default();
    p.rasterize_after = true;
    p.target_mode = TargetMode::Fixed;
    p.fixed_w = 32;
    p.fixed_h = 32;
    let inputs = vec![sprite(64, 64, 1.0, 1.0)];
    let out = run_equalize_sizes(&inputs, &p);
    assert_eq!(out.len(), 1);
    let o = &out[0];
    assert_eq!((o.width, o.height), (32, 32));
    assert!((o.new_scale_x.abs() - 1.0).abs() < 1e-4);
    assert!((o.new_scale_y.abs() - 1.0).abs() < 1e-4);
    assert!(o.changed);
    assert_eq!(o.rgba.len(), 32 * 32 * 4);
}

#[test]
fn fit_by_scale_keeps_buffer_changes_scale_only() {
    let mut p = EqualizeSizesParams::default();
    p.rasterize_after = false;
    p.target_mode = TargetMode::Fixed;
    p.fixed_w = 128;
    p.fixed_h = 64;
    let inputs = vec![sprite(64, 64, 1.0, 1.0)];
    let out = run_equalize_sizes(&inputs, &p);
    let o = &out[0];
    // Buffer unchanged (64x64), scale rewritten so visual is 128x64.
    assert_eq!((o.width, o.height), (64, 64));
    assert!((o.new_scale_x - 2.0).abs() < 1e-4);
    assert!((o.new_scale_y - 1.0).abs() < 1e-4);
    assert!(o.changed);
}

#[test]
fn flip_sign_is_preserved() {
    let mut p = EqualizeSizesParams::default();
    p.rasterize_after = true;
    p.target_mode = TargetMode::Fixed;
    p.fixed_w = 16;
    p.fixed_h = 16;
    let inputs = vec![sprite(8, 8, -1.0, 1.0)];
    let out = run_equalize_sizes(&inputs, &p);
    assert!(out[0].new_scale_x < 0.0, "horizontal flip must survive");
    assert!(out[0].new_scale_y > 0.0);
}

#[test]
fn upscale_if_smaller_grows_source_buffer_first() {
    let mut p = EqualizeSizesParams::default();
    p.upscale_if_smaller = true;
    p.upscale_algorithm = UpscaleAlgorithm::Nearest;
    p.rasterize_after = false;
    p.target_mode = TargetMode::Fixed;
    p.fixed_w = 64;
    p.fixed_h = 64;
    let inputs = vec![sprite(16, 16, 1.0, 1.0)];
    let out = run_equalize_sizes(&inputs, &p);
    let o = &out[0];
    // Source buffer grew (nearest by factor 4) to 64x64; fit-by-scale
    // path then leaves scale ≈ 1.
    assert_eq!((o.width, o.height), (64, 64));
    assert!((o.new_scale_x - 1.0).abs() < 1e-4);
    assert!(o.changed);
}

#[test]
fn no_change_returns_changed_false() {
    let mut p = EqualizeSizesParams::default();
    p.target_mode = TargetMode::Fixed;
    p.fixed_w = 32;
    p.fixed_h = 32;
    p.rasterize_after = false;
    p.upscale_if_smaller = false;
    // Sprite ALREADY at visual 32x32 with no scaling, no upscale, no
    // rasterize → nothing to change.
    let inputs = vec![sprite(32, 32, 1.0, 1.0)];
    let out = run_equalize_sizes(&inputs, &p);
    assert!(!out[0].changed);
}

#[test]
fn nearest_upscale_is_pixel_exact_at_integer_factor() {
    let src = solid(2, 2, [10, 20, 30]);
    let (out, w, h) = nearest_upscale(&src, 2, 2, 3);
    assert_eq!((w, h), (6, 6));
    // Every pixel should be [10,20,30,255].
    for chunk in out.as_chunks::<4>().0 {
        assert_eq!(chunk, &[10, 20, 30, 255]);
    }
}

#[test]
fn lanczos3_resample_preserves_solid_color() {
    // Resampling a solid color must produce the same solid color
    // (all weights normalize, brightness invariant).
    let src = solid(8, 8, [50, 100, 150]);
    let (out, w, h) = lanczos3_resample(bytemuck::cast_slice(&src), 8, 8, 16, 16);
    assert_eq!((w, h), (16, 16));
    for chunk in out.as_chunks::<4>().0 {
        assert_eq!(chunk[0], 50);
        assert_eq!(chunk[1], 100);
        assert_eq!(chunk[2], 150);
        assert_eq!(chunk[3], 255);
    }
}

#[test]
fn mitchell_resample_preserves_solid_color() {
    let src = solid(8, 8, [50, 100, 150]);
    let (out, w, h) = mitchell_resample(bytemuck::cast_slice(&src), 8, 8, 5, 11);
    assert_eq!((w, h), (5, 11));
    for chunk in out.as_chunks::<4>().0 {
        assert_eq!(chunk[0], 50);
        assert_eq!(chunk[1], 100);
        assert_eq!(chunk[2], 150);
        assert_eq!(chunk[3], 255);
    }
}

#[test]
fn output_buffer_length_matches_reported_dimensions() {
    let mut p = EqualizeSizesParams::default();
    p.target_mode = TargetMode::Fixed;
    p.fixed_w = 47;
    p.fixed_h = 23;
    p.rasterize_after = true;
    let inputs = vec![sprite(60, 60, 1.0, 1.0)];
    let out = run_equalize_sizes(&inputs, &p);
    let o = &out[0];
    assert_eq!(o.rgba.len(), (o.width * o.height * 4) as usize);
}
