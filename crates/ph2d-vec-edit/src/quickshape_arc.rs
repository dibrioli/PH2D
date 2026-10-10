//! **O ARCO do QuickShape** (F7c, pedido do dono 10/10: *«QuickShape com arcos»*) — irmão de
//! `quickshape.rs` pelo tecto de 700 LOC.
//!
//! Entra onde a lei poligonal desiste de um traço ABERTO: os lados não são rectos. É arco se a
//! curvatura tem UM sinal (o S e o rabisco viram para os dois lados) e um círculo o desenha bem — o
//! círculo de Taubin, que não encolhe nos arcos curtos como o de Kåsa. O oráculo (Xournal++) deixa os
//! arcos crus; a lei é a do PRODUTO do dono (o QuickShape do Procreate faz arcos).
//! As faixas medidas de cada constante: `00_resultados.md` §3g.

use crate::quickshape::{Law, QuickShape, REGULAR_STEP_DEG};
use crate::quickshape_geom::{
    P2, add, angle, centroid, dist, mul, seg_dist, snap, sub, unit, wrap_pi,
};
use ph2d_vec_scene::{VecPath, VecVertex, VertexKind};
use std::f64::consts::{FRAC_PI_2, TAU};

/// O arco ajustado a um traço, antes do `regular`; `rms` é a distância média dele ao traço.
pub(crate) struct ArcFit {
    centre: P2,
    radius: f64,
    start: f64,
    sweep: f64,
    pub(crate) rms: f64,
}

impl ArcFit {
    /// A forma; `p0` é o início do traço (o pivô que o `regular` não move).
    pub(crate) fn shape(&self, p0: P2, regular: bool) -> QuickShape {
        if regular {
            return regularise(p0, self.radius, self.start, self.sweep);
        }
        QuickShape::Arc {
            centre: self.centre,
            radius: self.radius,
            start: self.start,
            sweep: self.sweep,
        }
    }
}

/// `r` é o traço reamostrado por comprimento (as pontas são as do traço); `diag`, a diagonal da
/// caixa dele.
pub(crate) fn fit(law: &Law, r: &[P2], diag: f64) -> Option<ArcFit> {
    if bends_back(r, law.arc_chords) > law.arc_bend_back {
        return None;
    }
    let (centre, radius) = taubin(r)?;
    let rms = (r
        .iter()
        .map(|&p| (dist(p, centre) - radius).powi(2))
        .sum::<f64>()
        / r.len() as f64)
        .sqrt();
    // Relativo ao MENOR de raio e tamanho: um traço quase recto tem raio enorme, e o resíduo de um S
    // sobre ele seria nada em fracção do raio.
    if rms > law.arc_rms_max * radius.min(diag) {
        return None;
    }
    let sweep: f64 = r
        .windows(2)
        .map(|w| wrap_pi(angle(sub(w[1], centre)) - angle(sub(w[0], centre))))
        .sum();
    // Mais de uma volta não é arco (a volta e meia fica crua, como no oráculo).
    if sweep.abs() >= TAU {
        return None;
    }
    Some(ArcFit {
        centre,
        radius,
        start: angle(sub(r[0], centre)),
        sweep,
        rms,
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
fn regularise(p0: P2, radius: f64, start: f64, sweep: f64) -> QuickShape {
    let step = REGULAR_STEP_DEG.to_radians();
    let s = snap(sweep, step);
    let s = if s == 0.0 { step.copysign(sweep) } else { s };
    // A corda de um arco que começa em `a0` e varre `s` aponta para `a0 + s/2 + sgn(s)·90°`.
    let quarter = FRAC_PI_2.copysign(sweep);
    let chord = snap(start + sweep / 2.0 + quarter, step);
    let a0 = chord - s / 2.0 - quarter;
    QuickShape::Arc {
        centre: sub(p0, mul(unit(a0), radius)),
        radius,
        start: a0,
        sweep: s,
    }
}

/// O caminho do arco: troços de ≤ 90° com a alça `(4/3)·tan(α/4)·r` (a lei de
/// [`ph2d_vec_scene::arc`], com início e sentido próprios); nós `Smooth` no meio, `Corner` nas pontas.
/// Um troço pode passar 5 % dos 90° (a cúbica erra então ≤ 0,04 % do raio): o meio arco de uma mão
/// tem 181° e não deve ganhar um nó por isso.
pub(crate) fn arc_path(centre: P2, radius: f64, start: f64, sweep: f64) -> VecPath {
    let n = (sweep.abs() / FRAC_PI_2 - 0.05).ceil().max(1.0) as usize;
    let seg = sweep / n as f64;
    let h = radius * (4.0 / 3.0) * (seg / 4.0).tan();
    let verts = (0..=n)
        .map(|i| {
            let a = start + seg * i as f64;
            let anchor = add(centre, mul(unit(a), radius));
            let t = mul(unit(a + FRAC_PI_2), h);
            let mut v = VecVertex::smooth(anchor, sub(anchor, t), add(anchor, t));
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
