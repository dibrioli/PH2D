//! **Os gates de PRODUTO da segunda razão** — a grade de FLUXO (plano 30).
//!
//! Irmão do [`super::grid_ratio_tests`], e a divisão é o assunto: aquele prova
//! que a grade de FLUIDO desacopla do pixel, este que a de FLUXO desacopla do
//! fluido. As duas razões são independentes de propósito — *quão fino é o
//! pigmento?* e *quão grosso é o fluxo?* são perguntas diferentes, e o gate que
//! importa mais aqui é o que prova que a segunda **não** encolhe o pigmento.

use super::*;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushSpec, Falloff};

const W: usize = 256;
const H: usize = 256;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um tool de Wet Paint com as DUAS razões pedidas, canvas branco.
fn wet_tool(grid: u8, flow: u8) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    let b = BrushSpec {
        radius_px: 48.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.8, 0.1, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_tool_mode("wetpaint");
    // Pelas PORTAS (os sliders), não pelos campos — é o caminho do artista.
    t.set_wet_grid_ratio(f64::from(grid));
    t.set_wet_flow_ratio(f64::from(flow));
    t
}

fn stroke(t: &mut PainterTool) {
    t.on_canvas_pointer(cp([60.0, 60.0], PointerPhase::Down));
    for k in 1..=8 {
        t.on_canvas_pointer(cp([60.0 + 15.0 * k as f32, 60.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([180.0, 60.0], PointerPhase::Up));
}

/// **O que a wave É**: a segunda razão encolhe a grade de VELOCIDADE e deixa a
/// de pigmento onde estava.
///
/// ⚠️ É o gate que separa esta wave do `Grid Size` que já shipou — aquele
/// barateia o fluxo encolhendo TUDO, e é por isso que ele granula a borda.
/// Mutação: fazer a sessão nascer com `Engine::new(gw, gh)` (ignorando a razão
/// de fluxo) deixa as duas grades iguais e a segunda asserção sangra.
#[test]
fn the_flow_ratio_shrinks_the_velocity_grid_and_not_the_pigment_grid() {
    for (grid, flow) in [(1u8, 1u8), (1, 4), (2, 4), (4, 2)] {
        let mut t = wet_tool(grid, flow);
        stroke(&mut t);
        let sess = t
            .paint
            .wetpaint
            .session
            .as_ref()
            .expect("uma sessao viva apos o traco");
        let (gw, gh) = grid_map::grid_dims(W, H, grid);
        assert_eq!(sess.grid, (gw, gh), "a grade de FLUIDO segue o Grid Size");
        let g = sess.engine.active_grid();
        assert_eq!(
            (g.w, g.h),
            (gw, gh),
            "o pigmento NAO pode encolher com o Flow Grid (grid {grid}, flow {flow})"
        );
        let (fw, fh) = ph2d_wet_paint::flow::flow_dims(gw, gh, usize::from(flow));
        assert_eq!(
            (g.flow.w, g.flow.h),
            (fw, fh),
            "a grade de FLUXO segue o Flow Grid (grid {grid}, flow {flow})"
        );
        assert_eq!(g.vel_x.len(), g.flow.cells, "vel mora na grade de fluxo");
    }
}

/// Trocar a razão encerra a água viva; re-emitir o mesmo valor **não**.
///
/// O guard de igualdade é o que torna seguro o chip numérico re-emitir o valor
/// a cada frame de arrasto — sem ele, arrastar o slider mataria a sessão a cada
/// quadro. Mutação: tirar o guard faz a 1ª asserção sangrar; tirar o
/// `wetpaint_end_session` faz a 3ª (o motor seguiria com a grade antiga sob uma
/// razão nova).
#[test]
fn changing_the_flow_ratio_ends_the_live_session_and_keeping_it_does_not() {
    let mut t = wet_tool(1, 1);
    stroke(&mut t);
    assert!(t.paint.wetpaint.session.is_some(), "ha agua viva");

    t.set_wet_flow_ratio(1.0);
    assert!(
        t.paint.wetpaint.session.is_some(),
        "re-emitir o mesmo valor nao pode matar a sessao"
    );

    t.set_wet_flow_ratio(4.0);
    assert!(
        t.paint.wetpaint.session.is_none(),
        "trocar a razao de fluxo tem de encerrar a sessao"
    );
    assert_eq!(t.paint.wetpaint.flow_ratio, 4);
    stroke(&mut t);
    let sess = t.paint.wetpaint.session.as_ref().expect("uma sessao nova");
    let g = sess.engine.active_grid();
    let (fw, fh) = ph2d_wet_paint::flow::flow_dims(W, H, 4);
    assert_eq!(
        (g.flow.w, g.flow.h),
        (fw, fh),
        "a sessao nova nasce com a grade de fluxo autorada"
    );
}

/// A faixa do slider é honrada pela PORTA, não pelo chamador.
#[test]
fn the_flow_ratio_door_clamps_to_the_sliders_range() {
    let mut t = wet_tool(1, 1);
    t.set_wet_flow_ratio(0.0);
    assert_eq!(
        usize::from(t.paint.wetpaint.flow_ratio),
        ph2d_wet_paint::flow::MIN_FLOW_RATIO
    );
    t.set_wet_flow_ratio(9999.0);
    assert_eq!(
        usize::from(t.paint.wetpaint.flow_ratio),
        ph2d_wet_paint::flow::MAX_FLOW_RATIO
    );
}

/// **As duas razões são independentes** — mexer numa não move a outra.
///
/// Parece trivial e não é: as duas passam pelo MESMO roteador de `SetValue` e
/// pelo mesmo `wetpaint_end_session`, e um `id` trocado ali faria o slider de
/// cima escrever no campo de baixo com todo o resto verde.
#[test]
fn the_two_ratios_do_not_write_each_other() {
    let mut t = wet_tool(1, 1);
    t.set_wet_grid_ratio(3.0);
    assert_eq!(t.paint.wetpaint.grid_ratio, 3);
    assert_eq!(t.paint.wetpaint.flow_ratio, 1, "o Grid Size mexeu no Flow");
    t.set_wet_flow_ratio(5.0);
    assert_eq!(t.paint.wetpaint.flow_ratio, 5);
    assert_eq!(t.paint.wetpaint.grid_ratio, 3, "o Flow mexeu no Grid Size");
}

/// O DEFAULT é `1` nos dois — a grade de fluxo nasce COLADA na de fluido, que é
/// o motor que sempre shipou.
///
/// ⚠️ Mudar a resolução do fluido por default mudaria o desenho de toda arte já
/// feita; o ponto de operação é do artista.
#[test]
fn both_ratios_boot_at_one() {
    let t = PainterTool::default();
    assert_eq!(t.paint.wetpaint.grid_ratio, grid_map::DEFAULT_RATIO);
    assert_eq!(t.paint.wetpaint.flow_ratio, 1);
}
