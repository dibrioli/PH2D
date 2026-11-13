//! **O fundo do toque aquece na troca de camada e dá o mesmo bit** (n.º 7 item 3e): com a chave
//! `FUNDO_AQUECIDO_NA_TROCA`, o «+» e o clique na linha deixam o fundo de baixo e o chão preto prontos, e o 1.º
//! pen-down usa ESSE `Arc` (não compõe); sem a chave (o oráculo, no mesmo processo) o pen-down compõe. A tela, o fundo e
//! o quadro composto saem iguais ao bit. A 4K: 32,7–33,6 → 12,0–17,6 ms no 1.º pen-down da nova (`N7_ITEM3_mais.md`).

use super::watercolor_vidro::FUNDO_AQUECIDO_NA_TROCA;
use crate::tool::PainterTool;
use ph2d_editor_core::Tool as _;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use std::sync::Arc;

const W: u32 = 96;
const H: u32 = 64;

fn cp(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 0.8,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn traco(t: &mut PainterTool, y: f32) {
    t.on_canvas_pointer(cp(10.0, y, PointerPhase::Down));
    for k in 1..=5 {
        t.on_canvas_pointer(cp(10.0 + 12.0 * k as f32, y, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(80.0, y, PointerPhase::Up));
    for _ in 0..3 {
        t.on_tick(1000.0 / 60.0);
        let _ = t.take_preview_arc();
    }
}

/// O fundo em cache agora e se o chão preto congelado é o dele.
fn aquecido(t: &PainterTool) -> Option<(Arc<Vec<u8>>, bool)> {
    let f = t
        .paint
        .fundo_em_cache
        .as_ref()
        .map(|(_, f)| Arc::clone(f))?;
    let preto = t.chao_preto_e_de(&f);
    Some((f, preto))
}

/// O 1.º pen-down depois da troca usou o fundo que ela deixou pronto.
fn o_toque_usou(
    t: &mut PainterTool,
    antes: Option<(Arc<Vec<u8>>, bool)>,
    y: f32,
    fotos: &mut Vec<Vec<u8>>,
) -> bool {
    t.on_canvas_pointer(cp(10.0, y, PointerPhase::Down));
    let usado = t.paint.wet_backdrop.clone().expect("o fundo do toque");
    fotos.push(usado.to_vec());
    t.on_canvas_pointer(cp(80.0, y, PointerPhase::Up));
    for _ in 0..3 {
        t.on_tick(1000.0 / 60.0);
        let _ = t.take_preview_arc();
    }
    antes.is_some_and(|(f, preto)| preto && Arc::ptr_eq(&f, &usado))
}

/// `(tela, fundo, quadro)` de cada passo, e se cada 1.º toque usou o aquecido.
fn corre(chave: bool) -> (Vec<Vec<u8>>, Vec<bool>) {
    FUNDO_AQUECIDO_NA_TROCA.set(chave);
    let mut t = PainterTool::default();
    let base: Vec<u8> = (0..W * H * 4)
        .map(|i| (i.wrapping_mul(2_654_435_761) >> 9) as u8 | 0x40)
        .collect();
    t.abre_a_sprite(base, W, H);
    let fundo_id = t.layers.active().expect("a base");
    t.paint.brush.watercolor = true;
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    t.set_brush_size_px(10.0);
    assert!(t.watercolor_render_active(), "a fixture: a aguada");
    traco(&mut t, 20.0);
    let mut fotos = Vec::new();
    let mut usou = Vec::new();
    let foto = |t: &mut PainterTool, fotos: &mut Vec<Vec<u8>>| {
        fotos.push(t.canvas_rgba.to_vec());
        fotos.push(
            t.take_preview_arc()
                .map(|a| a.0.to_vec())
                .unwrap_or(vec![1]),
        );
    };
    // O «+».
    let nova = t.add_raster_layer("nova").expect("a nova");
    let a = aquecido(&t);
    usou.push(o_toque_usou(&mut t, a, 30.0, &mut fotos));
    foto(&mut t, &mut fotos);
    traco(&mut t, 40.0);
    foto(&mut t, &mut fotos);
    // O clique na linha: a base (nada abaixo, o chão puro) e de volta à nova.
    t.select_layer(fundo_id);
    let a = aquecido(&t);
    usou.push(o_toque_usou(&mut t, a, 50.0, &mut fotos));
    foto(&mut t, &mut fotos);
    t.select_layer(nova);
    let a = aquecido(&t);
    usou.push(o_toque_usou(&mut t, a, 55.0, &mut fotos));
    foto(&mut t, &mut fotos);
    FUNDO_AQUECIDO_NA_TROCA.set(true);
    (fotos, usou)
}

#[test]
fn o_fundo_do_toque_aquece_na_troca_e_e_o_mesmo_bit() {
    let (com, usou) = corre(true);
    let (sem, usou_sem) = corre(false);
    assert_eq!(
        usou,
        [true, true, true],
        "«+», clique na base, clique na nova: o 1.º toque usa o fundo e o chão preto aquecidos"
    );
    assert_eq!(
        usou_sem,
        [false, false, false],
        "CONTROLO: sem a chave o 1.º toque compõe o seu"
    );
    assert_eq!(com.len(), sem.len());
    for (i, (a, b)) in com.iter().zip(&sem).enumerate() {
        assert!(!a.is_empty(), "a foto {i} existe");
        assert!(
            a == b,
            "a foto {i} (fundo · tela · quadro) é a mesma ao bit"
        );
    }
}
