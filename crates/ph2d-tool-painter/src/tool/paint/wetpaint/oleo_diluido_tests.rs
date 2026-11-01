//! ⭐ **O óleo diluído escorre** (smoke do dono 01/11, E10g `docs/Painter/Arquitetura/E10g_oleo_diluido.md`): o
//! diluente é líquido com qualquer Paint type; o óleo corre mais devagar e fica molhado mais tempo, a tinta do tubo
//! fica onde se pôs. Tela inclinada de fábrica (o dial a apontar para baixo).

use super::agua_sobre_oleo_tests::{H, PAPEL, W, grava, mostrada, ponteiro, preset, relato};
use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PanelEvent, PointerPhase};
use ph2d_wet_paint::tuning::Knob;

const Y0: f32 = 40.0;

/// Um traço horizontal em `y = Y0` com `rel`, Dilution `d` (`None` = a do preset) e Paint type `m`.
fn traco(rel: &str, d: Option<f64>, m: f64) -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", PAPEL[0], PAPEL[1], PAPEL[2]),
    ));
    t.preset_aplicar(&preset(rel)).expect("o meio");
    if let Some(d) = d {
        t.paint.wetpaint.knobs.set(Knob::Dilution, d);
    }
    t.paint.wetpaint.knobs.set(Knob::Medium, m);
    t.set_brush_size_px(24.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    t.on_canvas_pointer(ponteiro(30.0, Y0, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(30.0 + 4.5 * k as f32, Y0, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro(210.0, Y0, PointerPhase::Up));
    t
}

/// (fracção da massa abaixo de `Y0 + 16`, a linha mais baixa com massa, a água que resta).
fn leitura(t: &mut PainterTool) -> (f64, usize, f64) {
    t.wet_bring_home();
    let g = &t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessão")
        .engine
        .layers[0]
        .grid;
    let (mut tot, mut baixo, mut fundo, mut agua) = (0.0, 0.0, 0, 0.0);
    for y in 0..H {
        for x in 0..W {
            let i = (x + 1) + (y + 1) * g.s;
            let m = f64::from(g.sett[i]) + f64::from(g.susp[i]);
            agua += f64::from(g.film[i]);
            tot += m;
            if m > 1.0 {
                fundo = fundo.max(y);
            }
            if y as f32 > Y0 + 16.0 {
                baixo += m;
            }
        }
    }
    (baixo / tot.max(1e-9), fundo, agua)
}

fn ticks(t: &mut PainterTool, n: u32) {
    for _ in 0..n {
        t.on_tick(1000.0 / 60.0);
    }
}

/// SONDA — por preset, Paint type e Dilution: quanto escorre a 2 s e a 15 s, e a água que resta.
#[test]
#[ignore = "sonda E10g"]
fn diag_o_oleo_diluido_escorre() {
    for rel in [
        "Watercolor/Fluid/Wet Round.ph2d-brush.png",
        "Oil/Flat/Filbert.ph2d-brush.png",
        "Oil/Dabber/Dabber.ph2d-brush.png",
        "Oil/Flat/Thick Impasto.ph2d-brush.png",
    ] {
        for m in [0.0, 1.0] {
            for d in [None, Some(0.3), Some(0.6), Some(0.9)] {
                let mut t = traco(rel, d, m);
                let nome = format!(
                    "{}_m{m}_d{}",
                    rel.rsplit('/')
                        .next()
                        .unwrap_or(rel)
                        .trim_end_matches(".ph2d-brush.png")
                        .replace(' ', "_"),
                    d.map_or("p".into(), |d| format!("{d}"))
                );
                ticks(&mut t, 120);
                grava(&format!("{nome}_2s"), &mostrada(&mut t));
                let a = leitura(&mut t);
                ticks(&mut t, 780);
                grava(&format!("{nome}_15s"), &mostrada(&mut t));
                let b = leitura(&mut t);
                relato(format!(
                    "{rel} | Paint type {m} | Dilution {d:?} | 2 s: abaixo {:.3} fundo {} água {:.0} | 15 s: abaixo {:.3} fundo {} água {:.0}",
                    a.0, a.1, a.2, b.0, b.1, b.2
                ));
            }
        }
    }
}

/// SONDA — o que trava o Filbert diluído: cada variante desliga UMA coisa do preset (Paint type Water, Dilution 0,9).
#[test]
#[ignore = "sonda E10g"]
fn diag_o_que_trava_o_filbert_diluido() {
    type Muda = fn(&mut PainterTool);
    let variantes: [(&str, Muda); 6] = [
        ("preset", |_| {}),
        ("sem Real Oil", |t| t.set_wet_oleo_real_ligado(false)),
        ("sem relevo", |t| {
            t.paint.wetpaint.knobs.set(Knob::Emboss, 0.0)
        }),
        ("Paper gate 0,4", |t| {
            t.paint.wetpaint.knobs.set(Knob::PaperGate, 0.4)
        }),
        ("Edge 12", |t| {
            t.paint.wetpaint.knobs.set(Knob::EdgeDarkening, 12.0)
        }),
        ("pigmento 78", |t| {
            t.paint.wetpaint.knobs.set(Knob::PigmentPerDab, 78.0)
        }),
    ];
    for (nome, muda) in variantes {
        let mut t = PainterTool::default();
        t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
        t.set_paint_tool_mode("wetpaint");
        t.set_wet_relogio_fixo(true);
        t.preset_aplicar(&preset("Oil/Flat/Filbert.ph2d-brush.png"))
            .expect("o meio");
        t.paint.wetpaint.knobs.set(Knob::Dilution, 0.9);
        t.paint.wetpaint.knobs.set(Knob::Medium, 0.0);
        muda(&mut t);
        t.set_brush_size_px(24.0);
        t.set_brush_color_srgb8([40, 90, 220]);
        t.on_canvas_pointer(ponteiro(30.0, Y0, PointerPhase::Down));
        for k in 1..=40 {
            t.on_canvas_pointer(ponteiro(30.0 + 4.5 * k as f32, Y0, PointerPhase::Move));
            t.on_tick(1000.0 / 60.0);
        }
        t.on_canvas_pointer(ponteiro(210.0, Y0, PointerPhase::Up));
        ticks(&mut t, 120);
        let a = leitura(&mut t);
        relato(format!(
            "Filbert d 0,9 Water · {nome}: 2 s abaixo {:.3} fundo {} água {:.0}",
            a.0, a.1, a.2
        ));
    }
}
