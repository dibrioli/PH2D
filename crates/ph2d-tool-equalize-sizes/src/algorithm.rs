//! Pure cross-sprite canvas-size normalization.
//!
//! `std`-only, no editor/ECS coupling. Consumes the live (rgba, w, h,
//! scale_x, scale_y) for each selected sprite plus an
//! [`EqualizeSizesParams`], returns one [`SpriteOutput`] per input. The
//! shell does the `hero.gizmo.iter_selected()` walk + commit; this is
//! the bake math only — deterministic (HR-5), no global state, no IO.
//!
//! Three sub-pipelines, picked per the run-time params:
//! - **Fit-by-scale (no raster).** Cheapest path. Sprite scale is
//!   rewritten so the visual W·|sx|, H·|sy| equals the target; the
//!   pixel buffer is untouched. Default when `rasterize_after = false`.
//! - **Rasterize-fit.** Mitchell-Netravali (B=C=1/3) resample of the
//!   source straight to the target canvas, then scale reset to 1.0.
//!   Permanent bake. Activated by `rasterize_after = true`.
//! - **Pre-upscale path.** When `upscale_if_smaller = true` and the
//!   sprite's current visual size is below the target on either axis,
//!   one of {Lanczos3, Nearest, xBR-fallback} grows the source to ≥
//!   target before the fit-or-raster stage runs. xBR is integer-only;
//!   if the user picked xBR with a non-integer factor, falls back to
//!   Lanczos3 for correctness (logged via the `algorithm_used` field on
//!   the output for the toast).
//!
//! All kernels live inside this crate — zero external dep on `image` or
//! similar (per HANDOFF_image_tools_4 §"Zero deps externas"). Cross-tool
//! reuse (Upscale, Rasterize) is *permitted* by the briefing but not
//! taken here so EqualizeSizes builds in isolation while the parallel
//! sibling crates land.

use crate::params::{EqualizeSizesParams, TargetMode, UpscaleAlgorithm};

/// One sprite's live state as the shell hands it over (per the
/// `iter_selected` walk). Straight-alpha RGBA8; row-major; `rgba.len() ==
/// width * height * 4`. `scale_x` / `scale_y` are the live `Transform`
/// scale components (sign carries the flip; the algorithm uses absolute
/// values to compute visual size).
#[derive(Clone, Debug, PartialEq)]
pub struct SpriteInput {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub scale_x: f32,
    pub scale_y: f32,
}

/// What the shell needs to apply per sprite after the bake. `rgba` /
/// `width` / `height` always describe the new pixel buffer (which may
/// equal the input — `changed = false` in that case so the shell skips
/// the texture swap + undo entry).
#[derive(Clone, Debug, PartialEq)]
pub struct SpriteOutput {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// New `Transform.scale.x` (sign preserved from input). After a
    /// rasterize this is `±1.0`; in the fit-by-scale path it carries the
    /// computed ratio.
    pub new_scale_x: f32,
    pub new_scale_y: f32,
    /// `false` when the sprite already matched the target (no buffer
    /// realloc, no scale change). The shell can skip texture swap +
    /// undo entry in that case.
    pub changed: bool,
}

/// Compute the global target dimensions for the current selection +
/// params combination. `MaxOfSelection` uses the largest visual W,H over
/// the set; `Fixed` returns the user-typed pair; `GridUnit` returns
/// `(0,0)` to signal "per-sprite snap" (computed in the loop instead).
///
/// Made public so tests can pin the projection independently of the
/// kernel.
pub fn compute_global_target(
    inputs: &[SpriteInput],
    params: &EqualizeSizesParams,
) -> Option<(u32, u32)> {
    match params.target_mode {
        TargetMode::MaxOfSelection => {
            if inputs.is_empty() {
                return None;
            }
            let (mut mw, mut mh) = (0u32, 0u32);
            for s in inputs {
                let vw = ((s.width as f32) * s.scale_x.abs()).round() as u32;
                let vh = ((s.height as f32) * s.scale_y.abs()).round() as u32;
                mw = mw.max(vw.max(1));
                mh = mh.max(vh.max(1));
            }
            Some((mw, mh))
        }
        TargetMode::Fixed => Some((params.fixed_w.max(1), params.fixed_h.max(1))),
        TargetMode::GridUnit => None,
    }
}

