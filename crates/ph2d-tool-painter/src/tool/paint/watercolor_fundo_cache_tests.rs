//! **O fundo de baixo da aguada em cache nunca fica velho** (n.º 6): depois de cada mudança da pilha, dos pixels de uma
//! camada de baixo, da opacidade, da visibilidade, da ordem ou do desfazer, o fundo congelado é o que a composição do
//! zero dá (o oráculo: `FUNDO_EM_CACHE` desligado, no mesmo processo); sem mudança ele é o MESMO `Arc` (não se
//! recompôs). A 4K a composição custa 10–14 ms por pen-down, e o chão preto do vidro, guardado pelo mesmo `Arc`, 8 ms.

use super::watercolor_backdrop::FUNDO_EM_CACHE;
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

fn variada(semente: u32) -> Vec<u8> {
    (0..W * H * 4)
        .map(|i| ((i.wrapping_mul(2_654_435_761) >> 7).wrapping_add(semente * 37) % 251) as u8)
        .collect()
}

/// O fundo que o pen-down congela agora, com a cache, e o do oráculo (composto do zero).
fn congela(t: &mut PainterTool) -> (Arc<Vec<u8>>, Vec<u8>) {
    t.freeze_watercolor_ground(false);
    let com = t.paint.wet_backdrop.clone().expect("o fundo");
    FUNDO_EM_CACHE.set(false);
    t.freeze_watercolor_ground(false);
    let oraculo = t.paint.wet_backdrop.clone().expect("o fundo do oráculo");
    FUNDO_EM_CACHE.set(true);
    t.paint.wet_backdrop = Some(Arc::clone(&com));
    (com, oraculo.to_vec())
}

#[test]
fn o_fundo_em_cache_e_o_composto_do_zero_e_so_se_recompoe_quando_algo_abaixo_muda() {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; (W * H * 4) as usize], W, H);
    let base = t.layers.active().expect("a base");
    let meio = t
        .add_raster_layer_with_pixels("meio", variada(1))
        .expect("meio");
    t.set_layer_opacity(meio, 0.7);
    let topo = t.add_raster_layer("topo").expect("topo");
    t.select_layer(topo);
    t.paint.brush.watercolor = true;
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);
    t.set_brush_size_px(10.0);
    assert!(t.watercolor_render_active(), "a fixture: a aguada");

    let (a, o) = congela(&mut t);
    assert!(*a == o, "o 1.º fundo é o do zero");
    let (b, o) = congela(&mut t);
    assert!(
        Arc::ptr_eq(&a, &b),
        "CONTROLO: sem mudança é o mesmo fundo (a cache serve)"
    );
    assert!(*b == o);

    traco(&mut t, 20.0);
    let (c, o) = congela(&mut t);
    assert!(*c == o, "depois de pintar na camada de cima");
    assert!(Arc::ptr_eq(&b, &c), "pintar em cima não muda o de baixo");

    let mut passos: Vec<(&str, Box<dyn Fn(&mut PainterTool)>)> = vec![
        (
            "opacidade",
            Box::new(move |t| t.set_layer_opacity(meio, 0.3)),
        ),
        (
            "visibilidade",
            Box::new(move |t| t.set_layer_visible(base, false)),
        ),
        (
            "pixels de baixo",
            Box::new(move |t| {
                t.select_layer(meio);
                traco(t, 40.0);
                t.select_layer(topo);
            }),
        ),
        (
            "desfazer o de baixo",
            Box::new(move |t| {
                t.select_layer(meio);
                assert!(t.undo_last(), "desfaz");
                t.select_layer(topo);
            }),
        ),
        (
            "só os pixels de baixo (a pilha igual)",
            Box::new(move |t| {
                let img = Arc::make_mut(t.images.get_mut(&meio).expect("o meio fora da tela"));
                img.rgba8_mut()[..(W * 8 * 4) as usize].fill(7);
            }),
        ),
        ("ordem", Box::new(move |t| t.move_layer_down(meio))),
    ];
    let mut antes = c;
    for (nome, passo) in passos.drain(..) {
        passo(&mut t);
        let (f, o) = congela(&mut t);
        assert!(*f == o, "{nome}: o fundo em cache não é o do zero");
        if *antes != o {
            assert!(!Arc::ptr_eq(&antes, &f), "{nome}: mudou e veio o velho");
        }
        antes = f;
    }
}
