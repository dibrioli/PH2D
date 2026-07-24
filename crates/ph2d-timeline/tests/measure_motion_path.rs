//! **Fatia 0 do [ADR-0141]** — quanto custa amostrar uma trajetória por comprimento de arco.
//!
//! É medição para DECIDIR, não regressão a proteger: o ADR declarou a barra *"o modo Path não
//! custa mais que 2× o Separate"* e mandou escrever o número **antes** de a barra valer (§0.0 da
//! casa: meça antes de limitar). O gate de perf nasce na Fatia 2, contra o que sair daqui.
//!
//! # A pergunta
//!
//! Em modo Path a track escalar mede **comprimento de arco** e a amostragem é
//! `ponto = caminho.em_arco(track.sample(t))`. O único custo NOVO é a **inversa**: dado `s`,
//! achar o `t` da cúbica. O motor que existe hoje ([`ph2d_vec_scene::arclen`]) a resolve por
//! **bisseção de 40 iterações**, e cada iteração chama `arclen_to` = Gauss-Legendre de 16 nós =
//! 32 avaliações de `|B'(t)|`, cada uma com um `sqrt`. **~1300 sqrt por amostra**, por entidade,
//! por frame.
//!
//! Este harness mede três estratégias contra a baseline escalar (o que uma entidade em modo
//! Separate custa hoje), e mede também o **erro** de cada uma — uma inversa mais barata que
//! aterrissa no lugar errado não é mais barata, é outra função.
//!
//! Rodar:
//! ```text
//! cargo test -p ph2d-timeline --release --test measure_motion_path -- --nocapture
//! ```
//!
//! [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md

use std::time::Instant;

use ph2d_anim::{AnimValue, AttributeEvaluator, Interp, Key, RationalTime, Track};
use ph2d_vec_scene::arclen::{Cubic, arclen, arclen_to, inv_arclen, point_at};

// ── a fixture: um caminho ondulado com N âncoras ────────────────────────────────────────────

/// Âncoras numa onda determinística (sem transcendental: a forma não muda o custo — GL16 e a
/// bisseção têm contagem FIXA de avaliações — mas uma fixture reta esconderia erro de inversa).
fn anchors(n: usize) -> Vec<[f64; 2]> {
    (0..n)
        .map(|i| {
            let x = i as f64 * 100.0;
            // triangular determinística, em vez de sin: a curvatura varia e nada é irracional.
            let phase = (i % 4) as f64;
            let y = (phase - 1.5).abs() * 80.0;
            [x, y]
        })
        .collect()
}

/// Cúbicas Auto-Bezier entre as âncoras (o default do AE): a tangente de cada âncora é paralela
/// à corda dos vizinhos, com um terço do comprimento de cada lado.
fn cubics(a: &[[f64; 2]]) -> Vec<Cubic> {
    let n = a.len();
    let tan = |i: usize| -> [f64; 2] {
        let p = a[i.saturating_sub(1)];
        let q = a[(i + 1).min(n - 1)];
        [(q[0] - p[0]) / 6.0, (q[1] - p[1]) / 6.0]
    };
    (0..n - 1)
        .map(|i| {
            let (t0, t1) = (tan(i), tan(i + 1));
            [
                a[i],
                [a[i][0] + t0[0], a[i][1] + t0[1]],
                [a[i + 1][0] - t1[0], a[i + 1][1] - t1[1]],
                a[i + 1],
            ]
        })
        .collect()
}

/// Prefixo somado dos comprimentos: `starts[i]` = arco onde o segmento `i` começa; a última
/// entrada é o total. É o que o `ArcPath` da linha Vector já faz — construir UMA vez.
fn prefix(cs: &[Cubic]) -> Vec<f64> {
    let mut v = Vec::with_capacity(cs.len() + 1);
    let mut acc = 0.0;
    v.push(0.0);
    for c in cs {
        acc += arclen(c);
        v.push(acc);
    }
    v
}

/// O segmento onde `s` cai (busca binária no prefixo).
fn seg_of(starts: &[f64], s: f64) -> usize {
    starts
        .partition_point(|&x| x <= s)
        .saturating_sub(1)
        .min(starts.len() - 2)
}

