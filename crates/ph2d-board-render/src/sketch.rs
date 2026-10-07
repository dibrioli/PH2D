//! **O traço à mão e a caneta no desenho** (W4): as formas e setas em RASCUNHO (`Style::sketch`,
//! o rough.js portado em `ph2d-board-rough`, com a semente do elemento), os traços da caneta (o
//! contorno do perfect-freehand, preenchido), o rasto do laser e o anel da borracha.
//!
//! O traço à mão é caro (centenas de cúbicas por forma): guarda-se por elemento e refaz-se só
//! quando a geometria que o decide muda (a chave é um hash dela).

use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use ph2d_board_geom::Outline;
use ph2d_board_model::{Element, Ink, Pen, Rgba, ShapeType, Style};
use ph2d_board_rough::freehand;
use ph2d_board_rough::rough::{self, FillStyle, Op, OpSet, Seg, SetKind};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Cap, Color, Join, PathEl, Point, Stroke, VectorScene};

use crate::{KEEP_FRAMES, doc_color, style_stroke};

/// A transparência do MARCADOR (o Miro não a deixa mudar — help «Pen», FAQ). ⚠️ O valor dele não
/// está publicado: por medir numa captura.
pub const HIGHLIGHTER_ALPHA: f32 = 0.4;

/// Um elemento em rascunho, pronto a pintar: o preenchimento cheio, as riscas do preenchimento
/// (traçadas com a cor dele) e o traço.
#[derive(Default)]
pub(crate) struct Rough {
    fill: BezPath,
    hachure: BezPath,
    line: BezPath,
}

#[derive(Default)]
pub(crate) struct SketchCache {
    rough: BTreeMap<(u64, u64), (u64, Rough, u64)>,
    inks: BTreeMap<(u64, u64), (u64, BezPath, u64)>,
}

impl SketchCache {
    pub(crate) fn end_frame(&mut self, frame: u64) {
        if frame.is_multiple_of(KEEP_FRAMES) {
            let cut = frame.saturating_sub(KEEP_FRAMES);
            self.rough.retain(|_, e| e.2 >= cut);
            self.inks.retain(|_, e| e.2 >= cut);
        }
    }
}

/// O valor guardado de `owner` se a chave é a mesma; senão refaz-se com `make`.
fn cached<'a, V>(
    map: &'a mut BTreeMap<(u64, u64), (u64, V, u64)>,
    owner: (u64, u64),
    key: u64,
    frame: u64,
    make: impl FnOnce() -> V,
) -> &'a V {
    use std::collections::btree_map::Entry;
    let e = match map.entry(owner) {
        Entry::Occupied(o) => {
            let e = o.into_mut();
            if e.0 != key {
                *e = (key, make(), frame);
            }
            e
        }
        Entry::Vacant(v) => v.insert((key, make(), frame)),
    };
    e.2 = frame;
    &e.1
}

fn hash(f: impl FnOnce(&mut DefaultHasher)) -> u64 {
    let mut h = DefaultHasher::new();
    f(&mut h);
    h.finish()
}

fn path_hash(h: &mut DefaultHasher, p: &BezPath) {
    for el in p.elements() {
        let pts: &[Point] = match el {
            PathEl::MoveTo(a) | PathEl::LineTo(a) => std::slice::from_ref(a),
            PathEl::QuadTo(a, b) => &[*a, *b],
            PathEl::CurveTo(a, b, c) => &[*a, *b, *c],
            PathEl::ClosePath => &[],
        };
        std::mem::discriminant(el).hash(h);
        for q in pts {
            q.x.to_bits().hash(h);
            q.y.to_bits().hash(h);
        }
    }
}

/// O caminho do kurbo nos segmentos que o rough.js lê.
fn segs(p: &BezPath) -> Vec<Seg> {
    let pt = |q: &Point| [q.x, q.y];
    p.elements()
        .iter()
        .map(|el| match el {
            PathEl::MoveTo(a) => Seg::M(pt(a)),
            PathEl::LineTo(a) => Seg::L(pt(a)),
            PathEl::QuadTo(a, b) => Seg::Q(pt(a), pt(b)),
            PathEl::CurveTo(a, b, c) => Seg::C(pt(a), pt(b), pt(c)),
            PathEl::ClosePath => Seg::Z,
        })
        .collect()
}

fn bez(ops: &[Op], into: &mut BezPath) {
    for op in ops {
        match *op {
            Op::Move([x, y]) => into.move_to((x, y)),
            Op::Line([x, y]) => into.line_to((x, y)),
            Op::Cubic(a, b, c) => into.curve_to((a[0], a[1]), (b[0], b[1]), (c[0], c[1])),
        }
    }
}

fn collect(sets: Vec<OpSet>, into: &mut Rough) {
    for s in sets {
        let target = match s.kind {
            SetKind::Path => &mut into.line,
            SetKind::FillPath => &mut into.fill,
            SetKind::FillSketch => &mut into.hachure,
        };
        bez(&s.ops, target);
    }
}

/// As opções do rough.js de um elemento: a mão do Excalidraw «artista» (`roughness 1`, `bowing 1`),
/// a espessura do estilo, o preenchimento CHEIO de borda tremida e a SEMENTE do elemento.
///
/// ⛔ As riscas (`hachure`) não: a cor da letra de uma forma lê-se sobre o PREENCHIMENTO, e entre as
/// riscas está o quadro — a mesma letra ficava ilegível num dos dois (foto da cena 5, 07/10: branca
/// sobre riscas pastel num quadro escuro). Cheio, a letra lê-se igual em rascunho e em final.
fn options(el: &Element, st: &Style, fill: bool) -> rough::Options {
    rough::Options {
        seed: el.seed(),
        stroke_width: st.stroke_width.max(1.0),
        fill: fill && st.fill.is_some(),
        fill_style: FillStyle::Solid,
        stroke: st.stroke.is_some() && st.stroke_width > 0.0,
        ..rough::Options::default()
    }
}

