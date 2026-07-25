//! **Measurement probes for the PROTECTION GATE** — the paint crossing a mask, not the mask itself
//! (`docs/Painter/25_avaliacao_gpu.md` §13.11 diagnosis + §13.12 cure). Sibling of `mask_probe`, which
//! owns the mask's own coverage and the shared oracle helpers this file stands on; split off it when the
//! two waves together crossed the 700-line file cap.
//!
//! Run: `cargo test -p ph2d-tool-painter mask_probe_gate -- --ignored --nocapture` (the cost probe wants
//! `--release`: it reports memory-bandwidth numbers, and a debug build measures the debug build).

use super::mask_probe::{coverage, cp, cross_x, dump, vstroke};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, PanelEvent, PointerPhase, RasterEditTool, Tool as _,
};

/// **PROBE 12 — o reporte de 2026-07-25 (2ª rodada): a TINTA atravessando a proteção saía CRAQUELADA.**
///
/// A sonda que diagnosticou o defeito e a que o declara curado são a MESMA, e é isso que a torna útil:
/// ela mede a MESMA cena (proteção com orla macia + N traços de tinta cruzando) a duas taxas de polling
/// muito diferentes e imprime as duas linhas lado a lado. Duas linhas iguais = a força da proteção é uma
/// propriedade da máscara. Duas linhas diferentes = ela virou uma propriedade do mouse.
///
/// ## O histórico MEDIDO (não re-derive; doc 25 §13.11 → §13.12)
///
/// | lei | tinta onde `keep ≈ 0.5`, 4 ev | 60 ev | serra do contorno | contorno médio |
/// |---|---|---|---|---|
/// | pull-back contra o snapshot do BATCH (o bug) | 0,886 | **0,992** | 0,061 → **0,164 px** | andava **4 px** |
/// | pull-back contra a base do TRAÇO (cura mínima, REFUTADA) | 0,667 | **0,141** | 0,077 → 0,039 px | — |
/// | **plano LIVRE por-traço, `keep` aplicado UMA vez** (hoje) | **0,800** | **0,800** | **0,082 px** nas duas | **x=73,36** nas duas |
///
/// As duas primeiras erram em direções opostas e as duas dependem do nº de batches: **puxar de volta por
/// batch era a doença, não a referência escolhida.** A terceira é a semântica de máscara de camada, e o
/// controle desta mesma sonda (a serra do contorno da MÁSCARA, 0,040 px) diz que 0,082 px é a ordem do
/// próprio traçado, não um resíduo do gate.
#[allow(clippy::doc_overindented_list_items)]
#[test]
#[ignore]
fn probe_paint_through_the_protection() {
    const SZ: u32 = 256;
    for (label, events, strokes) in [("poucos-eventos", 4u32, 8u32), ("muitos-eventos", 60, 8)] {
        // Canvas BRANCO (o `mask_tool` pinta uma arte avermelhada, e aqui a TINTA é que tem de
        // destacar-se do fundo — com o fundo já vermelho não há contorno a medir; a 1ª versão desta
        // sonda mediu n=0 amostras por isso).
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (SZ * SZ * 4) as usize], SZ, SZ);
        t.handle_panel_event(PanelEvent::SelectOption(
            ph2d_editor_core::ids::PAINTER_PAINT_MODE,
            "mask".to_string(),
        ));
        t.set_brush_size_px(40.0);
        // 1) A proteção: um traço de máscara VERTICAL, com a orla macia atravessando o meio.
        vstroke(&mut t, 100.0, 40.0, 220.0, 30);
        let prot = coverage(&t, SZ);
        // 2) A tinta: N traços HORIZONTAIS de vermelho cruzando a zona protegida.
        t.set_paint_tool_mode("brush");
        t.set_brush_color_srgb8([0, 0, 0]); // tinta PRETA sobre branco: a cobertura é `1 − luma`
        t.set_brush_size_px(18.0);
        for k in 0..strokes {
            let y = 70.0 + k as f32 * 12.0;
            t.on_canvas_pointer(cp([40.0, y], PointerPhase::Down));
            for i in 1..=events {
                let x = 40.0 + 170.0 * (i as f32) / (events as f32);
                t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp([210.0, y], PointerPhase::Up));
            let _ = t.take_preview_arc();
        }
        // 3) Onde a tinta morre dentro da zona de alpha parcial. Numa proteção lisa, liso.
        let red: Vec<f32> = (0..(SZ as usize * SZ as usize))
            .map(|i| 1.0 - f32::from(t.canvas_rgba[i * 4]) / 255.0)
            .collect();
        let xs: Vec<f32> = (70..160)
            .filter_map(|y| cross_x(&red, SZ, y, 0.5))
            .collect();
        let saw = if xs.len() > 3 {
            xs.windows(3)
                .map(|w| (w[0] - 2.0 * w[1] + w[2]).abs())
                .sum::<f32>()
                / (xs.len() - 2) as f32
        } else {
            f32::NAN
        };
        let (mn, mx) = (
            xs.iter().copied().fold(f32::MAX, f32::min),
            xs.iter().copied().fold(f32::MIN, f32::max),
        );
        let pxs: Vec<f32> = (70..160)
            .filter_map(|y| cross_x(&prot, SZ, y, 0.5))
            .collect();
        let psaw = pxs
            .windows(3)
            .map(|w| (w[0] - 2.0 * w[1] + w[2]).abs())
            .sum::<f32>()
            / (pxs.len().max(3) - 2) as f32;
        // O DEGRAU de verdade: o maior salto de linha para linha do contorno (é isso que lê como
        // craquelado), e a POSIÇÃO média do contorno — se ela anda com o nº de eventos, a força da
        // proteção depende da taxa de polling, que é a doença que esta linha já curou 4× no relevo.
        let step = xs
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0_f32, f32::max);
        let mean = xs.iter().sum::<f32>() / xs.len() as f32;
        // E quanta tinta sobrevive exactamente onde a proteção é meia (o texel que decide tudo).
        let half_x = (100..160)
            .find(|&x| prot[130 * SZ as usize + x] < 0.5)
            .unwrap_or(0);
        println!(
            "{label:15} ({events:2} ev/traço): TINTA serra {saw:.3} px, DEGRAU máx {step:.2} px, \
             contorno médio x={mean:.2} (p2p {:.2}) | MÁSCARA (controle) serra {psaw:.3} px | \
             tinta em keep≈0.5 (x={half_x}): {:.3}",
            mx - mn,
            red[130 * SZ as usize + half_x]
        );
        dump(&format!("through_{label}"), &red, SZ);
    }
}

