//! **A SETA do QuickShape** (F7d, o dono 10/10: *«desenhar uma linha ou um arco com um bico no fim
//! e SEGURAR ⇒ uma seta perfeita»*) — irmão de [`crate::quickshape`].
//!
//! Num traço SÓ (o lápis é um gesto: a mão não levanta): o cabo e, na ponta, o bico — a farpa, a
//! volta à ponta e a outra farpa (o V aberto), o mesmo e de volta à ponta (o V de ida-e-volta), as
//! duas farpas e a ponta (o V fechado); e o mesmo no INÍCIO (a mão começa numa farpa). A ponta sai
//! de cima do traço e o CABO é o resto, reconhecido pela lei que já existe — linha, polilinha, arco,
//! biarco: a forma do cabo é a de o desenhar sozinho. A ponta é a do traço ([`ph2d_vec_scene::Marker`],
//! posta pelo [`crate::Pencil`]), e o tamanho dela o das farpas desenhadas.
//!
//! Os oráculos ([`00_resultados.md`](../../../docs/UI_New_and_Simple/vetor_nova_ui/oraculo_quickshape/00_resultados.md)
//! §3h): o Xournal++ não tem seta (lê-a LINHA, a ponta engolida como gancho); o Excalidraw (`next`,
//! MIT, corrido e não lido) tem — cabo RECTO, V aberto, ida-e-volta ou fechado, farpas de 20–60°;
//! uma farpa só, farpas de 4 % do cabo, duas pontas e cabo curvo NÃO são seta para ele. O cabo curvo
//! e as duas pontas são a lei do dono (o pedido), não a do oráculo.

use crate::quickshape::{Law, QuickShape, recognise_with};
use crate::quickshape_geom::{P2, cross, dist, dot, seg_dist, sub};

/// A seta reconhecida, em MUNDO.
#[derive(Clone, Debug, PartialEq)]
pub struct ArrowShape {
    /// O cabo: [`QuickShape::Line`], polilinha aberta, [`QuickShape::Arc`] ou [`QuickShape::Biarc`].
    pub shaft: Box<QuickShape>,
    /// O comprimento médio das farpas desenhadas no `[início, fim]` do cabo; `None` = sem ponta.
    pub heads: [Option<f64>; 2],
}

impl ArrowShape {
    /// A semelhança à volta de `pivot`: o cabo e as farpas.
    #[must_use]
    pub fn transformed(&self, pivot: P2, scale: f64, rot: f64) -> Self {
        Self {
            shaft: Box::new(self.shaft.transformed(pivot, scale, rot)),
            heads: self.heads.map(|h| h.map(|h| h * scale)),
        }
    }
}

/// **As constantes da seta** — campo de [`Law`] para as sondas as varrerem no mesmo processo
/// (`quickshape_arrow_probe_tests.rs`). Cada uma com a FAIXA em que as setas de mão (20–400 px,
/// farpas de 20–60°, tremor ±0,5–2 px, o estabilizador) e os «não é seta» (o gancho, zigue-zague,
/// L, V, taco, S, rabisco, o tremor do repouso) continuam certos — `measure_arrow_ranges`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ArrowLaw {
    /// O ângulo de cada farpa com o cabo (graus, medido da ponta para TRÁS).
    pub(crate) barb_min_deg: f64,
    pub(crate) barb_max_deg: f64,
    /// A farpa mais curta é pelo menos esta fracção da mais comprida.
    pub(crate) barb_ratio: f64,
    /// A farpa média, em fracção do cabo.
    pub(crate) head_min: f64,
    pub(crate) head_max: f64,
    /// O traço da ponta fica junto do V (as duas farpas e a corda entre elas): resíduo rms em
    /// fracção da farpa média.
    pub(crate) head_rms: f64,
    /// A farpa mais curta tem pelo menos isto (px de tela): abaixo, o tremor da mão em repouso.
    pub(crate) barb_min_px: f64,
}

impl ArrowLaw {
    pub(crate) const MEASURED: Self = Self {
        barb_min_deg: 10.0,
        barb_max_deg: 80.0,
        barb_ratio: 0.4,
        head_min: 0.04,
        head_max: 0.8,
        head_rms: 0.15,
        barb_min_px: 6.0,
    };
}

