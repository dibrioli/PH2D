//! ⭐ W23 B13 — **o QuickShape do Vetor na caneta do quadro** (dono 20/10: *«exatamente como no
//! módulo Vector»*; doc 17 §2.2). A lei (`ph2d_vec_edit::quickshape::recognise`) e o gatilho
//! (`quickshape_hold::Hold`: parado 500 ms dentro de 10 px; o arrasto sem soltar escala e roda; Shift
//! regulariza; Esc devolve o traço) são OS do vetor — o quadro não tem limiares seus.
//!
//! Ao soltar, a forma vira o objecto do quadro que a desenha IGUAL (a menos de um píxel de ecrã):
//! rectângulo, losango, elipse, triângulo…, e a recta/seta vira uma ligação (presa à forma onde a ponta
//! cai, o Excalidraw). Onde nenhum a desenha igual (um triângulo qualquer, um arco), fica uma tinta com
//! a geometria LIMPA da lei — nunca uma forma do quadro que mude o desenho. O traço cru nasce num passo
//! e a troca é outro: um `Ctrl+Z` devolve o traço como foi desenhado.

use ph2d_board_model::{
    BoardDoc, Connector, Element, ElementId, FracKey, Head, Ink, Pen, RichText, Route, Shape,
    ShapeType, Style,
};
use ph2d_vec_edit::quickshape::QuickShape;
use ph2d_vec_edit::quickshape_hold::{HOLD_SLOP_PX, Hold, HoldTick, rest_index};

use crate::{Editor, Mods, Pointer};

/// O estado do gatilho num traço.
#[derive(Default)]
pub(crate) struct Smart {
    pub(crate) hold: Hold,
    /// O relógio (ns) de cada ponto do traço.
    pub(crate) times: Vec<u128>,
    /// Px de ecrã → mundo (as folgas do vetor são de ecrã).
    pub(crate) px: f64,
    /// O último `Shift` (o `regular` do vetor).
    pub(crate) regular: bool,
}

/// Os tipos do quadro que se tentam, por esta ordem, para um polígono fechado.
const POLYGON_KINDS: [ShapeType; 7] = [
    ShapeType::Rectangle,
    ShapeType::Diamond,
    ShapeType::Triangle,
    ShapeType::Parallelogram,
    ShapeType::Trapezoid,
    ShapeType::Hexagon,
    ShapeType::Star,
];

/// O ângulo das farpas da seta desenhada como tinta (a ponta `Arrow` do quadro).
const BARB_DEG: f64 = 30.0;

impl Smart {
    pub(crate) fn new(now: u128, px: f64) -> Self {
        Self {
            hold: Hold::default(),
            times: vec![now],
            px: px.max(f64::MIN_POSITIVE),
            regular: false,
        }
    }

    /// Um quadro do gatilho sobre os pontos `pts` do traço. `enabled` = o interruptor.
    pub(crate) fn tick(&mut self, pts: &[[f64; 3]], now: u128, enabled: bool) -> HoldTick {
        let samples: Vec<[f64; 2]> = pts.iter().map(|p| [p[0], p[1]]).collect();
        let j = rest_index(&samples, HOLD_SLOP_PX * self.px);
        let rest = self.times.get(j).copied().unwrap_or(now);
        self.hold
            .tick(&samples, rest, now, self.px, self.regular, enabled)
    }
}

