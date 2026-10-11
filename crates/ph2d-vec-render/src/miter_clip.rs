//! ⭐ **O bico cortado, do lado da `kurbo`** (F12 A) — a lei vive em
//! [`ph2d_vec_scene::miter_clip`]; aqui o `BezPath` vira troços, os cortes viram quadriláteros
//! orientados pelo winding do corpo do traço, e o contorno expandido ganha-os. Uma porta para
//! todo leitor que expande um traço: o desenho, a caixa visual, a placa.

use ph2d_vec_scene::miter_clip::{ClipPatch, El, clip_patches, oriented, subpaths};
use ph2d_vector::{BezPath, Join, PathEl, Point, Shape, Stroke, StrokeOpts, dash, expand_stroke};

/// O `PathEl` da `kurbo` no espelho da lei.
fn el(e: PathEl) -> El {
    let pt = |p: Point| [p.x, p.y];
    match e {
        PathEl::MoveTo(p) => El::Move(pt(p)),
        PathEl::LineTo(p) => El::Line(pt(p)),
        PathEl::QuadTo(a, p) => El::Quad(pt(a), pt(p)),
        PathEl::CurveTo(a, b, p) => El::Cubic(pt(a), pt(b), pt(p)),
        PathEl::ClosePath => El::Close,
    }
}

/// Os cortes que a caneta `pen` pede sobre `bp` (o tracejado já aplicado: só as juntas que se
/// desenham). Vazio fora da junta `Miter`.
#[must_use]
pub fn patches(bp: &BezPath, pen: &Stroke) -> Vec<ClipPatch> {
    if !matches!(pen.join, Join::Miter) || pen.width <= 0.0 {
        return Vec::new();
    }
    let subs = if pen.dash_pattern.is_empty() {
        subpaths(bp.iter().map(el))
    } else {
        subpaths(dash(bp.iter(), pen.dash_offset, &pen.dash_pattern).map(el))
    };
    clip_patches(&subs, pen.width * 0.5, pen.miter_limit)
}

/// Há junta além do [`ph2d_vec_scene::MITER_LIMIT`] em `bp`? Só ângulos (a largura não conta) —
/// a [`crate::PathTess`] paga-o UMA vez por geometria e o desenho por cópia só lê o sinal.
#[must_use]
pub fn any_sharp(bp: &BezPath) -> bool {
    !clip_patches(
        &subpaths(bp.iter().map(el)),
        1.0,
        ph2d_vec_scene::MITER_LIMIT,
    )
    .is_empty()
}

/// ⭐ **O contorno do traço com os bicos cortados** — o `expand_stroke` da `kurbo` mais um
/// quadrilátero por corte, cada um com o sentido do corpo (sem costura no preenchimento NonZero).
#[must_use]
pub fn stroke_outline(bp: &BezPath, pen: &Stroke, tol: f64) -> BezPath {
    let mut out = expand_stroke(bp.iter(), pen, &StrokeOpts::default(), tol);
    let cuts = patches(bp, pen);
    append_oriented(&mut out, &cuts);
    out
}

/// Junta os `cuts` a `body`, cada quadrilátero no sentido do winding do corpo junto ao chanfro.
pub fn append_oriented(body: &mut BezPath, cuts: &[ClipPatch]) {
    let quads = oriented(cuts, |p| body.winding(Point::new(p[0], p[1])));
    for q in quads {
        body.move_to((q[0][0], q[0][1]));
        for p in &q[1..] {
            body.line_to((p[0], p[1]));
        }
        body.close_path();
    }
}

#[cfg(test)]
#[path = "miter_clip_tests.rs"]
mod tests;
