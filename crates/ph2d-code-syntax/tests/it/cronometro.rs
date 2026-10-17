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

/// Que FORMA do topo do ficheiro encarece o reparse por tecla (E14: `0.38` → `2.1` ms com ~1 000
/// `ph2d.property(…)` vivas)? Um factor de cada vez, todas as variantes intercaladas no mesmo
/// processo; fora do relógio, o diário do tree-sitter conta o que o reparse refez. Relatório:
/// `docs/CodeEditor/ferramentas/medicoes_onda2/I0a_reparse_topo.md`. Corra com
/// `--profile smoke -- --ignored --nocapture reparse_por_forma_do_topo`.
#[test]
#[ignore = "medição: corra à mão no perfil smoke"]
fn reparse_por_forma_do_topo() {
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};
    use tree_sitter::{InputEdit, LogType, Parser};

    const ALVO: &str = "function alvo(self, dt)\n  self.t = self.t + dt\nend\n";
    const ABERTO: &str = "function aberto(self)\n  local z = 1\n";
    let forma = |tipo: &str, k: usize| match tipo {
        "ph2d.property" => format!("ph2d.property(\"p{k}\", 1)\n"),
        "foo(K)" => format!("foo({k})\n"),
        "local _ = foo(K)" => format!("local _ = foo({k})\n"),
        "local pK = 1" => format!("local p{k} = 1\n"),
        "x.pK = 1" => format!("x.p{k} = 1\n"),
        "pK = 1" => format!("p{k} = 1\n"),
        "-- comentário" => format!("-- comentario {k}\n"),
        "function (controle)" => format!("function f{k}(self)\n  self.t = {k}\nend\n"),
        _ => unreachable!(),
    };
    // (nome, texto)
    let mut variantes: Vec<(String, String)> = Vec::new();
    let tipos = [
        "ph2d.property",
        "foo(K)",
        "local _ = foo(K)",
        "local pK = 1",
        "x.pK = 1",
        "pK = 1",
        "-- comentário",
        "function (controle)",
    ];
    for n in [100, 1_000, 3_000] {
        for t in tipos {
            let corpo: String = (0..n).map(|k| forma(t, k)).collect();
            variantes.push((format!("{n} × `{t}`, tecla no fim"), corpo + ALVO));
        }
    }
    for t in ["ph2d.property", "function (controle)"] {
        let corpo: String = (0..1_000).map(|k| forma(t, k)).collect();
        variantes.push((
            format!("1000 × `{t}`, tecla no início"),
            format!("{ALVO}{corpo}"),
        ));
        variantes.push((
            format!("1000 × `{t}`, bloco aberto no fim"),
            format!("{corpo}{ALVO}{ABERTO}"),
        ));
    }

    let ts_point = |p: ph2d_code_buffer::Point| tree_sitter::Point {
        row: p.row,
        column: p.col,
    };
    let input_edit = |e: &ph2d_code_buffer::TextEdit| InputEdit {
        start_byte: e.start_byte,
        old_end_byte: e.old_end_byte,
        new_end_byte: e.new_end_byte,
        start_position: ts_point(e.start),
        old_end_position: ts_point(e.old_end),
        new_end_position: ts_point(e.new_end),
    };
    let cursor_no_alvo = |b: &mut Buffer, src: &str| {
        let linha = src
            .lines()
            .position(|l| l == "  self.t = self.t + dt")
            .expect("alvo");
        let at = b.text().line_end(linha);
        b.set_selection(Selection::single(Range::cursor(at)));
    };
    // Uma tecla do traço: 5 letras escritas, 5 apagadas (o texto volta ao de partida a cada 10).
    let tecla = |b: &mut Buffer, i: usize| {
        if i % 10 < 5 {
            b.insert("x")
        } else {
            b.delete_backward()
        }
    };

    // 1) O mecanismo, fora do relógio: 10 teclas por variante com o diário ligado.
    let diario: Arc<Mutex<BTreeMap<String, usize>>> = Arc::default();
    let reusados: Arc<Mutex<BTreeMap<String, usize>>> = Arc::default();
    let mut p = Parser::new();
    p.set_language(&tree_sitter_luau::LANGUAGE.into())
        .expect("luau");
    let mut mecanismo = Vec::new();
    for (nome, src) in &variantes {
        let mut b = Buffer::new(src);
        cursor_no_alvo(&mut b, src);
        p.set_logger(None);
        let mut tree = p.parse(src, None).expect("árvore");
        let raiz = tree.root_node();
        let (filhos, nos, erro) = (
            raiz.child_count(),
            raiz.descendant_count(),
            raiz.has_error(),
        );
        diario.lock().unwrap().clear();
        reusados.lock().unwrap().clear();
        let (d, r) = (diario.clone(), reusados.clone());
        p.set_logger(Some(Box::new(move |tipo, msg: &str| {
            let chave = if tipo == LogType::Lex {
                msg.split([' ', ':']).next().unwrap_or("").to_string()
            } else if let Some(v) = msg.strip_prefix("process version:") {
                let multi = v
                    .split("version_count:")
                    .nth(1)
                    .and_then(|s| s.split(',').next())
                    .is_some_and(|c| c.trim() != "1");
                (if multi { "process_glr" } else { "process" }).to_string()
            } else {
                let sim = if let Some(s) = msg.strip_prefix("reuse_node symbol:") {
                    Some(format!("reuso:{s}"))
                } else if let Some(s) = msg.strip_prefix("cant_reuse_node_is_fragile tree:") {
                    Some(format!("frágil:{s}"))
                } else {
                    msg.split_once("first_leaf_symbol:")
                        .map(|(_, s)| format!("folha:{s}"))
                };
                if let Some(s) = sim {
                    *r.lock().unwrap().entry(s).or_default() += 1;
                }
                msg.split([' ', ':', ',']).next().unwrap_or("").to_string()
            };
            *d.lock().unwrap().entry(chave).or_default() += 1;
        })));
        let mut mudadas = 0;
        for i in 0..10 {
            let edits = tecla(&mut b, i);
            for e in edits.iter() {
                tree.edit(&input_edit(e));
            }
            let s = b.text().to_string();
            let novo = p.parse(&s, Some(&tree)).expect("árvore");
            mudadas += tree.changed_ranges(&novo).len();
            tree = novo;
        }
        p.set_logger(None);
        let d = diario.lock().unwrap();
        let g = |k: &str| *d.get(k).unwrap_or(&0) as f64 / 10.0;
        let hist = reusados.lock().unwrap().clone();
        let top = |pre: &str| {
            let mut v: Vec<_> = hist
                .iter()
                .filter(|(_, c)| **c >= 5)
                .filter_map(|(s, c)| s.strip_prefix(pre).map(|s| (s, *c)))
                .collect();
            v.sort_by_key(|x| std::cmp::Reverse(x.1));
            v.iter()
                .take(2)
                .map(|(s, c)| format!("`{s}` {:.0}", *c as f64 / 10.0))
                .collect::<Vec<_>>()
                .join(" · ")
        };
        mecanismo.push(format!(
            "| {nome} | {} | {} / {} / {} | {:.0} | {:.0} | {:.0} | {:.0} | {} | {:.0} | {} | {:.0} | {:.0} | {:.0} | {:.0} | {:.1} | {} |",
            src.lines().count(),
            filhos,
            nos,
            erro,
            g("lexed_lookahead"),
            g("consume") + g("skip"),
            g("reuse_node"),
            g("cant_reuse_node_is_fragile"),
            top("frágil:"),
            g("cant_reuse_node"),
            top("folha:"),
            g("cant_reuse_node_has_changes"),
            g("process_glr"),
            g("shift"),
            g("reduce"),
            mudadas as f64 / 10.0,
            top("reuso:")
        ));
    }

    // 2) O relógio: `Syntax::edit` (o caminho do produto), blocos de 10 teclas por variante,
    // ordem rodada a cada rodada.
    let load0 = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let t_rodada = Instant::now();
    let mut estado: Vec<(Buffer, Syntax, Vec<f64>)> = variantes
        .iter()
        .map(|(_, src)| {
            let mut b = Buffer::new(src);
            cursor_no_alvo(&mut b, src);
            let syn = Syntax::new(b.text());
            (b, syn, Vec::new())
        })
        .collect();
    let rodadas = 80;
    let v = estado.len();
    for r in 0..rodadas {
        for j in 0..v {
            let (b, syn, ms) = &mut estado[(r + j) % v];
            for i in 0..10 {
                let edits = tecla(b, i);
                let t = Instant::now();
                syn.edit(&edits, b.text());
                ms.push(t.elapsed().as_secs_f64() * 1e3);
            }
        }
    }
    let duracao = t_rodada.elapsed();
    let load1 = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let mut tabela = Vec::new();
    for ((nome, _), (b, syn, ms)) in variantes.iter().zip(estado.iter_mut()) {
        assert_eq!(syn.sexp(), Syntax::new(b.text()).sexp(), "{nome}");
        ms.sort_by(f64::total_cmp);
        let q = |x: f64| ms[((ms.len() - 1) as f64 * x) as usize];
        tabela.push(format!(
            "| {nome} | {:.3} / {:.3} / {:.3} / {:.3} |",
            ms[0],
            q(0.5),
            q(0.99),
            q(0.999)
        ));
    }
    println!(
        "\nreparse por tecla (`Syntax::edit`), ms (mín / p50 / p99 / p99.9); {rodadas} rodadas × 10 teclas = {} por célula; rodada inteira {:.1} s; loadavg início {} · fim {}",
        rodadas * 10,
        duracao.as_secs_f64(),
        load0.trim(),
        load1.trim()
    );
    println!("| topo do ficheiro | ms |");
    println!("|---|---|");
    for l in tabela {
        println!("{l}");
    }
    println!(
        "\nmecanismo (diário do tree-sitter, média por tecla sobre 10 teclas, fora do relógio)"
    );
    println!(
        "| topo do ficheiro | linhas | filhos da raiz / nós / erro | tokens lexados | caracteres lidos pelo lexer | nós reusados | recusa: frágil | frágeis (símbolo n) | recusa: 1.ª folha | 1.ª folha recusada (símbolo n) | recusa: mudou | passos com >1 versão (GLR) | shift | reduce | changed_ranges | reusados (símbolo n) |"
    );
    println!("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|");
    for l in mecanismo {
        println!("{l}");
    }
}
