//! ⭐ **O encaixe de um gesto** (W5b): quem se move, contra que alvos, em que eixos. As leis e as
//! fontes (Miro, Figma, tldraw, Excalidraw…) estão em `docs/MiroClone/03_guias_e_alinhamento.md`.
//!
//! - Os alvos são os vivos à vista que não se movem: formas, notas, traços e **frames** (setas não).
//! - Cada referencial é uma família de eixos: o do MUNDO (todas as caixas envolventes) e, se quem se
//!   move está rodado, o DELE — onde só entram os alvos ao mesmo ângulo (mod 90°), pelas caixas
//!   rodadas. A lei de [`snap_box`] é a mesma nos dois; o próprio ganha quando encaixa.
//! - Rodar encaixa no ângulo de um alvo da casa (mod 90°), com a tolerância medida em px no ARCO do
//!   ponteiro ou do canto mais longe, o maior.

use std::collections::BTreeSet;
use std::f64::consts::{FRAC_PI_2, PI};

use ph2d_board_model::{BoardDoc, Element, ElementId};

use crate::snap::{Guide, Snap, Stops, snap_box};
use crate::{Frame, Pointer};

/// O passo do ângulo com `Shift` a rodar.
const ROTATE_STEP: f64 = PI / 12.0;

/// Dois ângulos dão os mesmos eixos se diferem menos que isto (mod 90°): rodar um frame soma o MESMO
/// delta a ele e aos filhos, e a conta pode mexer no último dígito.
const SAME_AXES: f64 = 1e-7;

/// O ângulo reduzido a `[0, 90°)` — rodar uma caixa 90° só lhe troca os lados.
fn reduced(angle: f64) -> f64 {
    let a = angle.rem_euclid(FRAC_PI_2);
    if FRAC_PI_2 - a < SAME_AXES { 0.0 } else { a }
}

fn same_axes(a: f64, b: f64) -> bool {
    let d = (reduced(a) - reduced(b)).abs();
    d < SAME_AXES || FRAC_PI_2 - d < SAME_AXES
}

/// Os eixos rodados de um ângulo.
#[derive(Clone, Copy, Debug)]
struct Axes {
    s: f64,
    k: f64,
}

impl Axes {
    const WORLD: Axes = Axes { s: 0.0, k: 1.0 };

    fn of(angle: f64) -> Self {
        let (s, k) = angle.sin_cos();
        Self { s, k }
    }

    fn local(self, p: [f64; 2]) -> [f64; 2] {
        [p[0] * self.k + p[1] * self.s, p[1] * self.k - p[0] * self.s]
    }

    fn world(self, q: [f64; 2]) -> [f64; 2] {
        [q[0] * self.k - q[1] * self.s, q[0] * self.s + q[1] * self.k]
    }

    /// A caixa `[u0, v0, u1, v1]` dos pontos nestes eixos — e, com os QUATRO cantos de um só
    /// elemento, os dois de dentro de cada eixo (um rodado prende pelos cantos e o centro).
    fn stops(self, pts: impl IntoIterator<Item = [f64; 2]>) -> Stops {
        let q: Vec<[f64; 2]> = pts.into_iter().map(|p| self.local(p)).collect();
        let b = q.iter().fold(
            [
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ],
            |b, q| {
                [
                    b[0].min(q[0]),
                    b[1].min(q[1]),
                    b[2].max(q[0]),
                    b[3].max(q[1]),
                ]
            },
        );
        let mut s = Stops::rect(b);
        if q.len() == 4 {
            for (axis, inner) in s.inner.iter_mut().enumerate() {
                let mut v: Vec<f64> = q.iter().map(|p| p[axis]).collect();
                v.sort_by(f64::total_cmp);
                *inner = [v[1], v[2]];
            }
        }
        s
    }
}

/// Os alvos num referencial: as caixas deles nesses eixos.
struct Family {
    axes: Axes,
    boxes: Vec<Stops>,
}

impl Family {
    /// O encaixe dos pontos `pts` (já deslocados) neste referencial, nos eixos `free` dele, com o
    /// deslocamento e as guias devolvidos no MUNDO.
    fn snap(&self, pts: impl IntoIterator<Item = [f64; 2]>, tol: f64, free: [bool; 2]) -> Snap {
        let s = snap_box(self.axes.stops(pts), &self.boxes, tol, free);
        Snap {
            delta: self.axes.world(s.delta),
            best: s.best,
            guides: s
                .guides
                .into_iter()
                .map(|g| Guide {
                    a: self.axes.world(g.a),
                    b: self.axes.world(g.b),
                })
                .collect(),
        }
    }
}

