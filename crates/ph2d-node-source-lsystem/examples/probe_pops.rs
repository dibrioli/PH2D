//! **OS PEQUENOS PULOS** — report do Enio (2026-08-31): *"Dá pequenos pulos, não é
//! perfeitamente liso"*.
//!
//! # ⚠️⚠️ Por que TODAS as réguas desta linha dizem «liso»
//!
//! O [`probe_flicker`] e o [`probe_drift`] medem **um escalar de TAMANHO** (a largura média de
//! Cauchy, o span de eixo, o centroide) e perguntam se a série dele é uma recta. A lei do
//! produto (`crate::build`) **normaliza exactamente essa grandeza**: ela resolve o factor de
//! comprimento para que o tamanho caia na rampa recta. ⇒ *a régua e o produto partilham a lei*,
//! e uma régua que partilha a lei do que julga é um espelho (memória
//! `an_oracle_that_shares_the_law_of_what_it_judges_is_a_mirror`).
//!
//! ⇒ Esta sonda mede a **IMAGEM**: rasteriza o esqueleto numa grelha e pergunta *quanto da
//! figura MUDOU* entre dois instantes consecutivos do arrasto. Um crescimento perfeitamente
//! liso dá uma série **plana**; um pulo dá um **pico**.
//!
//! # As duas molduras, e as duas apanham defeitos diferentes
//!
//! | moldura | o que mede | cega a |
//! |---|---|---|
//! | `mundo` | janela FIXA (a caixa do último quadro) — o que se vê da cadeira | nada; é a soma |
//! | `forma` | cada quadro re-centrado e re-escalado para tamanho `1` | mudança de TAMANHO |
//!
//! ⚠️ A moldura `forma` re-centra pelo **centro da caixa**, nunca pela média dos vértices: a
//! contagem de elementos multiplica por 5 numa travessia e os novos nascem coincidentes com os
//! pais, então a média salta por AMOSTRAGEM (memória
//! `a_centroid_by_vertex_average_measures_the_sampling_not_the_shape`, e as duas primeiras
//! réguas do `probe_flicker` caíram pela mesma causa).
//!
//! # O que a saída diz
//!
//! - `mediana` — a mudança típica de um passo do arrasto (a unidade é «fracção da tinta»).
//! - `máx ÷ mediana` — a **ondulação**: `1,0` é perfeito, `6` quer dizer que um passo mudou
//!   seis vezes mais do que o passo típico.
//! - `@t` — ONDE está o pior passo, na escala do slider `Growth`.
//! - `Δn` — se a contagem de elementos mudou nesse mesmo passo (⇒ o pior passo é a **fronteira
//!   de geração**, e não uma travessia qualquer).
//!
//! ⚠️ **Determinística**: nenhuma leitura de relógio, então a carga da máquina não a move
//! (`CLAUDE.md` §5.0).

use ph2d_node_source_lsystem::{PRESETS, Preset, param, probe_build};
use ph2d_nodegraph::attr::Column;

/// Direcções da largura média. Não paga o orçamento do produto, então é generoso.
const K_OBS: usize = 32;
/// Lado da grelha da imagem. `192` põe o traço mais fino do corpus em ~1 célula.
const GRID: usize = 192;
/// Passos do arrasto. `0,005` é ~4× mais fino do que a fronteira de geração mais estreita.
const STEP: f32 = 0.005;
const T0: f32 = 0.06;

fn mean_width(v: &[[f32; 2]]) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    let mut acc = 0.0f64;
    for k in 0..K_OBS {
        let a = std::f32::consts::PI * k as f32 / K_OBS as f32;
        let (c, s) = (a.cos(), a.sin());
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for q in v {
            let t = q[0] * c + q[1] * s;
            lo = lo.min(t);
            hi = hi.max(t);
        }
        acc += f64::from(hi - lo);
    }
    (acc / K_OBS as f64) as f32
}

/// `(x0, y0, x1, y1)` da caixa alinhada aos eixos.
fn bbox(v: &[[f32; 2]]) -> [f32; 4] {
    let mut b = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
    for q in v {
        b[0] = b[0].min(q[0]);
        b[1] = b[1].min(q[1]);
        b[2] = b[2].max(q[0]);
        b[3] = b[3].max(q[1]);
    }
    b
}