// ── A: a inversa de hoje (bisseção de 40 iterações) ─────────────────────────────────────────

fn sample_bisect(cs: &[Cubic], starts: &[f64], s: f64) -> [f64; 2] {
    let i = seg_of(starts, s);
    point_at(&cs[i], inv_arclen(&cs[i], s - starts[i]))
}

// ── B: Newton, usando `|B'(t)|` como derivada exata do comprimento ──────────────────────────

/// `ds/dt = |B'(t)|` — a derivada da função que estamos invertendo está DISPONÍVEL de graça,
/// que é exatamente a condição em que Newton bate bisseção. Palpite inicial linear em `s/total`.
fn inv_newton(c: &Cubic, s: f64, total: f64) -> f64 {
    inv_newton_counted(c, s, total, &mut 0)
}

/// Newton com PARADA por tolerância (e a contagem de iterações, que é o que decide se vale
/// gastar 4 fixas). `tol` é relativo ao comprimento do segmento: um erro de arco abaixo de
/// 1e-9 do segmento é indistinguível do da bisseção de 40 iterações.
fn inv_newton_counted(c: &Cubic, s: f64, total: f64, iters: &mut usize) -> f64 {
    if s <= 0.0 || total <= 0.0 {
        return 0.0;
    }
    if s >= total {
        return 1.0;
    }
    let tol = total * 1e-9;
    let mut t = s / total;
    for _ in 0..8 {
        *iters += 1;
        let err = arclen_to(c, t) - s;
        if err.abs() <= tol {
            break;
        }
        // |B'(t)|, recomputado aqui para não expor `speed` (que é privado na crate de origem).
        let u = 1.0 - t;
        let mut d = [0.0; 2];
        for k in 0..2 {
            let (p, q, r) = (c[1][k] - c[0][k], c[2][k] - c[1][k], c[3][k] - c[2][k]);
            d[k] = 3.0 * (p * u * u + 2.0 * q * u * t + r * t * t);
        }
        let sp = (d[0] * d[0] + d[1] * d[1]).sqrt();
        if sp <= 1e-12 {
            break;
        }
        t = (t - err / sp).clamp(0.0, 1.0);
    }
    t
}

fn sample_newton(cs: &[Cubic], starts: &[f64], s: f64) -> [f64; 2] {
    let i = seg_of(starts, s);
    let total = starts[i + 1] - starts[i];
    point_at(&cs[i], inv_newton(&cs[i], s - starts[i], total))
}

// ── C: LUT por segmento (K+1 amostras de t→s), construída uma vez ───────────────────────────

const LUT_K: usize = 16;

fn build_lut(cs: &[Cubic]) -> Vec<[f64; LUT_K + 1]> {
    cs.iter()
        .map(|c| {
            let mut row = [0.0; LUT_K + 1];
            for (k, slot) in row.iter_mut().enumerate() {
                *slot = arclen_to(c, k as f64 / LUT_K as f64);
            }
            row
        })
        .collect()
}

fn sample_lut(cs: &[Cubic], starts: &[f64], lut: &[[f64; LUT_K + 1]], s: f64) -> [f64; 2] {
    let i = seg_of(starts, s);
    let local = s - starts[i];
    let row = &lut[i];
    // busca linear curta na LUT (K = 16: cache-friendly, e o branch predictor ganha)
    let mut k = 0;
    while k + 1 < LUT_K && row[k + 1] < local {
        k += 1;
    }
    let (a, b) = (row[k], row[k + 1]);
    let frac = if b > a { (local - a) / (b - a) } else { 0.0 };
    let t = (k as f64 + frac) / LUT_K as f64;
    point_at(&cs[i], t)
}

// ── a baseline: o que uma entidade em modo SEPARATE custa hoje ──────────────────────────────

fn scalar_track(n: usize) -> Track {
    Track::new(
        (0..n)
            .map(|i| Key {
                t: RationalTime::from_seconds(i as f64 * 0.25),
                value: AnimValue::Float(i as f32 * 10.0),
                interp: Interp::Linear,
            })
            .collect(),
    )
}