/// Um alvo para o encaixe do ÂNGULO: o ângulo reduzido, o centro e a meia diagonal.
struct Turn {
    angle: f64,
    center: [f64; 2],
    reach: f64,
}

/// ⭐ Os alvos de um gesto: recolhidos ao carregar, e de novo quando a CASA debaixo do ponteiro
/// muda (é ela que o largar decide) ou quando o `Alt` deixa os originais no sítio.
pub(crate) struct Snapper {
    skip: BTreeSet<ElementId>,
    view: [f64; 4],
    homes: BTreeSet<Option<ElementId>>,
    /// Os eixos próprios (um ângulo não recto), se quem se move os tem.
    angle: Option<f64>,
    /// O ângulo comum de quem se move (recto incluído): a base do encaixe do ângulo.
    base: Option<f64>,
    /// Os cantos de quem se move, no início do gesto.
    corners: Vec<[f64; 2]>,
    world: Family,
    own: Option<Family>,
    turns: Vec<Turn>,
}

fn intersects(a: &[f64; 4], b: &[f64; 4]) -> bool {
    a[0] <= b[2] && b[0] <= a[2] && a[1] <= b[3] && b[1] <= a[3]
}

/// O ângulo de `els` se TODOS têm os mesmos eixos (mod 90°).
fn common_axes(els: &[&Element]) -> Option<f64> {
    let first = els.first()?.angle;
    els.iter()
        .all(|e| same_axes(e.angle, first))
        .then_some(first)
}

impl Snapper {
    /// Os alvos à vista (`view`), menos `skip`, que vivem numa das `homes` (o frame onde se vai
    /// largar, ou `None` = solto no quadro) — e esses frames. `movers` são as caixas que se
    /// encaixam (as raízes do que se move), `angle` os eixos próprios a juntar (os de um rodado a
    /// redimensionar).
    pub(crate) fn new(
        doc: &BoardDoc,
        skip: BTreeSet<ElementId>,
        view: [f64; 4],
        movers: &[&Element],
        angle: Option<f64>,
        homes: BTreeSet<Option<ElementId>>,
    ) -> Self {
        let base = common_axes(movers);
        let mut s = Self {
            skip,
            view,
            homes,
            angle: angle.or(base).filter(|a| reduced(*a) != 0.0),
            base,
            corners: movers.iter().flat_map(|e| e.corners()).collect(),
            world: Family {
                axes: Axes::WORLD,
                boxes: Vec::new(),
            },
            own: None,
            turns: Vec::new(),
        };
        s.collect(doc);
        s
    }

    fn collect(&mut self, doc: &BoardDoc) {
        self.world.boxes.clear();
        self.turns.clear();
        self.own = self.angle.map(|a| Family {
            axes: Axes::of(a),
            boxes: Vec::new(),
        });
        for el in doc.live() {
            let home = self.homes.contains(&el.frame) || self.homes.contains(&Some(el.id));
            if el.connector().is_some() || self.skip.contains(&el.id) || !home {
                continue;
            }
            let b = el.aabb();
            if !intersects(&b, &self.view) {
                continue;
            }
            // Ao MESMO ângulo de quem se move, um alvo conta só nos eixos próprios: canto a canto no
            // ecrã, dois rodados iguais dão alinhamentos que não se vêem (o gate da cena 7).
            match (self.own.as_mut(), self.angle) {
                (Some(f), Some(a)) if same_axes(el.angle, a) => {
                    f.boxes.push(f.axes.stops(el.corners()));
                }
                _ if el.angle == 0.0 => self.world.boxes.push(Stops::rect(b)),
                _ => self.world.boxes.push(Axes::WORLD.stops(el.corners())),
            }
            self.turns.push(Turn {
                angle: reduced(el.angle),
                center: el.center(),
                reach: el.w.hypot(el.h) / 2.0,
            });
        }
    }

    /// O ponteiro em `p`: se o frame onde se largaria mudou, os alvos passam a ser os dessa casa.
    pub(crate) fn follow(&mut self, doc: &BoardDoc, p: [f64; 2]) {
        let homes = BTreeSet::from([doc.deepest_frame_at(p, &self.skip)]);
        if homes != self.homes {
            self.homes = homes;
            self.collect(doc);
        }
    }