/// Um quadro do arrasto, já lido do stream.
struct Frame {
    p: Vec<[f32; 2]>,
    parent: Vec<f32>,
    ink: f32,
    n: usize,
}

fn frame(pr: &Preset, t: f32) -> Frame {
    let s = probe_build(
        pr.axiom,
        pr.rules,
        pr.generations,
        &[
            (param::ANGLE, pr.angle),
            (param::STEP, pr.step),
            (param::WIDTH, pr.width),
            (param::GROWTH, t),
        ],
    );
    let p = match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    let parent = match s.get("parent") {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    let ink = match s.get("len") {
        Some(Column::Scalar(v)) => v.iter().sum(),
        _ => 0.0,
    };
    let n = p.len();
    Frame { p, parent, ink, n }
}

/// Marca as células que o esqueleto toca, dentro da janela `(cx, cy, half)`.
fn cover(f: &Frame, cx: f32, cy: f32, half: f32) -> Vec<bool> {
    let mut g = vec![false; GRID * GRID];
    if half <= 0.0 {
        return g;
    }
    let to_cell = |x: f32, y: f32| -> (f32, f32) {
        (
            (x - cx + half) / (2.0 * half) * GRID as f32,
            (y - cy + half) / (2.0 * half) * GRID as f32,
        )
    };
    let mut put = |u: f32, v: f32| {
        if u >= 0.0 && v >= 0.0 {
            let (iu, iv) = (u as usize, v as usize);
            if iu < GRID && iv < GRID {
                g[iv * GRID + iu] = true;
            }
        }
    };
    for i in 0..f.n {
        let par = f.parent.get(i).copied().unwrap_or(-1.0);
        if par < 0.0 {
            continue;
        }
        let a = f.p[par as usize];
        let b = f.p[i];
        let (ua, va) = to_cell(a[0], a[1]);
        let (ub, vb) = to_cell(b[0], b[1]);
        // Meia célula por passo: nenhum buraco na diagonal.
        let steps = ((ub - ua).abs().max((vb - va).abs()) * 2.0).ceil().max(1.0) as usize;
        for k in 0..=steps {
            let w = k as f32 / steps as f32;
            put(ua + (ub - ua) * w, va + (vb - va) * w);
        }
    }
    g
}

/// Distância de Jaccard entre duas coberturas: `|A Δ B| / |A ∪ B|`.
fn jaccard(a: &[bool], b: &[bool]) -> f32 {
    let (mut inter, mut union) = (0u32, 0u32);
    for (x, y) in a.iter().zip(b) {
        if *x || *y {
            union += 1;
            if *x && *y {
                inter += 1;
            }
        }
    }
    if union == 0 {
        return 0.0;
    }
    1.0 - inter as f32 / union as f32
}

fn median(v: &[f32]) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    s[s.len() / 2]
}

/// `(mediana, máximo, índice do máximo)`.
fn peak(v: &[f32]) -> (f32, f32, usize) {
    let m = median(v);
    let mut best = (f32::MIN, 0usize);
    for (i, x) in v.iter().enumerate() {
        if *x > best.0 {
            best = (*x, i);
        }
    }
    (m, best.0, best.1)
}

