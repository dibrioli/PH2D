//! **O traço à mão e a caneta no desenho** (W4): as formas e setas em RASCUNHO (`Style::sketch`,
//! o rough.js portado em `ph2d-board-rough`, com a semente do elemento), os traços da caneta (o
//! contorno do perfect-freehand, preenchido), o rasto do laser e o anel da borracha.
//!
//! O traço à mão é caro (centenas de cúbicas por forma): guarda-se por elemento e refaz-se só
//! quando a geometria que o decide muda (a chave é um hash dela).

use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use ph2d_board_geom::Outline;
use ph2d_board_model::{Dash, Element, Ink, Pen, Rgba, ShapeType, Style};
use ph2d_board_rough::freehand;
use ph2d_board_rough::rough::{self, FillStyle, Op, OpSet, Seg, SetKind};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{
    Affine, BezPath, Brush, Cap, Color, Join, ParamCurve, PathEl, PathSeg, Point, Stroke,
    StrokeOpts, VectorScene, expand_stroke,
};

use crate::{KEEP_FRAMES, doc_color, style_stroke};

/// A transparência do MARCADOR (o Miro não a deixa mudar — help «Pen», FAQ). ⚠️ O valor dele não
/// está publicado: por medir numa captura.
pub const HIGHLIGHTER_ALPHA: f32 = 0.4;

/// Um elemento em rascunho, pronto a pintar: o preenchimento cheio, as riscas do preenchimento
/// (traçadas com a cor dele), o traço e o traço POSTO em contorno ([`outline_of`]), o que se pinta.
#[derive(Default)]
pub(crate) struct Rough {
    fill: BezPath,
    hachure: BezPath,
    line: BezPath,
    outline: BezPath,
    hachure_outline: BezPath,
}

impl Rough {
    /// Do espaço de nascença ao mundo (`f` = [`ph2d_board_model::BirthScale::factor`]), e o traço
    /// posto em contorno à espessura `width` (mundo).
    fn finish(mut self, f: f64, width: f64, dash: Dash) -> Self {
        if f != 1.0 {
            for p in [&mut self.fill, &mut self.hachure, &mut self.line] {
                p.apply_affine(Affine::scale(f));
            }
        }
        self.outline = outline_of(&self.line, width, dash);
        if !self.hachure.is_empty() {
            let hatch = Stroke::new(width.max(1.0) / 2.0).with_caps(Cap::Round);
            self.hachure_outline =
                expand_stroke(&self.hachure, &hatch, &StrokeOpts::default(), width / 16.0);
        }
        self
    }
}

/// ⭐ O traço à mão POSTO em contorno: a placa ENCHE-o em vez de o traçar — a traçar as passagens
/// finas ela soltava espigões até 35 px fora da caixa (auditoria 21/10, doc 17 §9: 3/60 rectângulos a
/// 0,5; a geometria do rough fica a ≤ 3,9 px em 5 000 sementes). Cabe na cache do elemento.
fn outline_of(line: &BezPath, width: f64, dash: Dash) -> BezPath {
    if line.is_empty() || width <= 0.0 {
        return BezPath::new();
    }
    let s = style_stroke(width, dash)
        .with_caps(Cap::Round)
        .with_join(Join::Round);
    expand_stroke(line, &s, &StrokeOpts::default(), width / 16.0)
}

/// ⭐ O espaço em que o traço NASCEU ([`ph2d_board_model::BirthScale`], decisão do dono 21/10): o
/// rascunho gera-se no tamanho que tinha no ecrã de quem o desenhou e volta ao mundo por `f`. Devolve
/// `f` e o estilo lá (a espessura do ecrã de nascença).
fn born(st: &Style) -> (f64, Style) {
    let f = st.birth.factor();
    let mut b = st.clone();
    b.stroke_width /= f;
    (f, b)
}