/// Os pontos (mundo) que DESENHAM a forma como tinta: o contorno achatado e, numa seta, as farpas.
#[must_use]
pub fn shape_points(shape: &QuickShape, px: f64) -> Vec<[f64; 2]> {
    let tol = (px * 0.25).max(1e-9);
    let mut pts: Vec<[f64; 2]> = ph2d_board_geom::cover::vec_polyline(&shape.to_path(), tol)
        .into_iter()
        .flatten()
        .collect();
    if let QuickShape::Arrow(a) = shape {
        let shaft = pts.clone();
        for (i, len) in a.heads.iter().enumerate() {
            let (Some(len), true) = (len, shaft.len() >= 2) else {
                continue;
            };
            let (tip, back) = if i == 0 {
                (shaft[0], shaft[1])
            } else {
                (shaft[shaft.len() - 1], shaft[shaft.len() - 2])
            };
            let d = [back[0] - tip[0], back[1] - tip[1]];
            let n = d[0].hypot(d[1]).max(f64::MIN_POSITIVE);
            let u = [d[0] / n, d[1] / n];
            let barb = |s: f64| {
                let (sn, cs) = (s * BARB_DEG).to_radians().sin_cos();
                [
                    tip[0] + len * (u[0] * cs - u[1] * sn),
                    tip[1] + len * (u[0] * sn + u[1] * cs),
                ]
            };
            if i == 1 {
                pts.extend([barb(1.0), tip, barb(-1.0)]);
            } else {
                let mut head = vec![barb(1.0), tip, barb(-1.0), tip];
                head.extend(pts);
                pts = head;
            }
        }
    }
    pts
}

fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

/// A distância de `p` à linha fechada `poly`.
fn to_poly(p: [f64; 2], poly: &[[f64; 2]]) -> f64 {
    (0..poly.len())
        .map(|i| crate::ink::point_segment(p, poly[i], poly[(i + 1) % poly.len()]))
        .fold(f64::INFINITY, f64::min)
}

/// Pontos ao longo da linha fechada `poly`, um a cada `step`.
fn along(poly: &[[f64; 2]], step: f64) -> Vec<[f64; 2]> {
    let mut out = Vec::new();
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        let l = sub(b, a)[0].hypot(sub(b, a)[1]);
        let n = (l / step).ceil().max(1.0) as usize;
        out.extend((0..n).map(|k| {
            let t = k as f64 / n as f64;
            [a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])]
        }));
    }
    out
}

/// As duas linhas fechadas desenham o MESMO a menos de `tol` (Hausdorff pelas amostras).
fn same_outline(a: &[[f64; 2]], b: &[[f64; 2]], tol: f64) -> bool {
    let step = tol / 2.0;
    along(a, step).into_iter().all(|p| to_poly(p, b) <= tol)
        && along(b, step).into_iter().all(|p| to_poly(p, a) <= tol)
}

/// O estilo de um objecto feito pela caneta: o traço e a cor dela, sem preenchimento, cantos vivos.
fn shape_style(pen: &Style) -> Style {
    let ink = pen.stroke.unwrap_or(pen.text_color);
    let mut s = Style::new(None, Some(ink), ink);
    s.stroke_width = pen.stroke_width;
    s.round = false;
    s.opacity = pen.opacity;
    s
}

/// O polígono `corners` como forma do quadro, se uma o desenha igual (a menos de `tol`).
fn polygon_shape(
    corners: &[[f64; 2]],
    style: &Style,
    id: ElementId,
    z: &FracKey,
    tol: f64,
) -> Option<Element> {
    let n = corners.len();
    let mut dirs: Vec<f64> = (0..n)
        .map(|i| {
            let d = sub(corners[(i + 1) % n], corners[i]);
            d[1].atan2(d[0])
        })
        .collect();
    if n == 4 {
        for i in 0..2 {
            let d = sub(corners[i + 2], corners[i]);
            dirs.push(d[1].atan2(d[0]));
        }
    }
    for kind in POLYGON_KINDS {
        for &base in &dirs {
            for q in 0..4 {
                let angle = base + f64::from(q) * std::f64::consts::FRAC_PI_2;
                let (s, c) = angle.sin_cos();
                let local: Vec<[f64; 2]> = corners
                    .iter()
                    .map(|p| [p[0] * c + p[1] * s, -p[0] * s + p[1] * c])
                    .collect();
                let (lo, hi) =
                    local
                        .iter()
                        .fold(([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]), |(l, h), p| {
                            (
                                [l[0].min(p[0]), l[1].min(p[1])],
                                [h[0].max(p[0]), h[1].max(p[1])],
                            )
                        });
                let (w, h) = (hi[0] - lo[0], hi[1] - lo[1]);
                if w <= tol || h <= tol {
                    continue;
                }
                let m = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
                let centre = [m[0] * c - m[1] * s, m[0] * s + m[1] * c];
                let shape = Shape {
                    kind,
                    style: style.clone(),
                    text: RichText::default(),
                };
                let mut el = Element::new_shape(
                    id,
                    z.clone(),
                    shape,
                    [centre[0] - w / 2.0, centre[1] - h / 2.0, w, h],
                );
                el.angle = angle;
                let Some(o) = ph2d_board_geom::world_outline(&el) else {
                    continue;
                };
                let outline = ph2d_board_geom::cover::polygons(&o.fill, tol / 4.0);
                if let [poly] = outline.as_slice()
                    && same_outline(poly, corners, tol)
                {
                    return Some(el);
                }
            }
        }
    }
    None
}