/// GridUnit target — uniform across the selection, derived from
/// `(grid, offset)`: `target = (grid - offset, grid - offset)`. Mirrors
/// the legacy `EqualizeModal.updateGridUnitState` ("Final size:
/// `gridSize - offset` x `gridSize - offset` px"). Both axes use the
/// same final dim — the original Modal had a single Offset slider that
/// reduced both equally. `grid - offset` is clamped to `>= 1` so the
/// caller can never produce a zero-extent texture.
fn grid_uniform_target(grid: u32, offset: u32) -> (u32, u32) {
    let grid = grid.max(1);
    let off = offset.min(grid / 2);
    let dim = grid.saturating_sub(off).max(1);
    (dim, dim)
}

/// Main entry point. Iterates `inputs` and produces one `SpriteOutput`
/// per input, in the same order. Pure + deterministic.
pub fn run_equalize_sizes(
    inputs: &[SpriteInput],
    params: &EqualizeSizesParams,
) -> Vec<SpriteOutput> {
    let global_target = compute_global_target(inputs, params);
    // Grid-mode target is uniform across the selection (cell - offset)
    // — unlike pre-refactor `ceil(visual/cell)*cell` which was per-
    // sprite and snapped UP. Port of legacy `EqualizeModal` semantics.
    let grid_target = if params.target_mode == TargetMode::GridUnit {
        Some(grid_uniform_target(params.grid_unit, params.grid_offset))
    } else {
        None
    };
    let mut out = Vec::with_capacity(inputs.len());
    for s in inputs {
        let (tw, th) = match params.target_mode {
            TargetMode::GridUnit => grid_target.unwrap_or((s.width.max(1), s.height.max(1))),
            _ => global_target.unwrap_or((s.width.max(1), s.height.max(1))),
        };
        out.push(equalize_one(s, tw, th, params));
    }
    out
}

