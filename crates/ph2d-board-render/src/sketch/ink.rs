//! Os traços da CANETA (o contorno do perfect-freehand, preenchido; a tinta limpa à mão no rascunho)
//! e o rasto do laser — partidos do [`super`] pelo tecto de LOC.

use super::*;

/// As opções do perfect-freehand de um traço: a espessura do estilo, sem afinar quando a pressão não
/// foi medida (a caneta do Miro não varia a largura), alisado a meio, terminado.
fn freehand_options(ink: &Ink) -> freehand::Options {
    freehand::Options {
        size: ink.style.stroke_width,
        thinning: if ink.pressure { 0.5 } else { 0.0 },
        simulate_pressure: false,
        last: true,
        ..freehand::Options::default()
    }
}

/// O contorno de um traço (caixa LOCAL), fechado por quadráticas pelos pontos médios — a receita do
/// README do perfect-freehand (`getSvgPathFromStroke`).
#[must_use]
pub fn ink_outline(ink: &Ink, w: f64, h: f64) -> BezPath {
    let input: Vec<freehand::Input> = ink
        .placed([0.0, 0.0, w, h])
        .into_iter()
        .map(|[x, y, p]| freehand::Input {
            x,
            y,
            pressure: Some(p),
        })
        .collect();
    let o = freehand_options(ink);
    let pts = freehand::outline(&freehand::stroke_points(&input, &o), &o);
    let mut path = BezPath::new();
    let Some(first) = pts.first() else {
        return path;
    };
    path.move_to((first[0], first[1]));
    for (i, p) in pts.iter().enumerate() {
        let q = pts[(i + 1) % pts.len()];
        path.quad_to((p[0], p[1]), ((p[0] + q[0]) / 2.0, (p[1] + q[1]) / 2.0));
    }
    path.close_path();
    path
}

/// A distância (espaço de nascença, px de ecrã de quem desenhou) abaixo da qual um ponto da tinta
/// limpa está sobre o lado que liga os vértices vizinhos: a lei põe um ponto por px EXACTAMENTE sobre
/// a forma, por isso meio px separa os vértices dos pontos de passagem.
const CLEAN_VERTEX_PX: f64 = 0.5;

/// Os vértices de uma polilinha (Douglas–Peucker a `tol`).
fn vertices(pts: &[[f64; 2]], tol: f64) -> Vec<[f64; 2]> {
    fn keep(pts: &[[f64; 2]], tol: f64, out: &mut Vec<[f64; 2]>) {
        let (a, b) = (pts[0], pts[pts.len() - 1]);
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let len = dx.hypot(dy);
        let far = pts[1..pts.len() - 1]
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let d = if len > 0.0 {
                    ((p[0] - a[0]) * dy - (p[1] - a[1]) * dx).abs() / len
                } else {
                    (p[0] - a[0]).hypot(p[1] - a[1])
                };
                (i + 1, d)
            })
            .max_by(|x, y| x.1.total_cmp(&y.1));
        match far {
            Some((i, d)) if d > tol => {
                keep(&pts[..=i], tol, out);
                keep(&pts[i..], tol, out);
            }
            _ => out.push(b),
        }
    }
    let mut out = vec![pts[0]];
    if pts.len() > 1 {
        keep(pts, tol, &mut out);
    }
    out
}

