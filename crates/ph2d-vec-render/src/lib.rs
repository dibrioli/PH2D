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

use ph2d_vec_scene::{
    FillRule as VecFillRule, LineCap, LineJoin, Paint, Rgba8, StrokeSpec, VecPath, VecPathId,
    VecScene, VecViewState, VecXforms,
};
use ph2d_vector::{
    Affine, BezPath, Brush, Cap, Circle, Color, ColorStop, Fill, Gradient, Join, Point, Rect,
    Stroke, VectorScene,
};

/// Gradient rendering (multi-point IDW fill) + on-canvas editing handles live in a
/// sibling module (LOC cap).
mod gradient;
use gradient::fill_multipoint;
pub use gradient::{GradHandle, drag_gradient_handle, draw_gradient_handles, hit_gradient_handle};

/// Smart guides (o feedback visual do snap), likewise a sibling.
mod guides;
pub use guides::{Guide, draw_snap_guides, draw_text_caret};

/// Constrói o `BezPath` (world-space) de um path editável: para CADA contorno
/// (primário + `subpaths`), `move_to` na 1ª âncora, depois uma cúbica por segmento
/// usando `out_handle(i)` e `in_handle(i+1)`; fecha com uma cúbica final se
/// `closed`. Um compound vira um só `BezPath` de vários sub-caminhos — é a
/// [`Fill`] rule que decide o que é buraco.
pub fn build_bezpath(path: &VecPath) -> BezPath {
    build_path(path, None)
}

/// O path do **PREENCHIMENTO** — só os contornos FECHADOS.
///
/// **Um contorno aberto não tem interior.** Ele é uma linha de construção: as três arestas
/// internas do cubo isométrico, a boca da base do cone, a tampa do cilindro, as barras da
/// sub-rotina, a cruz da junção. Essas coisas se DESENHAM, não se preenchem.
///
/// Sem esta distinção, o preenchimento **fecha cada contorno aberto implicitamente** (é a
/// semântica de fill de qualquer rasterizador) e a corda que fecha a linha de construção
/// vira uma região com winding próprio — que, com `NonZero`, CANCELA a silhueta onde
/// coincide. Foi exatamente o que o Enio fotografou no cubo: as arestas internas
/// `V1 → M → V3`, fechadas pela corda `V3 → V1`, abriam um triângulo escuro no meio da face
/// direita. O cone e o cilindro tinham a mesma doença em forma de lente (o arco aberto
/// fechado pela sua corda), só que menos visível.
///
/// O traço (`build_bezpath`) continua levando TUDO — é ele que desenha as linhas de
/// construção, que é a razão de elas existirem.
#[must_use]
pub fn build_fill_bezpath(path: &VecPath) -> BezPath {
    build_path(path, Some(true))
}

/// As **linhas de construção** — só os contornos ABERTOS. É o complemento exato de
/// [`build_fill_bezpath`]: o que dá volume ao sólido (as arestas internas do cubo, a boca
/// do cone, a tampa do cilindro) e que o preenchimento tem de ignorar.
///
/// Vazio para as 40 formas que não têm sub-contorno aberto.
#[must_use]
pub fn build_lines_bezpath(path: &VecPath) -> BezPath {
    build_path(path, Some(false))
}

/// `want`: `None` = todos os contornos · `Some(true)` = só os fechados · `Some(false)` = só
/// os abertos.
fn build_path(path: &VecPath, want: Option<bool>) -> BezPath {
    let mut bp = BezPath::new();
    for c in 0..path.contour_count() {
        let Some((verts, closed)) = path.contour(c) else {
            continue;
        };
        if want.is_some_and(|w| w != closed) {
            continue;
        }
        let Some(first) = verts.first() else {
            continue;
        };
        bp.move_to(pt(first.anchor));
        for pair in verts.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            bp.curve_to(pt(a.out_handle), pt(b.in_handle), pt(b.anchor));
        }
        if closed && verts.len() >= 2 {
            let last = verts.last().unwrap();
            bp.curve_to(pt(last.out_handle), pt(first.in_handle), pt(first.anchor));
            bp.close_path();
        }
    }
    bp
}

/// A [`Fill`] rule do Vello para o `fill_rule` do path.
pub(crate) fn fill_rule(path: &VecPath) -> Fill {
    match path.fill_rule {
        VecFillRule::NonZero => Fill::NonZero,
        VecFillRule::EvenOdd => Fill::EvenOdd,
    }
}

/// O afim local→tela do path: o `Transform` dele (ADR-0111), depois a câmera.
///
/// A geometria do path é LOCAL; quem a põe no mundo é `xforms`. Path ausente do
/// mapa ⇒ identidade ⇒ local é mundo, que é o estado de todo path recém-criado.
#[must_use]
pub fn path_to_screen(xforms: &VecXforms, id: VecPathId, camera: Affine) -> Affine {
    camera * Affine::new(ph2d_vec_scene::xform_of(xforms, id).0)
}

