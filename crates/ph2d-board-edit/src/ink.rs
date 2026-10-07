//! **A caneta** (W4): o traço livre e o marcador, as duas borrachas e o ponteiro laser — as leis do
//! Miro (`docs/MiroClone/ferramentas/miro_caneta_notas.txt`, help «Pen»):
//!
//! - três predefinições de cor e espessura POR caneta, que se editam e não se apagam;
//! - a caneta e a borracha ficam activas depois de usadas (as outras ferramentas também ficam aqui);
//! - a borracha apaga SÓ desenho da caneta, o traço inteiro que tocar; a de precisão corta só o
//!   pedaço por onde passa (o resto fica em traços separados);
//! - o laser (o `K` do Excalidraw — o Miro não o tem no help) deixa um rasto que se apaga sozinho e
//!   nunca entra no documento nem no desfazer.
//!
//! Cada gesto é UM passo de desfazer; o documento muda AO VIVO (o idioma do [`crate::Editor`]).

use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use ph2d_board_model::{BoardDoc, BoardOp, Element, ElementId, History, Ink, Pen, Rgba, Style};

use crate::{Editor, Pointer, Tool, live};

/// As espessuras (diâmetro, mundo) que o painel da caneta oferece — fina a muito larga (o marcador
/// vive nas largas). O Miro não as publica (por medir numa captura).
pub const PEN_WIDTHS: [f64; 5] = [2.0, 4.0, 8.0, 16.0, 32.0];
/// Quanto tempo um ponto do rasto do laser vive.
pub const LASER_LIFE: Duration = Duration::from_millis(1000);

/// Uma predefinição da caneta: a cor e a espessura (diâmetro, mundo).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PenPreset {
    pub color: Rgba,
    pub width: f64,
}

/// As predefinições das duas canetas (`[caneta, marcador]`), a escolhida de cada uma, e a última
/// caneta usada (a que o `P` volta a pegar).
#[derive(Clone, Debug, PartialEq)]
pub struct PenBox {
    pub presets: [[PenPreset; 3]; 2],
    pub active: [usize; 2],
    pub last: Pen,
}

impl Default for PenBox {
    fn default() -> Self {
        let ink = Rgba(ph2d_board_model::DEFAULT_INK);
        let p = |width| PenPreset { color: ink, width };
        Self {
            presets: [
                [p(PEN_WIDTHS[0]), p(PEN_WIDTHS[1]), p(PEN_WIDTHS[2])],
                [p(PEN_WIDTHS[3]), p(PEN_WIDTHS[3]), p(PEN_WIDTHS[4])],
            ],
            active: [0, 0],
            last: Pen::Pen,
        }
    }
}

fn slot(pen: Pen) -> usize {
    match pen {
        Pen::Pen => 0,
        Pen::Highlighter => 1,
    }
}

impl PenBox {
    /// A predefinição escolhida da caneta `pen`.
    #[must_use]
    pub fn current(&self, pen: Pen) -> PenPreset {
        self.presets[slot(pen)][self.active[slot(pen)]]
    }

    /// A predefinição `i` da caneta `pen`, para mudar.
    pub fn preset_mut(&mut self, pen: Pen, i: usize) -> &mut PenPreset {
        &mut self.presets[slot(pen)][i]
    }

    pub fn choose(&mut self, pen: Pen, i: usize) {
        self.active[slot(pen)] = i;
    }

    #[must_use]
    pub fn active(&self, pen: Pen) -> usize {
        self.active[slot(pen)]
    }
}

pub(crate) enum InkGesture {
    Draw {
        id: ElementId,
        z: ph2d_board_model::FracKey,
        pen: Pen,
        style: Style,
        points: Vec<[f64; 3]>,
    },
    Erase {
        precise: bool,
        last: [f64; 2],
        /// O estado de ANTES do gesto de cada traço que ele mexeu (os que já existiam).
        originals: BTreeMap<ElementId, Element>,
        /// Os pedaços que o gesto criou e ainda vivem.
        created: BTreeSet<ElementId>,
    },
    Laser,
}

