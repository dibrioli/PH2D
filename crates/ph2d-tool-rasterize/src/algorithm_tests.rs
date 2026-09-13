//! Testes de `algorithm.rs` — irmão por tecto de LOC (`workspace_src_files_under_loc_cap`).
//!
//! Corte mecânico: o `mod tests` saiu inteiro, verbatim, do ficheiro que o continha.

use super::*;

fn pixel_at(buf: &[u8], w: u32, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * w + x) * 4) as usize;
    [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
}

/// Fill an `RGBA8` buffer of size `w*h` with `colour`.
fn solid(w: u32, h: u32, colour: [u8; 4]) -> Vec<u8> {
    let mut v = vec![0u8; (w * h * 4) as usize];
    for i in (0..v.len()).step_by(4) {
        v[i] = colour[0];
        v[i + 1] = colour[1];
        v[i + 2] = colour[2];
        v[i + 3] = colour[3];
    }
    v
}

#[test]
fn identity_transform_returns_unchanged_with_did_change_false() {
    let rgba = solid(4, 3, [10, 20, 30, 255]);
    let r = rasterize(&rgba, 4, 3, 1.0, 1.0, 0.0);
    assert_eq!(r.width, 4);
    assert_eq!(r.height, 3);
    assert!(!r.did_change);
    assert_eq!(r.pixels, rgba);
}

#[test]
fn non_finite_inputs_fall_back_to_identity() {
    let rgba = solid(2, 2, [50, 60, 70, 255]);
    let r = rasterize(&rgba, 2, 2, f32::NAN, f32::INFINITY, f32::NAN);
    assert!(!r.did_change);
    assert_eq!(r.width, 2);
    assert_eq!(r.height, 2);
    assert_eq!(r.pixels, rgba);
}

#[test]
fn zero_width_returns_sentinel() {
    let r = rasterize(&[], 0, 5, 2.0, 2.0, 0.0);
    assert_eq!(r.width, 1);
    assert_eq!(r.height, 1);
    assert_eq!(r.pixels, vec![0, 0, 0, 0]);
    assert!(r.did_change);
}

#[test]
fn zero_height_returns_sentinel() {
    let r = rasterize(&[], 5, 0, 1.5, 1.5, 0.0);
    assert_eq!(r.width, 1);
    assert_eq!(r.height, 1);
    assert!(r.did_change);
}

#[test]
#[should_panic(expected = "rgba buffer length must equal")]
fn buffer_length_mismatch_panics() {
    let rgba = vec![0u8; 3];
    let _ = rasterize(&rgba, 4, 4, 1.0, 1.0, 0.0);
}

#[test]
fn upscale_doubles_dimensions() {
    let rgba = solid(4, 3, [200, 100, 50, 255]);
    let r = rasterize(&rgba, 4, 3, 2.0, 2.0, 0.0);
    assert_eq!(r.width, 8);
    assert_eq!(r.height, 6);
    assert_eq!(r.pixels.len(), 8 * 6 * 4);
    assert!(r.did_change);
    // Solid colour stays solid (Mitchell partition of unity + edge
    // clamp preserves a constant input exactly within RGB rounding
    // noise of ±1).
    let centre = pixel_at(&r.pixels, r.width, 4, 3);
    assert!(
        (centre[0] as i32 - 200).abs() <= 1
            && (centre[1] as i32 - 100).abs() <= 1
            && (centre[2] as i32 - 50).abs() <= 1
            && centre[3] == 255,
        "centre={:?}",
        centre,
    );
}

#[test]
fn downscale_halves_dimensions() {
    let rgba = solid(8, 6, [128, 64, 32, 255]);
    let r = rasterize(&rgba, 8, 6, 0.5, 0.5, 0.0);
    assert_eq!(r.width, 4);
    assert_eq!(r.height, 3);
    assert!(r.did_change);
    let centre = pixel_at(&r.pixels, r.width, 2, 1);
    assert!(
        (centre[0] as i32 - 128).abs() <= 1
            && (centre[1] as i32 - 64).abs() <= 1
            && (centre[2] as i32 - 32).abs() <= 1
            && centre[3] == 255,
    );
}

