//! Painter on-canvas WETNESS sheen overlay — the damp-paper veil drawn over the wet region while the
//! Watercolor render-path is active (Wetness card). Split from `painter_bridge_overlays.rs` for the
//! HR-18 file-LOC cap. Pure draw: reads the active `PainterTool` moisture view + selection + camera and
//! writes a translucent veil image into the overlay `VectorScene`; mutates no tool or model state.
//! Called once per frame by `painter_bridge_overlays::draw_overlays`.

use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_painter::PainterTool;
use ph2d_vector::VectorScene;

/// #12a (doc 14): the on-canvas WETNESS sheen — a subtle cool tint over the wet paper (Rebelle
/// "show wetness"), alpha ∝ the local moisture byte. Built over the moisture RECT only (transient,
/// session-scoped) and drawn via the same image→screen affine as the sprite. Read-only.
pub(super) fn draw_wetness_overlay(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
) {
    // Wetness-preview strength (Wetness card slider): 0 ⇒ no preview.
    let intensity = painter.wet_preview_intensity();
    if intensity <= 0.0 {
        return;
    }
    let Some((wet, cw, ch, [rx0, ry0, rx1, ry1])) = painter.canvas_wet_view() else {
        return;
    };
    let Some(bits) = hero.gizmo.selection else {
        return;
    };
    let entity = ph2d_ecs::Entity::from_bits(bits);
    let (Some(tr), Some(sprite)) = (
        sim.world().get::<crate::Transform>(entity),
        sim.world().get::<ph2d_render::Sprite>(entity),
    ) else {
        return;
    };
    let (rw, rh) = ((rx1 - rx0) as usize, (ry1 - ry0) as usize);
    if rw == 0 || rh == 0 {
        return;
    }
    // Straight-alpha DAMP-PAPER darkening over the wet region (Enio 2026-07-11: "sem tonalidade
    // azulada" — wet paper just darkens, no water-blue sheen). Near-neutral dark tint, a hair warm so it
    // reads as damp paper, not a grey wash; the slider (`intensity`) scales the max veil alpha. Vello
    // premultiplies on draw.
    const TINT: [u8; 3] = [34, 31, 28]; // LITERAL-COLOR-OK: damp-paper darkening (near-neutral, faint warm)
    const BLUR_R: usize = 4; // LITERAL-PX-OK: gentle veil softening — wet paper has no 1-px hard edges
    let max_alpha = intensity * 0.55; // LITERAL-COLOR-OK: slider 0..1 → veil alpha 0..0.55 at full wetness
    let cwu = cw as usize;
    // Local moisture → veil alpha, then a GENTLE box blur so the damp reads with a soft organic fringe
    // (a stroke's footprint edge / the fresh-vs-decayed step at a junction is ~1 px hard otherwise). The
    // moisture MAP is untouched — this is preview-only. (Safe now that the pour is per-footprint, Bug #9:
    // an earlier over-blur only spread the union-pour RECTANGLE — that root is fixed, so this just softens.)
    let mut alpha = vec![0.0f32; rw * rh];
    for y in 0..rh {
        let src = (ry0 as usize + y) * cwu + rx0 as usize;
        let dst = y * rw;
        for x in 0..rw {
            alpha[dst + x] = f32::from(wet[src + x]) / 255.0 * max_alpha;
        }
    }
    let alpha = box_blur_f32(&alpha, rw, rh, BLUR_R);
    let mut veil = vec![0u8; rw * rh * 4];
    for (i, &a) in alpha.iter().enumerate() {
        if a > 0.002 {
            let p = i * 4;
            veil[p] = TINT[0];
            veil[p + 1] = TINT[1];
            veil[p + 2] = TINT[2];
            veil[p + 3] = (a * 255.0).clamp(0.0, 255.0) as u8;
        }
    }
    // `base` maps FULL image-px → screen; the sub-image rides it after a translate to the rect origin.
    let base = super::bgremoval_preview::sprite_image_to_screen_affine(
        cw,
        ch,
        tr,
        sprite,
        camera,
        window_size,
    );
    let affine = base * ph2d_vector::Affine::translate((f64::from(rx0), f64::from(ry0)));
    vector_scene.draw_image_rgba_transformed(
        &std::sync::Arc::new(veil),
        rw as u32,
        rh as u32,
        affine,
        ph2d_vector::ImageQuality::Low,
    );
}

/// Separable box blur of a `w×h` f32 map (sliding window, O(w·h) — safe on a full-canvas wet map). Edge
/// pixels normalize by the FULL window, so the field FADES softly at the boundary (the damp fringe we want).
fn box_blur_f32(src: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    if r == 0 || w == 0 || h == 0 {
        return src.to_vec();
    }
    let win = (2 * r + 1) as f32;
    let mut tmp = vec![0.0f32; w * h];
    for y in 0..h {
        let base = y * w;
        let mut acc = 0.0f32;
        for x in 0..(r + 1).min(w) {
            acc += src[base + x];
        }
        for x in 0..w {
            tmp[base + x] = acc / win;
            if x + r + 1 < w {
                acc += src[base + x + r + 1];
            }
            if x >= r {
                acc -= src[base + x - r];
            }
        }
    }
    let mut out = vec![0.0f32; w * h];
    for x in 0..w {
        let mut acc = 0.0f32;
        for y in 0..(r + 1).min(h) {
            acc += tmp[y * w + x];
        }
        for y in 0..h {
            out[y * w + x] = acc / win;
            if y + r + 1 < h {
                acc += tmp[(y + r + 1) * w + x];
            }
            if y >= r {
                acc -= tmp[(y - r) * w + x];
            }
        }
    }
    out
}
