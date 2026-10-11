//! ⭐ **A CAIXA que a forma PINTA, EXACTA** (F11, 11/10) — a *Use Preview Bounds* do Illustrator:
//! a geometria ∪ cada marca da pilha de aparência, com o contorno do traço EXPANDIDO pela kurbo
//! (a junta, a ponta, o bico do miter tal como saem) sob a caneta redonda do renderer
//! (`stroke_uniform`: largura × `√|det|`). Lei, consumidores e recusas:
//! `docs/UI_New_and_Simple/vetor_nova_ui/17_gizmos_do_illustrator.md` §2.
//!
//! ⚠️ A irmã [`crate::path_bounds_under`] é CONSERVADORA (o pior miter sempre — recortes de FX,
//! onde folga é o lado certo do erro). Esta é a que se DESENHA: a caixa de transformação, a ④ e o
//! cartão, o enquadrar — uma caixa folgada lê-se como «a caixa não abraça a forma».

use ph2d_vec_scene::{PaintRef, StrokeAlign, StrokePiece, StrokeSpec, VecPath};
use ph2d_vector::{Affine, Rect, Shape, Stroke};

use crate::{build_bezpath, kurbo_stroke, stroke_uniform::uniform_scale};

/// Tolerância da expansão, RELATIVA à largura da caneta (o erro da caixa fica abaixo dela).
const TOL: f64 = 1e-3;

/// ⭐ **A caixa do que `path` pinta sob `xf`** — `None` = sem geometria. Cada camada da pilha
/// ([`VecPath::paint_stack`]) entra no sítio dela (`offset`) e com o que cresce (`dilate`); o
/// traço Outer chega `w` para fora, o Inner fica dentro da forma (a banda do
/// `ph2d_vec_boolean::aligned_stroke`); as pontas de seta pelo `stroke_plan`. O tracejado não
/// encolhe a caixa (ela não salta com o padrão).
#[must_use]
pub fn path_visual_bounds(path: &VecPath, xf: Affine) -> Option<Rect> {
    let cooked = path.cooked();
    let geo = build_bezpath(&cooked);
    if geo.elements().is_empty() {
        return None;
    }
    let [a, b, c, d, _, _] = xf.as_coeffs();
    let (sx, sy) = ((a * a + b * b).sqrt(), (c * c + d * d).sqrt());
    let mut acc = (xf * geo.clone()).bounding_box();
    for layer in path.paint_stack() {
        let m = xf * Affine::translate((layer.offset[0], layer.offset[1]));
        let mark = match layer.paint {
            PaintRef::Fill(_) => Some((m * geo.clone()).bounding_box()),
            PaintRef::Stroke(s) => stroke_box(&cooked, s, m),
        };
        if let Some(r) = mark {
            let g = layer.dilate.max(0.0);
            acc = acc.union(r.inflate(g * sx, g * sy));
        }
    }
    Some(acc)
}

/// A caixa do traço `s` de `p` (cozido) sob `m`. `None` = nada fora da geometria.
fn stroke_box(p: &VecPath, s: &StrokeSpec, m: Affine) -> Option<Rect> {
    if !s.lays_a_band() {
        return None;
    }
    let aligned = s.is_aligned() && p.all_contours_closed();
    if aligned && s.align == StrokeAlign::Inner {
        return None;
    }
    // Outer = a metade de FORA da faixa `2w` (a lei do `aligned_stroke`).
    let k = uniform_scale(m);
    let w = if aligned { 2.0 * s.width } else { s.width } * k;
    let mut acc: Option<Rect> = None;
    let mut eat = |r: Rect| acc = Some(acc.map_or(r, |cur| cur.union(r)));
    for piece in ph2d_vec_scene::stroke_plan(p, s) {
        match piece {
            StrokePiece::Line { path } => {
                let mut pen = kurbo_stroke(s, None);
                pen.width = w;
                eat(expanded(&build_bezpath(&path), &pen, m));
            }
            StrokePiece::Symbol { path } => {
                eat(expanded(
                    &build_bezpath(&path),
                    &Stroke::new(s.width * k),
                    m,
                ));
            }
            StrokePiece::Fill { path } => eat((m * build_bezpath(&path)).bounding_box()),
        }
    }
    acc
}

fn expanded(bp: &ph2d_vector::BezPath, pen: &Stroke, m: Affine) -> Rect {
    let screen = m * bp.clone();
    crate::miter_clip::stroke_outline(&screen, pen, (TOL * pen.width).max(1e-9)).bounding_box()
}

/// A caixa de MUNDO do que o caminho `id` pinta, `(min, max)` — a pose das `VecXforms`.
#[must_use]
pub fn path_world_visual_bbox(
    scene: &ph2d_vec_scene::VecScene,
    xforms: &ph2d_vec_scene::VecXforms,
    id: ph2d_vec_scene::VecPathId,
) -> Option<([f64; 2], [f64; 2])> {
    let path = scene.path(id)?;
    let r = path_visual_bounds(path, crate::path_to_screen(xforms, id, Affine::IDENTITY))?;
    Some(([r.x0, r.y0], [r.x1, r.y1]))
}

/// ⭐ **A caixa INTRÍNSECA (pré-escala) do que o caminho pinta**, na língua do gizmo de sprite
/// (`anchor ± half`, que o `gizmo_view_from` multiplica por `scale`). O traço mede-se JÁ sob a
/// escala do mundo (a caneta é redonda: `w·√|sx·sy|` nos dois eixos, não `w·sx` e `w·sy`) e
/// volta dividido por ela — a caixa desenhada abraça o traço também sob escala desigual.
#[must_use]
pub fn path_intrinsic_visual_bbox(path: &VecPath, scale: [f64; 2]) -> Option<([f64; 2], [f64; 2])> {
    let [sx, sy] = scale;
    if !(sx.abs() > f64::EPSILON && sy.abs() > f64::EPSILON) {
        return None;
    }
    let r = path_visual_bounds(path, Affine::scale_non_uniform(sx, sy))?;
    let (x0, x1) = (r.x0 / sx, r.x1 / sx);
    let (y0, y1) = (r.y0 / sy, r.y1 / sy);
    Some(([x0.min(x1), y0.min(y1)], [x0.max(x1), y0.max(y1)]))
}

#[cfg(test)]
#[path = "visual_bounds_tests.rs"]
mod tests;