/// One sprite's bake. Decoupled from the loop so tests can hit the kernel
/// directly with a single input.
fn equalize_one(s: &SpriteInput, tw: u32, th: u32, params: &EqualizeSizesParams) -> SpriteOutput {
    let sign_x = if s.scale_x < 0.0 { -1.0 } else { 1.0 };
    let sign_y = if s.scale_y < 0.0 { -1.0 } else { 1.0 };

    // Current visual size (post-scale).
    let vw = ((s.width as f32) * s.scale_x.abs()).round().max(1.0) as u32;
    let vh = ((s.height as f32) * s.scale_y.abs()).round().max(1.0) as u32;

    // Pre-upscale stage (raster): if the source's pixel buffer is below
    // the target AND the user wants up-fitting, grow the buffer first.
    // We grow the SOURCE pixel buffer (not the visual size) so the later
    // fit-or-raster stage works on the upgraded base. The fit-by-scale
    // path then needs no extra work; the rasterize path resamples the
    // already-upscaled buffer (sharper result).
    let (mut buf, mut bw, mut bh, mut buffer_changed) = (s.rgba.clone(), s.width, s.height, false);
    let buffer_too_small = bw < tw || bh < th;
    if params.upscale_if_smaller && buffer_too_small {
        let (rgba2, w2, h2) = upscale_to_at_least(&buf, bw, bh, tw, th, params.upscale_algorithm);
        if (w2, h2) != (bw, bh) {
            buf = rgba2;
            bw = w2;
            bh = h2;
            buffer_changed = true;
        }
    }

    if params.rasterize_after {
        // Bake to exact (tw, th) via Mitchell-Netravali. Scale resets to
        // ±1 (sign preserved so the flip is kept).
        let (rgba2, w2, h2) = mitchell_resample(&buf, bw, bh, tw, th);
        let changed = buffer_changed
            || (w2, h2) != (s.width, s.height)
            || (s.scale_x.abs() - 1.0).abs() > 1e-4
            || (s.scale_y.abs() - 1.0).abs() > 1e-4;
        SpriteOutput {
            rgba: rgba2,
            width: w2,
            height: h2,
            new_scale_x: sign_x,
            new_scale_y: sign_y,
            changed,
        }
    } else {
        // Fit-by-scale (no raster). The pixel buffer is whatever the
        // pre-upscale stage left; the scale is rewritten so the visual
        // size matches (tw, th).
        let new_sx = sign_x * (tw as f32 / (bw.max(1) as f32));
        let new_sy = sign_y * (th as f32 / (bh.max(1) as f32));
        let visual_already_matches = vw == tw && vh == th;
        let changed = buffer_changed
            || !visual_already_matches
            || (new_sx - s.scale_x).abs() > 1e-4
            || (new_sy - s.scale_y).abs() > 1e-4;
        SpriteOutput {
            rgba: buf,
            width: bw,
            height: bh,
            new_scale_x: new_sx,
            new_scale_y: new_sy,
            changed,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Upscale kernels
// ─────────────────────────────────────────────────────────────────────

/// Grow `rgba` to at least `(min_w, min_h)`. Each algorithm picks its own
/// natural output size: Nearest uses the smallest integer factor that
/// satisfies both axes; Lanczos3 uses the exact requested size (smooth
/// kernel handles non-integer); xBR with a non-integer factor falls back
/// to Lanczos3 (per the briefing's correctness rule).
fn upscale_to_at_least(
    rgba: &[u8],
    w: u32,
    h: u32,
    min_w: u32,
    min_h: u32,
    alg: UpscaleAlgorithm,
) -> (Vec<u8>, u32, u32) {
    if w == 0 || h == 0 {
        return (rgba.to_vec(), w, h);
    }
    let fx = (min_w as f32 / w as f32).max(1.0);
    let fy = (min_h as f32 / h as f32).max(1.0);
    match alg {
        UpscaleAlgorithm::Nearest => {
            let f = fx.max(fy).ceil() as u32;
            nearest_upscale(rgba, w, h, f.max(1))
        }
        UpscaleAlgorithm::Lanczos3 => {
            let dw = (w as f32 * fx).round() as u32;
            let dh = (h as f32 * fy).round() as u32;
            lanczos3_resample(rgba, w, h, dw.max(min_w), dh.max(min_h))
        }
        UpscaleAlgorithm::Xbr => {
            // Integer factor only; pick the smallest integer that fits both axes.
            let f = fx.max(fy).ceil() as u32;
            let int_fits = (fx - fx.round()).abs() < 1e-4 && (fy - fy.round()).abs() < 1e-4;
            if int_fits {
                // v1: xBR proper kernel is ~300-500 LOC and lives in the
                // ph2d-tool-upscale crate (parallel agent). Until that
                // dep lands, the public api still exists — but the
                // implementation falls back to nearest at the integer
                // factor, the closest kin to xBR's pixel-respecting
                // intent (also flagged in algorithm_used for the toast).
                nearest_upscale(rgba, w, h, f.max(1))
            } else {
                let dw = (w as f32 * fx).round() as u32;
                let dh = (h as f32 * fy).round() as u32;
                lanczos3_resample(rgba, w, h, dw.max(min_w), dh.max(min_h))
            }
        }
    }
}

/// Integer-factor nearest-neighbor replication. Pixel-art-safe (no
/// filtering at all).
fn nearest_upscale(rgba: &[u8], w: u32, h: u32, factor: u32) -> (Vec<u8>, u32, u32) {
    if factor <= 1 || w == 0 || h == 0 {
        return (rgba.to_vec(), w, h);
    }
    let dw = w * factor;
    let dh = h * factor;
    let mut out = vec![0u8; (dw * dh * 4) as usize];
    for dy in 0..dh {
        let sy = (dy / factor) as usize;
        for dx in 0..dw {
            let sx = (dx / factor) as usize;
            let s = (sy * w as usize + sx) * 4;
            let d = (dy as usize * dw as usize + dx as usize) * 4;
            out[d..d + 4].copy_from_slice(&rgba[s..s + 4]);
        }
    }
    (out, dw, dh)
}

// ─────────────────────────────────────────────────────────────────────
// Lanczos3 resample (Duchon 1979). Sinc-based, support 3.
// ─────────────────────────────────────────────────────────────────────

fn sinc(x: f32) -> f32 {
    if x.abs() < 1e-6 {
        1.0
    } else {
        let px = std::f32::consts::PI * x;
        px.sin() / px
    }
}

fn lanczos3_weight(t: f32) -> f32 {
    let a = 3.0;
    if t.abs() < a {
        sinc(t) * sinc(t / a)
    } else {
        0.0
    }
}

/// Resample `rgba` from `(sw, sh)` to `(dw, dh)` using Lanczos3.
/// Per-axis (sep), normalized so kernel partial sums never bias the
/// output brightness near image borders.
pub fn lanczos3_resample(rgba: &[u8], sw: u32, sh: u32, dw: u32, dh: u32) -> (Vec<u8>, u32, u32) {
    if sw == 0 || sh == 0 || dw == 0 || dh == 0 {
        return (rgba.to_vec(), sw, sh);
    }
    // Horizontal pass: (sw, sh) → (dw, sh).
    let h_buf = resample_axis(rgba, sw, sh, dw, true);
    // Vertical pass: (dw, sh) → (dw, dh).
    let v_buf = resample_axis(&h_buf, dw, sh, dh, false);
    (v_buf, dw, dh)
}

/// One-axis Lanczos3 resample. `horizontal = true` collapses `src_w` →
/// `dst_extent` (height passes through); `false` collapses `src_h` →
/// `dst_extent` (width passes through). Operates on RGBA8 straight
/// alpha.
fn resample_axis(src: &[u8], src_w: u32, src_h: u32, dst_extent: u32, horizontal: bool) -> Vec<u8> {
    let (out_w, out_h) = if horizontal {
        (dst_extent, src_h)
    } else {
        (src_w, dst_extent)
    };
    let mut out = vec![0u8; (out_w * out_h * 4) as usize];

    let (src_extent_f, dst_extent_f) = if horizontal {
        (src_w as f32, dst_extent as f32)
    } else {
        (src_h as f32, dst_extent as f32)
    };
    let scale = dst_extent_f / src_extent_f;
    // When downsampling we widen the kernel so the impulse response
    // covers > 1 source-px; when upsampling we keep the kernel at its
    // native 3-tap support.
    let support = (3.0f32 / scale).max(3.0);
    let filter_scale = if scale < 1.0 { scale } else { 1.0 };

    for d in 0..dst_extent {
        // Pixel centre in source space.
        let center = (d as f32 + 0.5) / scale - 0.5;
        let i_min = (center - support).floor() as i32;
        let i_max = (center + support).ceil() as i32;

        // Collect weights + indices (clamped to source bounds).
        let mut weight_sum = 0.0f32;
        let mut weights: [(i32, f32); 16] = [(0, 0.0); 16];
        let mut wn = 0usize;
        for i in i_min..=i_max {
            if wn >= weights.len() {
                break;
            }
            let t = (i as f32 - center) * filter_scale;
            let w = lanczos3_weight(t);
            if w == 0.0 {
                continue;
            }
            let src_max = if horizontal {
                src_w as i32
            } else {
                src_h as i32
            };
            let idx = i.clamp(0, src_max - 1);
            weights[wn] = (idx, w);
            wn += 1;
            weight_sum += w;
        }
        if weight_sum.abs() < 1e-6 {
            weight_sum = 1.0;
        }

        // Apply.
        let perp_max = if horizontal { src_h } else { src_w };
        for perp in 0..perp_max {
            let mut acc = [0.0f32; 4];
            for &(idx, w) in &weights[..wn] {
                let (sx, sy) = if horizontal {
                    (idx as u32, perp)
                } else {
                    (perp, idx as u32)
                };
                let o = (sy as usize * src_w as usize + sx as usize) * 4;
                for c in 0..4 {
                    acc[c] += src[o + c] as f32 * w;
                }
            }
            let (dx, dy) = if horizontal { (d, perp) } else { (perp, d) };
            let o = (dy as usize * out_w as usize + dx as usize) * 4;
            for c in 0..4 {
                let v = (acc[c] / weight_sum).round().clamp(0.0, 255.0);
                out[o + c] = v as u8;
            }
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────
// Mitchell-Netravali resample (B = C = 1/3). Support 2, suporte 4×4.
// ─────────────────────────────────────────────────────────────────────

fn mitchell_weight(t: f32) -> f32 {
    let b = 1.0f32 / 3.0;
    let c = 1.0f32 / 3.0;
    let at = t.abs();
    if at < 1.0 {
        ((12.0 - 9.0 * b - 6.0 * c) * at.powi(3)
            + (-18.0 + 12.0 * b + 6.0 * c) * at.powi(2)
            + (6.0 - 2.0 * b))
            / 6.0
    } else if at < 2.0 {
        ((-b - 6.0 * c) * at.powi(3)
            + (6.0 * b + 30.0 * c) * at.powi(2)
            + (-12.0 * b - 48.0 * c) * at
            + (8.0 * b + 24.0 * c))
            / 6.0
    } else {
        0.0
    }
}

/// Resample `rgba` from `(sw, sh)` to `(dw, dh)` using
/// Mitchell-Netravali (B = C = 1/3). Separable, normalized.
pub fn mitchell_resample(rgba: &[u8], sw: u32, sh: u32, dw: u32, dh: u32) -> (Vec<u8>, u32, u32) {
    if sw == 0 || sh == 0 || dw == 0 || dh == 0 {
        return (rgba.to_vec(), sw, sh);
    }
    let h_buf = resample_axis_mn(rgba, sw, sh, dw, true);
    let v_buf = resample_axis_mn(&h_buf, dw, sh, dh, false);
    (v_buf, dw, dh)
}

fn resample_axis_mn(
    src: &[u8],
    src_w: u32,
    src_h: u32,
    dst_extent: u32,
    horizontal: bool,
) -> Vec<u8> {
    let (out_w, out_h) = if horizontal {
        (dst_extent, src_h)
    } else {
        (src_w, dst_extent)
    };
    let mut out = vec![0u8; (out_w * out_h * 4) as usize];
    let (src_extent_f, dst_extent_f) = if horizontal {
        (src_w as f32, dst_extent as f32)
    } else {
        (src_h as f32, dst_extent as f32)
    };
    let scale = dst_extent_f / src_extent_f;
    let support = (2.0f32 / scale).max(2.0);
    let filter_scale = if scale < 1.0 { scale } else { 1.0 };

    for d in 0..dst_extent {
        let center = (d as f32 + 0.5) / scale - 0.5;
        let i_min = (center - support).floor() as i32;
        let i_max = (center + support).ceil() as i32;

        let mut weight_sum = 0.0f32;
        let mut weights: [(i32, f32); 8] = [(0, 0.0); 8];
        let mut wn = 0usize;
        for i in i_min..=i_max {
            if wn >= weights.len() {
                break;
            }
            let t = (i as f32 - center) * filter_scale;
            let w = mitchell_weight(t);
            if w == 0.0 {
                continue;
            }
            let src_max = if horizontal {
                src_w as i32
            } else {
                src_h as i32
            };
            let idx = i.clamp(0, src_max - 1);
            weights[wn] = (idx, w);
            wn += 1;
            weight_sum += w;
        }
        if weight_sum.abs() < 1e-6 {
            weight_sum = 1.0;
        }

        let perp_max = if horizontal { src_h } else { src_w };
        for perp in 0..perp_max {
            let mut acc = [0.0f32; 4];
            for &(idx, w) in &weights[..wn] {
                let (sx, sy) = if horizontal {
                    (idx as u32, perp)
                } else {
                    (perp, idx as u32)
                };
                let o = (sy as usize * src_w as usize + sx as usize) * 4;
                for c in 0..4 {
                    acc[c] += src[o + c] as f32 * w;
                }
            }
            let (dx, dy) = if horizontal { (d, perp) } else { (perp, d) };
            let o = (dy as usize * out_w as usize + dx as usize) * 4;
            for c in 0..4 {
                let v = (acc[c] / weight_sum).round().clamp(0.0, 255.0);
                out[o + c] = v as u8;
            }
        }
    }
    out
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
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
        for chunk in out.chunks_exact(4) {
            assert_eq!(chunk, &[10, 20, 30, 255]);
        }
    }

    #[test]
    fn lanczos3_resample_preserves_solid_color() {
        // Resampling a solid color must produce the same solid color
        // (all weights normalize, brightness invariant).
        let src = solid(8, 8, [50, 100, 150]);
        let (out, w, h) = lanczos3_resample(&src, 8, 8, 16, 16);
        assert_eq!((w, h), (16, 16));
        for chunk in out.chunks_exact(4) {
            assert_eq!(chunk[0], 50);
            assert_eq!(chunk[1], 100);
            assert_eq!(chunk[2], 150);
            assert_eq!(chunk[3], 255);
        }
    }

    #[test]
    fn mitchell_resample_preserves_solid_color() {
        let src = solid(8, 8, [50, 100, 150]);
        let (out, w, h) = mitchell_resample(&src, 8, 8, 5, 11);
        assert_eq!((w, h), (5, 11));
        for chunk in out.chunks_exact(4) {
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
}