fn sweep(pr: &Preset, trace: bool) {
    let steps = ((1.0 - T0) / STEP).round() as usize;
    let frames: Vec<Frame> = (0..=steps)
        .map(|k| frame(pr, T0 + k as f32 * STEP))
        .collect();
    // A janela do MUNDO é a do último quadro, com folga — é a moldura em que o artista vê.
    let last = bbox(&frames[steps].p);
    let (wcx, wcy) = ((last[0] + last[2]) * 0.5, (last[1] + last[3]) * 0.5);
    let whalf = ((last[2] - last[0]).max(last[3] - last[1]) * 0.55).max(1e-6);

    let world: Vec<Vec<bool>> = frames.iter().map(|f| cover(f, wcx, wcy, whalf)).collect();
    let shape: Vec<Vec<bool>> = frames
        .iter()
        .map(|f| {
            // Re-centrada pelo CENTRO DA CAIXA e re-escalada pela largura média — as duas
            // grandezas são extensos, logo insensíveis a pontos coincidentes.
            let b = bbox(&f.p);
            let w = mean_width(&f.p).max(1e-9);
            let (cx, cy) = ((b[0] + b[2]) * 0.5, (b[1] + b[3]) * 0.5);
            let norm: Vec<[f32; 2]> =
                f.p.iter()
                    .map(|q| [(q[0] - cx) / w, (q[1] - cy) / w])
                    .collect();
            let g = Frame {
                p: norm,
                parent: f.parent.clone(),
                ink: 0.0,
                n: f.n,
            };
            cover(&g, 0.0, 0.0, 0.75)
        })
        .collect();

    let dw: Vec<f32> = world.windows(2).map(|w| jaccard(&w[0], &w[1])).collect();
    let ds: Vec<f32> = shape.windows(2).map(|w| jaccard(&w[0], &w[1])).collect();
    let di: Vec<f32> = frames
        .windows(2)
        .map(|f| (f[1].ink - f[0].ink).abs())
        .collect();

    if trace {
        println!("\n=== {} ===", pr.label);
        println!("     t      n    Δimg(mundo)  Δimg(forma)      tinta     Δtinta");
        for k in 0..dw.len() {
            let mark = if frames[k + 1].n != frames[k].n {
                " <-- fronteira"
            } else {
                ""
            };
            println!(
                "{:6.3} {:6}   {:10.4}   {:10.4} {:10.3} {:10.4}{mark}",
                T0 + (k + 1) as f32 * STEP,
                frames[k + 1].n,
                dw[k],
                ds[k],
                frames[k + 1].ink,
                di[k]
            );
        }
    }

    let (mw, xw, iw) = peak(&dw);
    let (ms, xs, is_) = peak(&ds);
    let (mi, xi, ii) = peak(&di);
    let born = |i: usize| {
        if frames[i + 1].n != frames[i].n {
            "sim"
        } else {
            "nao"
        }
    };
    println!(
        "{:8} mundo: med {mw:.4} max {xw:.4} ondul {:5.2}x @t {:.3} fronteira {}  |  \
         forma: med {ms:.4} max {xs:.4} ondul {:5.2}x @t {:.3} fronteira {}  |  \
         tinta: ondul {:6.2}x @t {:.3} fronteira {}",
        pr.label,
        if mw > 0.0 { xw / mw } else { 0.0 },
        T0 + (iw + 1) as f32 * STEP,
        born(iw),
        if ms > 0.0 { xs / ms } else { 0.0 },
        T0 + (is_ + 1) as f32 * STEP,
        born(is_),
        if mi > 0.0 { xi / mi } else { 0.0 },
        T0 + (ii + 1) as f32 * STEP,
        born(ii),
    );
}