/// **PROBE 13 — o que a sessão de proteção CUSTA** (doc 25 §13.12). O plano livre é canvas-sized, então
/// a semeadura é proporcional à TELA (uma vez por traço) e a projeção é proporcional à PEGADA (por batch).
/// Mede as duas separadamente, nos dois tamanhos, para que as barras do gate saiam da medição.
#[test]
#[ignore]
fn probe_gated_stroke_cost() {
    for (size, gated) in [(2048u32, true), (2048, false), (4096, true), (4096, false)] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let c = size as f32 * 0.5;
        if gated {
            t.handle_panel_event(PanelEvent::SelectOption(
                ph2d_editor_core::ids::PAINTER_PAINT_MODE,
                "mask".to_string(),
            ));
            t.set_brush_size_px(120.0);
            vstroke(&mut t, c, c - 200.0, c + 200.0, 20);
            t.set_paint_tool_mode("brush");
        }
        t.set_brush_size_px(120.0);
        // O PEN-DOWN paga a semeadura (clone do canvas, canvas-proporcional, UMA vez por traço);
        // os moves seguintes pagam só a projeção, que é limitada pela pegada.
        let t0 = std::time::Instant::now();
        t.on_canvas_pointer(cp([c - 300.0, c], PointerPhase::Down));
        let _ = t.take_preview_arc();
        let seed_ms = t0.elapsed().as_secs_f64() * 1000.0;
        t.on_canvas_pointer(cp([c - 280.0, c], PointerPhase::Move));
        let _ = t.take_preview_arc();
        let n = 20;
        let t1 = std::time::Instant::now();
        for i in 1..=n {
            t.on_canvas_pointer(cp([c - 280.0 + i as f32 * 12.0, c], PointerPhase::Move));
            let _ = t.take_preview_arc();
        }
        let per_move = t1.elapsed().as_secs_f64() * 1000.0 / f64::from(n);
        t.on_canvas_pointer(cp([c + 300.0, c], PointerPhase::Up));
        let tag = if gated {
            "COM proteção"
        } else {
            "sem proteção (controle)"
        };
        println!("{size}^2 {tag:24}: pen-down {seed_ms:.2} ms | move {per_move:.2} ms");
    }
}
