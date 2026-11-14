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
//! cargo test -p ph2d-tool-painter --profile smoke measure_ -- --ignored --nocapture --test-threads=1
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

/// **UM TRAÇO FICA MAIS CARO CONFORME A HISTÓRIA CRESCE?** — o report do Enio (2026-08-01):
/// *"depois de dar vário traço o impasto fica muito lento"*, com `INPUT (fora do frame) p50=0,0
/// max=1016,5 ms` no log. Mediana grátis e um evento de UM SEGUNDO é a assinatura de um custo
/// **por-TRAÇO** (pen-down ou pen-up), nunca por-movimento — e *"depois de vários"* é a assinatura de
/// algo O(histórico).
///
/// ⚠️ A sonda não instrumenta o laço: ela dirige o **ciclo do produto** e imprime cada fase por traço.
/// Se a curva for plana, a causa não está aqui e a hipótese morre em vez de virar wave.
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn does_a_stroke_get_more_expensive_as_the_history_grows() {
    use std::time::Instant;

    for side in [4096u32] {
        let mut t = armed(side);
        println!(
            "\n{side}²  {:>4} {:>10} {:>10} {:>10} {:>12}",
            "n", "pen-down", "moves", "pen-up", "hist MB"
        );
        let mut worst = 0.0f64;
        for n in 0..80u32 {
            let y = 120.0 + (n % 12) as f32 * 40.0;
            let t0 = Instant::now();
            t.on_canvas_pointer(cp([60.0, y], PointerPhase::Down));
            let down = t0.elapsed().as_secs_f64() * 1e3;
            let t1 = Instant::now();
            for k in 1..=6u8 {
                t.on_canvas_pointer(cp([60.0 + f32::from(k) * 30.0, y], PointerPhase::Move));
            }
            let moves = t1.elapsed().as_secs_f64() * 1e3;
            let t2 = Instant::now();
            t.on_canvas_pointer(cp([260.0, y], PointerPhase::Up));
            let up = t2.elapsed().as_secs_f64() * 1e3;
            let ev = down.max(moves).max(up);
            if ev > worst * 1.6 && n > 0 {
                println!(
                    "     !! n={n}: PICO — down {down:.1} moves {moves:.1} up {up:.1} (pior anterior {worst:.1})"
                );
            }
            worst = worst.max(ev);
            if n % 10 == 0 || n == 79 {
                println!(
                    "     {n:>4} {down:>10.2} {moves:>10.2} {up:>10.2} {:>12.1}",
                    t.undo.retained_bytes() as f64 / 1e6
                );
            }
        }
    }
}

/// **A CADÊNCIA e o PINCEL** — as duas diferenças entre a sonda acima (que NÃO reproduz o report) e o
/// produto: o app drena o preview a cada quadro, e o artista pinta com pincel grande.
///
/// ⚠️ A sonda anterior mede o handler de ponteiro isolado, e `INPUT (fora do frame)` é exatamente esse
/// handler — mas sem o dreno o tool nunca publica, e §5.49 já mostrou que o handshake por quadro muda
/// o custo por evento. *Uma fixture que não contém o fenômeno não pode julgá-lo.*
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn does_the_frame_cadence_or_a_big_brush_reproduce_the_slow_stroke() {
    use std::time::Instant;

    for radius in [40.0f32, 100.0, 200.0, 300.0] {
        let mut t = armed(4096);
        t.sonda_raio(radius);
        let (mut worst_ev, mut worst_n) = (0.0f64, 0u32);
        let mut total = 0.0f64;
        for n in 0..40u32 {
            let y = 200.0 + (n % 16) as f32 * 60.0;
            let mut ev = 0.0f64;
            let t0 = Instant::now();
            t.on_canvas_pointer(cp([100.0, y], PointerPhase::Down));
            ev = ev.max(t0.elapsed().as_secs_f64() * 1e3);
            for k in 1..=10u8 {
                let t1 = Instant::now();
                t.on_canvas_pointer(cp([100.0 + f32::from(k) * 60.0, y], PointerPhase::Move));
                ev = ev.max(t1.elapsed().as_secs_f64() * 1e3);
                // A CADÊNCIA: o produto drena o preview a cada quadro.
                t.paint_tick(1.0 / 60.0);
                let _ = t.take_preview_arc();
            }
            let t2 = Instant::now();
            t.on_canvas_pointer(cp([700.0, y], PointerPhase::Up));
            ev = ev.max(t2.elapsed().as_secs_f64() * 1e3);
            t.paint_tick(1.0 / 60.0);
            let _ = t.take_preview_arc();
            total += ev;
            if ev > worst_ev {
                worst_ev = ev;
                worst_n = n;
            }
        }
        println!(
            "raio {radius:>5.0}: pior evento {worst_ev:>8.2} ms (traco {worst_n}) · media do pior-por-traco {:>7.2} ms",
            total / 40.0
        );
    }
}

