//! J9 §3 — a tinta SEM Real Oil (o meio, a solubilidade e o corpo vivem no mesmo campo `Grid::oleo`) não paga a
//! óptica do Óleo Real: a composição com o campo presente e sem uma célula de Real Oil é a do caminho sem óleo.

use super::agua_sobre_oleo_tests::{H, PAPEL, W, ponteiro, preset};
use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PanelEvent, PointerPhase};
use ph2d_wet_paint::tuning::Knob;

fn wet_drip_oleo(real_oil: bool) -> PainterTool {
    pincel_oleo("Watercolor/Fluid/Wet Drip.ph2d-brush.png", real_oil, None)
}

fn pincel_oleo(
    rel: &str,
    real_oil: bool,
    lei: Option<ph2d_wet_paint::sim::LeiDaTinta>,
) -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", PAPEL[0], PAPEL[1], PAPEL[2]),
    ));
    if let Some(l) = lei {
        t.set_wet_lei_da_tinta(l);
    }
    t.preset_aplicar(&preset(rel)).expect("o meio");
    t.paint.wetpaint.knobs.set(Knob::Dilution, 0.9);
    t.paint.wetpaint.knobs.set(Knob::Medium, 1.0);
    t.set_wet_oleo_real_ligado(real_oil);
    t.set_brush_size_px(24.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    t.on_canvas_pointer(ponteiro(30.0, 60.0, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(30.0 + 4.5 * k as f32, 60.0, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro(210.0, 60.0, PointerPhase::Up));
    for _ in 0..90 {
        t.on_tick(1000.0 / 60.0);
    }
    t
}

/// A tela inteira recomposta; `forca` = a lei da caixa com o Real Oil na mão (o caminho do óleo sempre).
fn recompoe(t: &mut PainterTool, forca: bool) -> (Vec<u8>, bool) {
    let antes = t.paint.wetpaint.oleo_real.is_some();
    if forca {
        t.set_wet_oleo_real_ligado(true);
    }
    super::oleo_plano::lei_da_caixa::set(forca);
    let s = t.paint.wetpaint.session.as_mut().expect("a sessão");
    s.bring_home();
    s.engine.mark_dirty_full();
    t.wetpaint_composite();
    super::oleo_plano::lei_da_caixa::set(false);
    t.set_wet_oleo_real_ligado(antes);
    (t.canvas_rgba.to_vec(), t.paint.wetpaint.oleo_na_tela)
}

/// ⭐ Medido antes da cura (Wet Drip Oil d0,9, sonda `desempenho_oleo_tests`): o campo de altura e a sombra eram 6,6 s
/// dos 7,3 s de composição em 120 s, numa tela sem uma célula de Real Oil (o campo existia pelo meio). O caminho do
/// óleo e o sem óleo dão a MESMA tela ao byte sem Real Oil; com ele (o controlo) 4 465 bytes mudam.
#[test]
fn a_tinta_sem_real_oil_nao_paga_a_optica_do_oleo() {
    let mut t = wet_drip_oleo(false);
    let (produto, na_tela) = recompoe(&mut t, false);
    assert!(
        !na_tela,
        "sem Real Oil a composição tomou o caminho do óleo"
    );
    let (oleo, _) = recompoe(&mut t, true);
    let difs = produto.iter().zip(&oleo).filter(|(a, b)| a != b).count();
    assert_eq!(
        difs, 0,
        "o caminho do óleo muda {difs} bytes de uma tela sem Real Oil"
    );
    let mut t = wet_drip_oleo(true);
    let (produto, na_tela) = recompoe(&mut t, false);
    assert!(na_tela, "com Real Oil a composição saiu do caminho do óleo");
    t.set_wet_oleo_real_ligado(false);
    super::oleo_plano::lei_da_caixa::set(true);
    let s = t.paint.wetpaint.session.as_mut().expect("a sessão");
    s.bring_home();
    s.engine.mark_dirty_full();
    t.wetpaint_composite();
    super::oleo_plano::lei_da_caixa::set(false);
    let difs = produto
        .iter()
        .zip(t.canvas_rgba.iter())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        difs > 1000,
        "controlo: o Real Oil já não muda a tela ({difs} bytes)"
    );
}

#[test]
#[ignore = "sonda J9 §3"]
fn diag_os_dois_caminhos_por_lei() {
    use ph2d_wet_paint::sim::LeiDaTinta;
    for lei in [LeiDaTinta::Velha, LeiDaTinta::Pigmento] {
        for rel in [
            "Watercolor/Fluid/Wet Drip.ph2d-brush.png",
            "Oil/Flat/Thick Impasto.ph2d-brush.png",
        ] {
            let mut t = pincel_oleo(rel, false, Some(lei));
            let (a, na) = recompoe(&mut t, false);
            let (b, _) = recompoe(&mut t, true);
            let difs = a.iter().zip(&b).filter(|(x, y)| x != y).count();
            let max = a
                .iter()
                .zip(&b)
                .map(|(x, y)| x.abs_diff(*y))
                .max()
                .unwrap_or(0);
            eprintln!("{lei:?} {rel}: na tela {na} · bytes diferentes {difs} · máx {max}");
        }
    }
}