/// A pressão gravada quando o dispositivo não a mede (o meio, a de nascença do perfect-freehand).
const NO_PRESSURE: f64 = 0.5;

impl Editor {
    /// Carregar com uma ferramenta da caneta. `false` = não é uma delas.
    pub(crate) fn ink_down(&mut self, doc: &mut BoardDoc, p: Pointer) -> bool {
        self.cursor = Some(p.world);
        self.ink = Some(match self.tool {
            Tool::Pen(pen) => {
                self.pen.last = pen;
                let preset = self.pen.current(pen);
                let mut style = Style::new(None, Some(preset.color), preset.color);
                style.stroke_width = preset.width;
                let id = doc.mint_id();
                let z = doc.z_on_top();
                let points = vec![[p.world[0], p.world[1], NO_PRESSURE]];
                put_stroke(doc, id, &z, pen, &style, &points);
                self.selection.clear();
                InkGesture::Draw {
                    id,
                    z,
                    pen,
                    style,
                    points,
                }
            }
            Tool::Eraser { precise } => {
                self.selection.clear();
                let mut g = InkGesture::Erase {
                    precise,
                    last: p.world,
                    originals: BTreeMap::new(),
                    created: BTreeSet::new(),
                };
                self.erase_step(doc, &mut g, p);
                g
            }
            Tool::Laser => {
                self.laser.push((p.world, Instant::now()));
                InkGesture::Laser
            }
            _ => return false,
        });
        true
    }

    /// O ponteiro andou num gesto da caneta. `false` = não há um.
    pub(crate) fn ink_move(&mut self, doc: &mut BoardDoc, p: Pointer) -> bool {
        self.cursor = Some(p.world);
        let Some(mut g) = self.ink.take() else {
            return false;
        };
        match &mut g {
            InkGesture::Draw {
                id,
                z,
                pen,
                style,
                points,
            } => {
                let last = points[points.len() - 1];
                if last[0] != p.world[0] || last[1] != p.world[1] {
                    points.push([p.world[0], p.world[1], NO_PRESSURE]);
                    put_stroke(doc, *id, z, *pen, style, points);
                }
            }
            InkGesture::Erase { .. } => self.erase_step(doc, &mut g, p),
            InkGesture::Laser => self.laser.push((p.world, Instant::now())),
        }
        self.ink = Some(g);
        true
    }

    /// Largar: o gesto vira UM passo de desfazer. `false` = não havia gesto da caneta.
    pub(crate) fn ink_up(&mut self, doc: &mut BoardDoc, history: &mut History, p: Pointer) -> bool {
        if !self.ink_move(doc, p) {
            return false;
        }
        match self.ink.take() {
            Some(InkGesture::Draw { id, .. }) => history.record(vec![BoardOp::Delete(id)]),
            Some(g @ InkGesture::Erase { .. }) => history.record(erase_inverse(doc, g)),
            _ => {}
        }
        true
    }

    /// `Esc` a meio de um gesto da caneta: o documento volta ao início dele, sem passo.
    pub(crate) fn ink_cancel(&mut self, doc: &mut BoardDoc) -> bool {
        match self.ink.take() {
            Some(InkGesture::Draw { id, .. }) => {
                let _ = BoardOp::Delete(id).apply(doc);
            }
            Some(g @ InkGesture::Erase { .. }) => {
                let undo = erase_inverse(doc, g);
                let _ = ph2d_board_model::apply_batch(doc, undo);
            }
            Some(InkGesture::Laser) => {}
            None => return false,
        }
        true
    }

