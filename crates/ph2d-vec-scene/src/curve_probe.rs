//! **A SONDA DE CURVA** — projetar um ponto sobre a geometria, e achar onde duas curvas se
//! cruzam (plano 25 §9, a W6). Puro, em `[f64; 2]`, sem kurbo (esta crate é o modelo de
//! documento — a conversão para `BezPath` vive no render).
//!
//! # Por que isto não é o [`crate::nearest_point_on_path`]
//!
//! Aquele responde *"em que SEGMENTO e em que `t` o usuário clicou?"* — a pergunta do
//! hit-test, cuja resposta é um endereço no documento, e uma amostragem grosseira basta
//! porque o consumidor insere um vértice ali. Esta responde *"em que PONTO exato do desenho
//! isto encosta?"* — a pergunta do snap, cuja resposta é uma coordenada que o artista vai
//! ver ampliada 100×. Amostrar não serve: o erro de uma amostragem de `SAMPLES` por segmento
//! é da ordem de `comprimento / SAMPLES`, que a um zoom alto são pixels. Por isso as duas
//! funções daqui **refinam por Newton** depois de amostrar — a amostragem só escolhe a bacia.
//!
//! # A localização substitui o cache
//!
//! Cruzamentos são `O(n²)` sobre os segmentos da cena, e o plano previa cacheá-los por gesto.
//! Não é preciso: o snap só quer os cruzamentos a até `max_d` do cursor, e filtrar os
//! segmentos por essa vizinhança ANTES do laço par-a-par deixa o custo em `O(n + k·n)` com
//! `k` = quantos segmentos passam perto (tipicamente 0..4). Um cache seria memória que pode
//! ficar velha; a localização não tem estado nenhum.

use crate::{VecPath, Xform, compound::contour_segments};

/// Um segmento cúbico em MUNDO: os quatro pontos de controle.
///
/// É o que a sonda consome — não uma [`VecPath`] clonada. Os alvos de snap são recolhidos
/// por GESTO e o `Xform` já foi aplicado, então guardar estilo, efeitos e contornos seria
/// pagar por dado que a projeção nunca lê.
pub type CubicSeg = [[f64; 2]; 4];

/// Amostras por segmento na busca grosseira. Ela só precisa **escolher a bacia** certa para
/// o Newton — o valor final vem do refino, não daqui.
const SAMPLES: usize = 16;

/// Iterações de Newton. Convergência é quadrática; oito é folga confortável sobre as ~4 que
/// um caso normal usa, e o laço sai cedo quando o passo fica sob [`STEP_EPS`].
const NEWTON_ITERS: usize = 8;

/// Passo de Newton abaixo do qual o refino parou de aprender (em `t`, que é adimensional).
const STEP_EPS: f64 = 1e-12;

/// Tolerância RELATIVA à escala do segmento para decidir *"isto é a ponta da curva"*. Um
/// cruzamento que pousa numa ponta é uma **âncora**, que já é alvo distinto por outra via —
/// e é assim que dois segmentos vizinhos (que compartilham a âncora **por construção**)
/// deixam de reportar um cruzamento em cada junta do desenho.
const ENDPOINT_EPS: f64 = 1e-6;

/// Empurra os segmentos cúbicos de `path` em MUNDO para `out`.
///
/// Percorre TODOS os contornos (o buraco de uma rosquinha também encaixa) e usa a geometria
/// **COZIDA** ([`VecPath::cooked`]) — é ela que o artista vê, e encaixar na fonte afiada
/// deixaria o ímã a um raio de distância da linha desenhada num vértice arredondado.
pub fn world_segs(path: &VecPath, xf: &Xform, out: &mut Vec<CubicSeg>) {
    let cooked = path.cooked();
    for c in 0..cooked.contour_count() {
        let Some((verts, closed)) = cooked.contour(c) else {
            continue;
        };
        let n = verts.len();
        for seg in 0..contour_segments(verts, closed) {
            let (a, b) = (seg, (seg + 1) % n);
            out.push([
                xf.apply(verts[a].anchor),
                xf.apply(verts[a].out_handle),
                xf.apply(verts[b].in_handle),
                xf.apply(verts[b].anchor),
            ]);
        }
    }
}