/// ⭐⭐⭐ **SALTO ou MOVIMENTO? — o discriminador, e é o único que decide.**
///
/// Uma diferença entre dois quadros consecutivos é grande por duas razões opostas: ou a figura
/// **saltou** (descontinuidade), ou ela está simplesmente a **andar depressa** ali. As duas
/// leem-se iguais numa amostragem só — e é por isso que a régua tem de ser a **derivada**:
/// afina-se o passo `4×` e pergunta-se o que aconteceu.
///
/// | o que se vê ao afinar `4×` | o que era |
/// |---|---|
/// | a diferença encolhe `~4×` | MOVIMENTO — a figura anda, e a amostragem é que era grossa |
/// | a diferença **fica onde estava** | SALTO — há uma descontinuidade, e nenhuma taxa de quadros a esconde |
///
/// ⚠️ Medida **contra um controlo no meio da mesma geração**, nunca sozinha: a velocidade de
/// fundo do arrasto varia ao longo dele, e uma leitura absoluta misturaria as duas coisas.
/// *A pergunta não é «isto mudou muito?», é «isto mudou muito MAIS do que os vizinhos?»*
fn refine(pr: &Preset) {
    let steps = ((1.0 - T0) / STEP).round() as usize;
    let counts: Vec<(f32, usize)> = (0..=steps)
        .map(|k| {
            let t = T0 + k as f32 * STEP;
            (t, frame(pr, t).n)
        })
        .collect();
    // ⚠️ **TODAS as fronteiras, não a primeira** — a 1.ª redacção mediu só a `[0]` e caiu na
    // travessia mais precoce do arrasto, onde a planta tem 2 elementos e toda régua é ruído:
    // no `Weed` ela dizia `5,89x -> 1,19x` (parece movimento) sobre uma fronteira que **não é**
    // a que o report descreve. *Um extremo lido no primeiro membro de uma lista é o primeiro,
    // não o extremo.*
    let bounds: Vec<f32> = counts
        .windows(2)
        .filter(|w| w[1].1 != w[0].1)
        .map(|w| (w[0].0 + w[1].0) * 0.5)
        .collect();
    if bounds.is_empty() {
        println!("{:8} (nenhuma fronteira de geração no arrasto)", pr.label);
        return;
    }
    let gap = |t: f32, h: f32| -> (f32, f32) {
        let (a, b) = (frame(pr, t - h * 0.5), frame(pr, t + h * 0.5));
        let bb = bbox(&b.p);
        let half = ((bb[2] - bb[0]).max(bb[3] - bb[1]) * 0.55).max(1e-6);
        let (cx, cy) = ((bb[0] + bb[2]) * 0.5, (bb[1] + bb[3]) * 0.5);
        (
            jaccard(&cover(&a, cx, cy, half), &cover(&b, cx, cy, half)),
            (b.ink - a.ink).abs() / b.ink.max(1e-6),
        )
    };
    let (h0, h1) = (STEP, STEP * 0.25);
    let r = |b: f32, c: f32| if c > 1e-9 { b / c } else { f32::INFINITY };
    // A PIOR fronteira do arrasto, escolhida pela tinta no passo fino.
    let mut worst = (f32::MIN, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
    for tb in bounds {
        // O controlo vive no MEIO da geração anterior — mesmo regime, sem fronteira.
        let tc = (tb - 0.04).max(T0 + 0.01);
        let (ib0, kb0) = gap(tb, h0);
        let (ic0, kc0) = gap(tc, h0);
        let (ib1, kb1) = gap(tb, h1);
        let (ic1, kc1) = gap(tc, h1);
        let key = r(kb1, kc1);
        if key > worst.0 {
            worst = (key, tb, r(ib0, ic0), r(ib1, ic1), r(kb0, kc0), key);
        }
    }
    println!(
        "{:8} @t {:.3}  imagem: {:6.2}x -> {:6.2}x   tinta: {:8.2}x -> {:8.2}x",
        pr.label, worst.1, worst.2, worst.3, worst.4, worst.5,
    );
}

/// ⭐ **O TECTO da lei da escada** — o que uma escada de densidade `M` conseguiria, medido por
/// FORÇA BRUTA, antes de alguém a construir.
///
/// A bancada amostra o tamanho entregue em `g ∈ [1, G]` com passo `1/M`, inverte essa curva
/// para achar o `g` de cada `t`, e depois **mede o tamanho nesses `g`**. O desvio que sobra é o
/// que a lei com aquele `M` deixaria. *Uma cura provada por medição antes de existir.*
fn ideal(pr: &Preset, m: usize) {
    let gmax = pr.generations;
    let ov = [
        (param::ANGLE, pr.angle),
        (param::STEP, pr.step),
        (param::WIDTH, pr.width),
    ];
    let size = |g: f32| -> f32 {
        let s = probe_build(pr.axiom, pr.rules, g, &ov);
        match s.get("P") {
            Some(Column::Vec2(v)) => mean_width(v),
            _ => 0.0,
        }
    };
    // A escada de densidade `M`, em `g = 1 .. gmax`.
    let rungs = ((gmax - 1.0) * m as f32).round().max(1.0) as usize;
    let gs: Vec<f32> = (0..=rungs)
        .map(|i| 1.0 + (gmax - 1.0) * i as f32 / rungs as f32)
        .collect();
    let ss: Vec<f32> = gs.iter().map(|g| size(*g)).collect();
    let backs: Vec<String> = ss
        .windows(2)
        .enumerate()
        .filter(|(_, w)| w[1] <= w[0])
        .map(|(i, w)| {
            format!(
                "g={:.2}→{:.2} {:.4}→{:.4} ({:+.2}%)",
                gs[i],
                gs[i + 1],
                w[0],
                w[1],
                (w[1] / w[0] - 1.0) * 100.0
            )
        })
        .collect();
    if !backs.is_empty() {
        println!(
            "{:8} M={m}: ⛔ o tamanho ANDA PARA TRÁS em {} sítio(s): {}",
            pr.label,
            backs.len(),
            backs.join("  ")
        );
        return;
    }
    // Inverte: para cada `t`, o `g` cujo tamanho está na recta.
    let steps = 60usize;
    let mut got: Vec<f32> = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let alvo = ss[0] + t * (ss[rungs] - ss[0]);
        let mut k = 1;
        while k < rungs && ss[k] < alvo {
            k += 1;
        }
        let f = (alvo - ss[k - 1]) / (ss[k] - ss[k - 1]);
        got.push(size(gs[k - 1] + (gs[k] - gs[k - 1]) * f));
    }
    let (a, b) = (got[0], got[steps]);
    let worst = got
        .iter()
        .enumerate()
        .map(|(i, w)| (w - a) / (b - a) - i as f32 / steps as f32)
        .fold(0.0f32, |acc, d| if d.abs() > acc.abs() { d } else { acc });
    println!("{:8} M={m}: desvio máx {:+6.2} %", pr.label, worst * 100.0);
}

