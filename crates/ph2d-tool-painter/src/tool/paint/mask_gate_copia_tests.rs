//! B40.A4 — a projeção da proteção não copia a tela inteira por traço (`mask_gate.rs`). Antes: por época um
//! `free` (cópia) + o fork da tela que o `base` (2.º dono forte) forçava; por traço a captura da tela inteira no
//! journal (`fork_canvas(.., None)`) e um acesso não declarado (o commit varria a tela). Depois: o `free` é a
//! única cópia da época; o `base` é capturado por pedaço onde os lotes projetam.

use super::mask_gate::CAMINHO_ANTIGO;
use super::mask_probe::{cp, vstroke};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, PanelEvent, PointerPhase, RasterEditTool, Tool as _,
};
use std::sync::Arc;

fn armada(s: u32, protecao: bool, sel: bool) -> PainterTool {
    let mut t = PainterTool::default();
    let src: Vec<u8> = (0..s * s * 4).map(|i| (i * 7 % 251) as u8 | 0x80).collect();
    t.set_source(src, s, s);
    if protecao {
        t.handle_panel_event(PanelEvent::SelectOption(
            ph2d_editor_core::ids::PAINTER_PAINT_MODE,
            "mask".to_string(),
        ));
        t.set_brush_size_px(40.0);
        let c = s as f32 * 0.5;
        vstroke(&mut t, c, c - 120.0, c + 120.0, 24);
        t.set_paint_tool_mode("brush");
    }
    if sel {
        t.paint.selection_active = true;
        t.paint.selection_mask = Arc::new((0..s * s).map(|i| ((i % s) * 255 / s) as u8).collect());
    }
    t.set_brush_size_px(16.0);
    t.set_brush_color_srgb8([10, 20, 200]);
    t
}

/// Um traço horizontal em `y`; devolve `(cópias da tela inteira, pico do journal da tela em bytes)`.
fn traco(t: &mut PainterTool, y: f32) -> (u32, usize) {
    let s = t.source_size.0 as f32;
    let free0 = t.gate.as_ref().map(|g| Arc::as_ptr(&g.free));
    let tela0 = Arc::as_ptr(&t.canvas_rgba);
    t.on_canvas_pointer(cp([s * 0.3, y], PointerPhase::Down));
    let mut pico = 0;
    for i in 1..=6 {
        let x = s * 0.3 + s * 0.4 * (i as f32) / 6.0;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        pico = pico.max(t.undo.write_state.journal_heap_bytes());
    }
    t.on_canvas_pointer(cp([s * 0.7, y], PointerPhase::Up));
    let _ = t.take_preview_arc();
    let free1 = t.gate.as_ref().map(|g| Arc::as_ptr(&g.free));
    // O `free` novo é uma cópia; um `base` que é a tela do início é o 2.º dono forte que obriga o fork dela.
    let base_e_a_tela = t
        .gate
        .as_ref()
        .is_some_and(|g| Arc::as_ptr(&g.base.pixels) == tela0);
    let copias = u32::from(free1.is_some() && free1 != free0) + u32::from(base_e_a_tela);
    (copias, pico)
}

#[test]
fn a_protecao_nao_copia_a_tela_inteira_por_traco() {
    const S: u32 = 512;
    let tela = (S * S * 4) as usize;
    // CONTROLO: sem proteção nem seleção, o journal guarda só a caixa — a régua mede a porta, não o pincel.
    let mut livre = armada(S, false, false);
    for y in [200.0, 230.0] {
        let (c, pico) = traco(&mut livre, y);
        assert_eq!(c, 0, "CONTROLO: sem porta não há época");
        assert!(
            pico < tela / 4,
            "CONTROLO: o traço sem porta capturou {pico} B de {tela}"
        );
    }
    for (protecao, sel) in [(true, false), (false, true), (true, true)] {
        let mut t = armada(S, protecao, sel);
        let (c1, _) = traco(&mut t, 200.0);
        assert!(
            c1 <= 1,
            "({protecao},{sel}) 1.º traço: {c1} cópias da tela (≤ 1 por época)"
        );
        for y in [230.0, 260.0] {
            let (c, pico) = traco(&mut t, y);
            assert_eq!(
                c, 0,
                "({protecao},{sel}) traço seguinte: {c} cópias da tela (0)"
            );
            assert!(
                pico < tela / 4,
                "({protecao},{sel}) o journal capturou {pico} B de {tela} (só a caixa)"
            );
        }
        assert!(t.undo_last(), "desfez");
        let (cz, _) = traco(&mut t, 215.0);
        assert!(
            cz <= 1,
            "({protecao},{sel}) depois do Ctrl+Z: {cz} cópias (≤ 1 recaptura)"
        );
    }
}