/// **A ablação 2×2 que separa a CADÊNCIA do PINCEL** — as duas mudaram juntas na sonda anterior, e uma
/// diferença medida com dois fatores é uma diferença sem dono.
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn is_it_the_brush_or_the_drain() {
    use std::time::Instant;

    println!(
        "\n{:>6} {:>8} {:>14} {:>14}",
        "raio", "dreno", "pior evento", "mediana"
    );
    for radius in [40.0f32, 100.0, 200.0] {
        for drain in [false, true] {
            let mut t = armed(4096);
            t.sonda_raio(radius);
            let mut evs = Vec::new();
            for n in 0..12u32 {
                let y = 200.0 + (n % 8) as f32 * 60.0;
                t.on_canvas_pointer(cp([100.0, y], PointerPhase::Down));
                for k in 1..=10u8 {
                    let t1 = Instant::now();
                    t.on_canvas_pointer(cp([100.0 + f32::from(k) * 60.0, y], PointerPhase::Move));
                    evs.push(t1.elapsed().as_secs_f64() * 1e3);
                    if drain {
                        t.paint_tick(1.0 / 60.0);
                        let _ = t.take_preview_arc();
                    }
                }
                t.on_canvas_pointer(cp([700.0, y], PointerPhase::Up));
            }
            evs.sort_by(f64::total_cmp);
            println!(
                "{radius:>6.0} {:>8} {:>14.2} {:>14.2}",
                if drain { "SIM" } else { "nao" },
                evs[evs.len() - 1],
                evs[evs.len() / 2]
            );
        }
    }
}

/// **B17.5 — quanto um passo DIAGONAL retém, contra o que os tiles tocados reteriam** (3840×2160).
///
/// A janela declarada é a UNIÃO-retângulo do que se escreveu: um traço de canto a canto cobre a tela
/// inteira e o `from_window` guarda o `Whole` (dois planos de 33 MB). Os tiles de 128² que o traço de
/// facto toca são poucos; a sonda imprime os dois números — o «antes» do B17.5.
#[test]
#[ignore = "medicao — rode com --profile smoke --ignored"]
fn what_a_diagonal_step_retains_at_4k() {
    const W: u32 = 3840;
    const H: u32 = 2160;
    const T: u32 = crate::undo::journal::TILE as u32;
    for impasto in [false, true] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
        let mut b = t.paint.brush;
        b.radius_px = 40.0;
        b.strength = 1.0;
        b.impasto = impasto;
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        stroke(&mut t, 40.0); // instala o histórico e o relevo da camada
        let antes = t.undo.retained_bytes();
        let n = 64u16;
        let p = |k: u16| {
            let f = f32::from(k) / f32::from(n);
            [60.0 + f * (W as f32 - 120.0), 60.0 + f * (H as f32 - 120.0)]
        };
        t.on_canvas_pointer(super::measure_stroke_owners::cp(p(0), PointerPhase::Down));
        let mut tiles = std::collections::BTreeSet::new();
        for k in 1..=n {
            let [x, y] = p(k);
            t.on_canvas_pointer(super::measure_stroke_owners::cp([x, y], PointerPhase::Move));
            let r = b.radius_px + 2.0;
            for ty in ((y - r).max(0.0) as u32 / T)..=((y + r) as u32 / T).min((H - 1) / T) {
                for tx in ((x - r).max(0.0) as u32 / T)..=((x + r) as u32 / T).min((W - 1) / T) {
                    tiles.insert((tx, ty));
                }
            }
        }
        t.on_canvas_pointer(super::measure_stroke_owners::cp(p(n), PointerPhase::Up));
        let passo = t.undo.retained_bytes() - antes;
        let por_tiles = tiles.len() * (T * T * 4) as usize * 2;
        eprintln!(
            "impasto={impasto} passo retém {:.1} MB · {} tiles de {T}² tocados (canvas) = {:.1} MB nos dois lados",
            passo as f64 / 1e6,
            tiles.len(),
            por_tiles as f64 / 1e6
        );
        eprintln!("  variantes do topo: {:?}", t.undo.probe_top_variants());
        // Antes do B17.5: 66,4 MB (Digital) e 265,4 MB (Impasto) — a união-retângulo guardada inteira.
        let teto = if impasto {
            5 * por_tiles
        } else {
            2 * por_tiles
        };
        assert!(
            passo <= teto,
            "impasto={impasto}: o passo retém {passo} B > {teto} B dos tiles"
        );
    }
}