/// **A seta**, se o traço ABERTO `samples` tem ponta num extremo (ou nos dois) e o resto é um cabo
/// que a lei reconhece. `r` é o traço reamostrado por comprimento (`length`) da lei.
pub(crate) fn recognise(
    law: &Law,
    samples: &[P2],
    r: &[P2],
    length: f64,
    px_to_world: f64,
    regular: bool,
) -> Option<QuickShape> {
    let a = law.arrow?;
    let ds = length / (r.len() - 1) as f64;
    let end = head(&a, r, ds, px_to_world);
    let stop = end.map_or(r.len() - 1, |(i, _)| i);
    let back: Vec<P2> = r[..=stop].iter().rev().copied().collect();
    let start = head(&a, &back, ds, px_to_world).map(|(i, h)| (stop - i, h));
    if end.is_none() && start.is_none() {
        return None;
    }
    let from = start.map_or(0, |(i, _)| i);
    let body = between(samples, r[from], r[stop], from as f64 * ds, stop as f64 * ds);
    let plain = Law { arrow: None, ..*law };
    let shaft = recognise_with(&plain, &body, px_to_world, regular)?;
    matches!(
        shaft,
        QuickShape::Line { .. }
            | QuickShape::Arc(_)
            | QuickShape::Biarc(_)
            | QuickShape::Polygon { closed: false, .. }
    )
    .then(|| {
        QuickShape::Arrow(ArrowShape {
            shaft: Box::new(shaft),
            heads: [start.map(|s| s.1), end.map(|e| e.1)],
        })
    })
}

/// **A ponta no FIM de `r`** (passo `ds` por amostra): o índice da ponta e a farpa média. Cada
/// amostra é candidata a ponta `T` ([`tip_at`]); a ponta é o PRIMEIRO mínimo do resíduo — o cabo
/// acaba onde a ponta começa. ⚠️ Não o menor de todos: uma candidata em cima da farpa, cujo resto
/// percorre dois lados do V (farpa → ponta → outra farpa), passa ainda mais perto do modelo e cortava
/// a ponta 14 px ao lado (o L de duas pontas, sonda `measure_arrow_first_look`).
pub(crate) fn head(a: &ArrowLaw, r: &[P2], ds: f64, px_to_world: f64) -> Option<(usize, f64)> {
    let mut best: Option<(f64, usize, f64)> = None;
    for i in 1..r.len().saturating_sub(2) {
        match (tip_at(a, r, i, ds, px_to_world), best) {
            (Some((rms, _)), Some((b, _, _))) if rms > b => break,
            (Some((rms, h)), _) => best = Some((rms, i, h)),
            (None, Some(_)) => break,
            (None, None) => {}
        }
    }
    best.map(|(_, i, h)| (i, h))
}

/// A amostra `i` como ponta: o eixo do cabo é a corda que chega a ela, as farpas são as amostras
/// seguintes mais longe de `T` de CADA lado do eixo, e o resíduo é o do traço da ponta ao V (as duas
/// farpas e a corda entre elas). `(resíduo, farpa média)` se passa a lei.
fn tip_at(a: &ArrowLaw, r: &[P2], i: usize, ds: f64, px_to_world: f64) -> Option<(f64, f64)> {
    let n = r.len();
    let (tip, shaft, tail) = (r[i], i as f64 * ds, (n - 1 - i) as f64 * ds);
    // A corda do cabo mede-se sobre o tamanho de uma farpa (a ponta anda ~3 farpas).
    let win = (tail / 3.0).min(shaft / 2.0);
    let j = i - ((win / ds).round() as usize).clamp(1, i);
    let back = sub(r[j], tip);
    let nb = back[0].hypot(back[1]);
    if nb <= f64::MIN_POSITIVE {
        return None;
    }
    let u = [back[0] / nb, back[1] / nb];
    let (mut left, mut right) = ((0.0, tip), (0.0, tip));
    for &p in &r[i + 1..] {
        let (v, d) = (sub(p, tip), dist(p, tip));
        let side = if cross(u, v) > 0.0 {
            &mut left
        } else {
            &mut right
        };
        if d > side.0 {
            *side = (d, p);
        }
    }
    let ((h1, b1), (h2, b2)) = (left, right);
    let (lo, hi, h) = (h1.min(h2), h1.max(h2), (h1 + h2) / 2.0);
    let barb = |b: P2| {
        let v = sub(b, tip);
        cross(u, v).abs().atan2(dot(u, v)).to_degrees()
    };
    if lo < a.barb_min_px * px_to_world
        || lo < a.barb_ratio * hi
        || h < a.head_min * shaft
        || h > a.head_max * shaft
        || ![barb(b1), barb(b2)]
            .iter()
            .all(|t| (a.barb_min_deg..=a.barb_max_deg).contains(t))
    {
        return None;
    }
    let ms = r[i..]
        .iter()
        .map(|&p| {
            seg_dist(p, tip, b1)
                .min(seg_dist(p, tip, b2))
                .min(seg_dist(p, b1, b2))
                .powi(2)
        })
        .sum::<f64>()
        / (n - i) as f64;
    let rms = ms.sqrt() / h;
    (rms <= a.head_rms).then_some((rms, h))
}

/// As amostras cruas entre os comprimentos de arco `s0` e `s1`, com as pontas `p0` e `p1` (os
/// pontos reamostrados nesses comprimentos).
fn between(samples: &[P2], p0: P2, p1: P2, s0: f64, s1: f64) -> Vec<P2> {
    let mut out = vec![p0];
    let mut s = 0.0;
    for w in samples.windows(2) {
        s += dist(w[0], w[1]);
        if s > s0 && s < s1 {
            out.push(w[1]);
        }
    }
    out.push(p1);
    out
}
