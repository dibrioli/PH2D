//! Micro-cronómetro (medição, sem limite): reparse incremental por tecla num ficheiro de 7 500
//! linhas. Corra com `--profile smoke -- --ignored --nocapture reparse_por_tecla`.

use crate::BOB;
use ph2d_code_buffer::{Buffer, Range, Selection};
use ph2d_code_syntax::Syntax;
use std::time::Instant;

#[test]
#[ignore = "medição: corra à mão no perfil smoke"]
fn reparse_por_tecla() {
    let mut src = String::new();
    while src.lines().count() < 7_500 {
        src.push_str(BOB);
    }
    let src: String = src.lines().take(7_500).map(|l| format!("{l}\n")).collect();
    let mut b = Buffer::new(&src);
    let t0 = Instant::now();
    let mut syn = Syntax::new(b.text());
    let inteiro = t0.elapsed();
    let mut rng = fastrand::Rng::with_seed(42);
    let mut us: Vec<f64> = Vec::new();
    // O parse do ZERO do mesmo texto, intercalado tecla a tecla (o custo de não ser incremental).
    let mut zero: Vec<f64> = Vec::new();
    // Cada rodada: um sítio ao acaso, uma palavra escrita tecla a tecla e apagada tecla a tecla.
    for _ in 0..200 {
        let line = rng.usize(0..7_500);
        let at = b.text().line_end(line);
        b.set_selection(Selection::single(Range::cursor(at)));
        for k in [" ", "x", "=", "(", "1", ")", "\n"] {
            let edits = b.insert(k);
            let t = Instant::now();
            syn.edit(&edits, b.text());
            us.push(t.elapsed().as_secs_f64() * 1e6);
            let t = Instant::now();
            std::hint::black_box(Syntax::new(b.text()));
            zero.push(t.elapsed().as_secs_f64() * 1e6);
        }
        for _ in 0..7 {
            let edits = b.delete_backward();
            let t = Instant::now();
            syn.edit(&edits, b.text());
            us.push(t.elapsed().as_secs_f64() * 1e6);
        }
    }
    us.sort_by(f64::total_cmp);
    zero.sort_by(f64::total_cmp);
    let p = |q: f64| us[((us.len() - 1) as f64 * q) as usize];
    let pz = |q: f64| zero[((zero.len() - 1) as f64 * q) as usize];
    println!(
        "parse do zero, {} amostras: min {:.1} µs · p50 {:.1} µs · p99 {:.1} µs",
        zero.len(),
        zero[0],
        pz(0.5),
        pz(0.99)
    );
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "reparse_por_tecla: {} linhas, {} bytes, parse inteiro {:.0} µs; {} teclas: min {:.1} µs · p50 {:.1} µs · p99 {:.1} µs · max {:.1} µs; loadavg {}",
        b.text().line_count(),
        b.text().len(),
        inteiro.as_secs_f64() * 1e6,
        us.len(),
        us[0],
        p(0.5),
        p(0.99),
        us[us.len() - 1],
        load.trim()
    );
    assert_eq!(syn.sexp(), Syntax::new(b.text()).sexp());
}
