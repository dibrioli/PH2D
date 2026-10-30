//! ⭐ **As rampas NOVAS do Painter nascem em OKLCH** (smoke do dono 30/10: as Grain Colors azul → amarelo saíam
//! esbranquiçadas — o modo RGB passa por um cinza a meio; o OKLCH roda pela matiz, azul → verde → amarelo). A rampa
//! partilhada do `ph2d-color` fica RGB (outros módulos); o que um preset ou projeto antigo gravou lê-se como era.

use crate::tool::PainterTool;
use ph2d_color::{ColorRamp, RampColorMode};

#[test]
fn as_rampas_novas_do_painter_nascem_em_oklch() {
    let mut t = PainterTool::default();
    let modos = |t: &PainterTool| {
        [
            t.texture_ramp().color_mode,
            t.shape_color_ramp().color_mode,
            t.shape_falloff_ramp().color_mode,
        ]
    };
    assert_eq!(modos(&t), [RampColorMode::Oklch; 3], "uma ferramenta nova");
    assert_eq!(
        crate::layers::TextureLayer::default().ramp.color_mode,
        RampColorMode::Oklch,
        "uma camada de textura nova"
    );
    t.set_texture_ramp(ColorRamp::default());
    t.reset_brush_color_ramp();
    assert_eq!(t.texture_ramp().color_mode, RampColorMode::Oklch, "o Reset");
    assert_eq!(
        super::shape_ramp::EstadoDaShapeColor::default().para_json(),
        t.shape_color_estado().para_json(),
        "a Shape Color de fábrica é a de uma ferramenta nova"
    );
    assert_eq!(
        ColorRamp::default().color_mode,
        RampColorMode::Rgb,
        "a rampa partilhada não muda"
    );
}

/// O que um preset de ANTES gravou: as Grain Colors e a Shape Color com a rampa de então (RGB) lêem-se como eram; um
/// preset sem elas (`None`) deixa-as desligadas — a rampa de fábrica não pinta nada.
#[test]
fn um_preset_antigo_le_as_rampas_como_eram() {
    let velha = ColorRamp::default();
    let mut t = PainterTool::default();
    t.grain_color_de_json(Some(
        &serde_json::json!({ "ligada": true, "bw": false, "rampa": velha }),
    ));
    assert_eq!(t.texture_ramp(), &velha);
    t.grain_color_de_json(None);
    assert!(!t.paint.texture_ramp_enabled);
    assert!(
        t.grain_color_para_json().is_none(),
        "a de fábrica grava nada"
    );

    let mut sc = super::shape_ramp::EstadoDaShapeColor::default();
    let mut v = sc.para_json();
    v["rampa"] = serde_json::to_value(&velha).unwrap();
    v["queda_rampa"] = serde_json::to_value(&velha).unwrap();
    sc.ler_json(&v);
    t.set_shape_color_estado(&sc);
    assert_eq!(t.shape_color_ramp(), &velha);
    assert_eq!(t.shape_falloff_ramp(), &velha);

    let camada: crate::layers::TextureLayer = serde_json::from_value({
        let mut c = serde_json::to_value(crate::layers::TextureLayer::default()).unwrap();
        c["ramp"] = serde_json::to_value(&velha).unwrap();
        c
    })
    .unwrap();
    assert_eq!(camada.ramp, velha, "a camada de um projeto antigo");
}