// ── o harness ───────────────────────────────────────────────────────────────────────────────

const SAMPLES: usize = 20_000;

fn bench(label: &str, mut f: impl FnMut(f64) -> [f64; 2], total: f64) -> f64 {
    // aquece (o primeiro toque paga cache)
    let mut sink = 0.0;
    for i in 0..1000 {
        sink += f(total * i as f64 / 1000.0)[0];
    }
    let t0 = Instant::now();
    for i in 0..SAMPLES {
        sink += f(total * i as f64 / SAMPLES as f64)[0];
    }
    let ns = t0.elapsed().as_nanos() as f64 / SAMPLES as f64;
    assert!(sink.is_finite(), "{label}: a soma virou não-finita");
    ns
}

#[test]
fn measure_the_arc_length_inverse() {
    eprintln!("\n=== ADR-0141 Fatia 0 — custo de UMA amostra (release, ns) ===\n");

    // A baseline escalar: uma entidade em modo Separate paga DUAS (X e Y).
    let tr = scalar_track(8);
    let t0 = Instant::now();
    let mut sink = 0.0;
    for i in 0..SAMPLES {
        let t = i as f64 * 2.0 / SAMPLES as f64;
        if let AnimValue::Float(v) = tr.sample(t) {
            sink += v;
        }
    }
    let scalar_ns = t0.elapsed().as_nanos() as f64 / SAMPLES as f64;
    assert!(sink.is_finite());
    eprintln!("  baseline: 1 Track::sample          = {scalar_ns:8.1} ns");
    eprintln!(
        "            1 entidade Separate (X+Y) = {:8.1} ns",
        scalar_ns * 2.0
    );
    eprintln!(
        "            a BARRA declarada no ADR (2x) = {:8.1} ns por entidade Path\n",
        scalar_ns * 4.0
    );

    eprintln!("  âncoras │   bisseção │     Newton │        LUT │  err New │  err LUT │ it.New");
    eprintln!("  ────────┼────────────┼────────────┼────────────┼──────────┼──────────┼───────");

    for n in [2usize, 8, 32, 128] {
        let a = anchors(n);
        let cs = cubics(&a);
        let st = prefix(&cs);
        let lut = build_lut(&cs);
        let total = *st.last().unwrap();

        let ns_b = bench("bisect", |s| sample_bisect(&cs, &st, s), total);
        let ns_n = bench("newton", |s| sample_newton(&cs, &st, s), total);
        let ns_l = bench("lut", |s| sample_lut(&cs, &st, &lut, s), total);

        // Quantas iterações o Newton de fato gasta (é o que diz se 4 fixas eram desperdício).
        let (mut it, mut calls) = (0usize, 0usize);
        for i in 0..2000 {
            let s = total * i as f64 / 2000.0;
            let j = seg_of(&st, s);
            inv_newton_counted(&cs[j], s - st[j], st[j + 1] - st[j], &mut it);
            calls += 1;
        }
        let avg_it = it as f64 / calls as f64;

        // erro: a bisseção é a REFERÊNCIA (é a que já shipou); as outras são medidas contra ela.
        let (mut e_n, mut e_l) = (0.0f64, 0.0f64);
        for i in 0..2000 {
            let s = total * i as f64 / 2000.0;
            let r = sample_bisect(&cs, &st, s);
            let dn = sample_newton(&cs, &st, s);
            let dl = sample_lut(&cs, &st, &lut, s);
            e_n = e_n.max((dn[0] - r[0]).hypot(dn[1] - r[1]));
            e_l = e_l.max((dl[0] - r[0]).hypot(dl[1] - r[1]));
        }
        eprintln!(
            "  {n:7} │ {ns_b:7.1} ns │ {ns_n:7.1} ns │ {ns_l:7.1} ns │ {e_n:8.2e} │ {e_l:8.2e} │ {avg_it:6.2}"
        );
    }

    eprintln!(
        "\n  (erro em unidades de MUNDO; o caminho da fixture mede ~{:.0} unidades)",
        {
            let a = anchors(8);
            let cs = cubics(&a);
            *prefix(&cs).last().unwrap()
        }
    );
    eprintln!();
}
