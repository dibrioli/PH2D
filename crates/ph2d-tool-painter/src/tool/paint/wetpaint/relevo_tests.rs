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

/// Bug #48 (dono 11/10): o relevo é da TINTA que o pincel pousou, não do pincel de agora. Pinta com o
/// Oil Bristle (Emboss 0,7), troca para o Wet Round (sem relevo) e pinta ao lado: o 1.º traço guarda-se
/// ao byte. Controlo: o 1.º traço TEM relevo (o mesmo traço com o Emboss a 0 é outro).
#[test]
fn o_relevo_fica_com_a_tinta_ao_trocar_para_um_pincel_sem_relevo() {
    const H2: usize = 160;
    let b = super::mistura_wet_drip_tests::biblioteca();
    let preset = |nome: &str| {
        b.entradas
            .iter()
            .find(|e| e.preset.parametros.name == nome)
            .expect("o pincel de fábrica")
            .preset
            .clone()
    };
    let p = |x: f32, y: f32, phase| CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let traco_em = |t: &mut PainterTool, y: f32| {
        t.on_canvas_pointer(p(30.0, y, PointerPhase::Down));
        for k in 1..=60 {
            t.on_canvas_pointer(p(30.0 + 3.0 * k as f32, y, PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(p(210.0, y, PointerPhase::Up));
        for _ in 0..60 {
            t.on_tick(1000.0 / 60.0);
        }
    };
    let oleo = |emboss: Option<f64>| {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; W * H2 * 4], W as u32, H2 as u32);
        t.set_paint_tool_mode("wetpaint");
        t.set_wet_relogio_fixo(true);
        assert!(t.wet_preset_aplicar(&preset("Oil Bristle")).is_empty());
        if let Some(v) = emboss {
            t.paint.wetpaint.knobs.set(Knob::Emboss, v);
        }
        assert!(t.paint.wetpaint.knobs.get(Knob::Emboss) > 0.0 || emboss.is_some());
        t.set_brush_size_px(14.0);
        t.set_brush_color_srgb8([40, 60, 160]);
        traco_em(&mut t, 40.0);
        // Seca (o botão Dry): a tinta parada, o que muda depois é só o que a troca de pincel faz.
        t.wetpaint_dry_canvas();
        for _ in 0..4 {
            t.on_tick(1000.0 / 60.0);
        }
        t
    };
    // As linhas do 1.º traço (y = 40 ± o raio e a água): longe do 2.º (y = 120).
    let faixa = |c: &[u8]| c[..80 * W * 4].to_vec();
    let mut t = oleo(None);
    let antes = faixa(&t.canvas_rgba);
    let plano = faixa(&oleo(Some(0.0)).canvas_rgba);
    assert!(
        antes != plano,
        "controlo: o traço do Oil Bristle tem relevo"
    );
    assert!(t.wet_preset_aplicar(&preset("Wet Round")).is_empty());
    assert_eq!(
        t.paint.wetpaint.knobs.get(Knob::Emboss),
        0.0,
        "o Wet Round não tem relevo"
    );
    t.set_brush_size_px(14.0);
    t.set_brush_color_srgb8([180, 30, 30]);
    traco_em(&mut t, 120.0);
    let fundo = &t.canvas_rgba[120 * W * 4..121 * W * 4];
    assert!(
        fundo.chunks(4).any(|px| px != [255, 255, 255, 255]),
        "o 2.º traço pintou"
    );
    let depois = faixa(&t.canvas_rgba);
    let mudou = antes
        .chunks(4)
        .zip(depois.chunks(4))
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        mudou, 0,
        "trocar para um pincel sem relevo mudou {mudou} px do 1.º traço"
    );
}
