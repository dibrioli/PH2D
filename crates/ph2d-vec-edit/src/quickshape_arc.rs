//! **O ARCO do QuickShape** (F7c, pedido do dono 10/10: *«QuickShape com arcos»*) — irmão de
//! `quickshape.rs` pelo tecto de 700 LOC.
//!
//! Entra onde a lei poligonal desiste de um traço ABERTO: os lados não são rectos. É arco se a
//! curvatura tem UM sinal (o S e o rabisco viram para os dois lados) e um círculo o desenha bem — o
//! círculo de Taubin, que não encolhe nos arcos curtos como o de Kåsa — ou, se o arco é ASSIMÉTRICO
//! (abre largo e fecha apertado: o smoke do dono, 10/10), uma ELIPSE ([`crate::quickshape_conic`]). O oráculo (Xournal++) deixa os
//! arcos crus; a lei é a do PRODUTO do dono (o QuickShape do Procreate faz arcos).
//! As faixas medidas de cada constante: `00_resultados.md` §3g.

use crate::quickshape::{Law, QuickShape, REGULAR_STEP_DEG};
use crate::quickshape_conic::{Conic, fit_ellipse};
use crate::quickshape_geom::{
    P2, add, angle, centroid, mul, rotate, seg_dist, snap, sub, unit, wrap_pi,
};
use ph2d_vec_scene::{VecPath, VecVertex, VertexKind};
use std::f64::consts::{FRAC_PI_2, TAU};

/// **O gancho ASSIMÉTRICO da captura do dono** (smoke de 10/10, px da tela, y para baixo): abre largo
/// e fecha apertado — o traço que só o biarco segue. Um só sítio para a cena 12 e os gates.
pub const OWNERS_HOOK_PX: [P2; 23] = [
    [390.0, 333.0],
    [401.0, 306.0],
    [415.0, 281.0],
    [433.0, 258.0],
    [451.0, 242.0],
    [476.0, 222.0],
    [493.0, 210.0],
    [513.0, 203.0],
    [543.0, 197.0],
    [579.0, 194.0],
    [600.0, 193.0],
    [631.0, 196.0],
    [651.0, 200.0],
    [681.0, 210.0],
    [695.0, 217.0],
    [714.0, 232.0],
    [727.0, 245.0],
    [735.0, 263.0],
    [735.0, 271.0],
    [726.0, 283.0],
    [706.0, 294.0],
    [681.0, 302.0],
    [668.0, 307.0],
];

/// **Um arco** de círculo (`rx == ry`, `rot == 0`) ou de ELIPSE: começa no ângulo PARAMÉTRICO `start`
/// (`centre + R(rot)·(rx cos t, ry sin t)`) e varre `sweep` (radianos; o SINAL é o sentido do
/// desenho, `|sweep| < 2π`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArcShape {
    pub centre: P2,
    pub rx: f64,
    pub ry: f64,
    pub rot: f64,
    pub start: f64,
    pub sweep: f64,
}

impl ArcShape {
    #[must_use]
    pub fn at(&self, t: f64) -> P2 {
        add(
            self.centre,
            rotate([self.rx * t.cos(), self.ry * t.sin()], self.rot),
        )
    }

    /// O ponto onde o desenho começou — o pivô de uma forma aberta.
    #[must_use]
    pub fn pivot(&self) -> P2 {
        self.at(self.start)
    }

    /// A semelhança à volta de `pivot`: escala `scale`, roda `rot` radianos.
    #[must_use]
    pub fn transformed(&self, pivot: P2, scale: f64, rot: f64) -> Self {
        Self {
            centre: add(pivot, mul(rotate(sub(self.centre, pivot), rot), scale)),
            rx: self.rx * scale,
            ry: self.ry * scale,
            rot: self.rot + rot,
            ..*self
        }
    }

    /// O caminho: o arco do círculo unitário (troços de ≤ 90° com a alça `(4/3)·tan(α/4)`, a lei de
    /// [`ph2d_vec_scene::arc`] com início e sentido próprios) levado pela afinidade da elipse — uma
    /// afinidade leva uma cúbica a outra EXACTAMENTE. Nós `Smooth` no meio, `Corner` nas pontas. Um
    /// troço pode passar 5 % dos 90° (a cúbica erra então ≤ 0,04 %): o meio arco de uma mão tem 181°
    /// e não deve ganhar um nó por isso.
    #[must_use]
    pub fn to_path(&self) -> VecPath {
        let n = (self.sweep.abs() / FRAC_PI_2 - 0.05).ceil().max(1.0) as usize;
        let seg = self.sweep / n as f64;
        let h = (4.0 / 3.0) * (seg / 4.0).tan();
        let map = |q: P2| {
            add(
                self.centre,
                rotate([self.rx * q[0], self.ry * q[1]], self.rot),
            )
        };
        let verts = (0..=n)
            .map(|i| {
                let a = self.start + seg * i as f64;
                let (u, t) = (unit(a), mul(unit(a + FRAC_PI_2), h));
                let anchor = map(u);
                let mut v = VecVertex::smooth(anchor, map(sub(u, t)), map(add(u, t)));
                if i == 0 || i == n {
                    v.kind = VertexKind::Corner;
                }
                if i == 0 {
                    v.in_handle = anchor;
                }
                if i == n {
                    v.out_handle = anchor;
                }
                v
            })
            .collect();
        VecPath {
            verts,
            closed: false,
            ..VecPath::default()
        }
    }
}