#[derive(Default)]
pub(crate) struct SketchCache {
    /// A tinta do tema desta imagem (posta pelo `paint`).
    pub(crate) ink: crate::ThemeInk,
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
fn cached<V>(
    map: &mut BTreeMap<(u64, u64), (u64, V, u64)>,
    owner: (u64, u64),
    key: u64,
    frame: u64,
    make: impl FnOnce() -> V,
) -> &V {
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

/// O tremor do rascunho: entre o «artista» (1) e o «cartunista» (2) do Excalidraw — ordem do dono
/// (07/10: *«linhas um pouco mais irregulares, irregularidades mais intensas»*). Abaixo de 2, para os
/// vértices continuarem PRESOS (no 2 as esquinas cruzam-se — o que o dono recusou antes).
pub const ROUGHNESS: f64 = 1.5;
/// Passagens de cada traço: TRÊS — ordem do dono (07/10: *«a linha parece dar 2 voltas por
/// desenho; coloque 3»*). As duas primeiras são as do Excalidraw (o portão do oráculo confere-as),
/// a terceira junta-se com sorteio próprio (`ph2d_board_rough::rough::Options::passes`).
pub const HAND_PASSES: u8 = 3;

/// ⭐ O tremor EFECTIVO de um elemento `w × h` no quadro (a lei abaixo sobre [`ROUGHNESS`]) — o `adjustRoughness` do Excalidraw 0.18.1, MEDIDO
/// primeiro por ajuste exacto do `d` dos SVG dele (`ferramentas/excalidraw_oracle/ajuste/`, entradas
/// `rascunho_*`: 49×49 metade, 51×51 inteiro; 19×300 metade, 20×300 inteiro; 8×8 um terço) e depois
/// lido no código dele (`dist/dev`, por ordem do dono, 07/10), que acrescentou os ramos que a amostra
/// não tinha: INTEIRO quando os dois lados são grandes (menor ≥ 20 e maior ≥ 50), ou a forma é
/// redonda com o menor ≥ 15, ou é uma LINHA (`linear`, as setas) com o maior ≥ 50; senão o tremor a
/// dividir por 2 (por 3 abaixo de 10), no máximo 2,5.
/// O tremor de base é o do estilo ([`base_roughness`]).
#[must_use]
pub fn hand_roughness(st: &Style, w: f64, h: f64, round: bool, linear: bool) -> f64 {
    adjust_roughness(base_roughness(st), w, h, round, linear)
}

/// O tremor de BASE do estilo `st`: o do rascunho do quadro ([`ROUGHNESS`]); o «cartoonist» (2) só no
/// elemento que o trouxe de um `.excalidraw` (W9, `Look::cartoonist` — o dono recusou-o para o
/// quadro, não para o desenho de quem o fez lá); 0 no final (o preenchimento às riscas de um final
/// importado desenha-se direito, como o «architect» dele).
#[must_use]
pub fn base_roughness(st: &Style) -> f64 {
    match (st.sketch, st.look.cartoonist) {
        (false, _) => 0.0,
        (true, false) => ROUGHNESS,
        (true, true) => CARTOONIST,
    }
}

/// A rugosidade 2 do Excalidraw («cartoonist»).
const CARTOONIST: f64 = 2.0;

/// A lei do Excalidraw para um tremor de base `base` qualquer (o `adjustRoughness` dele) — o que o
/// portão do oráculo confere com o `base` das saídas gravadas (1).
#[must_use]
pub fn adjust_roughness(base: f64, w: f64, h: f64, round: bool, linear: bool) -> f64 {
    let (lo, hi) = (w.abs().min(h.abs()), w.abs().max(h.abs()));
    if (lo >= 20.0 && hi >= 50.0) || (lo >= 15.0 && round) || (linear && hi >= 50.0) {
        base
    } else {
        (base / if hi < 10.0 { 3.0 } else { 2.0 }).min(2.5)
    }
}

/// As opções do rough.js de uma FORMA `w × h`: as do Excalidraw, medidas (as mesmas do `d` dele,
/// carácter a carácter) — o tremor do tamanho ([`hand_roughness`]), os VÉRTICES PRESOS com tremor
/// < 2 (as esquinas encontram-se em vez de se cruzarem), a espessura do estilo, a semente.
///
/// ⛔ O preenchimento às riscas (`hachure`) não nasce no quadro: a cor da letra lê-se sobre o
/// PREENCHIMENTO, e entre as riscas está o quadro (foto da cena 5, 07/10). Só o elemento que o TROUXE
/// de um `.excalidraw` (W9, `Look::fill_style`) o desenha. O `zigzag` (não portado nem medido, estudo
/// 07 §6.4) desenha-se às riscas.
#[must_use]
pub fn hand_options(seed: u32, roughness: f64, st: &Style, fill: bool) -> rough::Options {
    rough::Options {
        seed,
        roughness,
        preserve_vertices: roughness < 2.0,
        // Tracejado e pontilhado: um traço só (dois sobrepunham os traços — o do Excalidraw).
        disable_multi_stroke: st.dash != ph2d_board_model::Dash::Solid,
        passes: HAND_PASSES,
        stroke_width: st.stroke_width,
        fill: fill && st.fill.is_some(),
        fill_style: match st.look.fill_style {
            ph2d_board_model::FillStyle::Solid => FillStyle::Solid,
            ph2d_board_model::FillStyle::Hachure | ph2d_board_model::FillStyle::Zigzag => {
                FillStyle::Hachure
            }
            ph2d_board_model::FillStyle::CrossHatch => FillStyle::CrossHatch,
        },
        stroke: st.stroke.is_some() && st.stroke_width > 0.0,
        ..rough::Options::default()
    }
}

/// O rectângulo de cantos redondos como o Excalidraw o percorre (medido, `rascunho_cantos`): a
/// partir do fim do canto de cima à esquerda, quadráticas com o controlo no vértice, raio
/// [`ph2d_board_geom::corner_radius`].
fn rounded_rect(w: f64, h: f64) -> Vec<Seg> {
    let r = ph2d_board_geom::corner_radius(w.min(h));
    vec![
        Seg::M([r, 0.0]),
        Seg::L([w - r, 0.0]),
        Seg::Q([w, 0.0], [w, r]),
        Seg::L([w, h - r]),
        Seg::Q([w, h], [w - r, h]),
        Seg::L([r, h]),
        Seg::Q([0.0, h], [0.0, h - r]),
        Seg::L([0.0, r]),
        Seg::Q([0.0, 0.0], [r, 0.0]),
    ]
}

/// ⭐ Os sets do rascunho de uma forma na caixa LOCAL `w × h`, pelas primitivas do Excalidraw
/// (medidas): o rectângulo pelo `rectangle` (ou o caminho de cantos redondos), a elipse pelo
/// `ellipse` com `curveFitting 1`, o losango pelo `polygon` dos quatro vértices; as formas que o
/// Excalidraw não tem, pelo `path` do contorno, com as mesmas opções.
#[must_use]
pub fn hand_shape(
    kind: ShapeType,
    round: bool,
    w: f64,
    h: f64,
    o: &Outline,
    opts: &rough::Options,
) -> Vec<OpSet> {
    match (kind, round) {
        (ShapeType::Rectangle, false) => rough::rectangle(0.0, 0.0, w, h, opts),
        (ShapeType::Rectangle, true) => rough::path(&rounded_rect(w, h), opts),
        (ShapeType::Ellipse, _) => rough::ellipse(
            w / 2.0,
            h / 2.0,
            w,
            h,
            &rough::Options {
                curve_fitting: 1.0,
                ..opts.clone()
            },
        ),
        (ShapeType::Diamond, false) => rough::polygon(
            &[[w / 2.0, 0.0], [w, h / 2.0], [w / 2.0, h], [0.0, h / 2.0]],
            opts,
        ),
        _ => rough::path(&segs(&o.fill), opts),
    }
}

/// ⭐ Até onde o traço À MÃO de uma forma `w × h` sai da caixa dela (mundo): o vértice mexe até
/// `rugosidade · max_randomness_offset` e o arco do meio de uma aresta até `bowing ·
/// max_randomness_offset · L / 200` vezes a rugosidade (o `line_ops` do porto, ganho ≤ 1) — a folga da
/// camada de opacidade. ⛔ Só o traço não chegava: no «cartoonist» importado (W9, vértices soltos) os
/// cantos e as barrigas saíam da camada e eram cortados a direito (auditoria W9).
#[must_use]
pub fn hand_reach(st: &Style, w: f64, h: f64) -> f64 {
    let o = rough::Options::default();
    let f = st.birth.factor();
    let (w, h) = (w / f, h / f);
    let rough = hand_roughness(st, w, h, st.round, false);
    f * rough * o.max_randomness_offset * (1.0 + o.bowing * w.abs().max(h.abs()) / 200.0)
}

/// O rascunho de uma forma, na caixa LOCAL dela — gerado no espaço de nascença ([`born`]).
fn rough_shape(el: &Element, kind: ShapeType, st: &Style, o: &Outline) -> Rough {
    let mut r = Rough::default();
    let (f, b) = born(st);
    let (w, h) = (el.w / f, el.h / f);
    let to_birth = Affine::scale(1.0 / f);
    let o = &Outline {
        fill: to_birth * o.fill.clone(),
        lines: to_birth * o.lines.clone(),
    };
    let rough = hand_roughness(&b, w, h, b.round, false);
    let opts = hand_options(el.seed(), rough, &b, true);
    collect(hand_shape(kind, b.round, w, h, o, &opts), &mut r);
    if !o.lines.is_empty() {
        collect(
            rough::path(&segs(&o.lines), &hand_options(el.seed(), rough, &b, false)),
            &mut r,
        );
    }
    r.finish(f, st.stroke_width, st.dash)
}

/// O maior afastamento (mundo) do traço à mão SEM tremor contra a curva da seta: meio traço do de
/// nascença — à vista a guia da selecção e o traço coincidem.
pub const HAND_CURVE_TOL: f64 = 0.5;

/// ⭐ Os sets do rascunho de uma LINHA aberta (a rota de uma seta), pelas primitivas do Excalidraw
/// (medidas, `rascunho_setas`): só segmentos ⇒ `linearPath` com os vértices presos; com curvas ⇒ a
/// `curve` pelos pontos (três por cúbica — a curva do Excalidraw passa pelos pontos dela). O tremor
/// é o de uma LINHA ([`hand_roughness`] com `linear`): inteiro a partir de 50 de comprimento.
#[must_use]
pub fn hand_line(path: &BezPath, opts: &rough::Options) -> Vec<OpSet> {
    let segs: Vec<PathSeg> = path.segments().collect();
    if segs.is_empty() {
        return Vec::new();
    }
    if segs.iter().all(|s| matches!(s, PathSeg::Line(_))) {
        let mut pts = vec![[segs[0].start().x, segs[0].start().y]];
        pts.extend(segs.iter().map(|s| [s.end().x, s.end().y]));
        return rough::linear_path(&pts, opts);
    }
    rough::curve(&curve_points(&segs), opts)
}

/// ⭐ Os pontos por onde a `curve` do rough.js passa para seguir `segs`: os três por cúbica de antes
/// (o idioma do Excalidraw), e MAIS onde o Catmull-Rom 1/6 dela (o `_curve`, pontas duplicadas) se
/// afasta da curva verdadeira mais que [`HAND_CURVE_TOL`] — só onde é preciso, para o tremor continuar o
/// dele. ⛔ Só com os três, a curva do Miro em «U» afastava-se 31,8 da rota: cortava o canto à saída e
/// chegava torta à ponta (smoke do dono, 10/10).
fn curve_points(segs: &[PathSeg]) -> Vec<[f64; 2]> {
    use ph2d_vector::ParamCurveNearest;
    /// Tecto de pontos: muito acima de qualquer rota (uma curva que precisasse dele é patológica).
    const MAX_POINTS: usize = 512;
    // `(segmento, t)` de cada ponto; o 1.º é o início.
    let mut at: Vec<(usize, f64)> = vec![(0, 0.0)];
    for (i, s) in segs.iter().enumerate() {
        match s {
            PathSeg::Line(_) => at.push((i, 1.0)),
            _ => at.extend([1.0 / 3.0, 2.0 / 3.0, 1.0].map(|t| (i, t))),
        }
    }
    let point = |(i, t): (usize, f64)| {
        let p = segs[i].eval(t);
        [p.x, p.y]
    };
    // A distância à curva verdadeira, só nos segmentos do intervalo (a 1e-3 do mundo: a tolerância é 0,5).
    let far = |q: Point, from: usize, to: usize| {
        segs[from..=to]
            .iter()
            .map(|s| s.nearest(q, 1e-3).distance_sq)
            .fold(f64::INFINITY, f64::min)
            .sqrt()
    };
    loop {
        let pts: Vec<[f64; 2]> = at.iter().map(|&a| point(a)).collect();
        let n = pts.len();
        let p = |k: isize| pts[k.clamp(0, n as isize - 1) as usize];
        let mut split = Vec::new();
        for k in 0..n - 1 {
            // O troço `k` do Catmull-Rom 1/6 do rough.js (as pontas repetidas).
            let (a, b) = (p(k as isize), p(k as isize + 1));
            let (pa, nb) = (p(k as isize - 1), p(k as isize + 2));
            let c1 = [a[0] + (b[0] - pa[0]) / 6.0, a[1] + (b[1] - pa[1]) / 6.0];
            let c2 = [b[0] - (nb[0] - a[0]) / 6.0, b[1] - (nb[1] - a[1]) / 6.0];
            let eval = |u: f64| {
                let v = 1.0 - u;
                let (w0, w1, w2, w3) = (v * v * v, 3.0 * v * v * u, 3.0 * v * u * u, u * u * u);
                Point::new(
                    w0 * a[0] + w1 * c1[0] + w2 * c2[0] + w3 * b[0],
                    w0 * a[1] + w1 * c1[1] + w2 * c2[1] + w3 * b[1],
                )
            };
            let (from, to) = (at[k].0, at[k + 1].0);
            if [0.25, 0.5, 0.75]
                .into_iter()
                .any(|u| far(eval(u), from, to) > HAND_CURVE_TOL)
            {
                split.push(k);
            }
        }
        if split.is_empty() || at.len() + split.len() > MAX_POINTS {
            return pts;
        }
        // O meio de cada troço afastado, no segmento dele (no fim do anterior quando muda de segmento).
        for &k in split.iter().rev() {
            let ((i0, t0), (i1, t1)) = (at[k], at[k + 1]);
            let mid = if i0 == i1 {
                (i0, (t0 + t1) / 2.0)
            } else {
                (i1, t1 / 2.0)
            };
            at.insert(k + 1, mid);
        }
    }
}

/// ⭐ Pinta uma forma em RASCUNHO (o preenchimento e o contorno; o texto é de quem chama).
#[allow(clippy::too_many_arguments)]
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
        (st.birth, st.dash as u8).hash(h);
        (st.fill.is_some(), st.stroke.is_some(), st.round).hash(h);
        (st.sketch, st.look.cartoonist, st.look.fill_style as u8).hash(h);
    });
    let ink = cache.ink;
    let r = cached(&mut cache.rough, owner, key, frame, || {
        rough_shape(el, kind, st, o)
    });
    paint_rough(scene, r, st, t, ink);
}