/// Paridade ao bit contra o caminho antigo guardado (`CAMINHO_ANTIGO`): a tela depois de cada passo de vários
/// traços com proteção, seleção e as duas, com Ctrl+Z e refazer.
#[test]
fn a_protecao_esparsa_pinta_o_mesmo_bit_que_o_caminho_antigo() {
    const S: u32 = 256;
    let roteiro = |antigo: bool, protecao: bool, sel: bool| -> Vec<Vec<u8>> {
        CAMINHO_ANTIGO.with(|c| c.set(antigo));
        let mut t = armada(S, protecao, sel);
        let mut fotos = Vec::new();
        for y in [100.0, 118.0, 136.0] {
            traco(&mut t, y);
            fotos.push(t.canvas_rgba.as_ref().clone());
        }
        t.paint.brush.strength = 0.4;
        traco(&mut t, 110.0);
        fotos.push(t.canvas_rgba.as_ref().clone());
        assert!(t.undo_last());
        fotos.push(t.canvas_rgba.as_ref().clone());
        assert!(t.undo_last());
        fotos.push(t.canvas_rgba.as_ref().clone());
        assert!(t.redo_last());
        fotos.push(t.canvas_rgba.as_ref().clone());
        traco(&mut t, 128.0);
        fotos.push(t.canvas_rgba.as_ref().clone());
        CAMINHO_ANTIGO.with(|c| c.set(false));
        fotos
    };
    for (protecao, sel) in [(true, false), (false, true), (true, true)] {
        let velho = roteiro(true, protecao, sel);
        let novo = roteiro(false, protecao, sel);
        assert!(
            velho[0] != armada(S, protecao, sel).canvas_rgba.as_ref()[..],
            "CONTROLO: o traço pintou"
        );
        for (k, (a, b)) in velho.iter().zip(&novo).enumerate() {
            let dif = a.iter().zip(b).filter(|(x, y)| x != y).count();
            assert_eq!(
                dif, 0,
                "({protecao},{sel}) passo {k}: {dif} bytes diferem do caminho antigo"
            );
        }
    }
}

/// Sonda (B40.A4): 4K com seleção ativa — ms e bytes do journal no 1.º lote e nos seguintes, antigo × novo
/// intercalados no mesmo binário, o mínimo de 5. `--profile smoke -- --ignored sonda_b40a4`.
#[test]
#[ignore = "sonda de custo"]
fn sonda_b40a4_custo_da_protecao_4k() {
    let (w, h) = (3840u32, 2160u32);
    let mede = |antigo: bool| -> (f64, f64, usize) {
        CAMINHO_ANTIGO.with(|c| c.set(antigo));
        let mut t = PainterTool::default();
        t.set_source(vec![200u8; (w * h * 4) as usize], w, h);
        t.paint.selection_active = true;
        t.paint.selection_mask = Arc::new((0..w * h).map(|i| ((i % w) * 255 / w) as u8).collect());
        t.set_brush_size_px(40.0);
        let mut primeiro = f64::MAX;
        let mut seguintes = f64::MAX;
        let mut bytes = 0;
        for k in 0..3 {
            let y = 800.0 + k as f32 * 60.0;
            t.on_canvas_pointer(cp([1500.0, y], PointerPhase::Down));
            let t0 = std::time::Instant::now();
            t.on_canvas_pointer(cp([1520.0, y], PointerPhase::Move));
            let a = t0.elapsed().as_secs_f64() * 1e3;
            bytes = bytes.max(t.undo.write_state.journal_heap_bytes());
            let t1 = std::time::Instant::now();
            for i in 2..=8 {
                t.on_canvas_pointer(cp([1500.0 + 20.0 * i as f32, y], PointerPhase::Move));
            }
            let b = t1.elapsed().as_secs_f64() * 1e3 / 7.0;
            t.on_canvas_pointer(cp([1700.0, y], PointerPhase::Up));
            let _ = t.take_preview_arc();
            if k == 0 {
                primeiro = primeiro.min(a);
            } else {
                seguintes = seguintes.min(a.min(b));
            }
        }
        CAMINHO_ANTIGO.with(|c| c.set(false));
        (primeiro, seguintes, bytes)
    };
    let (mut va, mut na) = ((f64::MAX, f64::MAX, 0), (f64::MAX, f64::MAX, 0));
    for _ in 0..5 {
        for antigo in [true, false] {
            let r = mede(antigo);
            let acc = if antigo { &mut va } else { &mut na };
            *acc = (acc.0.min(r.0), acc.1.min(r.1), acc.2.max(r.2));
        }
    }
    eprintln!(
        "B40.A4 4K seleção: 1.º lote antigo {:.2} ms → novo {:.2} ms · lotes seguintes (2.º/3.º traço) {:.2} → {:.2} ms · journal pico {} → {} B",
        va.0, na.0, va.1, na.1, va.2, na.2
    );
}