/// O caminho de um BIARCO: os dois arcos, com o nó da junção `Smooth` (são tangentes).
pub(crate) fn biarc_path(b: &[ArcShape; 2]) -> VecPath {
    let (mut p, q) = (b[0].to_path(), b[1].to_path());
    if let Some(join) = p.verts.last_mut() {
        join.kind = VertexKind::Smooth;
        join.out_handle = q.verts[0].out_handle;
    }
    p.verts.extend_from_slice(&q.verts[1..]);
    p
}

/// O arco ajustado a um traço, antes do `regular`; `rms` é a distância média dele ao traço.
pub(crate) struct ArcFit {
    arc: QuickShape,
    /// O círculo de Taubin, mesmo quando ganhou a elipse: o `regular` faz o arco de CÍRCULO.
    circle: ArcShape,
    pub(crate) rms: f64,
}

impl ArcFit {
    /// A forma; `p0` é o início do traço (o pivô que o `regular` não move).
    pub(crate) fn shape(&self, p0: P2, regular: bool) -> QuickShape {
        if regular {
            QuickShape::Arc(regularise(
                p0,
                self.circle.rx,
                self.circle.start,
                self.circle.sweep,
            ))
        } else {
            self.arc.clone()
        }
    }
}

/// `r` é o traço reamostrado por comprimento (as pontas são as do traço); `diag`, a diagonal da
/// caixa dele. O CÍRCULO primeiro; o arco ASSIMÉTRICO (o smoke do dono, 10/10) — o mais perto de uma
/// ELIPSE e de um BIARCO ([`crate::quickshape_biarc`]) — quando o círculo não desenha o traço, ou
/// quando o desenha mal (`arc_circle_keep`) e o outro MUITO melhor (`arc_asymmetric_gain`).
pub(crate) fn fit(law: &Law, r: &[P2], diag: f64) -> Option<ArcFit> {
    if bends_back(r, law.arc_chords) > law.arc_bend_back {
        return None;
    }
    let (centre, radius) = taubin(r)?;
    let circle = Conic {
        c: centre,
        rx: radius,
        ry: radius,
        rot: 0.0,
    };
    let rms =
        |e: &Conic| (r.iter().map(|&p| e.distance(p).powi(2)).sum::<f64>() / r.len() as f64).sqrt();
    // Relativo ao MENOR de raio e tamanho: um traço quase recto tem raio enorme, e o resíduo de um S
    // sobre ele seria nada em fracção do raio.
    let fits = |e: &Conic, rms: f64| rms <= law.arc_rms_max * e.rx.max(e.ry).min(diag);
    let rc = rms(&circle);
    let round = sweep_of(&circle, r)?;
    let ellipse = fit_ellipse(r)
        .map(|e| (rms(&e), e))
        .filter(|(re, e)| fits(e, *re))
        .and_then(|(re, e)| Some((re, QuickShape::Arc(sweep_of(&e, r)?))));
    // O biarco parte do círculo: curvatura `±1/R`, a tangente no início.
    let k = round.sweep.signum() / radius;
    let biarc = crate::quickshape_biarc::fit(r, k, round.start + FRAC_PI_2.copysign(k))
        .filter(|(b, rb)| {
            let big = b[0].rx.max(b[1].rx);
            *rb <= law.arc_rms_max * big.min(diag) && (b[0].sweep + b[1].sweep).abs() < TAU
        })
        .map(|(b, rb)| (rb, QuickShape::Biarc(b)));
    let other = match (ellipse, biarc) {
        (Some(e), Some(b)) => Some(if e.0 <= b.0 { e } else { b }),
        (e, b) => e.or(b),
    };
    let (rms, arc) = match other {
        Some((ro, o))
            if !fits(&circle, rc)
                || (rc > law.arc_circle_keep * radius.min(diag)
                    && ro < law.arc_asymmetric_gain * rc) =>
        {
            (ro, o)
        }
        _ if fits(&circle, rc) => (rc, QuickShape::Arc(round)),
        _ => return None,
    };
    Some(ArcFit {
        arc,
        circle: round,
        rms,
    })
}

/// O arco de `e` que o traço percorre; mais de uma volta não é arco (a volta e meia fica crua,
/// como no oráculo).
fn sweep_of(e: &Conic, r: &[P2]) -> Option<ArcShape> {
    let sweep: f64 = r
        .windows(2)
        .map(|w| wrap_pi(e.param(w[1]) - e.param(w[0])))
        .sum();
    (sweep.abs() < TAU).then(|| ArcShape {
        centre: e.c,
        rx: e.rx,
        ry: e.ry,
        rot: e.rot,
        start: e.param(r[0]),
        sweep,
    })
}