/// ⭐⭐⭐ **O ARRASTO É UMA RECTA? — a 2.ª pergunta, e é outra lei que a dos pulos.**
///
/// Report do Enio (2026-08-31, depois da lei do recém-nascido): *"está mais suave mas não é
/// perfeitamente linear"*. Suave e linear são coisas diferentes: a 1.ª é a AUSÊNCIA de degraus
/// (a derivada existe), a 2.ª é a derivada ser **CONSTANTE**.
///
/// ⚠️ A régua é o TAMANHO ao longo do `Growth`, contra a recta que une as duas pontas do
/// arrasto — e a coluna que decide é a **posição** do desvio: se ele se repete **uma vez por
/// geração**, a causa é a composição de duas leis, não ruído.
fn linear(pr: &Preset, trace: bool) {
    let steps = ((1.0 - T0) / STEP).round() as usize;
    let f: Vec<Frame> = (0..=steps)
        .map(|k| frame(pr, T0 + k as f32 * STEP))
        .collect();
    let w: Vec<f32> = f.iter().map(|x| mean_width(&x.p)).collect();
    let (a, b) = (w[0], w[steps]);
    if (b - a).abs() < 1e-6 {
        println!("{:8} (o arrasto não cresce)", pr.label);
        return;
    }
    // Normalizado: `0` no início do arrasto, `1` no fim. A recta é `k/steps`.
    let u: Vec<f32> = w.iter().map(|x| (x - a) / (b - a)).collect();
    let dev: Vec<f32> = u
        .iter()
        .enumerate()
        .map(|(k, v)| v - k as f32 / steps as f32)
        .collect();
    let mut worst = (0.0f32, 0usize);
    for (k, d) in dev.iter().enumerate() {
        if d.abs() > worst.0.abs() {
            worst = (*d, k);
        }
    }
    // O desvio no MEIO de cada geração — a assinatura de «duas leis compostas».
    let mut mids: Vec<(f32, f32)> = Vec::new();
    let mut k = 0usize;
    while k < steps {
        let n0 = f[k].n;
        let mut j = k + 1;
        while j <= steps && f[j].n == n0 {
            j += 1;
        }
        if j - k > 4 {
            let m = (k + j) / 2;
            mids.push((T0 + m as f32 * STEP, dev[m] * 100.0));
        }
        k = j;
    }
    if trace {
        println!("\n=== {} ===", pr.label);
        println!("     t      n   tamanho   fracção do arrasto   desvio da recta");
        for (k, x) in u.iter().enumerate() {
            println!(
                "{:6.3} {:6} {:9.4} {:18.4} {:17.4}",
                T0 + k as f32 * STEP,
                f[k].n,
                w[k],
                x,
                dev[k]
            );
        }
    }
    let mids_txt: Vec<String> = mids
        .iter()
        .map(|(t, d)| format!("t={t:.2}:{d:+.1}%"))
        .collect();
    println!(
        "{:8} desvio máx {:+6.2} % @t {:.3}   |  no meio de cada geração: {}",
        pr.label,
        worst.0 * 100.0,
        T0 + worst.1 as f32 * STEP,
        mids_txt.join("  "),
    );
}

