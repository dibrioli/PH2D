//! ⭐ **Over Oil — a aguada por cima do óleo** (dono 30/10: «o melhor dos dois mundos»; as três escolhas da prancha
//! `docs/Painter/Pinceis/pranchas/J8_aguada_sobre_oleo_4_variantes.jpg` viram um dropdown). Antes, a aguada amarela
//! sobre o óleo azul saía CREME: a lei `Cobre` pinta o tapar extra com a tinta diluída em branco.

use super::agua_sobre_oleo_tests::{aguada_por_cima_do_oleo, na_faixa, ponteiro, preset};
use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

/// ⭐ **Mix (o de fábrica): amarelo sobre azul = VERDE** a meio do traço, na faixa da aguada sobre o óleo; **Tint**
/// tinge (fica azul-petróleo) e **Blend** fica entre os dois. Medido 30/10 (média `[r,g,b]` da faixa): Tint
/// `[63,107,170]` · Mix `[72,132,105]` · Blend `[67,119,137]`; a lei de antes (o tapar com branco) era creme.
#[test]
fn a_aguada_sobre_o_oleo_mistura_as_cores() {
    let [tinge, mistura, meio] = [0u8, 1, 2].map(|v| na_faixa(&aguada_por_cima_do_oleo(v).0).0);
    let [r, g, b] = mistura.map(i64::from);
    assert!(
        g > r + 30 && g > b + 15,
        "Mix: amarelo sobre azul tem de dar verde, deu {mistura:?}"
    );
    assert!(
        tinge[2] > meio[2] && meio[2] > mistura[2],
        "o azul: Tint {tinge:?} > Blend {meio:?} > Mix {mistura:?}"
    );
}

/// Um traço amarelo que CRUZA tinta assentada, com o **Over Oil** `v`: sem óleo nenhum (um Wet Flat azul por baixo e o
/// Wet Flat por cima), ou o óleo de [`super::agua_sobre_oleo_tests::com_oleo`] cruzado por um pincel COM o Real Oil.
fn cruza(v: u8, com_oleo: bool) -> Vec<u8> {
    let mut t = if com_oleo {
        super::agua_sobre_oleo_tests::com_oleo()
    } else {
        let mut t = tela_de_cor();
        t.preset_aplicar(&preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png"))
            .expect("o meio");
        t.set_brush_size_px(30.0);
        t.set_brush_color_srgb8([40, 90, 220]);
        traco(&mut t, (40.0, 90.0), (200.0, 90.0), 120);
        t
    };
    let de_cima = if com_oleo {
        "Oil/Dabber/Dabber.ph2d-brush.png"
    } else {
        "Watercolor/Fluid/Wet Flat.ph2d-brush.png"
    };
    t.preset_aplicar(&preset(de_cima)).expect("o meio");
    t.set_wet_sobre_oleo(v);
    t.set_brush_size_px(30.0);
    t.set_brush_color_srgb8([220, 200, 40]);
    traco(&mut t, (120.0, 30.0), (120.0, 142.0), 60);
    t.wet_bring_home();
    t.canvas_rgba.to_vec()
}

/// A tela vazia no papel de cor de [`super::agua_sobre_oleo_tests::com_oleo`].
fn tela_de_cor() -> PainterTool {
    use super::agua_sobre_oleo_tests::{H, PAPEL, W};
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.handle_panel_event(ph2d_editor_core::tool::PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", PAPEL[0], PAPEL[1], PAPEL[2]),
    ));
    t
}

fn traco(t: &mut PainterTool, de: (f32, f32), ate: (f32, f32), assenta: u32) {
    t.on_canvas_pointer(ponteiro(de.0, de.1, PointerPhase::Down));
    for k in 1..=40 {
        let f = k as f32 / 40.0;
        t.on_canvas_pointer(ponteiro(
            de.0 + (ate.0 - de.0) * f,
            de.1 + (ate.1 - de.1) * f,
            PointerPhase::Move,
        ));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro(ate.0, ate.1, PointerPhase::Up));
    for _ in 0..assenta {
        t.on_tick(1000.0 / 60.0);
    }
}

/// ⭐ **Neutro ao bit fora do óleo**: a aguada sobre tinta SEM óleo, e o pincel COM o Real Oil por cima do óleo, pintam
/// o mesmo byte nas três escolhas — e o de ANTES do Over Oil (as impressões tiradas com `com_oleo` desligado, J8).
#[test]
fn fora_do_oleo_o_over_oil_nao_muda_um_byte() {
    for com_oleo in [false, true] {
        let [a, b, c] = [0u8, 1, 2].map(|v| cruza(v, com_oleo));
        assert!(
            a == b && b == c,
            "Over Oil mexeu (óleo debaixo com o Real Oil na mão: {com_oleo})"
        );
        let h = a.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &x| {
            (h ^ u64::from(x)).wrapping_mul(0x0100_0000_01b3)
        });
        assert_eq!(
            h,
            if com_oleo {
                0x8dbc_ddfd_2e91_07ce
            } else {
                0xb8bd_7def_6b32_0385
            },
            "a lei de antes mudou (óleo debaixo com o Real Oil na mão: {com_oleo}): {h:#018x}"
        );
    }
}

/// O pincel GRAVA o Over Oil só quando sai do de fábrica, e aplicá-lo traz o dele — ou o de fábrica, num preset de
/// antes dele (um preset é o pincel inteiro).
#[test]
fn o_preset_leva_o_over_oil() {
    let mut t = PainterTool::default();
    let fabrica = preset("Watercolor/Fluid/Wet Flat.ph2d-brush.png");
    assert_eq!(fabrica.parametros.over_oil, None);
    t.preset_aplicar(&fabrica).expect("o meio");
    assert_eq!(t.wet_sobre_oleo(), 1, "o de fábrica é o Mix");
    let base = || ph2d_brush_presets::Parametros::novo("WetPainter", "Teste", "Over Oil");
    assert_eq!(t.wet_preset_capturar(&base()).0.over_oil, None);
    t.set_wet_sobre_oleo(2);
    assert_eq!(t.wet_preset_capturar(&base()).0.over_oil, Some(2));
    let mut tinge = fabrica.clone();
    tinge.parametros.over_oil = Some(0);
    t.preset_aplicar(&tinge).expect("o meio");
    assert_eq!(t.wet_sobre_oleo(), 0);
    t.preset_aplicar(&fabrica).expect("o meio");
    assert_eq!(
        t.wet_sobre_oleo(),
        1,
        "o preset de antes volta ao de fábrica"
    );
}
