use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

/// Um recto à pressão `p` (3 px por evento) com a linha `pressureJitter` a `j`: a tela inteira.
fn recto(p: f32, j: f64) -> Vec<u8> {
    let mut t = super::super::super::PainterTool::default();
    t.set_source(vec![255u8; 600 * 120 * 4], 600, 120);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_size_px(16.0);
    t.set_mao(LinhaDaMao::PressureJitter, j);
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, 60.0],
        pressure: p,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(20.0, PointerPhase::Down));
    let mut x: f32 = 20.0;
    while x < 580.0 {
        x += 3.0;
        t.on_canvas_pointer(cp(x.min(580.0), PointerPhase::Move));
        t.paint_tick(0.008);
    }
    t.on_canvas_pointer(cp(580.0, PointerPhase::Up));
    t.canvas_rgba.to_vec()
}

/// As linhas da tela com tinta (algum texel abaixo de 250 no vermelho).
fn linhas_com_tinta(px: &[u8]) -> usize {
    (0..120)
        .filter(|y| (100..500).any(|x| px[(y * 600 + x) * 4] < 250))
        .count()
}

/// ⭐ **A pressão leve espalha** (`Dynamics::jitter_pressure`; a esferográfica do MyPaint, `offset_by_random`
/// 0,354 → 0 com a pressão): à pressão 0,2 a tinta espalha-se por mais linhas do que sem a lei; à pressão 1 a
/// lei não move um dab (a tela é a de sem ela, ao byte).
#[test]
fn a_pressao_leve_espalha_e_a_cheia_nao() {
    let (sem, com) = (
        linhas_com_tinta(&recto(0.2, 0.0)),
        linhas_com_tinta(&recto(0.2, 0.5)),
    );
    assert!(
        com >= sem + 4,
        "à pressão 0,2 a lei não espalhou: {sem} → {com} linhas"
    );
    assert!(
        recto(1.0, 0.0) == recto(1.0, 0.5),
        "à pressão 1 a lei mexeu na tinta"
    );
}