/// Desenha toda a `scene` no `target` (o `VectorScene` do frame) sob `camera`
/// (o world→screen). Fill primeiro, stroke por cima.
///
/// `view` diz quem a ÁRVORE do editor esconde — a visibilidade é da entidade ECS
/// do path e dos ancestrais dela, não do documento (ADR-0110). `xforms` diz onde
/// cada path está — o `Transform` da entidade dele (ADR-0111). O stroke escala
/// junto com a forma, como o contorno de um sprite escalado.
pub fn dispatch(
    scene: &VecScene,
    view: &VecViewState,
    xforms: &VecXforms,
    camera: Affine,
    target: &mut VectorScene,
) {
    for path in scene.paths() {
        if view.is_hidden(path.id) {
            continue;
        }
        let transform = path_to_screen(xforms, path.id, camera);
        let bp = build_bezpath(path);
        if let Some(fill) = &path.fill {
            // O preenchimento ignora os contornos ABERTOS (linhas de construção — as
            // arestas internas do cubo, a tampa do cilindro): eles não têm interior, e
            // fechá-los implicitamente recorta a silhueta. Ver [`build_fill_bezpath`].
            let fp = build_fill_bezpath(path);
            if let Paint::MultiPoint { points } = fill {
                fill_multipoint(target, &fp, path, points, transform);
            } else {
                // `VectorScene::fill_path` assume NonZero; um compound precisa da
                // regra do path (EvenOdd vaza o contorno de dentro).
                target.inner_mut().fill(
                    fill_rule(path),
                    transform,
                    &fill_brush(fill, path),
                    None,
                    &fp,
                );
            }
        }
        if let Some(s) = path.stroke {
            target.inner_mut().stroke(
                &kurbo_stroke(&s),
                transform,
                &Brush::Solid(color(s.color)),
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
#[allow(clippy::too_many_arguments)]
pub fn draw_overlays(
    scene: &VecScene,
    view: &VecViewState,
    selected: Option<VecPathId>,
    selected_paths: &[VecPathId],
    selected_verts: &[usize],
    xforms: &VecXforms,
    camera: Affine,
    target: &mut VectorScene,
) {
    for path in scene.paths() {
        if view.is_hidden(path.id) {
            continue; // um path escondido não mostra âncoras
        }
        // Âncoras e handles são LOCAIS: passam pelo Transform do path antes da câmera.
        let transform = path_to_screen(xforms, path.id, camera);
        let is_sel = Some(path.id) == selected;
        // Any path in the OBJECT selection set is highlighted; the primary also shows
        // its Bézier handles + per-vertex picks.
        let in_set = selected_paths.contains(&path.id);
        if is_sel {
            for (i, v) in path.verts_all().enumerate() {
                let a = transform * Point::new(v.anchor[0], v.anchor[1]);
                // Draw each handle at its DISPLAY position: a real (offset) handle
                // as-is, and a Smooth/Symmetric point's zero-length ("invisible")
                // handle as a grabbable GHOST stub along the smooth tangent — without
                // touching geometry (the curve only changes when the user drags it).
                // A straight Corner's zero handle stays hidden (`ghost_handle` = None).
                for out in [false, true] {
                    let real = if out { v.out_handle } else { v.in_handle };
                    let (dx, dy) = (real[0] - v.anchor[0], real[1] - v.anchor[1]);
                    let is_ghost = dx * dx + dy * dy <= 1e-18;
                    let Some(h) = ph2d_vec_scene::ghost_handle(path, i, out) else {
                        continue;
                    };
                    let hp = transform * Point::new(h[0], h[1]);
                    let mut line = BezPath::new();
                    line.move_to(a);
                    line.line_to(hp);
                    // Ghost stubs are dimmer + hollow so the user reads them as
                    // "suggested, not yet affecting the curve".
                    let line_a = if is_ghost { 120 } else { 200 };
                    target.inner_mut().stroke(
                        &Stroke::new(1.0),
                        Affine::IDENTITY,
                        &Brush::Solid(Color::from_rgba8(120, 190, 230, line_a)),
                        None,
                        &line,
                    );
                    if is_ghost {
                        target.inner_mut().stroke(
                            &Stroke::new(1.2),
                            Affine::IDENTITY,
                            &Brush::Solid(Color::from_rgba8(120, 190, 230, 220)),
                            None,
                            &Circle::new(hp, 3.5),
                        );
                    } else {
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
        }
        // Flat index across contours — the same space `hit_test` / `selected_verts`
        // address, so a hole's anchors pick exactly like the outer contour's.
        for (i, v) in path.verts_all().enumerate() {
            let a = transform * Point::new(v.anchor[0], v.anchor[1]);
            // A vertex in the multi-selection (selected path only) is drawn bigger
            // + cyan; other anchors of the selected path are orange; other paths gray.
            let picked = is_sel && selected_verts.contains(&i);
            let s = if picked { 4.5 } else { 3.5 };
            let col = if picked {
                Color::from_rgba8(90, 200, 235, 255) // ciano = vértice selecionado (grupo)
            } else if in_set {
                Color::from_rgba8(250, 180, 90, 255) // laranja = path na seleção de objeto
            } else {
                Color::from_rgba8(230, 230, 235, 220)
            };
            target.fill_rect(Rect::new(a.x - s, a.y - s, a.x + s, a.y + s), col);
        }
    }
}

/// Desenha a caixa de **marquee** (box-select) em **screen-space** (o shell
/// passa cantos de tela): preenchimento translúcido + contorno. Chamada só
/// enquanto o Shift+arrasto está ativo.
pub fn draw_marquee(min: [f64; 2], max: [f64; 2], target: &mut VectorScene) {
    let (x0, x1) = (min[0].min(max[0]), min[0].max(max[0]));
    let (y0, y1) = (min[1].min(max[1]), min[1].max(max[1]));
    let rect = Rect::new(x0, y0, x1, y1);
    target.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(Color::from_rgba8(90, 200, 235, 40)),
        None,
        &rect,
    );
    let mut outline = BezPath::new();
    outline.move_to(Point::new(x0, y0));
    outline.line_to(Point::new(x1, y0));
    outline.line_to(Point::new(x1, y1));
    outline.line_to(Point::new(x0, y1));
    outline.close_path();
    target.inner_mut().stroke(
        &Stroke::new(1.0),
        Affine::IDENTITY,
        &Brush::Solid(Color::from_rgba8(90, 200, 235, 200)),
        None,
        &outline,
    );
}

/// `StrokeSpec` → `kurbo::Stroke` (ponta/junção + dash). Larguras/dashes ficam em
/// world-units; o `transform` do render escala p/ screen.
fn kurbo_stroke(s: &StrokeSpec) -> Stroke {
    let cap = match s.cap {
        LineCap::Butt => Cap::Butt,
        LineCap::Round => Cap::Round,
        LineCap::Square => Cap::Square,
    };
    let join = match s.join {
        LineJoin::Miter => Join::Miter,
        LineJoin::Round => Join::Round,
        LineJoin::Bevel => Join::Bevel,
    };
    let stroke = Stroke::new(s.width).with_caps(cap).with_join(join);
    // `dash` carries width MULTIPLES `(dash, gap)` (width-aware — a thicker line
    // gets proportionally longer dash + gap, so the cap projection never swallows
    // the gap). A zero-length gap collapses to a solid look; clamp it off zero so
    // kurbo never emits a degenerate dash element.
    match s.dash {
        Some((d, g)) if d > 0.0 => {
            let dash_len = d * s.width;
            let gap_len = (g * s.width).max(f64::EPSILON);
            stroke.with_dashes(0.0, [dash_len, gap_len])
        }
        _ => stroke,
    }
}

#[inline]
fn pt(p: [f64; 2]) -> Point {
    Point::new(p[0], p[1])
}

#[inline]
pub(crate) fn color(c: Rgba8) -> Color {
    Color::from_rgba8(c.r, c.g, c.b, c.a)
}

/// Peniko color stops from our gradient stops (`(offset f32, Color)` → `ColorStop`),
/// SORTED by offset — interior stops may cross one another in the editor (their Vec
/// order isn't guaranteed monotonic), but peniko wants non-decreasing offsets.
fn stops_of(stops: &[ph2d_vec_scene::GradientStop]) -> Vec<ColorStop> {
    let mut out: Vec<ColorStop> = stops
        .iter()
        .map(|s| ColorStop::from((s.offset as f32, color(s.color))))
        .collect();
    out.sort_by(|a, b| {
        a.offset
            .partial_cmp(&b.offset)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

/// Build the Vello fill brush for a path's [`Paint`]. Linear/Radial map to native
/// peniko gradients using the paint's OWN world-space geometry (start/end,
/// center/radius) — which transforms rigidly with the path, so the gradient never
/// "breathes" under rotation. The frame's world→screen transform maps them.
/// MultiPoint is handled by `fill_multipoint` (image-clip path), never here.
fn fill_brush(paint: &Paint, _path: &VecPath) -> Brush {
    match paint {
        Paint::Solid(c) => Brush::Solid(color(*c)),
        Paint::Linear { stops, start, end } => {
            let a = Point::new(start[0], start[1]);
            let b = Point::new(end[0], end[1]);
            Brush::Gradient(Gradient::new_linear(a, b).with_stops(stops_of(stops).as_slice()))
        }
        Paint::Radial {
            stops,
            center,
            radius,
        } => {
            let c = Point::new(center[0], center[1]);
            let r = (*radius as f32).max(f32::MIN_POSITIVE);
            Brush::Gradient(Gradient::new_radial(c, r).with_stops(stops_of(stops).as_slice()))
        }
        // MultiPoint is handled by `fill_multipoint` (image-clip path), never here.
        Paint::MultiPoint { .. } => Brush::Solid(color(paint.primary_color())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_path_yields_empty_bezpath() {
        let p = VecPath::default();
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
            let xf = VecXforms::new();
            dispatch(&scene, &VecViewState::default(), &xf, affine, &mut target); // warm
            let iters = 30;
            let t = Instant::now();
            for _ in 0..iters {
                target.reset();
                dispatch(&scene, &VecViewState::default(), &xf, affine, &mut target);
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

#[cfg(test)]
mod open_contour_tests {
    use super::*;
    use ph2d_vector::Shape;

    /// **O triângulo escuro do cubo, executável.**
    ///
    /// O cubo isométrico é uma silhueta hexagonal FECHADA + três arestas internas, que são
    /// contornos ABERTOS. Preencher tudo junto faz o rasterizador **fechar cada contorno
    /// aberto implicitamente** (é a semântica de fill, em qualquer engine): a corda que
    /// fecha a polilinha `V1 → M → V3` vira uma região de winding próprio que, com
    /// `NonZero`, CANCELA o hexágono onde coincide. O Enio fotografou exatamente isso — um
    /// triângulo escuro comendo metade da face direita.
    ///
    /// O teste mede o winding NO PONTO do triângulo. Ele prova as duas metades da história:
    /// o path do preenchimento cobre a face (winding ≠ 0) **e** o path completo — que era o
    /// que se preenchia antes — a perfura (winding = 0). Se alguém reverter a correção, a
    /// segunda asserção passa a valer para o primeiro path e o teste cai.
    #[test]
    fn an_open_contour_never_punches_a_hole_in_the_fill() {
        let cube = ph2d_vec_scene::iso_cube([-1.0, -1.0], [1.0, 1.0], 0.5, 0.5);
        // As três arestas internas vivem no sub-contorno; o vértice central é o do meio.
        let inner = &cube.subpaths[0].verts;
        let (v1, m, v3) = (inner[0].anchor, inner[1].anchor, inner[2].anchor);
        // Um ponto BEM dentro do triângulo V1–M–V3: o baricentro. É o miolo da mancha.
        let p = Point::new((v1[0] + m[0] + v3[0]) / 3.0, (v1[1] + m[1] + v3[1]) / 3.0);

        let fill = build_fill_bezpath(&cube);
        assert_ne!(
            fill.winding(p),
            0,
            "a face direita do cubo tem de ser PREENCHIDA em {p:?} — o triangulo escuro voltou"
        );

        // E a prova de que o bug era real: com os contornos abertos dentro do preenchimento,
        // o mesmo ponto FICA DE FORA. (É o que se pintava antes.)
        let everything = build_bezpath(&cube);
        assert_eq!(
            everything.winding(p),
            0,
            "o teste perdeu o poder de discriminar: o contorno aberto deveria furar o fill"
        );

        // As linhas de construção são exatamente as arestas internas — e não estão no fill.
        assert!(
            !build_lines_bezpath(&cube).is_empty(),
            "as arestas internas do cubo TEM de ser desenhadas (senao e um hexagono)"
        );
    }

    /// A regra vale para toda forma do catálogo: o path do preenchimento nunca contém um
    /// contorno aberto, e a soma dos dois caminhos é o path inteiro.
    #[test]
    fn fill_and_lines_partition_every_shape_in_the_catalogue() {
        for &kind in ph2d_vec_scene::ALL_SHAPES {
            let path = ph2d_vec_scene::cook(kind, [-1.0, -1.0], [1.0, 1.0], &kind.defaults());
            let (fill, lines, whole) = (
                build_fill_bezpath(&path),
                build_lines_bezpath(&path),
                build_bezpath(&path),
            );
            assert_eq!(
                fill.elements().len() + lines.elements().len(),
                whole.elements().len(),
                "{kind:?}: fill + linhas tem de particionar o path inteiro"
            );
            // Uma forma ABERTA (linha, arco, espiral, chave) não tem nada a preencher.
            if !kind.is_closed() {
                assert!(
                    fill.is_empty(),
                    "{kind:?} e aberta — nao tem interior para preencher"
                );
            }
        }
    }
}
