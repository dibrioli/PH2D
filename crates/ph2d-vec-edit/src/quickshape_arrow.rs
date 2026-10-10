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
use crate::quickshape_geom::{P2, cross, dist, dot, seg_dist, solve, sub};

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
/// (`quickshape_arrow_probe_tests.rs`, tabelas em `00_resultados.md` §3h). Cada FAIXA é onde as
/// fixtures dos dois oráculos continuam certas, as setas de 400 px saem ≥ 99 % e as de 24 px ≥ 20 %,
/// e nenhum «não é seta» (gancho, zigue-zague, L, V, T, taco, farpa só, S, estrela, rabisco, as
/// formas das sondas; 24–400 px, ±0,5–2, o repouso de ±1–3 px) sai seta — a linha que volta atrás
/// é a fronteira medida (§3h). O valor é o MEIO da faixa.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ArrowLaw {
    /// O ângulo de cada farpa com o cabo (graus, da ponta para TRÁS). Mínimo: uma faixa de um ponto,
    /// `4` — a 3 a linha que volta atrás passa a 25 % a 400 px, a 5 a farpa de 20° desenhada a 10°
    /// cai (98 %). Máximo `80–85` — a 75 a farpa de 60° desenhada a 70° cai, a 90 o T (a barra) é seta.
    pub(crate) barb_min_deg: f64,
    pub(crate) barb_max_deg: f64,
    /// A farpa média em fracção do cabo REAL: mínimo `0,04–0,05` (abaixo, a farpa de 4 % que os DOIS
    /// oráculos chamam linha vira seta; a 0,055 a seta comprida de ponta pequena, 8 %, cai), máximo
    /// `0,4–2` (a 0,35 a ponta grande, 40 %, cai; a 3 o T volta).
    pub(crate) head_min: f64,
    pub(crate) head_max: f64,
    /// O traço da ponta fica junto do V (as duas farpas e a corda entre elas): resíduo rms em
    /// fracção da farpa média, `0,09–0,10` (a 0,08 as de 400 px caem a 98 %; a 0,11 o rabisco).
    pub(crate) head_rms: f64,
    /// A farpa mais curta tem pelo menos isto (px de tela): `5,5–6` — abaixo, o repouso de ±3 px
    /// faz a 2.ª farpa de uma farpa só, de um L, de uma linha; a 7 as setas de 24 px caem a 9 %.
    pub(crate) barb_min_px: f64,
    /// A forma do cabo tem pelo menos esta fracção do comprimento do traço dele, `0,9–0,95`: um
    /// rabisco que vai e volta sobre a mesma recta lia-se polilinha curta (7 px de forma para 93 px
    /// de traço); a 0,875 o T de 400 px passa, a 0,97 as setas de 32 px caem.
    pub(crate) shaft_fill: f64,
    /// As duas farpas saem DA PONTA: depois da primeira, o traço volta a esta fracção da farpa média
    /// da ponta (no meio, o V aberto; no fim, o V fechado), `0,1–0,3`. Sem isto os dois últimos lados
    /// de um polígono — ponta, farpa, outra farpa, e acaba — liam-se V fechado (a estrela pequena);
    /// a 0,35 o T volta, a 0,05 as de 400 px caem a 95 %.
    pub(crate) tip_return: f64,
}