#[test]
fn horizontal_flip_reverses_columns() {
    // 4×1 strip with a known left→right gradient. After flipping on
    // X, column 0 holds the old column 3, etc.
    let mut rgba = vec![0u8; 4 * 1 * 4];
    for x in 0..4 {
        let i = (x * 4) as usize;
        rgba[i] = x as u8 * 50;
        rgba[i + 1] = 100;
        rgba[i + 2] = 200;
        rgba[i + 3] = 255;
    }
    let r = rasterize(&rgba, 4, 1, -1.0, 1.0, 0.0);
    assert_eq!(r.width, 4);
    assert_eq!(r.height, 1);
    assert!(r.did_change);
    for x in 0..4u32 {
        let p = pixel_at(&r.pixels, r.width, x, 0);
        assert_eq!(p[0], (3 - x) as u8 * 50, "x={x}");
        assert_eq!(p[3], 255);
    }
}

#[test]
fn vertical_flip_reverses_rows() {
    let mut rgba = vec![0u8; 1 * 4 * 4];
    for y in 0..4 {
        let i = (y * 4) as usize;
        rgba[i + 1] = y as u8 * 50;
        rgba[i + 3] = 255;
    }
    let r = rasterize(&rgba, 1, 4, 1.0, -1.0, 0.0);
    assert_eq!(r.width, 1);
    assert_eq!(r.height, 4);
    for y in 0..4u32 {
        let p = pixel_at(&r.pixels, r.width, 0, y);
        assert_eq!(p[1], (3 - y) as u8 * 50, "y={y}");
    }
}

#[test]
fn double_flip_returns_original_orientation() {
    // 2×2 with distinct corner colours; flipping on both axes is
    // a 180° point-mirror — every corner swaps with the diagonal.
    let mut rgba = vec![0u8; 2 * 2 * 4];
    let cols: [[u8; 4]; 4] = [
        [10, 0, 0, 255],
        [0, 20, 0, 255],
        [0, 0, 30, 255],
        [40, 40, 40, 255],
    ];
    for (idx, c) in cols.iter().enumerate() {
        let i = idx * 4;
        rgba[i..i + 4].copy_from_slice(c);
    }
    let r = rasterize(&rgba, 2, 2, -1.0, -1.0, 0.0);
    assert_eq!(r.width, 2);
    assert_eq!(r.height, 2);
    // top-left becomes bottom-right (cols[0] → idx 3), etc.
    assert_eq!(pixel_at(&r.pixels, 2, 1, 1), cols[0]);
    assert_eq!(pixel_at(&r.pixels, 2, 0, 1), cols[1]);
    assert_eq!(pixel_at(&r.pixels, 2, 1, 0), cols[2]);
    assert_eq!(pixel_at(&r.pixels, 2, 0, 0), cols[3]);
}

#[test]
fn rotation_by_90_degrees_swaps_dimensions() {
    let rgba = solid(8, 4, [100, 100, 100, 255]);
    let r = rasterize(&rgba, 8, 4, 1.0, 1.0, std::f32::consts::FRAC_PI_2);
    // |cos π/2| = 0, |sin π/2| = 1 → bbox = (h, w) = (4, 8).
    assert_eq!(r.width, 4);
    assert_eq!(r.height, 8);
    assert!(r.did_change);
}

#[test]
fn rotation_by_180_degrees_preserves_dimensions() {
    let rgba = solid(6, 4, [80, 80, 80, 255]);
    let r = rasterize(&rgba, 6, 4, 1.0, 1.0, std::f32::consts::PI);
    // |cos π| = 1, |sin π| = 0 → bbox = (w, h).
    assert_eq!(r.width, 6);
    assert_eq!(r.height, 4);
}

