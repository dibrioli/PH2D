//! **O undo mata a tinta viva** (smoke do dono, 30/10): desfazer um traço deixa a tinta que FICOU na tela sem a água,
//! o assentado e o óleo dela — o traço seguinte por cima não a re-molha nem a arrasta como faria sem o undo.
//!
//! Cena A = o óleo solúvel, um traço a mais LONGE dele, o undo desse traço e a aguada por cima; cena B = sem o traço
//! a mais. O undo devolve a tela de antes do traço: a grade e a imagem depois da aguada têm de ser as mesmas.

use super::agua_sobre_oleo_tests::{H, PAPEL, W, mostrada, ponteiro, preset};
use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PanelEvent, PointerPhase};
use ph2d_wet_paint::tuning::Knob;

fn tick(t: &mut PainterTool, n: usize) {
    for _ in 0..n {
        t.on_tick(1000.0 / 60.0);
    }
}

/// Um traço horizontal de `x0` a `x0 + 160` na linha `y`.
fn traco(t: &mut PainterTool, x0: f32, y: f32) {
    t.on_canvas_pointer(ponteiro(x0, y, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(x0 + 4.0 * k as f32, y, PointerPhase::Move));
        tick(t, 1);
    }
    t.on_canvas_pointer(ponteiro(x0 + 160.0, y, PointerPhase::Up));
}

fn base(meio: &str, cor: [u8; 3]) -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", PAPEL[0], PAPEL[1], PAPEL[2]),
    ));
    t.preset_aplicar(&preset(meio)).expect("o meio");
    t.paint.wetpaint.knobs.set(Knob::Solubility, 1.0);
    t.set_brush_size_px(30.0);
    t.set_brush_color_srgb8(cor);
    t
}

/// Os campos da tinta viva das células (sett/susp/film/wet/óleo), para comparar as duas cenas.
type Grade = (
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<u8>,
    Vec<ph2d_wet_paint::grid::oleo_celula::OleoCelula>,
);

fn grade(t: &mut PainterTool) -> Option<Grade> {
    t.wet_bring_home();
    let g = &t.paint.wetpaint.session.as_ref()?.engine.layers[0].grid;
    Some((
        g.sett.clone(),
        g.susp.clone(),
        g.film.clone(),
        g.wet.clone(),
        g.oleo.clone(),
    ))
}

/// O 1.º traço (`meio`/`cor`), o extra (só na cena A, desfeito) e a grade antes do 2.º traço.
fn ate_a_aguada(
    meio: &str,
    cor: [u8; 3],
    com_undo: bool,
    assenta: usize,
) -> (PainterTool, Option<Grade>) {
    let mut t = base(meio, cor);
    traco(&mut t, 40.0, 90.0);
    tick(&mut t, assenta);
    if com_undo {
        traco(&mut t, 40.0, 150.0);
        assert!(t.undo_last(), "havia o traço extra para desfazer");
    }
    let g = grade(&mut t);
    (t, g)
}

fn aguada(t: &mut PainterTool, meio: &str, cor: [u8; 3]) {
    t.preset_aplicar(&preset(meio)).expect("o meio");
    t.paint.wetpaint.knobs.set(Knob::Dissolve, 0.35);
    t.paint.wetpaint.knobs.set(Knob::RewetDrag, 0.15);
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8(cor);
    t.on_canvas_pointer(ponteiro(120.0, 30.0, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(120.0, 30.0 + 2.8 * k as f32, PointerPhase::Move));
        tick(t, 1);
    }
    t.on_canvas_pointer(ponteiro(120.0, 142.0, PointerPhase::Up));
    tick(t, 600);
}

fn compara(meio1: &str, cor1: [u8; 3], meio2: &str, cor2: [u8; 3], assenta: usize) {
    let (mut b, gb) = ate_a_aguada(meio1, cor1, false, assenta);
    let (mut a, ga) = ate_a_aguada(meio1, cor1, true, assenta);
    assert!(
        gb.is_some(),
        "a fixture não tem a tinta viva: a sessão da cena B morreu sem undo"
    );
    assert!(
        ga == gb,
        "{meio1}: depois do undo do traço extra a grade da tinta viva NÃO é a de sem o traço (sessão viva: {})",
        ga.is_some()
    );
    aguada(&mut b, meio2, cor2);
    aguada(&mut a, meio2, cor2);
    let (ib, ia) = (mostrada(&mut b), mostrada(&mut a));
    let dif = ib.iter().zip(&ia).filter(|(x, y)| x != y).count();
    assert_eq!(
        dif, 0,
        "{meio1} → {meio2}: o traço por cima depois do undo deu outra imagem ({dif} bytes)"
    );
}

/// ⭐ Óleo solúvel (Dabber, Solubility 1) + undo de outro traço + aguada (Re-wet 0,35, Drag 0,15) = sem o undo.
#[test]
fn undo_de_outro_traco_nao_mata_o_oleo_soluvel() {
    compara(
        "Oil/Dabber/Dabber.ph2d-brush.png",
        [40, 90, 220],
        "Watercolor/Fluid/Wet Flat.ph2d-brush.png",
        [220, 200, 40],
        120,
    );
}

/// ⭐ Aguarela × aguarela: Wet Flat azul, undo de outro traço, Wet Flat amarelo por cima ainda molhado.
#[test]
#[ignore = "ABERTO 30/10: a grade volta igual, a imagem depois difere em 13 bytes — o estado do motor fora da grade (relógio/RNG da sim) não volta"]
fn undo_de_outro_traco_nao_seca_a_aguarela_molhada() {
    compara(
        "Watercolor/Fluid/Wet Flat.ph2d-brush.png",
        [40, 90, 220],
        "Watercolor/Fluid/Wet Flat.ph2d-brush.png",
        [220, 200, 40],
        10,
    );
}
