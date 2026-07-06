#![forbid(unsafe_code)]
//! ph2d-vec-boolean — operações booleanas **edit-time** da pipeline vetorial nova
//! (ADR-0108, Fase 1). União / interseção / subtração de dois `VecPath` (tratados
//! como regiões fechadas) via o motor exato `linesweeper` (MIT/Apache). É
//! destrutivo por design: o resultado é um path editável, não um nó vivo — sem
//! boolean em runtime (ADR-0108 §2 D4).
//!
//! kurbo/linesweeper são internos a esta crate (não cruzam pro render, que fala a
//! kurbo do vello) → zero risco de version-skew.

use kurbo::{BezPath, PathEl, Point};
use linesweeper::{BinaryOp, FillRule};
use ph2d_vec_scene::{Rgba8, VecPath, VecVertex, VertexKind};

/// Operação booleana entre duas regiões.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BoolOp {
    Union,
    Intersect,
    Subtract,
}

/// Preenchimento do resultado (scaffold; migra p/ tokens no cutover).
const RESULT_FILL: Rgba8 = Rgba8::new(150, 120, 210, 150);

/// Aplica `op` entre `a` e `b` (fechados). Devolve os paths-resultado (podem ser
/// vários contornos); vazio se degenerar (ex.: interseção de disjuntos). O
/// resultado herda o traço de `a` (largura já na escala da câmera).
pub fn apply(a: &VecPath, b: &VecPath, op: BoolOp) -> Vec<VecPath> {
    let (ka, kb) = (to_bez(a), to_bez(b));
    let lop = match op {
        BoolOp::Union => BinaryOp::Union,
        BoolOp::Intersect => BinaryOp::Intersection,
        BoolOp::Subtract => BinaryOp::Difference,
    };
    let contours = match linesweeper::binary_op(&ka, &kb, FillRule::NonZero, lop) {
        Ok(c) => c.contours().map(|ct| ct.path.clone()).collect::<Vec<_>>(),
        Err(_) => return Vec::new(),
    };
    contours
        .iter()
        .filter_map(from_bez)
        .map(|mut p| {
            p.fill = Some(RESULT_FILL);
            p.stroke = a.stroke;
            p
        })
        .collect()
}

#[inline]
fn kp(p: [f64; 2]) -> Point {
    Point::new(p[0], p[1])
}

/// `VecPath` → kurbo `BezPath` fechado (cada segmento é uma cúbica; quina =
/// handles na âncora = reta).
fn to_bez(path: &VecPath) -> BezPath {
    let mut bp = BezPath::new();
    let Some(first) = path.verts.first() else {
        return bp;
    };
    bp.move_to(kp(first.anchor));
    for w in path.verts.windows(2) {
        bp.curve_to(kp(w[0].out_handle), kp(w[1].in_handle), kp(w[1].anchor));
    }
    if path.verts.len() >= 2 {
        let last = path.verts.last().unwrap();
        bp.curve_to(kp(last.out_handle), kp(first.in_handle), kp(first.anchor));
    }
    bp.close_path();
    bp
}

/// kurbo `BezPath` → `VecPath` (âncoras + handles reconstruídos dos elementos).
fn from_bez(bp: &BezPath) -> Option<VecPath> {
    let mut verts: Vec<VecVertex> = Vec::new();
    for el in bp.elements() {
        match *el {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => verts.push(VecVertex::corner([p.x, p.y])),
            PathEl::QuadTo(c, e) => {
                if let Some(prev) = verts.last().copied() {
                    let a = prev.anchor;
                    let c1 = [
                        a[0] + 2.0 / 3.0 * (c.x - a[0]),
                        a[1] + 2.0 / 3.0 * (c.y - a[1]),
                    ];
                    let c2 = [e.x + 2.0 / 3.0 * (c.x - e.x), e.y + 2.0 / 3.0 * (c.y - e.y)];
                    if let Some(pm) = verts.last_mut() {
                        pm.out_handle = c1;
                        pm.kind = VertexKind::Smooth;
                    }
                    verts.push(smooth([e.x, e.y], c2, [e.x, e.y]));
                }
            }
            PathEl::CurveTo(c1, c2, e) => {
                if let Some(pm) = verts.last_mut() {
                    pm.out_handle = [c1.x, c1.y];
                    pm.kind = VertexKind::Smooth;
                }
                verts.push(smooth([e.x, e.y], [c2.x, c2.y], [e.x, e.y]));
            }
            PathEl::ClosePath => {}
        }
    }
    // O contorno costuma terminar retornando ao 1º ponto → o último vértice
    // coincide com o primeiro. Funde o in-handle no primeiro e remove o duplicado.
    if verts.len() >= 2 {
        let (first_a, last_a) = (verts[0].anchor, verts[verts.len() - 1].anchor);
        if close_enough(first_a, last_a) {
            let last = verts.pop().unwrap();
            verts[0].in_handle = last.in_handle;
            if last.kind == VertexKind::Smooth {
                verts[0].kind = VertexKind::Smooth;
            }
        }
    }
    if verts.len() < 3 {
        return None; // contorno degenerado
    }
    Some(VecPath {
        id: 0,
        verts,
        closed: true,
        fill: None,
        stroke: None,
    })
}

fn smooth(anchor: [f64; 2], in_h: [f64; 2], out_h: [f64; 2]) -> VecVertex {
    VecVertex {
        anchor,
        in_handle: in_h,
        out_handle: out_h,
        kind: VertexKind::Smooth,
    }
}

fn close_enough(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-6 && (a[1] - b[1]).abs() < 1e-6
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(cx: f64, cy: f64, r: f64) -> VecPath {
        let v = |x: f64, y: f64| VecVertex::corner([x, y]);
        VecPath {
            id: 0,
            verts: vec![
                v(cx - r, cy - r),
                v(cx + r, cy - r),
                v(cx + r, cy + r),
                v(cx - r, cy + r),
            ],
            closed: true,
            fill: None,
            stroke: None,
        }
    }

    #[test]
    fn union_of_overlapping_squares_is_nonempty() {
        let a = square(0.0, 0.0, 2.0);
        let b = square(2.0, 2.0, 2.0);
        let r = apply(&a, &b, BoolOp::Union);
        assert!(!r.is_empty());
        assert!(r[0].closed);
        assert!(r[0].verts.len() >= 4);
    }

    #[test]
    fn intersect_overlapping_is_nonempty_disjoint_is_empty() {
        let a = square(0.0, 0.0, 2.0);
        let b = square(2.0, 2.0, 2.0);
        assert!(!apply(&a, &b, BoolOp::Intersect).is_empty());
        let far = square(100.0, 100.0, 2.0);
        assert!(apply(&a, &far, BoolOp::Intersect).is_empty());
    }

    #[test]
    fn subtract_overlapping_is_nonempty() {
        let a = square(0.0, 0.0, 3.0);
        let b = square(2.0, 2.0, 2.0);
        assert!(!apply(&a, &b, BoolOp::Subtract).is_empty());
    }
}
