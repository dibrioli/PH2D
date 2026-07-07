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

use ph2d_vec_scene::{Rgba8, VecPath, VecPathId, VecScene};
use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Rect, Stroke, VectorScene};

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

/// Desenha os **gizmos de edição** por cima da cena (screen-space, tamanho
/// constante em px): quadradinho em cada âncora; e — só no path `selected` — as
/// linhas âncora→handle + bolinhas nos handles dos vértices suaves. Cores
/// hardcoded de scaffold (Fase 1); migram p/ tokens no chrome do cutover (Fase R).
pub fn draw_overlays(
    scene: &VecScene,
    selected: Option<VecPathId>,
    transform: Affine,
    target: &mut VectorScene,
) {
    for path in scene.paths() {
        let is_sel = Some(path.id) == selected;
        if is_sel {
            for v in &path.verts {
                let a = transform * Point::new(v.anchor[0], v.anchor[1]);
                // Draw a handle only when it is offset from the anchor — cusps
                // (Corner), Smooth and Symmetric all show their non-degenerate
                // handles; a straight corner (handle == anchor) shows nothing.
                for h in [v.in_handle, v.out_handle] {
                    let (dx, dy) = (h[0] - v.anchor[0], h[1] - v.anchor[1]);
                    if dx * dx + dy * dy <= 1e-18 {
                        continue;
                    }
                    let hp = transform * Point::new(h[0], h[1]);
                    let mut line = BezPath::new();
                    line.move_to(a);
                    line.line_to(hp);
                    target.inner_mut().stroke(
                        &Stroke::new(1.0),
                        Affine::IDENTITY,
                        &Brush::Solid(Color::from_rgba8(120, 190, 230, 200)),
                        None,
                        &line,
                    );
                    target.inner_mut().fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        &Brush::Solid(Color::from_rgba8(120, 190, 230, 255)),
                        None,
                        &Circle::new(hp, 3.5),
                    );
                }
            }
        }
        for v in &path.verts {
            let a = transform * Point::new(v.anchor[0], v.anchor[1]);
            let s = 3.5;
            let col = if is_sel {
                Color::from_rgba8(250, 180, 90, 255) // laranja = selecionado
            } else {
                Color::from_rgba8(230, 230, 235, 220)
            };
            target.fill_rect(Rect::new(a.x - s, a.y - s, a.x + s, a.y + s), col);
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

    /// Spike de escala (ADR-0108 §5) — custo de re-encode NAIVE por frame (CPU,
    /// sem dirty-tracking), a fração dominante do custo em escala (achado Rive).
    /// `cargo test -p ph2d-vec-render --release -- --ignored --nocapture`
    #[test]
    #[ignore = "spike manual de medição; rode em --release --nocapture"]
    fn encode_cost_by_n() {
        use std::time::Instant;
        let affine = Affine::IDENTITY;
        println!("\n=== re-encode NAIVE por frame (CPU, sem dirty-tracking) ===");
        for &n in &[1_000usize, 5_000, 10_000, 20_000, 50_000] {
            let scene = VecScene::demo_grid(n);
            let mut target = VectorScene::new();
            target.reset();
            dispatch(&scene, affine, &mut target); // warm
            let iters = 30;
            let t = Instant::now();
            for _ in 0..iters {
                target.reset();
                dispatch(&scene, affine, &mut target);
            }
            let ms = t.elapsed().as_secs_f64() * 1000.0 / iters as f64;
            println!(
                "N={:>6}  encode={:>7.3} ms/frame   (teto encode-bound: {:>6.0} fps)",
                n,
                ms,
                1000.0 / ms
            );
        }
    }
}