/// O rascunho de uma forma, na caixa LOCAL dela.
fn rough_shape(el: &Element, kind: ShapeType, st: &Style, o: &Outline) -> Rough {
    let mut r = Rough::default();
    let opts = options(el, st, true);
    let sets = if kind == ShapeType::Ellipse {
        rough::ellipse(el.w / 2.0, el.h / 2.0, el.w, el.h, &opts)
    } else {
        rough::path(&segs(&o.fill), &opts)
    };
    collect(sets, &mut r);
    if !o.lines.is_empty() {
        collect(
            rough::path(&segs(&o.lines), &options(el, st, false)),
            &mut r,
        );
    }
    r
}

/// ⭐ Pinta uma forma em RASCUNHO (o preenchimento e o contorno; o texto é de quem chama).
pub(crate) fn paint_shape(
    scene: &mut VectorScene,
    cache: &mut SketchCache,
    frame: u64,
    owner: (u64, u64),
    el: &Element,
    kind: ShapeType,
    o: &Outline,
    t: Affine,
) {
    let st = el.style();
    let key = hash(|h| {
        kind.hash(h);
        el.seed().hash(h);
        [el.w, el.h, st.stroke_width].map(f64::to_bits).hash(h);
        (st.fill.is_some(), st.stroke.is_some(), st.round).hash(h);
    });
    let r = cached(&mut cache.rough, owner, key, frame, || {
        rough_shape(el, kind, st, o)
    });
    paint_rough(scene, r, st, t);
}

fn paint_rough(scene: &mut VectorScene, r: &Rough, st: &Style, t: Affine) {
    if let Some(c) = st.fill {
        let brush = Brush::Solid(doc_color(c));
        if !r.fill.is_empty() {
            scene.fill_path(&r.fill, &brush, t);
        }
        if !r.hachure.is_empty() {
            let hatch = Stroke::new(st.stroke_width.max(1.0) / 2.0).with_caps(Cap::Round);
            scene
                .inner_mut()
                .stroke(&hatch, t, &brush, None, &r.hachure);
        }
    }
    if let Some(c) = st.stroke.filter(|_| st.stroke_width > 0.0)
        && !r.line.is_empty()
    {
        let line = style_stroke(st.stroke_width, st.dash)
            .with_caps(Cap::Round)
            .with_join(Join::Round);
        scene
            .inner_mut()
            .stroke(&line, t, &Brush::Solid(doc_color(c)), None, &r.line);
    }
}

/// ⭐ Pinta a linha e as pontas de uma seta em RASCUNHO (`d` = o que `ph2d_board_route::drawn` deu).
pub(crate) fn paint_connector(
    scene: &mut VectorScene,
    cache: &mut SketchCache,
    frame: u64,
    owner: (u64, u64),
    el: &Element,
    d: &ph2d_board_route::Drawn,
    v: Affine,
) {
    let st = el.style();
    let key = hash(|h| {
        el.seed().hash(h);
        st.stroke_width.to_bits().hash(h);
        path_hash(h, &d.line);
        for (p, filled) in &d.heads {
            path_hash(h, p);
            filled.hash(h);
        }
    });
    let r = cached(&mut cache.rough, owner, key, frame, || {
        let mut r = Rough::default();
        let line = rough::Options {
            fill: false,
            ..options(el, st, false)
        };
        collect(rough::path(&segs(&d.line), &line), &mut r);
        for (i, (p, filled)) in d.heads.iter().enumerate() {
            let o = rough::Options {
                seed: line.seed.wrapping_add(i as u32 + 1),
                fill: *filled,
                fill_style: FillStyle::Solid,
                ..line.clone()
            };
            collect(rough::path(&segs(p), &o), &mut r);
        }
        r
    });
    let Some(c) = st.stroke else {
        return;
    };
    let brush = Brush::Solid(doc_color(c));
    if !r.fill.is_empty() {
        scene.fill_path(&r.fill, &brush, v);
    }
    let line = style_stroke(st.stroke_width, st.dash)
        .with_caps(Cap::Round)
        .with_join(Join::Round);
    scene.inner_mut().stroke(&line, v, &brush, None, &r.line);
}

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

/// ⭐ Pinta um traço da caneta (o marcador, translúcido).
pub(crate) fn paint_ink(
    scene: &mut VectorScene,
    cache: &mut SketchCache,
    frame: u64,
    owner: (u64, u64),
    el: &Element,
    ink: &Ink,
    t: Affine,
) {
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
    let Some(Rgba([r, g, b, a])) = ink.style.stroke else {
        return;
    };
    let mut alpha = f32::from(a) / 255.0 * f32::from(ink.style.opacity) / 100.0;
    if ink.pen == Pen::Highlighter {
        alpha *= HIGHLIGHTER_ALPHA;
    }
    // LITERAL-COLOR-OK: cor do DOCUMENTO (a do traço), com a transparência do marcador.
    let c = Color::from_rgba8(r, g, b, 255).with_alpha(alpha);
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
        scene.inner_mut().stroke(
            &stroke,
            Affine::IDENTITY,
            &Brush::Solid(color.with_alpha(life as f32)),
            None,
            &seg,
        );
    }
}

#[cfg(test)]
#[path = "sketch_tests.rs"]
mod tests;
