//! B14.4: o **Light angle** do cartão manda também no emboss da tinta SEM óleo; voltar à omissão é o mesmo byte.

use super::*;

/// O Light angle (`PAINTER_WETPAINT_OLEO_FIELDS[2]`, graus).
const AZIMUTE: usize = 2;

fn tela_sem_oleo(w: usize, h: usize) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![240u8; w * h * 4], w as u32, h as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(14.0);
    t.paint
        .wetpaint
        .knobs
        .set(ph2d_wet_paint::tuning::Knob::Emboss, 1.0);
    t.set_brush_color_srgb8([180, 40, 30]);
    let p = |k: f32, phase| CanvasPointer {
        pos: [0.1 * w as f32 + 0.8 * w as f32 * k, 0.5 * h as f32],
        pressure: 0.9,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(0.0, PointerPhase::Down));
    for i in 1..=30 {
        t.on_canvas_pointer(p(i as f32 / 30.0, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(p(1.0, PointerPhase::Up));
    t.on_tick(0.0);
    assert!(!t.paint.wetpaint.oleo_na_tela, "a cena é sem óleo");
    t
}

fn com_luz(t: &mut PainterTool, graus: f64) -> Vec<u8> {
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WETPAINT_OLEO_FIELDS[AZIMUTE],
        graus,
    ));
    varre(t);
    t.wet_pigmento().expect("há tinta").to_vec()
}

#[test]
fn o_light_angle_roda_o_relevo_da_tinta_sem_oleo() {
    let mut t = tela_sem_oleo(120, 80);
    oleo_no_quadro::faixa_fixa(Some(usize::MAX));
    let antes = com_luz(&mut t, 225.0);
    let oposta = com_luz(&mut t, 45.0);
    let difere = antes.iter().zip(&oposta).filter(|(a, b)| a != b).count();
    assert!(
        difere > 0,
        "o emboss da tinta sem óleo não segue o Light angle"
    );
    assert!(
        com_luz(&mut t, 225.0) == antes,
        "de volta à omissão, outro byte"
    );
    oleo_no_quadro::faixa_fixa(None);
}