/// ⭐ A TINTA LIMPA de uma forma da caneta + segurar em rascunho (W23 B13, auditoria 21/10): o traço à
/// mão dos vértices dela — fechada pelo `polygon` do Excalidraw, aberta pelo `linearPath` —, como as
/// formas da ferramenta, no espaço de nascença.
fn rough_clean(el: &Element, ink: &Ink) -> Rough {
    let mut r = Rough::default();
    let (f, b) = born(&ink.style);
    let pts: Vec<[f64; 2]> = ink
        .placed([0.0, 0.0, el.w, el.h])
        .iter()
        .map(|p| [p[0] / f, p[1] / f])
        .collect();
    if pts.len() < 2 {
        return r;
    }
    let v = vertices(&pts, CLEAN_VERTEX_PX);
    let (first, last) = (v[0], v[v.len() - 1]);
    let closed = v.len() > 3 && (first[0] - last[0]).hypot(first[1] - last[1]) <= CLEAN_VERTEX_PX;
    let (lo, hi) = v
        .iter()
        .fold(([f64::MAX; 2], [f64::MIN; 2]), |(lo, hi), p| {
            (
                [lo[0].min(p[0]), lo[1].min(p[1])],
                [hi[0].max(p[0]), hi[1].max(p[1])],
            )
        });
    let rough = hand_roughness(&b, hi[0] - lo[0], hi[1] - lo[1], false, !closed);
    let opts = hand_options(el.seed(), rough, &b, false);
    collect(
        if closed {
            rough::polygon(&v[..v.len() - 1], &opts)
        } else {
            rough::linear_path(&v, &opts)
        },
        &mut r,
    );
    r.finish(f, ink.style.stroke_width, ink.style.dash)
}

/// ⭐ Pinta um traço da caneta (o marcador, translúcido); a tinta LIMPA em rascunho, à mão.
pub(crate) fn paint_ink(
    scene: &mut VectorScene,
    cache: &mut SketchCache,
    frame: u64,
    owner: (u64, u64),
    el: &Element,
    ink: &Ink,
    t: Affine,
) {
    let Some(stroke) = ink.style.stroke else {
        return;
    };
    let mut alpha = f32::from(stroke.0[3]) / 255.0 * f32::from(ink.style.opacity) / 100.0;
    if ink.pen == Pen::Highlighter {
        alpha *= HIGHLIGHTER_ALPHA;
    }
    let c = cache
        .ink
        .color(Rgba([stroke.0[0], stroke.0[1], stroke.0[2], 255]))
        .with_alpha(alpha);
    if ink.clean && ink.style.sketch {
        let st = &ink.style;
        let key = hash(|h| {
            el.seed().hash(h);
            [el.w, el.h, st.stroke_width].map(f64::to_bits).hash(h);
            (st.birth, st.dash as u8, ink.points.len()).hash(h);
        });
        let r = cached(&mut cache.rough, owner, key, frame, || rough_clean(el, ink));
        scene.fill_path(&r.outline, &Brush::Solid(c), t);
        return;
    }
    let key = hash(|h| {
        [el.w, el.h, ink.style.stroke_width]
            .map(f64::to_bits)
            .hash(h);
        ink.points.len().hash(h);
        ink.pressure.hash(h);
    });
    let path = cached(&mut cache.inks, owner, key, frame, || {
        ink_outline(ink, el.w, el.h)
    });
    scene.fill_path(path, &Brush::Solid(c), t);
}

/// O rasto do laser (mundo → ecrã por `v`): trechos que afinam e somem com a idade, na cor de
/// alerta do tema. `width` = a espessura do ponto mais novo, em px de ecrã.
pub(crate) fn paint_laser(
    scene: &mut VectorScene,
    trail: &[([f64; 2], f64)],
    v: Affine,
    theme: Theme,
    width: f64,
) {
    let base = ColorToken::Danger.resolve(theme);
    let color = Color::from_rgba8(base.r, base.g, base.b, base.a);
    for w in trail.windows(2) {
        let ((a, _), (b, life)) = (w[0], w[1]);
        let life = life.clamp(0.0, 1.0);
        let mut seg = BezPath::new();
        seg.move_to(v * Point::new(a[0], a[1]));
        seg.line_to(v * Point::new(b[0], b[1]));
        let stroke = Stroke::new(width * life.max(0.2)).with_caps(Cap::Round);
        scene.stroke_path(
            &stroke,
            Affine::IDENTITY,
            &Brush::Solid(color.with_alpha(life as f32)),
            &seg,
        );
    }
}