impl ArrowLaw {
    pub(crate) const MEASURED: Self = Self {
        barb_min_deg: 4.0,
        barb_max_deg: 82.5,
        head_min: 0.0475,
        head_max: 1.2,
        head_rms: 0.095,
        barb_min_px: 5.75,
        shaft_fill: 0.925,
        tip_return: 0.2,
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
    // Cada ponta achou-se contra TUDO o que vinha antes dela; o cabo é só o meio. A pentagrama lia-se
    // seta de duas pontas com farpas maiores que o cabo (gate `regular_keeps_a_star_a_star`).
    let from = start.map_or(0, |(i, _)| i);
    let shaft = (stop - from) as f64 * ds;
    let fits = |h: &(usize, f64)| (a.head_min * shaft..=a.head_max * shaft).contains(&h.1);
    if !start.iter().chain(end.iter()).all(fits) || (end.is_none() && start.is_none()) {
        return None;
    }
    let body = between(
        samples,
        r[from],
        r[stop],
        from as f64 * ds,
        stop as f64 * ds,
    );
    let plain = Law {
        arrow: None,
        ..*law
    };
    let shaft = recognise_with(&plain, &body, px_to_world, regular)?;
    let drawn: f64 = body.windows(2).map(|w| dist(w[0], w[1])).sum();
    if length_of(&shaft) < a.shaft_fill * drawn {
        return None;
    }
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
    // O fim do cabo mede-se sobre o tamanho de uma farpa (a ponta anda ~3 farpas).
    let win = (tail / 3.0).min(shaft / 2.0);
    let j = i - ((win / ds).round() as usize).clamp(1, i);
    let u = shaft_end(&r[j..=i])?;
    let (mut left, mut right) = ((0.0, tip, i), (0.0, tip, i));
    for (k, &p) in r.iter().enumerate().skip(i + 1) {
        let (v, d) = (sub(p, tip), dist(p, tip));
        let side = if cross(u, v) > 0.0 {
            &mut left
        } else {
            &mut right
        };
        if d > side.0 {
            *side = (d, p, k);
        }
    }
    let ((h1, b1, k1), (h2, b2, k2)) = (left, right);
    let (lo, h) = (h1.min(h2), (h1 + h2) / 2.0);
    let barb = |b: P2| {
        let v = sub(b, tip);
        cross(u, v).abs().atan2(dot(u, v)).to_degrees()
    };
    if lo < a.barb_min_px * px_to_world
        || h < a.head_min * shaft
        || h > a.head_max * shaft
        || ![barb(b1), barb(b2)]
            .iter()
            .all(|t| (a.barb_min_deg..=a.barb_max_deg).contains(t))
        || r[k1.min(k2)..]
            .iter()
            .map(|&p| dist(p, tip))
            .fold(f64::MAX, f64::min)
            > a.tip_return * h
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

/// O comprimento do caminho de uma forma aberta (o arco amostrado a 64 passos por troço).
fn length_of(q: &QuickShape) -> f64 {
    let arc = |a: &crate::quickshape::ArcShape| {
        (0..64)
            .map(|k| {
                let t = |k: i32| a.start + a.sweep * f64::from(k) / 64.0;
                dist(a.at(t(k)), a.at(t(k + 1)))
            })
            .sum::<f64>()
    };
    match q {
        QuickShape::Line { a, b } => dist(*a, *b),
        QuickShape::Polygon { corners, .. } => corners.windows(2).map(|w| dist(w[0], w[1])).sum(),
        QuickShape::Arc(a) => arc(a),
        QuickShape::Biarc(b) => arc(&b[0]) + arc(&b[1]),
        _ => 0.0,
    }
}

/// A direcção do cabo para TRÁS na ponta (o último ponto de `w`): a tangente na ponta da parábola
/// `y = a·x² + b·x + c` ajustada a `w` no referencial da corda. ⚠️ A corda sozinha vira metade do
/// arco que abarca: num cabo curvo de raio 190 px e farpas de 100 px eram 15°, e as farpas de 20° e
/// de 60° saíam da faixa (sonda `measure_arrows`: o cabo em arco ficava livre).
fn shaft_end(w: &[P2]) -> Option<P2> {
    let tip = w[w.len() - 1];
    let chord = sub(w[0], tip);
    let len = chord[0].hypot(chord[1]);
    if len <= f64::MIN_POSITIVE {
        return None;
    }
    let u = [chord[0] / len, chord[1] / len];
    let n = [-u[1], u[0]];
    let mut m = [[0.0; 3]; 3];
    let mut v = [0.0; 3];
    for &p in w {
        let q = sub(p, tip);
        let (x, y) = (dot(q, u) / len, dot(q, n) / len);
        let row = [x * x, x, 1.0];
        for a in 0..3 {
            v[a] += row[a] * y;
            for b in 0..3 {
                m[a][b] += row[a] * row[b];
            }
        }
    }
    let b = solve(m, v).map_or(0.0, |s| s[1]);
    let d = [u[0] + b * n[0], u[1] + b * n[1]];
    let nd = d[0].hypot(d[1]);
    Some([d[0] / nd, d[1] / nd])
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
