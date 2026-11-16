//! ⭐ **Criar pontos numa curva PRONTA** (dono 15/11: «permita inserir pontos mesmo se a curva for
//! um easing»). Um Ease/Elastic/Bounce… não tem alças: o toque converte-o numa Custom com a MESMA
//! cara — nós escolhidos onde a cúbica mais erra (até [`NOS_MAX`], deixando lugar ao ponto do
//! artista), cada um no valor exato da curva e com as alças na inclinação dela de cada lado (Corner
//! onde ela quebra, como no Bounce) — e o ponto nasce onde se tocou.

use ph2d_anim::{EaseSpline, HandleKind, Interp, SPLINE_MAX_POINTS, SplineSide};

use crate::graph_pontos::segment_curve;
use crate::snapshot::KeyView;

/// Nós que a conversão pode usar: um lugar fica para o ponto que o toque cria.
const NOS_MAX: usize = SPLINE_MAX_POINTS - 1;
/// O erro (no quadrado unitário) abaixo do qual a forma já é a da curva pronta.
const ERRO_MAX: f64 = 0.004;
/// Amostras da medida do erro.
const AMOSTRAS: usize = 240;
/// A inclinação (no quadrado unitário) acima da qual a tangente conta como vertical: os presets
/// que não o são ficam muito abaixo (Back 3,6 · Bounce 5,5 · Expo 6,9 · Elastic ~7).
const VERTICAL: f64 = 20.0;
/// A alça de um arco de círculo, em frações da corda (`4/3·tan(θ/4)` a 90°).
const ARCO: f64 = 0.5523;
/// Lugares da grelha de candidatos a nó.
const CANDIDATOS: usize = 64;
/// Passo das inclinações de um lado (numéricas: a de um preset pode quebrar num ponto).
const H: f64 = 1.0e-6;
/// O passo do sinal na bissecção de um canto (bem abaixo de `H`, para o canto não sair desviado).
const BISSECAO_H: f64 = 1.0e-11;

/// A curva que um toque na linha do trecho edita: a Custom dele, ou a curva pronta convertida
/// ([`preset_as_curve`]). `None` onde não se cria ponto (Hold, Nearest, parado, sem duração).
#[must_use]
pub fn insertable_curve(k0: &KeyView, k1: &KeyView) -> Option<EaseSpline> {
    segment_curve(k0, k1).or_else(|| {
        let dv = f64::from(k1.value - k0.value);
        let ok = k1.t_seconds > k0.t_seconds && dv.abs() > f64::EPSILON;
        ok.then(|| preset_as_curve(k0.interp)).flatten()
    })
}

/// Uma curva pronta (`Interp::Eased`) como `EaseSpline` com a mesma forma. `None` para o resto.
#[must_use]
pub fn preset_as_curve(interp: Interp) -> Option<EaseSpline> {
    let Interp::Eased(_) = interp else {
        return None;
    };
    let e = |u: f64| interp.remap(u);
    let erro_de = |c: &EaseSpline| {
        (0..=AMOSTRAS)
            .map(|i| i as f64 / AMOSTRAS as f64)
            .map(|x| (c.eval(x) - e(x)).abs())
            .fold(0.0, f64::max)
    };
    // Candidatos: uma grelha e os pontos onde a curva QUEBRA ou VIRA (Bounce, Elastic).
    let mut cands: Vec<f64> = (1..CANDIDATOS)
        .map(|i| i as f64 / CANDIDATOS as f64)
        .collect();
    // Onde a inclinação troca de sinal (um pico do Elastic, um ressalto do Bounce), o lugar EXATO
    // por bissecção — um nó ao lado do canto não o representa.
    let passo = 1.0 / AMOSTRAS as f64;
    let mut viradas = Vec::new();
    for i in 0..AMOSTRAS {
        let (mut a, mut b) = (i as f64 * passo, (i + 1) as f64 * passo);
        let (sa, sb) = (lados(interp, a).1, lados(interp, b).0);
        if sa * sb >= 0.0 {
            continue;
        }
        for _ in 0..48 {
            let m = 0.5 * (a + b);
            // O sinal logo À ESQUERDA de `m`, sem desvio (a `lados` lê a 1–2 `H` dele).
            if (interp.remap(m) - interp.remap(m - BISSECAO_H)) * sa > 0.0 {
                a = m;
            } else {
                b = m;
            }
        }
        viradas.push(0.5 * (a + b));
    }
    cands.extend(&viradas);
    // Os lugares onde a curva VIRA entram primeiro, se cabem: são os nós que a forma pede (um
    // passo de cada vez nunca os acha — cada um sozinho ajuda menos que um ponto da grelha).
    let mut xs = vec![0.0, 1.0];
    if viradas.len() <= NOS_MAX {
        xs.extend(&viradas);
        xs.sort_by(f64::total_cmp);
    }
    let c = monta(interp, &xs);
    let mut erro = erro_de(&c);
    let (mut melhor_erro, mut melhor_c) = (erro, c);
    while erro >= ERRO_MAX && xs.len() - 2 < NOS_MAX {
        let melhor = cands
            .iter()
            .filter(|x| xs.iter().all(|q| (q - **x).abs() > 0.01))
            .map(|&x| {
                let mut ys = xs.clone();
                ys.push(x);
                ys.sort_by(f64::total_cmp);
                let c = monta(interp, &ys);
                (erro_de(&c), ys, c)
            })
            .min_by(|a, b| a.0.total_cmp(&b.0));
        let Some((novo, ys, cn)) = melhor else { break };
        if novo < melhor_erro {
            (melhor_erro, melhor_c) = (novo, cn);
        }
        (erro, xs) = (novo, ys);
    }
    // Ajuste fino: cada nó experimenta cada candidato; fica onde a forma melhora (duas voltas).
    for _ in 0..2 {
        for j in 1..xs.len() - 1 {
            for &x in &cands {
                if xs.iter().any(|q| (q - x).abs() <= 0.01) {
                    continue;
                }
                let mut ys = xs.clone();
                ys[j] = x;
                ys.sort_by(f64::total_cmp);
                let c = monta(interp, &ys);
                let novo = erro_de(&c);
                if novo < melhor_erro {
                    (melhor_erro, melhor_c, xs) = (novo, c, ys);
                }
            }
        }
    }
    Some(melhor_c)
}