    /// O rasto do laser agora: cada ponto com a VIDA que lhe resta (`1` = acabado de pôr, `0` =
    /// a sumir), do mais velho para o mais novo. Os mortos saem.
    pub fn laser_trail(&mut self, now: Instant) -> Vec<([f64; 2], f64)> {
        let life = LASER_LIFE.as_secs_f64();
        self.laser
            .retain(|(_, t)| now.saturating_duration_since(*t) < LASER_LIFE);
        self.laser
            .iter()
            .map(|(p, t)| {
                (
                    *p,
                    1.0 - now.saturating_duration_since(*t).as_secs_f64() / life,
                )
            })
            .collect()
    }

    /// O anel da borracha (centro e raio, mundo) com uma borracha na mão e o ponteiro no quadro.
    #[must_use]
    pub fn eraser_ring(&self, px: f64) -> Option<([f64; 2], f64)> {
        matches!(self.tool, Tool::Eraser { .. })
            .then_some(self.cursor?)
            .map(|c| (c, self.metrics.eraser * px))
    }

    fn erase_step(&mut self, doc: &mut BoardDoc, g: &mut InkGesture, p: Pointer) {
        let InkGesture::Erase {
            precise,
            last,
            originals,
            created,
        } = g
        else {
            return;
        };
        let r = self.metrics.eraser * p.px;
        let (a, b) = (*last, p.world);
        *last = p.world;
        let hit: Vec<Element> = doc
            .live()
            .filter(|el| el.ink().is_some())
            .filter(|el| {
                let [x0, y0, x1, y1] = el.aabb();
                a[0].min(b[0]) - r <= x1
                    && a[0].max(b[0]) + r >= x0
                    && a[1].min(b[1]) - r <= y1
                    && a[1].max(b[1]) + r >= y0
            })
            .filter(|el| touches(el, a, b, r))
            .cloned()
            .collect();
        for el in hit {
            if !created.remove(&el.id) {
                originals.entry(el.id).or_insert_with(|| el.clone());
            }
            let _ = BoardOp::Delete(el.id).apply(doc);
            if *precise {
                for piece in cut(&el, a, b, r) {
                    let id = doc.mint_id();
                    let (ink, bx) = piece;
                    let _ = BoardOp::Put(Element::new_ink(id, el.z.clone(), ink, bx)).apply(doc);
                    created.insert(id);
                }
            }
        }
    }
}

/// O lote que desfaz um gesto de borracha: tira os pedaços que ele criou e repõe os traços como
/// estavam antes.
fn erase_inverse(doc: &BoardDoc, g: InkGesture) -> Vec<BoardOp> {
    let InkGesture::Erase {
        originals, created, ..
    } = g
    else {
        return Vec::new();
    };
    created
        .into_iter()
        .filter(|id| doc.get(*id).is_some())
        .map(BoardOp::Delete)
        .chain(originals.into_values().map(BoardOp::Put))
        .collect()
}

fn put_stroke(
    doc: &mut BoardDoc,
    id: ElementId,
    z: &ph2d_board_model::FracKey,
    pen: Pen,
    style: &Style,
    points: &[[f64; 3]],
) {
    let (ink, bx) = Ink::from_world(points, style.clone(), pen, false);
    live(doc, Element::new_ink(id, z.clone(), ink, bx));
}

/// Os pontos do traço de `el` no MUNDO (rodados com ele).
#[must_use]
pub fn world_points(el: &Element) -> Vec<[f64; 3]> {
    let Some(ink) = el.ink() else {
        return Vec::new();
    };
    ink.placed([el.x, el.y, el.w, el.h])
        .into_iter()
        .map(|[x, y, pr]| {
            let [wx, wy] = el.rotate([x, y]);
            [wx, wy, pr]
        })
        .collect()
}