#[test]
fn tiny_rotation_is_treated_as_identity_shape() {
    let rgba = solid(4, 3, [50, 60, 70, 255]);
    // Below ROTATION_EPS — rotation pass skipped, only resample (no-op
    // since scale = 1) + flip (none) run. Output keeps (w, h).
    let r = rasterize(&rgba, 4, 3, 1.0, 1.0, ROTATION_EPS / 2.0);
    assert_eq!(r.width, 4);
    assert_eq!(r.height, 3);
    // Identity short-circuit catches scale=1 + rot=0; tiny non-zero
    // rotation goes through resample path but the resample also
    // short-circuits since w_s == width and h_s == height.
    assert!(!r.did_change);
}

#[test]
fn transparent_source_stays_transparent() {
    let rgba = solid(4, 4, [0, 0, 0, 0]);
    let r = rasterize(&rgba, 4, 4, 2.0, 2.0, 0.0);
    // Every output pixel must have alpha 0.
    for chunk in r.pixels.as_chunks::<4>().0 {
        assert_eq!(chunk[3], 0, "transparent input → transparent output");
    }
}

#[test]
fn opaque_solid_stays_solid_under_resample() {
    // Edge replication + Mitchell partition-of-unity should
    // reconstruct a constant input exactly (modulo rounding ±1).
    let rgba = solid(16, 16, [200, 50, 100, 255]);
    let r = rasterize(&rgba, 16, 16, 0.75, 0.75, 0.0);
    for chunk in r.pixels.as_chunks::<4>().0 {
        assert!((chunk[0] as i32 - 200).abs() <= 1);
        assert!((chunk[1] as i32 - 50).abs() <= 1);
        assert!((chunk[2] as i32 - 100).abs() <= 1);
        assert_eq!(chunk[3], 255);
    }
}

#[test]
fn premultiplied_alpha_prevents_colour_bleed_from_transparent_region() {
    // Half opaque red, half transparent magenta. After a 2× upscale
    // the opaque half must still report red — without premult, the
    // Mitchell kernel would blend the magenta RGB into the red
    // through the alpha-zero edge.
    let mut rgba = vec![0u8; 4 * 1 * 4];
    rgba[0..4].copy_from_slice(&[255, 0, 0, 255]); // opaque red
    rgba[4..8].copy_from_slice(&[255, 0, 0, 255]); // opaque red
    rgba[8..12].copy_from_slice(&[255, 0, 255, 0]); // transparent magenta
    rgba[12..16].copy_from_slice(&[255, 0, 255, 0]); // transparent magenta
    let r = rasterize(&rgba, 4, 1, 2.0, 1.0, 0.0);
    // Output is 8×1; inspect the deep-opaque end (pixel 1 — well
    // inside the opaque red half).
    let p = pixel_at(&r.pixels, 8, 1, 0);
    assert_eq!(p[3], 255, "alpha at opaque-deep pixel");
    // Red must be near 255; green near 0 (no magenta bleed).
    assert!((p[0] as i32 - 255).abs() <= 1, "red was {}", p[0]);
    assert_eq!(p[1], 0, "green was {}", p[1]);
}

#[test]
fn mitchell_kernel_partition_of_unity_at_integer_offsets() {
    // Weights at 4 integer source pixels around any sub-pixel
    // sample point sum to ~1.0 — the property that lets us skip
    // post-normalisation.
    for offset_steps in 0..32 {
        let xs_centre = offset_steps as f32 / 32.0;
        let floor = xs_centre.floor() as i32;
        let mut sum = 0.0;
        for k in -1..=2 {
            let xs = floor + k;
            let t = xs as f32 - xs_centre;
            sum += mitchell_kernel(t);
        }
        assert!(
            (sum - 1.0).abs() < 1e-4,
            "weights at xs_centre={xs_centre} sum to {sum}, expected ~1.0",
        );
    }
}