    /// O `Alt` duplicou: quem se move passa a ser `skip` (as cópias) e os originais viram alvos.
    pub(crate) fn reskip(&mut self, doc: &BoardDoc, skip: BTreeSet<ElementId>) {
        self.skip = skip;
        self.collect(doc);
    }

    /// O encaixe de quem se move, deslocado de `d`, nos eixos do mundo `free` (o `Shift` trava um):
    /// o deslocamento que falta e as guias — nos eixos PRÓPRIOS de um rodado quando lá há encaixe
    /// (é neles que o artista o vê); senão nos do mundo. Travado, só o mundo, no eixo livre.
    pub(crate) fn snap_moving(
        &self,
        d: [f64; 2],
        tol: f64,
        free: [bool; 2],
    ) -> ([f64; 2], Vec<Guide>) {
        if self.corners.is_empty() {
            return ([0.0; 2], Vec::new());
        }
        let pts = || self.corners.iter().map(|c| [c[0] + d[0], c[1] + d[1]]);
        if free != [true; 2] {
            let s = self.world.snap(pts(), tol, free);
            return (s.delta, s.guides);
        }
        self.best(|f| f.snap(pts(), tol, free))
    }

    /// O encaixe de um PONTO (o canto de uma forma a nascer) nos eixos do mundo `free`.
    pub(crate) fn snap_point(
        &self,
        p: [f64; 2],
        tol: f64,
        free: [bool; 2],
    ) -> ([f64; 2], Vec<Guide>) {
        let s = self.world.snap([p], tol, free);
        (s.delta, s.guides)
    }

    /// O encaixe da pega `sign` (`Dir::sign`) de uma moldura rodada `angle`: só os eixos que a pega
    /// mexe, nos eixos da moldura (os próprios, ou o mundo se ela é recta).
    pub(crate) fn snap_handle(
        &self,
        p: [f64; 2],
        tol: f64,
        (sx, sy): (f64, f64),
        angle: f64,
    ) -> ([f64; 2], Vec<Guide>) {
        let free = [sx != 0.0, sy != 0.0];
        let s = match &self.own {
            Some(f) => f.snap([p], tol, free),
            None => {
                // Rodada 90° (ou 270°), o lado da pega é o OUTRO eixo do mundo.
                let quarter = (angle.rem_euclid(PI) - FRAC_PI_2).abs() < SAME_AXES;
                let free = if quarter { [free[1], free[0]] } else { free };
                self.world.snap([p], tol, free)
            }
        };
        (s.delta, s.guides)
    }

    fn best(&self, snap: impl Fn(&Family) -> Snap) -> ([f64; 2], Vec<Guide>) {
        let world = snap(&self.world);
        let own = self.own.as_ref().map(&snap);
        let s = match own {
            Some(o) if o.best.is_finite() => o,
            _ => world,
        };
        (s.delta, s.guides)
    }

    /// O ângulo a SOMAR num gesto de rodar `frame` (de `start` até `p`) e as guias: com `Shift`,
    /// passos de 15° da moldura; sem ele, o encaixe do ângulo de quem roda (`tol` = `None` com
    /// `Ctrl`; nenhum se rodam ângulos diferentes).
    pub(crate) fn turn(
        &self,
        frame: &Frame,
        start: [f64; 2],
        p: Pointer,
        tol: Option<f64>,
    ) -> (f64, Vec<Guide>) {
        let c = frame.center;
        let a0 = (start[1] - c[1]).atan2(start[0] - c[0]);
        let a1 = (p.world[1] - c[1]).atan2(p.world[0] - c[0]);
        let delta = a1 - a0;
        if p.mods.shift {
            let target = ((frame.angle + delta) / ROTATE_STEP).round() * ROTATE_STEP;
            return (target - frame.angle, Vec::new());
        }
        // O arco medido é o do ponteiro ou o do canto mais longe, o MAIOR: perto do centro o canto
        // continua a andar 8 px antes de encaixar.
        let reach = frame.w.hypot(frame.h) / 2.0;
        let radius = (p.world[0] - c[0]).hypot(p.world[1] - c[1]).max(reach);
        let (Some(tol), Some(base)) = (tol, self.base) else {
            return (delta, Vec::new());
        };
        self.snap_angle(base + delta, c, reach, radius, tol)
            .map_or((delta, Vec::new()), |(a, g)| (a - base, g))
    }

