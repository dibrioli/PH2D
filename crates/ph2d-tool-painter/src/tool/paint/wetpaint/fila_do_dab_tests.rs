//! N10 F2c: o depósito pela FILA pinta a tela do depósito SÍNCRONO, ao bit, no mesmo processo (o oráculo é o
//! caminho síncrono), em 3 fundos (branco · blocos de cor · Textured com relevo) × Flat Bristle e Thick Impasto.

use super::super::super::agua_sobre_oleo_tests::{mostrada, preset};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};

fn cp(pos: [f32; 2], pressure: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn fundo(w: u32, h: u32, blocos: bool) -> Vec<u8> {
    let cores = [
        [200u8, 60, 50],
        [60, 150, 210],
        [240, 200, 60],
        [90, 170, 90],
    ];
    let mut v = vec![255u8; (w * h * 4) as usize];
    if blocos {
        for (i, p) in v.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let (x, y) = (i as u32 % w, i as u32 / w);
            p[..3].copy_from_slice(&cores[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)]);
        }
    }
    v
}

fn pinta(fila: bool, qual: u8, pincel: &str, tamanho: f32) -> Vec<u8> {
    let (w, h) = (320u32, 160u32);
    super::fila_no_teste(fila);
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.abre_a_sprite(fundo(w, h, qual == 1), w, h);
    }
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.preset_aplicar(&preset(pincel)).expect("o pincel");
    t.set_brush_size_px(tamanho);
    t.set_brush_color_srgb8([40, 90, 220]);
    t.on_canvas_pointer(cp([60.0, 80.0], 0.6, PointerPhase::Down));
    for k in 1..=12 {
        let p = [60.0 + 16.0 * k as f32, 80.0 + 3.0 * k as f32];
        t.on_canvas_pointer(cp(p, 0.6 + 0.03 * k as f32, PointerPhase::Move));
        t.ticks_em_blocos(1);
    }
    t.on_canvas_pointer(cp([252.0, 116.0], 1.0, PointerPhase::Up));
    t.ticks_em_blocos(2);
    let px = mostrada(&mut t);
    super::fila_no_teste(false);
    px
}

#[test]
fn o_deposito_pela_fila_pinta_a_tela_do_deposito_sincrono_ao_bit() {
    for (pincel, tamanho) in [
        ("Oil/Flat/Flat Bristle.ph2d-brush.png", 70.0),
        ("Oil/Flat/Thick Impasto.ph2d-brush.png", 60.0),
    ] {
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "Textured")] {
            let sincrono = pinta(false, qual, pincel, tamanho);
            let fila = pinta(true, qual, pincel, tamanho);
            let fundo_so = pinta_nada(qual);
            assert!(
                sincrono != fundo_so,
                "{pincel} · {nome}: o traço não pintou"
            );
            assert!(sincrono == fila, "{pincel} · {nome}: a fila mudou a tela");
        }
    }
}

/// O fundo sem traço, para provar que o traço pintou (no papel Textured o branco não é 255).
fn pinta_nada(qual: u8) -> Vec<u8> {
    let (w, h) = (320u32, 160u32);
    let mut t = PainterTool::default();
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo(w, h, false), w, h);
    } else {
        t.abre_a_sprite(fundo(w, h, qual == 1), w, h);
    }
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    mostrada(&mut t)
}
