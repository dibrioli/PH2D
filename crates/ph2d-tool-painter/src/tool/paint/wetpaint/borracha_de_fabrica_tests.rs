//! **A Relief Eraser** (dono 06/11): o preset de fábrica é a Borracha que o Wet abre sem preset depois do Dabber.

use super::agua_sobre_oleo_tests::{com_oleo, mostrada, ponteiro, preset};
use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_wet_paint::tuning::Knob;

const RELIEF: &str = "Eraser/Relief/Relief Eraser.ph2d-brush.png";

/// Uma passada de Borracha sobre o óleo, com a pressão a variar, e a imagem mostrada.
fn apaga(t: &mut PainterTool) -> Vec<u8> {
    let ponto = |k: usize, phase| {
        let mut p = ponteiro(120.0 + 0.7 * k as f32, 60.0 + 2.0 * k as f32, phase);
        p.pressure = 0.3 + 0.7 * (k as f32 / 30.0);
        p
    };
    t.on_canvas_pointer(ponto(0, PointerPhase::Down));
    for k in 1..=30 {
        t.on_canvas_pointer(ponto(k, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponto(30, PointerPhase::Up));
    t.ticks_em_blocos(60);
    mostrada(t)
}

fn diferentes(a: &[u8], b: &[u8]) -> usize {
    assert_eq!(a.len(), b.len());
    a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count()
}

/// ⭐ O preset apaga ao pixel como a Borracha sem preset, e não muda o tamanho que o artista tem na mão.
#[test]
fn a_relief_eraser_apaga_como_a_borracha_sem_preset_depois_do_dabber() {
    let antes = mostrada(&mut com_oleo());

    let mut sem = com_oleo();
    sem.set_paint_tool_mode("eraser");
    let raio = sem.paint.brush.radius_px;
    let sem = apaga(&mut sem);
    assert!(
        diferentes(&antes, &sem) > 100,
        "CONTROLO: a passada tem de apagar o óleo"
    );

    let mut com = com_oleo();
    com.set_paint_tool_mode("eraser");
    com.preset_aplicar(&preset(RELIEF)).expect("o meio");
    assert!(com.paint.eraser, "a Borracha na mão");
    assert_eq!(com.paint.brush.radius_px, raio, "o tamanho do artista");
    assert_eq!(diferentes(&sem, &apaga(&mut com)), 0);
}

/// A miniatura mostra a Borracha a apagar a tela vermelha: nunca um traço de tinta azul.
#[test]
fn a_miniatura_da_relief_eraser_mostra_o_que_ela_apaga() {
    let m = crate::tool::paint::wetpaint_preset::miniatura_do_preset(&preset(RELIEF));
    let px = m.rgba.as_chunks::<4>().0;
    let conta = |f: &dyn Fn(&[u8; 4]) -> bool| px.iter().filter(|p| f(p)).count();
    let total = px.len();
    let vermelhos = conta(&|p| p[0] > 150 && p[1] < 40 && p[2] < 60);
    let brancos = conta(&|p| p[0] > 240 && p[1] > 240 && p[2] > 240);
    let azuis = conta(&|p| p[2] > p[0].saturating_add(30));
    assert!(vermelhos > total / 2, "o fundo: {vermelhos} de {total}");
    assert!(
        brancos > total / 50,
        "o rasto apagado: {brancos} de {total}"
    );
    assert_eq!(azuis, 0, "a Borracha não pinta");
}

/// O preset devolve a Borracha mexida à de fábrica.
#[test]
fn a_relief_eraser_devolve_a_borracha_mexida_a_de_fabrica() {
    let mut sem = com_oleo();
    sem.set_paint_tool_mode("eraser");
    let sem = apaga(&mut sem);

    let mexe = || {
        let mut t = com_oleo();
        t.set_paint_tool_mode("eraser");
        t.paint.wetpaint.knobs.set(Knob::PaperGate, 1.2);
        t.paint.wetpaint.knobs.erase = 0.95;
        t.paint.brush.hardness = 0.1;
        t
    };
    assert!(
        diferentes(&sem, &apaga(&mut mexe())) > 100,
        "CONTROLO: a Borracha mexida apaga outra coisa"
    );
    let mut mexida = mexe();
    mexida.preset_aplicar(&preset(RELIEF)).expect("o meio");
    assert_eq!(diferentes(&sem, &apaga(&mut mexida)), 0);
}

/// ⭐ Aplicar o preset de Borracha não mexe em nada do Pincel: o Dabber volta igual, Dilution e Volume incluídos.
#[test]
fn a_relief_eraser_nao_mexe_no_pincel() {
    let mut t = com_oleo();
    t.paint.wetpaint.knobs.set(Knob::Dilution, 0.37);
    t.paint.wetpaint.knobs.set(Knob::Volume, 0.61);
    let retrato = t.retrato_do_pincel_da_agua();
    t.set_paint_tool_mode("eraser");
    t.preset_aplicar(&preset(RELIEF)).expect("o meio");
    t.set_paint_tool_mode("wet_paint");
    assert_eq!(t.retrato_do_pincel_da_agua(), retrato);
}
