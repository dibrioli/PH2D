//! **O fundo do toque compõe-se num fio e dá o mesmo bit** (n.º 7 item 3f): com `FUNDO_AQUECIDO_EM_FUNDO`, o «+»
//! lança o fio e o pen-down colhe-o — acabado, a meio (o pen-down espera) ou descartado (a pilha mudou entre o clique
//! e o toque: compõe como antes). O oráculo é a rota do clique (o item 3e), no mesmo processo.

use super::watercolor_aquece::{ATRASO_DO_AQUECIMENTO_MS, CONTAS, FUNDO_AQUECIDO_EM_FUNDO};
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

fn traco(t: &mut PainterTool, y: f32, fotos: &mut Vec<Vec<u8>>) {
    t.on_canvas_pointer(cp(10.0, y, PointerPhase::Down));
    fotos.push(t.paint.wet_backdrop.as_deref().cloned().unwrap_or_default());
    for k in 1..=5 {
        t.on_canvas_pointer(cp(10.0 + 12.0 * k as f32, y, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(80.0, y, PointerPhase::Up));
    for _ in 0..3 {
        t.on_tick(1000.0 / 60.0);
        let _ = t.take_preview_arc();
    }
    fotos.push(t.canvas_rgba.to_vec());
    fotos.push(t.chao_preto_congelado());
    fotos.push(
        t.take_preview_arc()
            .map(|a| a.0.to_vec())
            .unwrap_or(vec![1]),
    );
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Caso {
    Acabado,
    AMeio,
    Descartado,
}

/// As fotos `(fundo do toque, tela, chão preto, quadro)` e `(colhidos, descartados, o fio corria no pen-down)`.
fn corre(em_fundo: bool, caso: Caso) -> (Vec<Vec<u8>>, (u32, u32, bool)) {
    FUNDO_AQUECIDO_EM_FUNDO.set(em_fundo);
    CONTAS.set((0, 0));
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
    let mut fotos = Vec::new();
    traco(&mut t, 20.0, &mut fotos);
    ATRASO_DO_AQUECIMENTO_MS.set(if caso == Caso::AMeio { 150 } else { 0 });
    let nova = t.add_raster_layer("nova").expect("a nova");
    ATRASO_DO_AQUECIMENTO_MS.set(0);
    if caso != Caso::AMeio {
        while t.aquecimento_acabou() == Some(false) {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    match caso {
        Caso::Acabado | Caso::AMeio => {}
        Caso::Descartado => {
            // A base muda entre o clique e o toque: o fio compôs a de antes.
            let img = Arc::make_mut(t.images.get_mut(&fundo_id).expect("a base"));
            Arc::make_mut(&mut img.rgba8)[0] ^= 0xFF;
        }
    }
    let corria = t.aquecimento_acabou() == Some(false);
    traco(&mut t, 30.0, &mut fotos);
    traco(&mut t, 40.0, &mut fotos);
    t.select_layer(fundo_id);
    t.select_layer(nova);
    traco(&mut t, 50.0, &mut fotos);
    FUNDO_AQUECIDO_EM_FUNDO.set(true);
    let (c, d) = CONTAS.get();
    (fotos, (c, d, corria))
}

#[test]
fn o_fundo_do_toque_compoe_se_num_fio_e_e_o_mesmo_bit() {
    for caso in [Caso::Acabado, Caso::AMeio, Caso::Descartado] {
        let (com, contas) = corre(true, caso);
        let (sem, contas_sem) = corre(false, caso);
        // O 1.º toque da nova colhe (ou descarta); o clique na base lança e o de volta à nova descarta-o, porque a
        // nova já está na cache do traço dela (não lança).
        let esperado = match caso {
            Caso::Acabado => (1, 1, false),
            Caso::AMeio => (1, 1, true),
            Caso::Descartado => (0, 2, false),
        };
        assert_eq!(
            contas, esperado,
            "{caso:?}: colhidos, descartados, o fio corria no toque"
        );
        assert_eq!(
            contas_sem,
            (0, 0, false),
            "CONTROLO {caso:?}: a rota do clique não lança fio"
        );
        assert_eq!(com.len(), sem.len());
        for (i, (a, b)) in com.iter().zip(&sem).enumerate() {
            assert!(!a.is_empty(), "{caso:?}: a foto {i} existe");
            assert!(
                a == b,
                "{caso:?}: a foto {i} (fundo · tela · quadro) é a mesma ao bit"
            );
        }
    }
}
