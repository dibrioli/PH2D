//! **O CUSTO DE UM PEN-UP É FUNÇÃO DO QUE O TRAÇO COBRE** — a sonda que as outras não continham.
//!
//! Toda sonda desta família usa o [`super::measure_stroke_owners::stroke`], que vai de `x=60` a
//! `x=260`: **200 px numa tela de 4096², 0,1% da área**. É a fixture certa para *quem segura os planos*
//! e para *de que é feito o custo FIXO*, e é a errada para a pergunta do artista, que atravessa a tela.
//!
//! O produto reportou `INPUT (fora do frame) up max=512,9 ms` onde o
//! [`super::measure_commit_cost::what_the_two_halves_of_the_pen_up_cost`] mede **5,57**. Duas ordens de
//! grandeza não são deriva de máquina — são uma fixture que não contém o fenômeno
//! ([[reference_topic_fixture_discipline]]).
//!
//! ⚠️ **A janela é impressa AO LADO do relógio**, e é ela que nomeia o mecanismo: o commit não varre
//! mais o canvas (S1/S2 — ele RECEBE a janela), mas ainda **extrai os dois lados dela**, então um traço
//! cuja janela é a tela paga uma cópia de documento por plano. Um relógio sozinho diria *"ficou lento"*;
//! o par (janela, relógio) diz *por quê*.

use super::measure_stroke_owners::{armed, cp};
use super::*;
use std::time::Instant;

