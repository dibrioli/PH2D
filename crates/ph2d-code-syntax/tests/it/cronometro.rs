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

/// De quem é o reparse incremental que cresce com o ficheiro (E14: `0.02` → `0.58` ms de p50)?
/// O MESMO reparse, intercalado tecla a tecla: o nosso (`Syntax::edit`, a rope lida por pedaços)
/// contra o tree-sitter sozinho sobre um `&str` contíguo (a cópia para `String` fica fora do
/// relógio). Corra com `--profile smoke -- --ignored --nocapture reparse_rope_contra_str`.
#[test]
#[ignore = "medição: corra à mão no perfil smoke"]
fn reparse_rope_contra_str() {
    use ph2d_code_buffer::Point;
    use tree_sitter::{InputEdit, Parser};
    let ts_point = |p: Point| tree_sitter::Point {
        row: p.row,
        column: p.col,
    };
    let mut linhas = Vec::new();
    // O `bob.luau` sem o cabeçalho de 2 linhas da fixtura (o da bancada E14): juntado até 7 500
    // linhas acaba em comentário (texto válido). Com o cabeçalho, 7 500 linhas acabam a meio de uma
    // função: um bloco aberto no fim do ficheiro (texto quebrado).
    let bob: String = BOB.lines().skip(2).map(|l| format!("{l}\n")).collect();
    // A bancada comenta as `ph2d.property` das cópias depois da 1.ª (o motor recusa a repetida).
    let variantes = [
        ("51 linhas", bob.as_str(), 51, false),
        ("7 500 linhas", bob.as_str(), 7_500, false),
        (
            "7 500 linhas, propriedades comentadas",
            bob.as_str(),
            7_500,
            true,
        ),
        ("7 500 linhas, bloco aberto no fim", BOB, 7_500, false),
    ];
    for (nome, base, n, comenta) in variantes {
        let por = base.lines().count();
        let src: String = base
            .lines()
            .cycle()
            .take(n)
            .enumerate()
            .map(|(i, l)| {
                if comenta && i >= por && l.starts_with("ph2d.property(") {
                    format!("-- {l}\n")
                } else {
                    format!("{l}\n")
                }
            })
            .collect();
        let mut b = Buffer::new(&src);
        let mut syn = Syntax::new(b.text());
        let mut p = Parser::new();
        p.set_language(&tree_sitter_luau::LANGUAGE.into())
            .expect("luau");
        let mut tree = p.parse(&src, None).expect("árvore");
        // A linha do `self.t = self.t + dt` (33, 1-based, com o cabeçalho) na cópia do meio: o
        // texto fica válido a cada tecla (o da bancada E14).
        let alvo = base
            .lines()
            .position(|l| l == "  self.t = self.t + dt")
            .expect("alvo")
            + por * (n / por / 2);
        let at = b.text().line_end(alvo);
        b.set_selection(Selection::single(Range::cursor(at)));
        let (mut nosso, mut so_ts) = (Vec::new(), Vec::new());
        for i in 0..400 {
            let edits = if i % 10 < 5 {
                b.insert("x")
            } else {
                b.delete_backward()
            };
            let s = b.text().to_string();
            for e in edits.iter() {
                tree.edit(&InputEdit {
                    start_byte: e.start_byte,
                    old_end_byte: e.old_end_byte,
                    new_end_byte: e.new_end_byte,
                    start_position: ts_point(e.start),
                    old_end_position: ts_point(e.old_end),
                    new_end_position: ts_point(e.new_end),
                });
            }
            // Ordem rodada a cada amostra.
            let mut corre = |quem: usize| {
                let t = Instant::now();
                if quem == 0 {
                    syn.edit(&edits, b.text());
                    nosso.push(t.elapsed().as_secs_f64() * 1e3);
                } else {
                    tree = p.parse(&s, Some(&tree)).expect("árvore");
                    so_ts.push(t.elapsed().as_secs_f64() * 1e3);
                }
            };
            corre(i % 2);
            corre(1 - i % 2);
        }
        assert_eq!(
            syn.sexp(),
            tree.root_node().to_sexp(),
            "as duas árvores são a mesma"
        );
        for v in [&mut nosso, &mut so_ts] {
            v.sort_by(f64::total_cmp);
        }
        let r = |v: &[f64]| {
            let q = |x: f64| v[((v.len() - 1) as f64 * x) as usize];
            format!(
                "{:.3} / {:.3} / {:.3} / {:.3}",
                v[0],
                q(0.5),
                q(0.99),
                q(0.999)
            )
        };
        let erro = syn.tree().root_node().has_error();
        linhas.push(format!(
            "| {nome} (erro: {erro}) | {} | {} |",
            r(&nosso),
            r(&so_ts)
        ));
    }
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "\nreparse incremental por tecla, ms (mín / p50 / p99 / p99.9); loadavg {}",
        load.trim()
    );
    println!("| documento | `Syntax::edit` (rope por pedaços) | tree-sitter sobre `&str` |");
    println!("|---|---|---|");
    for l in linhas {
        println!("{l}");
    }
}
