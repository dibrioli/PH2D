//! **A MINIATURA de um assado** — irmã do [`crate::motion_object_bake`] por RESPONSABILIDADE (e
//! pelo tecto de LOC): aquele produz os pixels de um objecto; isto reduz-os ao que cabe num cartão
//! de painel, e não sabe nada sobre cenas, afins ou GPU.

use crate::motion_object_bake::THUMB_MAX;
use ph2d_panel_motion_graph::PreviewThumb;

/// Downsample straight RGBA8 (`w`×`h`) to a card thumbnail (doc 86 A5): at most
/// [`THUMB_MAX`] on its long side, aspect preserved, never upscaled. Box-average in
/// PREMULTIPLIED space (`Σ c·a / Σ a`) so a transparent edge does not bleed a dark
/// halo into the shrunk shape — the premul trap the overlay lesson names (ADR-0120
/// neighbourhood). One pass per bake; the result is cached with the tile.
pub(crate) fn thumbnail(rgba: &[u8], w: u32, h: u32) -> PreviewThumb {
    let (w, h) = (w.max(1), h.max(1));
    let long = w.max(h);
    let (tw, th) = if long <= THUMB_MAX {
        (w, h)
    } else {
        let s = THUMB_MAX as f32 / long as f32;
        (
            ((w as f32 * s).round() as u32).max(1),
            ((h as f32 * s).round() as u32).max(1),
        )
    };
    let mut out = vec![0u8; (tw * th * 4) as usize];
    for oy in 0..th {
        let sy0 = oy * h / th;
        let sy1 = ((oy + 1) * h / th).max(sy0 + 1).min(h);
        for ox in 0..tw {
            let sx0 = ox * w / tw;
            let sx1 = ((ox + 1) * w / tw).max(sx0 + 1).min(w);
            let (mut sr, mut sg, mut sb, mut sa, mut n) = (0u64, 0u64, 0u64, 0u64, 0u64);
            for sy in sy0..sy1 {
                for sx in sx0..sx1 {
                    let i = ((sy * w + sx) * 4) as usize;
                    let a = rgba[i + 3] as u64;
                    sr += rgba[i] as u64 * a;
                    sg += rgba[i + 1] as u64 * a;
                    sb += rgba[i + 2] as u64 * a;
                    sa += a;
                    n += 1;
                }
            }
            let o = ((oy * tw + ox) * 4) as usize;
            // `sa == 0` ⇒ the block was fully transparent; leave the colour at 0 (already
            // zeroed), which is what a transparent thumbnail texel should carry.
            if let (Some(r), Some(g), Some(b)) =
                (sr.checked_div(sa), sg.checked_div(sa), sb.checked_div(sa))
            {
                out[o] = r as u8;
                out[o + 1] = g as u8;
                out[o + 2] = b as u8;
            }
            out[o + 3] = (sa / n.max(1)) as u8;
        }
    }
    PreviewThumb {
        rgba: std::sync::Arc::new(out),
        w: tw,
        h: th,
    }
}
