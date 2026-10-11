//! **O relevo da tinta voltou ao Tuning** (dono 11/10: «tirou o relevo do papel junto. é só trazer de
//! volta»): a secção Relief, o slider Emboss pelo evento REAL do painel, morde SEM papel escolhido — o
//! relevo é da tinta (`ph2d_wet_paint::render::PigmentVisual::relevo`), não do papel. E nasce desligado:
//! o traço de quem não mexe nele é o de antes.

use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase};
use ph2d_wet_paint::tuning::Knob;

const W: usize = 240;
const H: usize = 120;

fn traco(emboss: Option<f64>) -> Vec<u8> {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(14.0);
    t.set_brush_color_srgb8([180, 30, 30]);
    t.paint.wetpaint.knobs.set(Knob::PigmentPerDab, 1500.0);
    assert!(!t.paint.brush.paper.is_active(), "sem papel escolhido");
    if let Some(v) = emboss {
        t.handle_panel_event(PanelEvent::SetValue(
            crate::ids::wet_tuning_slider_id("emboss"),
            v,
        ));
    }
    let p = |x: f32, phase| CanvasPointer {
        pos: [x, 60.0],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(30.0, PointerPhase::Down));
    for k in 1..=60 {
        t.on_canvas_pointer(p(30.0 + 3.0 * k as f32, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(p(210.0, PointerPhase::Up));
    for _ in 0..60 {
        t.on_tick(1000.0 / 60.0);
    }
    t.canvas_rgba.to_vec()
}

#[test]
fn o_emboss_morde_sem_papel_e_nasce_desligado() {
    assert_eq!(
        WetKnobs::DEFAULT.get(Knob::Emboss),
        0.0,
        "o relevo nasce desligado"
    );
    let base = traco(None);
    let zero = traco(Some(0.0));
    let forte = traco(Some(3.0));
    assert!(base == zero, "o Emboss a 0 tem de ser o traço de antes");
    let mudou = base
        .chunks(4)
        .zip(forte.chunks(4))
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        mudou > 200,
        "o slider Emboss a 3 não mudou a tinta sem papel ({mudou} px)"
    );
}