/// `C(t)` de um cúbico (de Casteljau expandido).
#[must_use]
fn at(s: &CubicSeg, t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        a * s[0][0] + b * s[1][0] + c * s[2][0] + d * s[3][0],
        a * s[0][1] + b * s[1][1] + c * s[2][1] + d * s[3][1],
    ]
}

/// `C'(t)`.
#[must_use]
fn deriv(s: &CubicSeg, t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (a, b, c) = (3.0 * u * u, 6.0 * u * t, 3.0 * t * t);
    [
        a * (s[1][0] - s[0][0]) + b * (s[2][0] - s[1][0]) + c * (s[3][0] - s[2][0]),
        a * (s[1][1] - s[0][1]) + b * (s[2][1] - s[1][1]) + c * (s[3][1] - s[2][1]),
    ]
}

/// `C''(t)`.
#[must_use]
fn deriv2(s: &CubicSeg, t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let mut o = [0.0; 2];
    for k in 0..2 {
        let d0 = s[2][k] - 2.0 * s[1][k] + s[0][k];
        let d1 = s[3][k] - 2.0 * s[2][k] + s[1][k];
        o[k] = 6.0 * (u * d0 + t * d1);
    }
    o
}

/// A caixa do **casco de controle** — superconjunto da curva, que é exactamente o que uma
/// rejeição precisa ser (nunca descarta um segmento que poderia ganhar).
#[must_use]
fn hull_box(s: &CubicSeg) -> ([f64; 2], [f64; 2]) {
    let mut lo = s[0];
    let mut hi = s[0];
    for p in &s[1..] {
        for k in 0..2 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    (lo, hi)
}

/// A caixa alcança `q` a até `d`?
#[must_use]
fn box_near(bx: ([f64; 2], [f64; 2]), q: [f64; 2], d: f64) -> bool {
    (0..2).all(|k| q[k] >= bx.0[k] - d && q[k] <= bx.1[k] + d)
}

/// As duas caixas se sobrepõem?
#[must_use]
fn boxes_overlap(a: ([f64; 2], [f64; 2]), b: ([f64; 2], [f64; 2])) -> bool {
    (0..2).all(|k| a.0[k] <= b.1[k] && b.0[k] <= a.1[k])
}

#[must_use]
fn dist2(a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    dx * dx + dy * dy
}

/// O `t` do ponto de `s` mais próximo de `q`, e a distância² ali.
///
/// Amostra para escolher a bacia, depois **refina por Newton** sobre
/// `g(t) = (C(t) − q)·C'(t) = 0` (a condição de perpendicularidade). O `t` é preso a
/// `[0, 1]`: fora dele a resposta é a ponta, e a ponta é uma âncora.
#[must_use]
fn nearest_t(s: &CubicSeg, q: [f64; 2]) -> (f64, f64) {
    let mut best = (0.0, f64::INFINITY);
    for k in 0..=SAMPLES {
        #[expect(clippy::cast_precision_loss, reason = "k <= SAMPLES = 16")]
        let t = k as f64 / SAMPLES as f64;
        let d2 = dist2(at(s, t), q);
        if d2 < best.1 {
            best = (t, d2);
        }
    }
    let mut t = best.0;
    for _ in 0..NEWTON_ITERS {
        let p = at(s, t);
        let d1 = deriv(s, t);
        let d2v = deriv2(s, t);
        let r = [p[0] - q[0], p[1] - q[1]];
        let g = r[0] * d1[0] + r[1] * d1[1];
        let gp = d1[0] * d1[0] + d1[1] * d1[1] + r[0] * d2v[0] + r[1] * d2v[1];
        if gp.abs() < f64::MIN_POSITIVE {
            break;
        }
        let step = g / gp;
        t = (t - step).clamp(0.0, 1.0);
        if step.abs() < STEP_EPS {
            break;
        }
    }
    (t, dist2(at(s, t), q))
}

/// O ponto de `segs` mais próximo de `q`, se algum estiver a até `max_d`.
///
/// `None` quando nada alcança — e é isso que faz o ímã **soltar** em vez de puxar de
/// longe.
#[must_use]
pub fn nearest_on_segs(segs: &[CubicSeg], q: [f64; 2], max_d: f64) -> Option<[f64; 2]> {
    let max2 = max_d * max_d;
    let mut best: Option<([f64; 2], f64)> = None;
    for s in segs {
        if !box_near(hull_box(s), q, max_d) {
            continue;
        }
        let (t, d2) = nearest_t(s, q);
        if d2 <= max2 && best.is_none_or(|(_, bd)| d2 < bd) {
            best = Some((at(s, t), d2));
        }
    }
    best.map(|(p, _)| p)
}

/// Cruzamento de dois SEGMENTOS DE RETA, em parâmetros `(u, v)` de cada um. `None` se são
/// paralelos ou se o encontro cai fora dos dois.
#[must_use]
fn line_cross(a0: [f64; 2], a1: [f64; 2], b0: [f64; 2], b1: [f64; 2]) -> Option<(f64, f64)> {
    let r = [a1[0] - a0[0], a1[1] - a0[1]];
    let s = [b1[0] - b0[0], b1[1] - b0[1]];
    let den = r[0] * s[1] - r[1] * s[0];
    if den == 0.0 {
        return None;
    }
    let q = [b0[0] - a0[0], b0[1] - a0[1]];
    let u = (q[0] * s[1] - q[1] * s[0]) / den;
    let v = (q[0] * r[1] - q[1] * r[0]) / den;
    ((0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v)).then_some((u, v))
}

/// Refina um par `(ta, tb)` até `Ca(ta) = Cb(tb)` por Newton 2×2 (Cramer sobre o
/// Jacobiano `J = [Ca'(ta), −Cb'(tb)]`). Devolve o ponto, ou `None` se o sistema degenerar
/// (curvas tangentes — onde "o cruzamento" não é um ponto isolado).
///
/// ⚠️ O passo é `Δ = J⁻¹·F` e depois `t ← t − Δ` — **uma negação, não duas**. A primeira
/// versão resolvia `J·Δ = −F` *e* subtraía, e o refino caminhava para LONGE da raiz. Um gate
/// entre duas RETAS não pode ver isso: ali a corda amostrada já é a resposta, então `F = 0`,
/// `Δ = 0`, e o Newton quebrado fica parado exactamente sobre o valor certo.
#[must_use]
fn refine_cross(a: &CubicSeg, b: &CubicSeg, mut ta: f64, mut tb: f64) -> Option<[f64; 2]> {
    for _ in 0..NEWTON_ITERS {
        let (pa, pb) = (at(a, ta), at(b, tb));
        let f = [pa[0] - pb[0], pa[1] - pb[1]];
        let (da, db) = (deriv(a, ta), deriv(b, tb));
        let den = db[0] * da[1] - da[0] * db[1];
        if den.abs() < f64::MIN_POSITIVE {
            return None;
        }
        let dta = (db[0] * f[1] - f[0] * db[1]) / den;
        let dtb = (da[0] * f[1] - f[0] * da[1]) / den;
        ta = (ta - dta).clamp(0.0, 1.0);
        tb = (tb - dtb).clamp(0.0, 1.0);
        if dta.abs() < STEP_EPS && dtb.abs() < STEP_EPS {
            break;
        }
    }
    let (pa, pb) = (at(a, ta), at(b, tb));
    // Newton pode ter sido preso pelo clamp numa ponta sem de facto fechar o sistema.
    let scale = seg_scale(a).max(seg_scale(b));
    (dist2(pa, pb) <= (ENDPOINT_EPS * scale).powi(2)).then_some(pa)
}

/// Uma medida de tamanho do segmento, para as tolerâncias serem relativas (uma cena
/// autorada em metros e outra em milhares de unidades merecem o mesmo comportamento).
#[must_use]
fn seg_scale(s: &CubicSeg) -> f64 {
    let (lo, hi) = hull_box(s);
    (hi[0] - lo[0]).abs().max((hi[1] - lo[1]).abs()).max(1.0)
}

/// O ponto é uma das quatro pontas dos dois segmentos?
#[must_use]
fn is_endpoint(p: [f64; 2], a: &CubicSeg, b: &CubicSeg) -> bool {
    let tol = (ENDPOINT_EPS * seg_scale(a).max(seg_scale(b))).powi(2);
    [a[0], a[3], b[0], b[3]].iter().any(|&e| dist2(p, e) <= tol)
}

/// Os cruzamentos entre segmentos de `segs` que caem a até `max_d` de `q`.
///
/// ⚠️ **Um cruzamento numa PONTA é descartado**: ele é a âncora que dois segmentos vizinhos
/// compartilham por construção, e uma âncora já é alvo por outra via. Sem esta regra toda
/// junta do desenho reportaria um "cruzamento", e o ímã dos cruzamentos passaria a competir
/// com o das âncoras em todo canto do traço.
#[must_use]
pub fn crossings_near(segs: &[CubicSeg], q: [f64; 2], max_d: f64) -> Vec<[f64; 2]> {
    let max2 = max_d * max_d;
    let boxes: Vec<_> = segs.iter().map(hull_box).collect();
    let near: Vec<usize> = (0..segs.len())
        .filter(|&i| box_near(boxes[i], q, max_d))
        .collect();
    let mut out: Vec<[f64; 2]> = Vec::new();
    for &i in &near {
        for j in 0..segs.len() {
            if j == i || !boxes_overlap(boxes[i], boxes[j]) {
                continue;
            }
            // Cada par é examinado uma vez: se os DOIS estão perto, só o menor índice roda.
            if near.contains(&j) && j < i {
                continue;
            }
            collect_pair(&segs[i], &segs[j], q, max2, &mut out);
        }
    }
    out
}

/// Os cruzamentos de UM par, filtrados e deduplicados dentro de `out`.
fn collect_pair(a: &CubicSeg, b: &CubicSeg, q: [f64; 2], max2: f64, out: &mut Vec<[f64; 2]>) {
    let dedup = (ENDPOINT_EPS * seg_scale(a).max(seg_scale(b))).powi(2);
    let poly = |s: &CubicSeg| -> Vec<[f64; 2]> {
        (0..=SAMPLES)
            .map(|k| {
                #[expect(clippy::cast_precision_loss, reason = "k <= SAMPLES = 16")]
                let t = k as f64 / SAMPLES as f64;
                at(s, t)
            })
            .collect()
    };
    let (pa, pb) = (poly(a), poly(b));
    #[expect(clippy::cast_precision_loss, reason = "SAMPLES = 16")]
    let n = SAMPLES as f64;
    for i in 0..SAMPLES {
        for j in 0..SAMPLES {
            let Some((u, v)) = line_cross(pa[i], pa[i + 1], pb[j], pb[j + 1]) else {
                continue;
            };
            #[expect(clippy::cast_precision_loss, reason = "i, j < SAMPLES = 16")]
            let (ta, tb) = ((i as f64 + u) / n, (j as f64 + v) / n);
            let Some(p) = refine_cross(a, b, ta, tb) else {
                continue;
            };
            if dist2(p, q) > max2 || is_endpoint(p, a, b) {
                continue;
            }
            if !out.iter().any(|&o| dist2(o, p) <= dedup) {
                out.push(p);
            }
        }
    }
}

#[cfg(test)]
#[path = "curve_probe_tests.rs"]
mod tests;
