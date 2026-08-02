//! **O QUE UM CTRL+Z CUSTA, e em qual das duas metades** — irmão de
//! [`super::measure_commit_cost`] (o pen-up) pela mesma linha de corte: uma coisa é *gravar* a
//! história, outra é *voltar* nela.
//!
//! # Por que esta sonda existe
//!
//! O smoke do S3 (2026-08-01) voltou **aprovado no comportamento** — a tinta e o relevo voltam iguais —
//! com um `[paint-perf]` trazendo `dispatch max = 71,2 ms`, **100% em `preview`**, num quadro
//! `branch=idle` a 4096² com `impasto=true`. Essa é a assinatura de um **re-fold do canvas inteiro**, e
//! um Ctrl+Z é exatamente o gesto que o força: ele reinstala os três planos de relevo, e o passe de luz
//! tem de reler a pintura toda.
//!
//! ⚠️ **A pergunta não é *"quanto custa"*, é *"a elisão MOVEU isso?"***, e ela só se responde
//! ablacionando **pela ENTRADA** (`UndoController::elide_relief` / `elide_cursor`), **costas-com-costas
//! na MESMA corrida** — a máquina é compartilhada e o mesmo número do produto já variou 14,5–30,2 ms
//! sem uma linha mudar (doc 28 §5.46). Um A/B cross-run atribuiria a deriva da máquina ao ganho.
//!
//! # As DUAS metades, e por que medi-las separadas
//!
//! O `71,2` está no **QUADRO**, não na chamada de undo — então somar as duas esconderia qual delas
//! move. A sonda cronometra:
//!
//! * **`undo`** — só [`PainterTool::undo_last`]: materializar o delta e instalar o modelo.
//! * **`+frame`** — o `paint_tick` + o dreno do preview que vêm DEPOIS, onde o fold do relevo e a luz
//!   repintam o canvas. É este o balde onde o `preview` do log vive.
//!
//! Rodar:
//! ```text
//! cargo test -p ph2d-tool-painter --release measure_ -- --ignored --nocapture --test-threads=1
//! ```

use super::measure_stroke_owners::{armed, stroke};
use super::*;

/// **O Ctrl+Z com e sem a elisão, na mesma corrida** — a atribuição do outlier do smoke.
///
/// ⚠️ **A ablação PROVA A SI MESMA:** a contagem de donos do plano de altura vai ao lado do relógio, e
/// o braço "os DOIS" tem de ter MENOS donos que o "nenhum". Sem esse controle os dois braços poderiam
/// estar medindo o mesmo caminho duas vezes — o verde-por-vácuo do ADR-0120, num relógio.
///
/// ⚠️ **O tool é reusado entre as repetições de propósito.** Um tool novo por amostra faz de todo undo
/// o primeiro da camada, e o primeiro paga a alocação preguiçosa dos três planos (192 MB a 4096²):
/// mediria a estreia repetidamente em vez do regime que o artista vive. O 1º é descartado.
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn what_a_ctrl_z_costs_with_and_without_the_elision() {
    use std::time::Instant;

    fn owners(t: &PainterTool) -> usize {
        t.layers
            .active()
            .and_then(|a| t.heights.get(&a).map(std::sync::Arc::strong_count))
            .unwrap_or(0)
    }

    /// `(undo ms, +frame ms, redo ms, +frame ms, donos)` — mediana de 7 ciclos, o 1º descartado.
    fn cycle(side: u32, elide_before: bool, elide_cursor: bool) -> (f64, f64, f64, f64, usize) {
        let mut t = armed(side);
        t.undo.elide_relief = elide_before;
        t.undo.elide_cursor = elide_cursor;
        // Três traços: o 1º instala o histórico, e sobram passos para desfazer sem esvaziar a fila.
        for k in 0..3u8 {
            stroke(&mut t, 200.0 + f32::from(k) * 40.0);
        }
        let _ = t.take_preview_arc();

        let (mut un, mut unf, mut re, mut ref_) = (vec![], vec![], vec![], vec![]);
        let mut own = 0;
        for k in 0..8u8 {
            own = owners(&t);

            let t0 = Instant::now();
            let did_undo = t.undo_last();
            let a = t0.elapsed().as_secs_f64() * 1e3;
            let t1 = Instant::now();
            t.paint_tick(1.0 / 60.0);
            let _ = t.take_preview_arc();
            let b = t1.elapsed().as_secs_f64() * 1e3;

            let t2 = Instant::now();
            let did_redo = t.redo_last();
            let c = t2.elapsed().as_secs_f64() * 1e3;
            let t3 = Instant::now();
            t.paint_tick(1.0 / 60.0);
            let _ = t.take_preview_arc();
            let d = t3.elapsed().as_secs_f64() * 1e3;

            // ⚠️ Controle: uma fila vazia devolve `false` e a sonda mediria o custo de NÃO desfazer —
            // zero, e indistinguível de um ganho enorme.
            assert!(
                did_undo && did_redo,
                "a fixture nao contem o fenomeno: undo={did_undo} redo={did_redo} (fila vazia?)"
            );
            if k > 0 {
                un.push(a);
                unf.push(b);
                re.push(c);
                ref_.push(d);
            }
        }
        let med = |v: &mut Vec<f64>| {
            v.sort_by(f64::total_cmp);
            v[v.len() / 2]
        };
        (
            med(&mut un),
            med(&mut unf),
            med(&mut re),
            med(&mut ref_),
            own,
        )
    }

    println!(
        "\n{:<6} {:<14} {:>9} {:>9} {:>9} {:>9} {:>7}",
        "tela", "elide", "undo", "+frame", "redo", "+frame", "donos"
    );
    for side in [2048u32, 4096] {
        let mut base_owners = 0;
        for (tag, b, c) in [
            ("nenhum", false, false),
            ("so o BEFORE", true, false),
            ("so o CURSOR", false, true),
            ("os DOIS", true, true),
        ] {
            let (u, uf, r, rf, o) = cycle(side, b, c);
            println!("{side:<6} {tag:<14} {u:>9.2} {uf:>9.2} {r:>9.2} {rf:>9.2} {o:>7}");
            if tag == "nenhum" {
                base_owners = o;
            } else if tag == "os DOIS" {
                assert!(
                    base_owners > o,
                    "controle: a ablacao nao mudou a contagem de donos ({base_owners} vs {o}) — os \
                     dois bracos medem o MESMO caminho, e a razao seria verde por vacuo"
                );
            }
        }
        println!();
    }
}