/// A distância rms do traço à polilinha dos `corners`.
pub(crate) fn polyline_rms(r: &[P2], corners: &[P2]) -> f64 {
    (r.iter()
        .map(|&p| {
            corners
                .windows(2)
                .map(|w| seg_dist(p, w[0], w[1]))
                .fold(f64::MAX, f64::min)
                .powi(2)
        })
        .sum::<f64>()
        / r.len() as f64)
        .sqrt()
}

/// A fracção da viragem que vai CONTRA o sentido do traço, medida entre `chords` cordas iguais (a
/// escala da corda apaga o tremor da mão): ~0 num arco, ~½ num S.
fn bends_back(r: &[P2], chords: usize) -> f64 {
    let pts: Vec<P2> = (0..=chords)
        .map(|i| r[i * (r.len() - 1) / chords])
        .collect();
    let turns: Vec<f64> = pts
        .windows(3)
        .map(|w| wrap_pi(angle(sub(w[2], w[1])) - angle(sub(w[1], w[0]))))
        .collect();
    let (net, all) = turns
        .iter()
        .fold((0.0, 0.0), |(n, a), t| (n + t, a + t.abs()));
    if all <= f64::MIN_POSITIVE {
        return 0.0;
    }
    turns
        .iter()
        .filter(|t| t.signum() != net.signum())
        .map(|t| t.abs())
        .sum::<f64>()
        / all
}

/// `regular` (`Shift` / o 2.º dedo): o ângulo varrido em múltiplos de 15° e a corda (início → fim)
/// a 15° — o degrau da linha. O início fica onde a mão o pôs e o raio é o ajustado.
fn regularise(p0: P2, radius: f64, start: f64, sweep: f64) -> ArcShape {
    let step = REGULAR_STEP_DEG.to_radians();
    let s = snap(sweep, step);
    let s = if s == 0.0 { step.copysign(sweep) } else { s };
    // A corda de um arco que começa em `a0` e varre `s` aponta para `a0 + s/2 + sgn(s)·90°`.
    let quarter = FRAC_PI_2.copysign(sweep);
    let chord = snap(start + sweep / 2.0 + quarter, step);
    let a0 = chord - s / 2.0 - quarter;
    ArcShape {
        centre: sub(p0, mul(unit(a0), radius)),
        rx: radius,
        ry: radius,
        rot: 0.0,
        start: a0,
        sweep: s,
    }
}

/// **O círculo de Taubin** (G. Taubin, IEEE PAMI 13, 1991), pelo método de Newton de N. Chernov
/// (*Circular and Linear Regression*, 2010, `CircleFitByTaubin`) — porte directo. O ajuste
/// algébrico de Kåsa encolhe o raio nos arcos curtos; o de Taubin é quase sem viés.
pub(crate) fn taubin(p: &[P2]) -> Option<(P2, f64)> {
    let n = p.len() as f64;
    let m = centroid(p);
    let (mut mxx, mut myy, mut mxy, mut mxz, mut myz, mut mzz) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for &q in p {
        let [x, y] = sub(q, m);
        let z = x * x + y * y;
        mxx += x * x;
        myy += y * y;
        mxy += x * y;
        mxz += x * z;
        myz += y * z;
        mzz += z * z;
    }
    let (mxx, myy, mxy, mxz, myz, mzz) = (mxx / n, myy / n, mxy / n, mxz / n, myz / n, mzz / n);
    let mz = mxx + myy;
    let cov_xy = mxx * myy - mxy * mxy;
    let var_z = mzz - mz * mz;
    let a3 = 4.0 * mz;
    let a2 = -3.0 * mz * mz - mzz;
    let a1 = var_z * mz + 4.0 * cov_xy * mz - mxz * mxz - myz * myz;
    let a0 = mxz * (mxz * myy - myz * mxy) + myz * (myz * mxx - mxz * mxy) - var_z * cov_xy;
    let (a22, a33) = (a2 + a2, a3 + a3 + a3);
    let (mut x, mut y) = (0.0_f64, a0);
    for _ in 0..99 {
        let dy = a1 + x * (a22 + a33 * x);
        let xn = x - y / dy;
        if xn == x || !xn.is_finite() {
            break;
        }
        let yn = a0 + xn * (a1 + xn * (a2 + xn * a3));
        if yn.abs() >= y.abs() {
            break;
        }
        (x, y) = (xn, yn);
    }
    let det = x * x - x * mz + cov_xy;
    let c = [
        (mxz * (myy - x) - myz * mxy) / det / 2.0,
        (myz * (mxx - x) - mxz * mxy) / det / 2.0,
    ];
    let radius = (c[0] * c[0] + c[1] * c[1] + mz).sqrt();
    (radius.is_finite() && c[0].is_finite() && c[1].is_finite()).then(|| (add(c, m), radius))
}