fn paint_rough(scene: &mut VectorScene, r: &Rough, st: &Style, t: Affine, ink: crate::ThemeInk) {
    if let Some(c) = st.fill {
        let brush = Brush::Solid(doc_color(c));
        if !r.fill.is_empty() {
            scene.fill_path(&r.fill, &brush, t);
        }
        if !r.hachure_outline.is_empty() {
            scene.fill_path(&r.hachure_outline, &brush, t);
        }
    }
    if let Some(c) = st.stroke.filter(|_| st.stroke_width > 0.0)
        && !r.outline.is_empty()
    {
        scene.fill_path(&r.outline, &Brush::Solid(ink.color(c)), t);
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
        (st.birth, st.dash as u8).hash(h);
        st.look.cartoonist.hash(h);
        path_hash(h, &d.line);
        for (p, filled) in &d.heads {
            path_hash(h, p);
            filled.hash(h);
        }
    });
    let r = cached(&mut cache.rough, owner, key, frame, || {
        let mut r = Rough::default();
        let (f, born_st) = born(st);
        let to_birth = Affine::scale(1.0 / f);
        let path = to_birth * d.line.clone();
        let b = ph2d_vector::Shape::bounding_box(&path);
        let rough = hand_roughness(&born_st, b.width(), b.height(), false, true);
        let line = hand_options(el.seed(), rough, &born_st, false);
        collect(hand_line(&path, &line), &mut r);
        for (i, (p, filled)) in d.heads.iter().enumerate() {
            let o = rough::Options {
                seed: line.seed.wrapping_add(i as u32 + 1),
                fill: *filled,
                fill_style: FillStyle::Solid,
                ..line.clone()
            };
            collect(rough::path(&segs(&(to_birth * p.clone())), &o), &mut r);
        }
        r.finish(f, st.stroke_width, st.dash)
    });
    let Some(c) = st.stroke else {
        return;
    };
    let brush = Brush::Solid(cache.ink.color(c));
    if !r.fill.is_empty() {
        scene.fill_path(&r.fill, &brush, v);
    }
    scene.fill_path(&r.outline, &brush, v);
}