impl Editor {
    /// ⭐ O objecto do quadro que a forma `shape` vira (o estilo `style` da caneta; `raw` = o traço cru,
    /// que não prende pontas); `None` = nenhum a desenha igual — fica a tinta limpa ([`shape_points`]).
    pub(crate) fn smart_element(
        &mut self,
        doc: &mut BoardDoc,
        shape: &QuickShape,
        style: &Style,
        px: f64,
        raw: ElementId,
    ) -> Option<Element> {
        let id = doc.mint_id();
        let z = doc.z_on_top();
        let st = shape_style(style);
        let tol = px;
        match shape {
            QuickShape::Ellipse {
                centre,
                rx,
                ry,
                rot,
            } => {
                let s = Shape {
                    kind: ShapeType::Ellipse,
                    style: st,
                    text: RichText::default(),
                };
                let mut el = Element::new_shape(
                    id,
                    z,
                    s,
                    [centre[0] - rx, centre[1] - ry, 2.0 * rx, 2.0 * ry],
                );
                el.angle = *rot;
                Some(el)
            }
            QuickShape::Polygon {
                corners,
                closed: true,
            } => polygon_shape(corners, &st, id, &z, tol),
            QuickShape::Line { a, b } => {
                Some(self.smart_wire(doc, [id, raw], z, [*a, *b], [None; 2], st, px))
            }
            QuickShape::Arrow(arrow) => match arrow.shaft.as_ref() {
                QuickShape::Line { a, b } => {
                    Some(self.smart_wire(doc, [id, raw], z, [*a, *b], arrow.heads, st, px))
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// A recta de `a` a `b` como ligação: cada ponta presa à forma onde cai (a porta da seta).
    #[allow(clippy::too_many_arguments)]
    fn smart_wire(
        &mut self,
        doc: &BoardDoc,
        [id, raw]: [ElementId; 2],
        z: FracKey,
        [a, b]: [[f64; 2]; 2],
        heads: [Option<f64>; 2],
        style: Style,
        px: f64,
    ) -> Element {
        let at = |p: [f64; 2]| Pointer {
            world: p,
            px,
            mods: Mods::default(),
        };
        let (start, _) = self.end_at(doc, at(a), &[raw]);
        let (end, _) = self.end_at(doc, at(b), &[raw]);
        let mut c = Connector::new(start, end, Route::Straight, style);
        c.heads = heads.map(|h| if h.is_some() { Head::Arrow } else { Head::None });
        Element::new_connector(id, z, c)
    }

    /// A tinta que DESENHA a forma (o que fica quando nenhum objecto do quadro a desenha igual).
    pub(crate) fn smart_ink(
        doc: &mut BoardDoc,
        shape: &QuickShape,
        pen: Pen,
        style: &Style,
        px: f64,
    ) -> Element {
        let pts: Vec<[f64; 3]> = shape_points(shape, px)
            .into_iter()
            .map(|p| [p[0], p[1], 0.5])
            .collect();
        let (ink, bx) = Ink::from_world(&pts, style.clone(), pen, false);
        Element::new_ink(doc.mint_id(), doc.z_on_top(), ink, bx)
    }
}
