//! ⭐ **A aguada RE-MOLHA o óleo** (dono 30/10, J10; prancha escolhida `docs/Painter/Pinceis/pranchas/J10_remolha_*.png`,
//! «re-molha médio»): a água do pincel dissolve `Re-wet × Solubility` do assentado solúvel debaixo dela; o
//! dissolvido viaja na água para fora do óleo. Cena = a de `agua_sobre_oleo_tests` (Dabber azul, Wet Flat amarelo).

use super::agua_sobre_oleo_tests::{H, PAPEL, W, grava, mostrada, ponteiro, preset, relato};
use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PanelEvent, PointerPhase};
use ph2d_wet_paint::tuning::Knob;

/// O óleo azul (Solubility `soluvel`) e, por cima, a aguada amarela (Re-wet `dissolve`, Drag `arrasto`), a meio.
fn cena(papel: [u8; 3], soluvel: f64, dissolve: f64, arrasto: f64) -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", papel[0], papel[1], papel[2]),
    ));
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o meio");
    t.paint.wetpaint.knobs.set(Knob::Solubility, soluvel);
    t.set_brush_size_px(30.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    t.on_canvas_pointer(ponteiro(40.0, 90.0, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(40.0 + 4.0 * k as f32, 90.0, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro(200.0, 90.0, PointerPhase::Up));
    for _ in 0..120 {
        t.on_tick(1000.0 / 60.0);
    }
    t.preset_aplicar(&preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png"))
        .expect("o meio");
    t.paint.wetpaint.knobs.set(Knob::Dissolve, dissolve);
    t.paint.wetpaint.knobs.set(Knob::RewetDrag, arrasto);
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([220, 200, 40]);
    t.on_canvas_pointer(ponteiro(120.0, 30.0, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(120.0, 30.0 + 2.8 * k as f32, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t
}

/// Acaba o traço da [`cena`] e deixa assentar.
fn assenta(t: &mut PainterTool) {
    t.on_canvas_pointer(ponteiro(120.0, 142.0, PointerPhase::Up));
    for _ in 0..600 {
        t.on_tick(1000.0 / 60.0);
    }
}

/// O assentado AZUL debaixo da aguada (`x 108..132`, as linhas do óleo `78..102`) e o pigmento azul que saiu do óleo
/// para a água (as linhas da aguada acima `40..70` e abaixo `112..140`: massa × a falta de vermelho da cor).
fn medidas(t: &mut PainterTool) -> (f64, f64) {
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
    let azul = |c: [f32; 3]| (f64::from(c[2]) - f64::from(c[0].max(c[1]))).max(0.0) / 255.0;
    let celula = |x: usize, y: usize| (x + 1) + (y + 1) * g.s;
    let mut oleo = 0.0;
    for y in 78..102 {
        for x in 108..132 {
            let i = celula(x, y);
            oleo += f64::from(g.sett[i]) * azul(g.sett_rgb[i]);
        }
    }
    // Fora do óleo o azul chega MISTURADO com o amarelo (verde, azul-ardósia): conta-se a falta de VERMELHO, que separa
    // o amarelo da aguada (r 220) do azul e do verde (r ≤ ~90) — a dominância do azul lia o verde como nada.
    let sem_vermelho = |c: [f32; 3]| (1.0 - f64::from(c[0]) / 255.0).max(0.0);
    let mut fora = 0.0;
    for y in (40..70).chain(112..140) {
        for x in 108..132 {
            let i = celula(x, y);
            fora += f64::from(g.sett[i]) * sem_vermelho(g.sett_rgb[i])
                + f64::from(g.susp[i]) * sem_vermelho(g.susp_rgb[i]);
        }
    }
    (oleo, fora)
}

/// ⛔ **O neutro é a tinta de sempre, ao bit:** Re-wet `0` (com o óleo solúvel) e Solubility `0` (com a água que
/// re-molha) dão a imagem de hoje.
#[test]
fn sem_re_molhar_a_aguada_sobre_o_oleo_e_a_de_sempre() {
    let hoje = {
        let mut t = cena(PAPEL, 0.0, 0.0, 0.0);
        assenta(&mut t);
        mostrada(&mut t)
    };
    for (soluvel, dissolve, arrasto) in [(1.0, 0.0, 0.0), (0.0, 0.35, 0.5)] {
        let mut t = cena(PAPEL, soluvel, dissolve, arrasto);
        assenta(&mut t);
        assert!(
            mostrada(&mut t) == hoje,
            "Solubility {soluvel} × Re-wet {dissolve} × Drag {arrasto} mudou a imagem"
        );
    }
}

/// ⭐ **Re-wet 0,35 × Solubility 1 dissolve ~35 % do azul debaixo da aguada e leva-o para a água** (a prancha
/// «re-molha médio»): o assentado azul sob a aguada cai 25–45 % e a aguada fora do óleo fica com azul.
#[test]
fn a_aguada_re_molha_o_oleo_soluvel() {
    let (oleo0, fora0) = {
        let mut t = cena(PAPEL, 0.0, 0.0, 0.0);
        assenta(&mut t);
        medidas(&mut t)
    };
    let mut t = cena(PAPEL, 1.0, 0.35, 0.0);
    assenta(&mut t);
    let (oleo, fora) = medidas(&mut t);
    let caiu = 1.0 - oleo / oleo0;
    assert!(
        (0.25..=0.45).contains(&caiu),
        "o azul debaixo da aguada caiu {caiu:.3} (alvo 0,35)"
    );
    assert!(
        fora > 1.15 * fora0,
        "o azul dissolvido não saiu do óleo: fora {fora:.0} (hoje {fora0:.0})"
    );
}

/// SONDA — a cena a meio e assentada com Re-wet 0,35 e Drag 0 · 0,15 · 0,3 · 0,5 (Solubility 1; `rw00` = hoje, sem
/// nada), no bege (`p`) e no branco (`b`).
#[test]
#[ignore = "diagnóstico"]
fn diag_a_aguada_re_molha_o_oleo() {
    for (pre, papel) in [("p", PAPEL), ("b", [255; 3])] {
        for (nome, d, a) in [
            ("00", 0.0, 0.0),
            ("15", 0.35, 0.15),
            ("30", 0.35, 0.3),
            ("50", 0.35, 0.5),
        ] {
            let mut t = cena(papel, 1.0, d, a);
            grava(&format!("{pre}_rw{nome}_meio"), &mostrada(&mut t));
            assenta(&mut t);
            grava(&format!("{pre}_rw{nome}_fim"), &mostrada(&mut t));
            let (oleo, fora) = medidas(&mut t);
            relato(format!(
                "{pre} Re-wet {d} Drag {a}: azul sob a aguada {oleo:.0} · azul fora do óleo {fora:.0}"
            ));
        }
    }
}
