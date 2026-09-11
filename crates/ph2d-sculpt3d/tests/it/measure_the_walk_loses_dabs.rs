//! **ONDE O PERCURSO PERDE UM CARIMBO** — a sonda do defeito que a
//! `measure_stroke_ripple` §3 mediu e não atribuiu.
//!
//! O MESMO caminho entregue em mais eventos deposita MENOS dabs — `40, 40, 39,
//! 39, 39, 38, 35` — e a altura média cai **26%**. É a lei que esta casa já
//! pagou quatro vezes no relevo do Painter: *o traço é fato do CAMINHO, nunca de
//! quão fino o motor amostrou o caminho*.
//!
//! ⚠️ **A aritmética diz que isto não devia acontecer**, e é por isso que a
//! sonda existe. Sobre uma RETA, com a âncora no último dab:
//!
//! ```text
//! n_k = n_{k-1} + floor((b_k − n_{k-1}·ms) / ms) = floor(b_k / ms)
//! ```
//!
//! — por indução, a contagem depende só de ONDE o caminho termina, e as
//! fronteiras dos eventos **cancelam**. O carry é exato por construção. Logo o
//! deficit vem de outra coisa, e a sonda mede QUAL antes de qualquer conserto.
//!
//! Rode com `-- --ignored --nocapture`.

use ph2d_sculpt3d::{min_spacing, walk};

/// O percurso REAL, com as fronteiras do `measure_stroke_ripple` (LCG idêntico).
fn boundaries(events: usize, len: f64) -> Vec<f64> {
    let mut st: u64 = 0x2545_F491_4F6C_DD1D;
    let mut w: Vec<f64> = (0..events)
        .map(|_| {
            st = st
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            0.5 + (st >> 40) as f64 / f64::from(1u32 << 24)
        })
        .collect();
    let total: f64 = w.iter().sum();
    let mut acc = 0.0;
    for x in &mut w {
        acc += *x / total * len;
        *x = acc;
    }
    if let Some(l) = w.last_mut() {
        *l = len;
    }
    w
}

/// Percorre em `f32` — o produto — e devolve `(dabs, âncora final, carries)`.
fn walk_f32(events: usize, len: f64, ms: f32) -> (usize, f64, usize) {
    let mut anchor = [0.0f32, 0.0];
    let (mut n, mut carries) = (0usize, 0usize);
    for bnd in boundaries(events, len) {
        let to = [bnd as f32, 0.0];
        match walk(anchor, to, ms) {
            Some(steps) => {
                n += steps.len() as usize;
                anchor = steps.anchor();
            }
            None => carries += 1,
        }
    }
    (n, f64::from(anchor[0]), carries)
}

/// A MESMA lei em `f64` puro — o oráculo aritmético, sem `f32` no meio.
fn walk_exact(events: usize, len: f64, ms: f64) -> (usize, f64) {
    let mut anchor = 0.0f64;
    let mut n = 0usize;
    for bnd in boundaries(events, len) {
        let dist = bnd - anchor;
        if !dist.is_finite() || dist <= ms {
            continue;
        }
        let steps = (dist / ms).floor() as usize;
        n += steps;
        anchor += steps as f64 * ms;
    }
    (n, anchor)
}

#[test]
#[ignore = "sonda: imprime, não afirma"]
fn measure_the_walk_loses_dabs() {
    // Os números do produto: raio do pincel 0,30 na cena, arco de 1,8 rad numa
    // esfera de raio ~1 ⇒ o caminho mede quase EXACTAMENTE 40 espaçamentos.
    let ms32 = min_spacing(0.30);
    let ms = f64::from(ms32);
    for &len in &[1.8f64, 1.8 * 1.013, 1.7] {
        println!(
            "\n=== caminho {len:.6} · espaçamento {ms:.8} · \
             len/ms = {:.6} ({} dabs se a lei valer) ===\n",
            len / ms,
            (len / ms).floor() as usize
        );
        println!(
            "  {:>8}  {:>6}  {:>6}  {:>8}  {:>14}  {:>12}",
            "eventos", "f32", "f64", "carries", "âncora f32", "deriva"
        );
        for &events in &[1usize, 2, 3, 5, 8, 20, 100, 400] {
            let (n32, a32, carries) = walk_f32(events, len, ms32);
            let (n64, a64) = walk_exact(events, len, ms);
            println!(
                "  {events:>8}  {n32:>6}  {n64:>6}  {carries:>8}  {a32:>14.9}  {:>12.3e}",
                a32 - a64
            );
        }
    }
}
