#![forbid(unsafe_code)]
//! ph2d-vec-render — pipeline de render da cena vetorial nova (ADR-0108, Fase 0).
//!
//! Converte o modelo editor-first (`ph2d-vec-scene`) em chamadas Vello, emitindo
//! no **`VectorScene` fundacional compartilhado do frame** (`ph2d-vector`) — sem
//! abrir passe de GPU novo, só anexando comandos de encode à cena que o compositor
//! já rasteriza. Toda a stack Linebender chega pelas re-exports de `ph2d-vector`
//! (gate-proof + skew-proof).
//!
//! Fase 0: draw estático da cena inteira. **Dirty-tracking** (só re-encodar a
//! sub-árvore que mudou — a alavanca de escala do ADR-0108) é o próximo passo.

use ph2d_vec_scene::{Rgba8, VecPath, VecScene};
use ph2d_vector::{Affine, BezPath, Brush, Color, Point, Stroke, VectorScene};

/// Constrói o `BezPath` (world-space) de um path editável: `move_to` na 1ª âncora,
/// depois uma cúbica por segmento usando `out_handle(i)` e `in_handle(i+1)`;
/// fecha com uma cúbica final se `closed`.
pub fn build_bezpath(path: &VecPath) -> BezPath {
    let mut bp = BezPath::new();
    let verts = &path.verts;
    let Some(first) = verts.first() else {
        return bp;
    };
    bp.move_to(pt(first.anchor));
    for pair in verts.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        bp.curve_to(pt(a.out_handle), pt(b.in_handle), pt(b.anchor));
    }
    if path.closed && verts.len() >= 2 {
        let last = verts.last().unwrap();
        bp.curve_to(pt(last.out_handle), pt(first.in_handle), pt(first.anchor));
        bp.close_path();
    }
    bp
}

/// Desenha toda a `scene` no `target` (o `VectorScene` do frame) sob `transform`
/// (o world→screen da câmera). Fill primeiro, stroke por cima.
pub fn dispatch(scene: &VecScene, transform: Affine, target: &mut VectorScene) {
    for path in scene.paths() {
        let bp = build_bezpath(path);
        if let Some(fill) = path.fill {
            target.fill_path(&bp, &Brush::Solid(color(fill)), transform);
        }
        if let Some((stroke, width)) = path.stroke {
            target.inner_mut().stroke(
                &Stroke::new(width),
                transform,
                &Brush::Solid(color(stroke)),
                None,
                &bp,
            );
        }
    }
}

#[inline]
fn pt(p: [f64; 2]) -> Point {
    Point::new(p[0], p[1])
}

#[inline]
fn color(c: Rgba8) -> Color {
    Color::from_rgba8(c.r, c.g, c.b, c.a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_path_yields_empty_bezpath() {
        let p = VecPath {
            id: 0,
            verts: vec![],
            closed: false,
            fill: None,
            stroke: None,
        };
        assert!(build_bezpath(&p).elements().is_empty());
    }

    #[test]
    fn demo_scene_builds_nonempty_paths() {
        let scene = VecScene::demo();
        for path in scene.paths() {
            assert!(!build_bezpath(path).elements().is_empty());
        }
    }
}
