//! **QUANTAS DIREÇÕES A RÉGUA PRECISA** — o número que o `WIDTH_DIRECTIONS` vai levar,
//! medido em vez de escolhido (§0.0).
//!
//! A pergunta é uma só: *rodando a MESMA figura, quanto a régua muda?* Uma régua perfeita
//! não muda nada. Roda-se o Dragon pelo `Root Angle` de `0` a `90°` e mede-se
//! `(máx − mín) / média` em %.

use ph2d_node_source_lsystem::{PRESETS, probe_build};
use ph2d_nodegraph::attr::Column;

/// A largura média sobre `k` direções — a mesma lei que o produto vai levar, com o `k` aberto.
fn mean_width(p: &ph2d_nodegraph::attr::Stream, k: usize) -> f32 {
    let Some(Column::Vec2(v)) = p.get("P") else {
        return 0.0;
    };
    if v.is_empty() {
        return 0.0;
    }
    let dirs: Vec<(f32, f32)> = (0..k)
        .map(|i| {
            let a = std::f32::consts::PI * i as f32 / k as f32;
            (a.cos(), a.sin())
        })
        .collect();
    let mut acc = 0.0f64;
    for (c, s) in &dirs {
        let mut lo = f32::MAX;
        let mut hi = f32::MIN;
        for q in v {
            let t = q[0] * c + q[1] * s;
            lo = lo.min(t);
            hi = hi.max(t);
        }
        acc += f64::from(hi - lo);
    }
    (acc / k as f64) as f32
}

fn axis_span(p: &ph2d_nodegraph::attr::Stream) -> f32 {
    let Some(Column::Vec2(v)) = p.get("P") else {
        return 0.0;
    };
    let x0 = v.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
    let x1 = v.iter().map(|q| q[0]).fold(f32::MIN, f32::max);
    let y0 = v.iter().map(|q| q[1]).fold(f32::MAX, f32::min);
    let y1 = v.iter().map(|q| q[1]).fold(f32::MIN, f32::max);
    (x1 - x0).max(y1 - y0)
}

fn ripple(vals: &[f32]) -> f32 {
    let hi = vals.iter().copied().fold(f32::MIN, f32::max);
    let lo = vals.iter().copied().fold(f32::MAX, f32::min);
    let mean = vals.iter().sum::<f32>() / vals.len() as f32;
    if mean > 0.0 {
        (hi - lo) / mean * 100.0
    } else {
        0.0
    }
}

fn main() {
    println!("ondulação da RÉGUA ao rodar a MESMA figura de 0 a 90° (0 % = invariante)\n");
    println!("molde      gens   eixo max(w,h)     K=2     K=4     K=8    K=16    K=32    K=64");
    for p in PRESETS {
        // A geração INTEIRA: aqui a lei do crescimento não corre, então o que varia é só a régua.
        let g = p.generations;
        let mut ax = vec![];
        let mut by_k: Vec<Vec<f32>> = vec![vec![]; 6];
        for deg in 0..=90 {
            let s = probe_build(
                p.axiom,
                p.rules,
                g,
                &[
                    ("angle", p.angle),
                    ("step", p.step),
                    ("width", p.width),
                    ("root_angle", 90.0 + deg as f32),
                ],
            );
            ax.push(axis_span(&s));
            for (i, k) in [2usize, 4, 8, 16, 32, 64].iter().enumerate() {
                by_k[i].push(mean_width(&s, *k));
            }
        }
        print!("{:8} {g:5.1}   {:9.2} %", p.label, ripple(&ax));
        for r in &by_k {
            print!("  {:5.2} %", ripple(r));
        }
        println!();
    }
    cost();
}

/// **O PREÇO** — a régua corre até 3 vezes por cozedura, e só numa geração fraccionária.
///
/// O pior caso é a maior cadeia que ainda cabe no orçamento com fracção: o Dragon a `16`
/// gerações são `262 143` módulos (`2^18 − 1`), e `17` já não cabe.
fn cost() {
    use std::time::Instant;
    let d = PRESETS.iter().find(|p| p.label == "Dragon").unwrap();
    let s = probe_build(
        d.axiom,
        d.rules,
        16.0,
        &[("angle", d.angle), ("step", d.step), ("width", d.width)],
    );
    let n = match s.get("P") {
        Some(Column::Vec2(v)) => v.len(),
        _ => 0,
    };
    println!("\nPREÇO da régua no pior caso — Dragon a 16 gerações, {n} elementos desenhados");
    println!("      K       uma passagem     3 por cozedura     % de um quadro de 16,7 ms");
    for k in [2usize, 4, 8, 16, 32, 64] {
        // Aquecer, e depois a MEDIANA de 5 — uma leitura só mede o cache frio.
        let mut ms = vec![];
        for _ in 0..5 {
            let t = Instant::now();
            std::hint::black_box(mean_width(&s, k));
            ms.push(t.elapsed().as_secs_f64() * 1e3);
        }
        ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let one = ms[2];
        println!(
            "{k:7}      {one:8.3} ms       {:8.3} ms          {:6.2} %",
            one * 3.0,
            one * 3.0 / 16.7 * 100.0
        );
    }
}