/// **DE QUE é feito o quadro depois do Ctrl+Z** — a atribuição, antes de qualquer conserto.
///
/// ⚠️ **381 ms é grande demais para o composite de UMA camada:** o fold do relevo custa 14,55 ms a
/// 4096² desde que virou paralelo (doc 28 §4.8.2). Somar as peças de cabeça diria *"é o fold"* e
/// mandaria a wave para o lugar errado — a lição da §5.13, que atribuiu um pen-up a um fork porque a
/// aritmética fechava **por coincidência**.
///
/// Ablação pela ENTRADA em dois eixos que se cruzam:
///
/// * **impasto ON/OFF** — separa *o fold + a luz* do *composite*. O flag é do pincel, não do laço.
/// * **quadro depois do UNDO × quadro NO MEIO de um traço** — o segundo passa pela pista PARCIAL
///   (`composited` quente + `dirty_rect` da pegada), e é o **limite superior do prêmio**: é o que o
///   quadro custaria se o undo publicasse a janela que ele reescreveu.
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn what_the_frame_after_a_ctrl_z_is_made_of() {
    use std::time::Instant;

    /// `(quadro pos-undo, quadro no MEIO de um traco)` em ms.
    fn split(side: u32, impasto: bool) -> (f64, f64) {
        let mut t = armed(side);
        if !impasto {
            t.paint.brush.impasto = false;
            for slot in &mut t.paint.brush_by_mode {
                slot.impasto = false;
            }
        }
        for k in 0..3u8 {
            stroke(&mut t, 200.0 + f32::from(k) * 40.0);
        }
        let _ = t.take_preview_arc();

        // (a) o quadro depois de um Ctrl+Z — a pista CHEIA
        let mut post = Vec::new();
        for _ in 0..6 {
            assert!(t.undo_last(), "a fixture nao contem o fenomeno: fila vazia");
            let t0 = Instant::now();
            t.paint_tick(1.0 / 60.0);
            let _ = t.take_preview_arc();
            post.push(t0.elapsed().as_secs_f64() * 1e3);
            assert!(
                t.redo_last(),
                "a fixture nao contem o fenomeno: nada a refazer"
            );
            t.paint_tick(1.0 / 60.0);
            let _ = t.take_preview_arc();
        }

        // (b) o quadro NO MEIO de um traço — a pista PARCIAL, o limite superior do prêmio
        let mut mid = Vec::new();
        t.on_canvas_pointer(cp([60.0, 400.0], PointerPhase::Down));
        for k in 1..=6u8 {
            t.on_canvas_pointer(cp([60.0 + f32::from(k) * 30.0, 400.0], PointerPhase::Move));
            let t0 = Instant::now();
            t.paint_tick(1.0 / 60.0);
            let _ = t.take_preview_arc();
            mid.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        t.on_canvas_pointer(cp([260.0, 400.0], PointerPhase::Up));

        let med = |v: &mut Vec<f64>| {
            v.sort_by(f64::total_cmp);
            v[v.len() / 2]
        };
        (med(&mut post), med(&mut mid))
    }

    println!(
        "\n{:<6} {:<9} {:>12} {:>12} {:>8}",
        "tela", "impasto", "pos-undo", "meio-traco", "razao"
    );
    for side in [2048u32, 4096] {
        for imp in [false, true] {
            let (p, m) = split(side, imp);
            println!(
                "{side:<6} {:<9} {p:>12.2} {m:>12.2} {:>7.1}x",
                if imp { "ON" } else { "OFF" },
                if m > 0.0 { p / m } else { f64::INFINITY }
            );
        }
    }
    println!();
}
