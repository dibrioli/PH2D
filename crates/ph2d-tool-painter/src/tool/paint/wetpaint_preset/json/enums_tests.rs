//! B50.1: todo enum do pincel grava-se pelo NOME da variante (a lei do [`super`]); um número no ficheiro não se
//! lê (§0.11: sem leitor do formato velho).

use super::{pincel_de_json, pincel_para_json};
use crate::tool::paint::wetpaint_preset::campos_da_macro;
use ph2d_painter_brush::{
    BrushSpec, DabRotation, DualMode, DualTip, GrainMode, HoseMode, ImageHose, PixelTip,
};
use serde_json::{Value, json};

fn mexido() -> BrushSpec {
    let mut b = BrushSpec {
        pixel: PixelTip::Perfect,
        hose: ImageHose {
            n: 3,
            mode: HoseMode::Pressure,
            ..ImageHose::NEUTRA
        },
        dual: Some(DualTip {
            modo: DualMode::Darken,
            ..DualTip::DE_FABRICA
        }),
        ..BrushSpec::default()
    };
    b.texture.lei.mode = GrainMode::Height;
    b.pen_tilt.rotation = DabRotation::PenTilt;
    // O Grain aplicado sem troca é o da textura (a ausência no ficheiro diz isso).
    b.grao_aplicado.grao = b.texture;
    b
}

#[test]
fn os_enums_do_preset_gravam_o_nome() {
    let b = mexido();
    let campos = campos_da_macro();
    let m = pincel_para_json(&b, &campos);
    let sub = |k: &str, s: &[&str]| s.iter().try_fold(m.get(k)?, |v, s| v.get(*s)).cloned();
    let gravados = [
        ("pixel", sub("pixel", &[]), "Perfect"),
        ("hose.mode", sub("hose", &["mode"]), "Pressure"),
        (
            "texture.lei.mode",
            sub("texture", &["lei", "mode"]),
            "Height",
        ),
        ("dual.modo", sub("dual", &["modo"]), "Darken"),
        (
            "pen_tilt.rotation",
            sub("pen_tilt", &["rotation"]),
            "PenTilt",
        ),
    ];
    let pelo_numero: Vec<_> = gravados
        .iter()
        .filter(|(_, v, nome)| v.as_ref() != Some(&Value::String((*nome).to_owned())))
        .map(|(k, v, _)| (*k, v.clone()))
        .collect();
    assert!(
        pelo_numero.is_empty(),
        "enum gravado sem o nome: {pelo_numero:?}"
    );
    assert_eq!(pincel_de_json(&m, &campos, BrushSpec::default()), b);

    // O número já não é lido: a variante de `base` (a de fábrica) fica.
    let mut velho = m;
    velho.insert("pixel".to_owned(), json!(2));
    let lido = pincel_de_json(&velho, &campos, BrushSpec::default());
    assert_eq!(lido.pixel, PixelTip::Off);
}
