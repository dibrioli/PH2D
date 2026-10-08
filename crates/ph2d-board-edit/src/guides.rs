//! ⭐ **O encaixe de um gesto** (W5b): quem se move, contra que alvos, em que eixos. As leis e as
//! fontes (Miro, Figma, tldraw, Excalidraw…) estão em `docs/MiroClone/03_guias_e_alinhamento.md`.
//!
//! - Os alvos são os vivos à vista que não se movem: formas, notas, traços e **frames** (setas não).
//! - Cada referencial é uma família de eixos: o do MUNDO (todas as caixas envolventes) e, se quem se
//!   move está rodado, o DELE — onde só entram os alvos ao mesmo ângulo (mod 90°), pelas caixas
//!   rodadas. A lei de [`snap_box`] é a mesma nos dois; o próprio ganha quando encaixa.
//! - Rodar encaixa no ângulo de um alvo à vista (mod 90°), com a tolerância medida em px no ARCO
//!   do ponteiro.

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

pub(crate) fn same_axes(a: f64, b: f64) -> bool {
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
    /// O encaixe dos pontos `pts` (já deslocados) neste referencial, com o deslocamento e as guias
    /// devolvidos no MUNDO.
    fn snap(&self, pts: impl IntoIterator<Item = [f64; 2]>, tol: f64) -> Snap {
        let s = snap_box(self.axes.stops(pts), &self.boxes, tol);
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

/// ⭐ Os alvos de um gesto, recolhidos ao carregar.
pub(crate) struct Snapper {
    /// Os cantos de quem se move, no início do gesto.
    corners: Vec<[f64; 2]>,
    world: Family,
    /// Os eixos de quem se move, quando está rodado (tudo o que se move ao mesmo ângulo).
    own: Option<Family>,
    turns: Vec<Turn>,
}

fn intersects(a: &[f64; 4], b: &[f64; 4]) -> bool {
    a[0] <= b[2] && b[0] <= a[2] && a[1] <= b[3] && b[1] <= a[3]
}

/// O ângulo comum (não recto) de `els`, se o têm.
fn common_angle(els: &[&Element]) -> Option<f64> {
    let first = els.first()?.angle;
    (reduced(first) != 0.0 && els.iter().all(|e| same_axes(e.angle, first))).then_some(first)
}

impl Snapper {
    /// Os alvos à vista (`view`), menos `skip`, que vivem numa das `homes` (o frame de quem se move,
    /// ou `None` = solto no quadro) — e esses frames. `movers` são as caixas que se encaixam (as
    /// raízes do que se move), `angle` os eixos próprios a juntar (os de um rodado a redimensionar).
    pub(crate) fn new(
        doc: &BoardDoc,
        skip: &BTreeSet<ElementId>,
        view: [f64; 4],
        movers: &[&Element],
        angle: Option<f64>,
        homes: &BTreeSet<Option<ElementId>>,
    ) -> Self {
        let angle = angle
            .filter(|a| reduced(*a) != 0.0)
            .or_else(|| common_angle(movers));
        let mut world = Family {
            axes: Axes::WORLD,
            boxes: Vec::new(),
        };
        let mut own = angle.map(|a| Family {
            axes: Axes::of(a),
            boxes: Vec::new(),
        });
        let mut turns = Vec::new();
        for el in doc.live() {
            let home = homes.contains(&el.frame) || homes.contains(&Some(el.id));
            if el.connector().is_some() || skip.contains(&el.id) || !home {
                continue;
            }
            let b = el.aabb();
            if !intersects(&b, &view) {
                continue;
            }
            world.boxes.push(if el.angle == 0.0 {
                Stops::rect(b)
            } else {
                Axes::WORLD.stops(el.corners())
            });
            if let (Some(f), Some(a)) = (own.as_mut(), angle)
                && same_axes(el.angle, a)
            {
                f.boxes.push(f.axes.stops(el.corners()));
            }
            turns.push(Turn {
                angle: reduced(el.angle),
                center: el.center(),
                reach: el.w.hypot(el.h) / 2.0,
            });
        }
        Self {
            corners: movers.iter().flat_map(|e| e.corners()).collect(),
            world,
            own,
            turns,
        }
    }

    /// O encaixe de quem se move, deslocado de `d`: o deslocamento que falta e as guias — nos eixos
    /// PRÓPRIOS de um rodado quando lá há encaixe (é neles que o artista o vê); senão nos do mundo.
    pub(crate) fn snap_moving(&self, d: [f64; 2], tol: f64) -> ([f64; 2], Vec<Guide>) {
        if self.corners.is_empty() {
            return ([0.0; 2], Vec::new());
        }
        let pts = || self.corners.iter().map(|c| [c[0] + d[0], c[1] + d[1]]);
        self.best(|f| f.snap(pts(), tol))
    }

    /// O encaixe de um PONTO (a pega a redimensionar, o canto de uma forma a nascer).
    pub(crate) fn snap_point(&self, p: [f64; 2], tol: f64) -> ([f64; 2], Vec<Guide>) {
        self.best(|f| f.snap([p], tol))
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
    /// passos de 15°; sem ele, o encaixe do ângulo (`tol` = `None` com `Ctrl`).
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
        let radius = (p.world[0] - c[0]).hypot(p.world[1] - c[1]);
        let reach = frame.w.hypot(frame.h) / 2.0;
        tol.and_then(|tol| self.snap_angle(frame.angle + delta, c, reach, radius, tol))
            .map_or((delta, Vec::new()), |(a, g)| (a - frame.angle, g))
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
    /// As casas da selecção (o frame de cada seleccionado, ou `None`): os alvos de um gesto vivem
    /// nelas (o tldraw G6: um filho prende ao irmão e ao frame, nunca ao que está fora).
    pub(crate) fn homes(&self, doc: &BoardDoc) -> BTreeSet<Option<ElementId>> {
        self.selected(doc).iter().map(|el| el.frame).collect()
    }
}