/// **A VIRAGEM DA GERAÇÃO, nua** — o salto relativo da tinta ao atravessar um `G` inteiro, em
/// dois `ε`. É a forma que o gate `tests/newborn_law.rs` usa, e existe aqui para as barras dele
/// serem **lidas** e não escolhidas.
fn turn(pr: &Preset) {
    let g = pr.generations.clamp(2.0, 4.0);
    let ink_at = |x: f32| -> f32 {
        let s = probe_build(
            pr.axiom,
            pr.rules,
            x,
            &[
                (param::ANGLE, pr.angle),
                (param::STEP, pr.step),
                (param::WIDTH, pr.width),
            ],
        );
        match s.get("len") {
            Some(Column::Scalar(v)) => v.iter().sum(),
            _ => 0.0,
        }
    };
    let a = ink_at(g);
    let j = |eps: f32| {
        if a <= 1e-6 {
            0.0
        } else {
            (ink_at(g + eps) - a).abs() / a
        }
    };
    let (big, small) = (j(8e-3), j(1e-3));
    println!(
        "{:8} G={g:>4.1}  salto(8e-3) {big:.4}   salto(1e-3) {small:.4}   encolhe {:.3}x",
        pr.label,
        if big > 1e-9 { small / big } else { 0.0 },
    );
}

fn main() {
    let mut want: Vec<String> = std::env::args().skip(1).collect();
    if want.first().map(String::as_str) == Some("--ideal") {
        want.remove(0);
        println!("o TECTO de uma escada de densidade M, por força bruta\n");
        for p in PRESETS {
            if want.is_empty() || want.iter().any(|w| w == p.label) {
                for m in [1usize, 2, 3, 4, 6] {
                    ideal(p, m);
                }
            }
        }
        return;
    }
    if want.first().map(String::as_str) == Some("--linear") {
        want.remove(0);
        let trace = !want.is_empty();
        println!(
            "TAMANHO ao longo do arrasto do `Growth`, contra a recta que une as duas pontas\n\
             desvio em % do arrasto inteiro · + = a figura vai ADIANTADA · - = atrasada\n"
        );
        for p in PRESETS {
            if want.is_empty() || want.iter().any(|w| w == p.label) {
                linear(p, trace);
            }
        }
        return;
    }
    if want.first().map(String::as_str) == Some("--turn") {
        want.remove(0);
        println!(
            "salto relativo da TINTA ao virar a geração inteira G, em dois ε\n\
             movimento -> encolhe ~0,125x (o passo caiu 8x) · SALTO -> encolhe ~1,0x\n"
        );
        for p in PRESETS {
            if want.is_empty() || want.iter().any(|w| w == p.label) {
                turn(p);
            }
        }
        return;
    }
    if want.first().map(String::as_str) == Some("--refine") {
        want.remove(0);
        println!(
            "fronteira ÷ controlo (meio da geração), no passo {STEP} e depois em {}\n\
             MOVIMENTO -> a razão CAI ~4x (a diferença encolhe com o passo) · SALTO -> ela SOBE ~4x\n",
            STEP * 0.25
        );
        for p in PRESETS {
            if want.is_empty() || want.iter().any(|w| w == p.label) {
                refine(p);
            }
        }
        return;
    }
    println!(
        "arrasto do `Growth` de {T0} a 1,0 em passos de {STEP}\n\
         Δimg = fracção da tinta que MUDOU entre dois passos (Jaccard) · ondul = max ÷ mediana\n\
         ondul = 1,00x -> perfeitamente liso · fronteira = a contagem de elementos mudou nesse passo\n"
    );
    for p in PRESETS {
        if want.is_empty() || want.iter().any(|w| w == p.label) {
            sweep(p, !want.is_empty());
        }
    }
}