/// Um traço em linha de `span` px, opcionalmente na diagonal (que é o gesto real: ele cobre área nos
/// DOIS eixos). Devolve o custo do **pen-up** e a janela que a entrada de undo guardou.
fn one_stroke(t: &mut PainterTool, y0: f32, span: f32, diagonal: bool) -> (f64, Option<Region>) {
    const STEPS: u8 = 12;
    let dy = if diagonal { span } else { 0.0 };
    t.on_canvas_pointer(cp([60.0, y0], PointerPhase::Down));
    for k in 1..=STEPS {
        let f = f32::from(k) / f32::from(STEPS);
        t.on_canvas_pointer(cp([60.0 + span * f, y0 + dy * f], PointerPhase::Move));
    }
    let t0 = Instant::now();
    t.on_canvas_pointer(cp([60.0 + span, y0 + dy], PointerPhase::Up));
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    (ms, t.undo.peek_confined_region(false))
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

/// Mediana do pen-up para um `span`, com a janela do último. O tool é **reusado** (o 1º traço paga a
/// alocação preguiçosa dos três planos de relevo — 192 MB a 4096²) e o 1º é descartado.
fn sweep(side: u32, span: f32, diagonal: bool) -> (f64, Option<Region>) {
    let mut t = armed(side);
    let (mut v, mut last) = (Vec::new(), None);
    for k in 0..5u8 {
        let (ms, win) = one_stroke(&mut t, 200.0 + f32::from(k) * 9.0, span, diagonal);
        if k > 0 {
            v.push(ms);
            last = win;
        }
    }
    (median(v), last)
}

fn pct(win: Option<Region>, side: u32) -> String {
    let total = f64::from(side) * f64::from(side);
    win.map_or_else(
        || "        —  (nao confinado)".to_string(),
        |r| {
            let a = f64::from(r.w) * f64::from(r.h);
            format!("{:5}x{:<5} = {:5.1}% da tela", r.w, r.h, 100.0 * a / total)
        },
    )
}

/// **O PEN-UP CONTRA A EXTENSÃO DO TRAÇO** — a sonda que responde ao report do Enio.
///
/// ⚠️ **O controle interno é a 1ª linha da tabela** (o traço de 200 px que as outras sondas medem). Sob
/// máquina carregada nenhum número absoluto se defende sozinho; o que se defende é ele ficar onde
/// sempre esteve enquanto as linhas de baixo explodem — a lição que salvou a wave do undo confinado
/// (doc 28 §5.63).
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn what_a_pen_up_costs_as_the_stroke_crosses_the_canvas() {
    for side in [2048u32, 4096] {
        eprintln!("\n[extensao] pen-up a {side}x{side}, impasto, pincel r=40");
        eprintln!(
            "  {:>6}  {:>4}  {:>9}   janela do delta",
            "span", "diag", "pen-up ms"
        );
        for (span, diag) in [
            (200.0f32, false), // <- o CONTROLE: o traço que todas as outras sondas medem
            (600.0, false),
            (1200.0, false),
            (f64::from(side) as f32 - 200.0, false),
            (600.0, true),
            (1200.0, true),
            (f64::from(side) as f32 - 200.0, true),
        ] {
            let (ms, win) = sweep(side, span, diag);
            eprintln!(
                "  {span:>6.0}  {:>4}  {ms:>9.2}   {}",
                if diag { "sim" } else { "nao" },
                pct(win, side)
            );
        }
    }
}

/// **AS DUAS METADES, no traço que atravessa a tela** — o irmão de
/// [`super::measure_commit_cost::what_the_two_halves_of_the_pen_up_cost`], com a fixture que contém o
/// fenômeno.
///
/// A ablação é pela ENTRADA: `paint.stroke_undo = None` faz o `close_stroke` pular
/// `commit_structural_edit`, então o que sobra é o `commit_stroke_height` (o fold do relevo).
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn which_half_of_a_long_pen_up_is_the_expensive_one() {
    fn halves(side: u32, span: f32, diagonal: bool) -> (f64, f64) {
        let mut both = Vec::new();
        let mut fold = Vec::new();
        for kill in [false, true] {
            let mut t = armed(side);
            for k in 0..5u8 {
                let y0 = 200.0 + f32::from(k) * 9.0;
                let dy = if diagonal { span } else { 0.0 };
                t.on_canvas_pointer(cp([60.0, y0], PointerPhase::Down));
                for j in 1..=12u8 {
                    let f = f32::from(j) / 12.0;
                    t.on_canvas_pointer(cp([60.0 + span * f, y0 + dy * f], PointerPhase::Move));
                }
                if kill {
                    t.paint.stroke_undo = None;
                }
                let t0 = Instant::now();
                t.on_canvas_pointer(cp([60.0 + span, y0 + dy], PointerPhase::Up));
                let ms = t0.elapsed().as_secs_f64() * 1000.0;
                if k > 0 {
                    if kill { fold.push(ms) } else { both.push(ms) }
                }
            }
        }
        (median(both), median(fold))
    }

    eprintln!("\n[metades] pen-up = commit de undo + fold do relevo");
    eprintln!("  (a ablacao e' pela ENTRADA: stroke_undo = None ⇒ close_stroke pula o commit)");
    eprintln!(
        "  {:>6}  {:>4}  {:>9} {:>9} {:>9}",
        "span", "diag", "completo", "so fold", "commit"
    );
    for side in [2048u32, 4096] {
        eprintln!("  --- {side}x{side} ---");
        for (span, diag) in [
            (200.0f32, false), // o CONTROLE
            (f64::from(side) as f32 - 200.0, false),
            (f64::from(side) as f32 - 200.0, true),
        ] {
            let (both, fold) = halves(side, span, diag);
            eprintln!(
                "  {span:>6.0}  {:>4}  {both:>9.2} {fold:>9.2} {:>9.2}",
                if diag { "sim" } else { "nao" },
                both - fold
            );
        }
    }
}

/// **DE QUE É FEITO O FOLD** — ablação pelos dois knobs que o artista de fato tem.
///
/// O `commit_stroke_height` é uma sequência de passadas sobre a MESMA janela, e duas delas são
/// governadas por um controle do painel: o **Smoothing** (que liga o `settle`, um box blur `O(n·r)` que
/// re-soma a janela por texel — deliberadamente, porque a soma corrida deriva e quebra a byte-identidade
/// do crop) e o **Push** (o deslocamento, que passa pelo mesmo blur). O que sobra com os dois em zero é
/// o piso: derivar a altura texel a texel, o `over` do material e a escrita com o bbox.
///
/// ⚠️ Ablacionar por knob mede o produto pela porta do artista — não instrumenta o laço, então ela não
/// pode ficar cega à porta se o fold for reescrito (doc 28 §4.8.2).
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn what_the_fold_of_a_canvas_wide_stroke_is_made_of() {
    fn fold_ms(side: u32, smoothing: Option<f32>, push: Option<f32>) -> f64 {
        let span = f64::from(side) as f32 - 200.0;
        let mut t = armed(side);
        if let Some(s) = smoothing {
            t.paint.brush.impasto_smoothing = s;
            for slot in &mut t.paint.brush_by_mode {
                slot.impasto_smoothing = s;
            }
        }
        if let Some(p) = push {
            t.paint.brush.impasto_push = p;
            for slot in &mut t.paint.brush_by_mode {
                slot.impasto_push = p;
            }
        }
        let mut v = Vec::new();
        for k in 0..5u8 {
            let y0 = 200.0 + f32::from(k) * 9.0;
            t.on_canvas_pointer(cp([60.0, y0], PointerPhase::Down));
            for j in 1..=12u8 {
                let f = f32::from(j) / 12.0;
                t.on_canvas_pointer(cp([60.0 + span * f, y0 + span * f], PointerPhase::Move));
            }
            t.paint.stroke_undo = None; // só o fold entra no relógio
            let t0 = Instant::now();
            t.on_canvas_pointer(cp([60.0 + span, y0 + span], PointerPhase::Up));
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            if k > 0 {
                v.push(ms);
            }
        }
        median(v)
    }

    let probe = armed(256);
    eprintln!(
        "\n[fold] defaults do pincel: smoothing efetivo {:.3} · push efetivo {:.3}",
        probe.paint.brush.effective_impasto_smoothing(),
        probe.paint.brush.effective_impasto_push()
    );
    for side in [2048u32, 4096] {
        let full = fold_ms(side, None, None);
        let no_settle = fold_ms(side, Some(0.0), None);
        let no_push = fold_ms(side, None, Some(0.0));
        let floor = fold_ms(side, Some(0.0), Some(0.0));
        eprintln!("[fold] {side}x{side}, traço na diagonal de canto a canto");
        eprintln!("  como shipa            {full:8.2} ms");
        eprintln!(
            "  sem Smoothing         {no_settle:8.2} ms   (o settle custa {:.2})",
            full - no_settle
        );
        eprintln!(
            "  sem Push              {no_push:8.2} ms   (o push   custa {:.2})",
            full - no_push
        );
        eprintln!(
            "  os dois em zero       {floor:8.2} ms   <- o PISO: derive + material + escrita"
        );
    }
}

/// **DE QUE É FEITA A METADE DO COMMIT** — a que sobrou sem atribuição, e é 72% do pen-up.
///
/// A `which_half_…` diz *quanto*; ela não diz *o quê*. E a hipótese óbvia — *"ele extrai uma janela
/// canvas-sized dos quatro planos"* — está **REFUTADA pelo próprio produto**: numa janela grande os
/// quatro caem em `Whole`, que guarda `Arc` e não copia byte nenhum. Então o custo é outra coisa.
///
/// Ablação pelas DUAS alavancas que o S3 deixou (`elide_cursor` · `elide_relief`) mais o histórico
/// inteiro fora do caminho. ⚠️ Elas são `cfg(test)`, e é justamente por isso que uma sonda pode
/// perguntar ao produto em vez de fingir o que ele faz.
#[test]
#[ignore = "medicao — rode com --release --ignored --test-threads=1"]
fn what_the_commit_half_is_made_of() {
    fn pen_up_ms(side: u32, cursor: bool, relief: bool, history: bool) -> f64 {
        let span = f64::from(side) as f32 - 200.0;
        let mut t = armed(side);
        t.undo.elide_cursor = cursor;
        t.undo.elide_relief = relief;
        let mut v = Vec::new();
        for k in 0..5u8 {
            let y0 = 200.0 + f32::from(k) * 9.0;
            t.on_canvas_pointer(cp([60.0, y0], PointerPhase::Down));
            for j in 1..=12u8 {
                let f = f32::from(j) / 12.0;
                t.on_canvas_pointer(cp([60.0 + span * f, y0 + span * f], PointerPhase::Move));
            }
            if !history {
                // A ablação mais grosseira: sem entrada nenhuma na pilha, o `record_structural` não
                // tem cursor para mover nem delta para partir.
                t.undo.clear();
            }
            let t0 = Instant::now();
            t.on_canvas_pointer(cp([60.0 + span, y0 + span], PointerPhase::Up));
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            if k > 0 {
                v.push(ms);
            }
        }
        median(v)
    }

    for side in [2048u32, 4096] {
        let full = pen_up_ms(side, true, true, true);
        let no_cursor = pen_up_ms(side, false, true, true);
        let no_relief = pen_up_ms(side, true, false, true);
        let no_hist = pen_up_ms(side, true, true, false);
        eprintln!("\n[commit] {side}x{side}, diagonal de canto a canto");
        eprintln!("  como shipa                 {full:8.2} ms");
        eprintln!(
            "  sem a elisao do CURSOR     {no_cursor:8.2} ms   (delta {:+.2})",
            no_cursor - full
        );
        eprintln!(
            "  sem a elisao do RELEVO     {no_relief:8.2} ms   (delta {:+.2})",
            no_relief - full
        );
        eprintln!(
            "  historico LIMPO no pen-up  {no_hist:8.2} ms   (delta {:+.2})",
            no_hist - full
        );
    }
}