    /// ⭐ O encaixe do ÂNGULO: `angle` (de quem roda, com o centro `center` e a meia diagonal
    /// `reach`) cola no ângulo do alvo mais perto em ângulo, mod 90°, se a diferença no arco do
    /// ponteiro (raio `radius`) é no máximo `tol`. Devolve o ângulo e as duas guias — uma por cada
    /// centro, nessa direcção.
    fn snap_angle(
        &self,
        angle: f64,
        center: [f64; 2],
        reach: f64,
        radius: f64,
        tol: f64,
    ) -> Option<(f64, Vec<Guide>)> {
        let mine = reduced(angle);
        let off = |t: f64| {
            let d = (t - mine).rem_euclid(FRAC_PI_2);
            if d > FRAC_PI_2 / 2.0 {
                d - FRAC_PI_2
            } else {
                d
            }
        };
        let near = |t: &&Turn| {
            let c = t.center;
            (c[0] - center[0]).hypot(c[1] - center[1])
        };
        let best = self
            .turns
            .iter()
            .filter(|t| off(t.angle).abs() * radius <= tol)
            .min_by(|a, b| {
                (off(a.angle).abs().total_cmp(&off(b.angle).abs()))
                    .then_with(|| near(a).total_cmp(&near(b)))
            })?;
        let snapped = angle + off(best.angle);
        let line = |c: [f64; 2], r: f64| {
            let (s, k) = snapped.sin_cos();
            Guide {
                a: [c[0] - k * r, c[1] - s * r],
                b: [c[0] + k * r, c[1] + s * r],
            }
        };
        Some((
            snapped,
            vec![line(center, reach), line(best.center, best.reach)],
        ))
    }
}

impl crate::Editor {
    /// As casas da selecção (o frame de cada seleccionado, ou `None`).
    fn homes(&self, doc: &BoardDoc) -> BTreeSet<Option<ElementId>> {
        self.selected(doc).iter().map(|el| el.frame).collect()
    }

    /// As raízes que rodam ou se movem (sem setas): a caixa delas é a que se encaixa.
    fn roots<'a>(&self, doc: &'a BoardDoc) -> Vec<&'a Element> {
        let mut v = self.selected(doc);
        v.retain(|el| !el.locked() && el.connector().is_none());
        v
    }

    /// Os alvos de uma forma a NASCER em `start`: a casa dela é o frame debaixo do início.
    pub(crate) fn snap_create(&self, doc: &BoardDoc, view: [f64; 4], start: [f64; 2]) -> Snapper {
        let home = doc.deepest_frame_at(start, &BTreeSet::new());
        Snapper::new(
            doc,
            BTreeSet::new(),
            view,
            &[],
            None,
            BTreeSet::from([home]),
        )
    }

    /// Os alvos de RODAR `originals` (a selecção e o que vive dentro dela).
    pub(crate) fn snap_rotate(
        &self,
        doc: &BoardDoc,
        view: [f64; 4],
        originals: &[Element],
    ) -> Snapper {
        let going = originals.iter().map(|o| o.id).collect();
        Snapper::new(doc, going, view, &self.roots(doc), None, self.homes(doc))
    }

    /// Os alvos de REDIMENSIONAR a moldura `frame`, nos eixos dela.
    pub(crate) fn snap_resize(&self, doc: &BoardDoc, view: [f64; 4], frame: &Frame) -> Snapper {
        let skip = self.selection.clone();
        Snapper::new(doc, skip, view, &[], Some(frame.angle), self.homes(doc))
    }

    /// Os alvos de MOVER `originals` a partir de `p`: o que FICA (o que se move — os de dentro de
    /// um frame arrastado também — não é alvo), na casa debaixo do ponteiro.
    pub(crate) fn snap_move(
        &self,
        doc: &BoardDoc,
        view: [f64; 4],
        p: [f64; 2],
        originals: &[Element],
    ) -> Snapper {
        let going: BTreeSet<ElementId> = originals.iter().map(|o| o.id).collect();
        let home = doc.deepest_frame_at(p, &going);
        Snapper::new(
            doc,
            going,
            view,
            &self.roots(doc),
            None,
            BTreeSet::from([home]),
        )
    }
}