/// **B17.5 (b) — o que um Ctrl+Z do passo diagonal COPIA a 4K, e quem segura a tela.**
///
/// O `side()` dos `Tiles` começa por um `par_clone` do plano do cursor; um `Arc::make_mut` só o
/// pouparia se o cursor fosse o ÚNICO dono. A sonda imprime os donos da tela viva antes do undo, as
/// cópias de plano inteiro (contador `PAR_CLONES`) e o relógio (mínimo de 5) do undo e do redo.
#[test]
#[ignore = "medicao — rode com --profile smoke --ignored"]
fn what_a_diagonal_ctrl_z_copies_at_4k() {
    use std::time::Instant;
    const W: u32 = 3840;
    const H: u32 = 2160;
    for impasto in [false, true] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
        let mut b = t.paint.brush;
        b.radius_px = 40.0;
        b.strength = 1.0;
        b.impasto = impasto;
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        stroke(&mut t, 40.0);
        let n = 64u16;
        let p = |k: u16| {
            let f = f32::from(k) / f32::from(n);
            [60.0 + f * (W as f32 - 120.0), 60.0 + f * (H as f32 - 120.0)]
        };
        let cp = super::measure_stroke_owners::cp;
        t.on_canvas_pointer(cp(p(0), PointerPhase::Down));
        for k in 1..=n {
            t.on_canvas_pointer(cp(p(k), PointerPhase::Move));
        }
        t.on_canvas_pointer(cp(p(n), PointerPhase::Up));
        let _ = t.take_preview_arc();
        let donos = std::sync::Arc::strong_count(&t.canvas_rgba);
        eprintln!("  variantes do topo: {:?}", t.undo.probe_top_variants());
        // Cura × ablação `COPIA_SEMPRE` (o mundo de antes do B17.5) no MESMO processo, intercaladas, o mínimo.
        let mut m = [(f64::MAX, f64::MAX, 0usize, 0usize); 2];
        let (mut su, mut sr) = (Vec::new(), Vec::new());
        for rodada in 0..10 {
            for k in [rodada % 2, 1 - rodada % 2] {
                crate::undo_delta::tiles::COPIA_SEMPRE.with(|c| c.set(k == 1));
                let _ = crate::plane_copy::par_clone_sites_desde_a_ultima();
                let c0 = crate::plane_copy::par_clones_nesta_thread();
                let t0 = Instant::now();
                assert!(
                    t.undo_last(),
                    "a fixture nao contem o fenomeno: nada a desfazer"
                );
                let un = t0.elapsed().as_secs_f64() * 1e3;
                let c1 = crate::plane_copy::par_clones_nesta_thread();
                let t1 = Instant::now();
                assert!(
                    t.redo_last(),
                    "a fixture nao contem o fenomeno: nada a refazer"
                );
                let re = t1.elapsed().as_secs_f64() * 1e3;
                let c2 = crate::plane_copy::par_clones_nesta_thread();
                let sites = crate::plane_copy::par_clone_sites_desde_a_ultima();
                crate::undo_delta::tiles::COPIA_SEMPRE.with(|c| c.set(false));
                let (a, b) = sites.split_at(c1 - c0);
                if k == 0 {
                    (su, sr) = (a.to_vec(), b.to_vec());
                } else {
                    assert!(
                        c1 > c0,
                        "controle: a ablacao COPIA_SEMPRE nao copiou — a sonda mede outra coisa"
                    );
                }
                let e = &mut m[k];
                *e = (e.0.min(un), e.1.min(re), c1 - c0, c2 - c1);
            }
        }
        for (k, nome) in [(0, "cura"), (1, "ablacao COPIA_SEMPRE")] {
            let (un, re, cu, cr) = m[k];
            eprintln!(
                "impasto={impasto} {nome}: donos da tela {donos} · undo {un:.2} ms ({cu} copias) · redo {re:.2} ms ({cr} copias)"
            );
        }
        eprintln!("  sitios do undo (cura): {su:?}\n  sitios do redo (cura): {sr:?}");
        let do_delta =
            |s: &[(&'static str, u32)]| s.iter().filter(|s| s.0.ends_with("undo_delta.rs")).count();
        assert_eq!(
            (do_delta(&su), do_delta(&sr)),
            (0, 0),
            "impasto={impasto}: a cura copia um plano no lado do delta"
        );
    }
}