/// A distância de `p` ao segmento `a–b`.
#[must_use]
pub fn point_segment(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 {
        (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (p[0] - (a[0] + t * dx)).hypot(p[1] - (a[1] + t * dy))
}

/// A distância entre os segmentos `a–b` e `c–d` (zero se se cruzam).
fn segment_segment(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> f64 {
    let cross = |o: [f64; 2], p: [f64; 2], q: [f64; 2]| {
        (p[0] - o[0]) * (q[1] - o[1]) - (p[1] - o[1]) * (q[0] - o[0])
    };
    let (d1, d2, d3, d4) = (
        cross(c, d, a),
        cross(c, d, b),
        cross(a, b, c),
        cross(a, b, d),
    );
    if d1 * d2 < 0.0 && d3 * d4 < 0.0 {
        return 0.0;
    }
    point_segment(a, c, d)
        .min(point_segment(b, c, d))
        .min(point_segment(c, a, b))
        .min(point_segment(d, a, b))
}

/// Um clique em `p` (a menos de `tol`) apanha o traço de `el`?
pub(crate) fn ink_hit(el: &Element, p: [f64; 2], tol: f64) -> bool {
    touches(el, p, p, tol)
}

/// O traço de `el` passa a menos de `r` (mais meia espessura dele) do caminho `a–b` da borracha?
fn touches(el: &Element, a: [f64; 2], b: [f64; 2], r: f64) -> bool {
    let reach = r + el.style().stroke_width / 2.0;
    let pts = world_points(el);
    match pts.as_slice() {
        [] => false,
        [one] => point_segment([one[0], one[1]], a, b) <= reach,
        many => many
            .windows(2)
            .any(|w| segment_segment([w[0][0], w[0][1]], [w[1][0], w[1][1]], a, b) <= reach),
    }
}

/// A borracha de PRECISÃO: o traço de `el` sem os pontos a menos de `r` (mais meia espessura) de
/// `a–b`, partido em pedaços (cada um um traço novo, com o estilo dele). Os trechos longos
/// subdividem-se antes, a `r / 2`, para a borracha não passar entre dois pontos.
fn cut(el: &Element, a: [f64; 2], b: [f64; 2], r: f64) -> Vec<(Ink, [f64; 4])> {
    let Some(ink) = el.ink() else {
        return Vec::new();
    };
    let reach = r + el.style().stroke_width / 2.0;
    let step = (r / 2.0).max(f64::EPSILON);
    let src = world_points(el);
    let mut dense: Vec<[f64; 3]> = Vec::with_capacity(src.len());
    for (i, p) in src.iter().enumerate() {
        if let Some(q) = i.checked_sub(1).map(|j| src[j]) {
            let len = (p[0] - q[0]).hypot(p[1] - q[1]);
            let n = (len / step).ceil() as usize;
            for k in 1..n {
                let t = k as f64 / n as f64;
                dense.push([
                    q[0] + (p[0] - q[0]) * t,
                    q[1] + (p[1] - q[1]) * t,
                    q[2] + (p[2] - q[2]) * t,
                ]);
            }
        }
        dense.push(*p);
    }
    let mut pieces = Vec::new();
    let mut run: Vec<[f64; 3]> = Vec::new();
    for p in dense {
        if point_segment([p[0], p[1]], a, b) <= reach {
            if !run.is_empty() {
                pieces.push(std::mem::take(&mut run));
            }
        } else {
            run.push(p);
        }
    }
    if !run.is_empty() {
        pieces.push(run);
    }
    pieces
        .into_iter()
        .map(|run| Ink::from_world(&run, ink.style.clone(), ink.pen, ink.pressure))
        .collect()
}

impl Editor {
    /// A espessura dos TRAÇOS seleccionados (as formas e setas da selecção ficam como estão, e o
    /// estilo das próximas formas também — a espessura da caneta é outra escala). UM passo.
    pub fn set_ink_width(&mut self, doc: &mut BoardDoc, history: &mut History, width: f64) {
        let ops = self
            .selected(doc)
            .into_iter()
            .filter(|el| el.ink().is_some())
            .map(|el| {
                let mut el = el.clone();
                el.style_mut().stroke_width = width;
                BoardOp::Put(el)
            })
            .collect();
        history.apply(doc, ops);
    }
}