/// A inclinação da curva pronta à esquerda e à direita de `x`, cada uma lida SÓ desse lado e sem
/// tocar em `x` (a `Interp::slope` de um preset é central: num ressalto do Bounce dava a média dos
/// dois lados; e o Expo salta no 0 por definição).
fn lados(interp: Interp, x: f64) -> (f64, f64) {
    let e = |u: f64| interp.remap(u.clamp(0.0, 1.0)); // CLAMP-OK: normalized u
    let esq = (e(x - H) - e(x - 2.0 * H)) / H;
    let dir = (e(x + 2.0 * H) - e(x + H)) / H;
    (
        if x <= 0.0 { dir } else { esq },
        if x >= 1.0 { esq } else { dir },
    )
}

/// A alça de Hermite a partir de `p` na inclinação `s` (para `lado` = ±1): ⅓ da distância em `x`
/// até `q` (exato numa cúbica). Uma tangente quase VERTICAL (o fim do Circ) não tem ⅓ em `x` que
/// sirva: a alça vai na direção dela com [`ARCO`] da corda, a do arco de círculo.
fn alca(p: [f64; 2], q: [f64; 2], s: f64, lado: f64) -> [f32; 2] {
    if s.abs() > VERTICAL {
        let corda = ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2)).sqrt() * ARCO;
        let n = (1.0 + s * s).sqrt();
        let (dx, dy) = (corda / n, s * corda / n);
        return [(p[0] + lado * dx) as f32, (p[1] + lado * dy) as f32];
    }
    let dx = (q[0] - p[0]).abs() / 3.0;
    [(p[0] + lado * dx) as f32, (p[1] + lado * s * dx) as f32]
}

/// A Hermite cúbica que passa por `e` nos `xs` (ordenados, com 0 e 1), alças na tangente.
fn monta(interp: Interp, xs: &[f64]) -> EaseSpline {
    let e = |u: f64| interp.remap(u);
    let f = |v: f64| v as f32;
    let fim = xs.len() - 1;
    let ponto = |x: f64| [x, e(x)];
    let mut c = EaseSpline::from_bezier(1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0);
    // Primeiro TODOS os nós (cada `insert` reescreve as alças dos vizinhos), depois as alças.
    let mut nos = Vec::new();
    for &x in &xs[1..fim] {
        let Some(i) = c.insert(x) else { break };
        c.move_point(i, f(x), f(e(x)));
        c.set_kind(i, HandleKind::Corner);
        nos.push(x);
    }
    let xs: Vec<f64> = std::iter::once(0.0).chain(nos).chain([1.0]).collect();
    let fim = xs.len() - 1;
    for j in 1..fim {
        let (x, p) = (xs[j], ponto(xs[j]));
        let (sl, sr) = lados(interp, x);
        let hi = alca(p, ponto(xs[j - 1]), sl, -1.0);
        let ho = alca(p, ponto(xs[j + 1]), sr, 1.0);
        c.move_handle(j, SplineSide::In, hi[0], hi[1]);
        c.move_handle(j, SplineSide::Out, ho[0], ho[1]);
        if (sl - sr).abs() <= 1.0e-3 * (1.0 + sl.abs()) {
            c.set_kind(j, HandleKind::Smooth);
        }
    }
    let (_, s0) = lados(interp, 0.0);
    let (s1, _) = lados(interp, 1.0);
    let out = alca(ponto(0.0), ponto(xs[1]), s0, 1.0);
    let inn = alca(ponto(1.0), ponto(xs[fim - 1]), s1, -1.0);
    c.with_end_handles(out, inn)
}

#[cfg(test)]
#[path = "graph_pontos_pronta_tests.rs"]
mod tests;