/// W23 B3 — o RAMO de um nó de mapa em rascunho: a curva dele (`path`, mundo) pelo traço à mão das
/// setas, na largura `width` (mundo) e na cor do ramo.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_branch(
    scene: &mut VectorScene,
    cache: &mut SketchCache,
    frame: u64,
    owner: (u64, u64),
    el: &Element,
    path: &BezPath,
    width: f64,
    color: ph2d_vector::Color,
    v: Affine,
) {
    let mut st = el.style().clone();
    st.stroke_width = width;
    // O ramo tem SEMPRE traço — o do nó pode não ter (os nós do mapa são só texto).
    st.stroke = Some(st.text_color);
    let key = hash(|h| {
        el.seed().hash(h);
        width.to_bits().hash(h);
        st.look.cartoonist.hash(h);
        path_hash(h, path);
    });
    let r = cached(&mut cache.rough, owner, key, frame, || {
        let mut r = Rough::default();
        let b = ph2d_vector::Shape::bounding_box(path);
        let rough = hand_roughness(&st, b.width(), b.height(), false, true);
        collect(
            hand_line(path, &hand_options(el.seed(), rough, &st, false)),
            &mut r,
        );
        r.finish(1.0, width, Dash::Solid)
    });
    scene.fill_path(&r.outline, &Brush::Solid(color), v);
}

mod ink;
pub use ink::ink_outline;
pub(crate) use ink::{paint_ink, paint_laser};

#[cfg(test)]
#[path = "sketch_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "sketch_oracle_tests.rs"]
mod oracle_tests;
