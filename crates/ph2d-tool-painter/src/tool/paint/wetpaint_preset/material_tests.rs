//! O material da tinta no ficheiro do preset (Óleo Real W2/W3): ida e volta por NOME e as chaves de antes da W3.
//! Filho de [`super::tests`] por LOC (E10h).

use super::tests::{ferramenta, pelo_ficheiro};
use super::*;
use ph2d_wet_paint::tuning::Knob;

/// ⭐ O material da tinta (Óleo Real W2 E3) vai e volta pelo ficheiro do preset, por NOME; o preset sem as chaves
/// (os de fábrica de antes) lê o NEUTRO e o Lock desligado.
#[test]
fn o_material_do_pincel_vai_e_volta_pelo_preset() {
    let mexidos = [
        (Knob::Metallic, 1.0),
        (Knob::Roughness, 0.4),
        (Knob::IorLevel, 0.8),
        (Knob::Coat, 0.6),
        (Knob::CoatRoughness, 0.2),
        (Knob::Sheen, 0.5),
        (Knob::Subsurface, 0.25),
        (Knob::SubsurfaceRadius, 7.5),
        (Knob::Transparency, 0.6),
        (Knob::LockMaterial, 1.0),
        // W3b: a Transmission e o Relevo do papel (ausentes = 0, o neutro)
        (Knob::Transmission, 0.7),
        (Knob::PaperRelief, 0.4),
        // o modo da Transmission (dono 28/10): ausente = 0, o Blender
        (Knob::TransmissionMode, 1.0),
    ];
    let mut a = ferramenta();
    for (k, v) in mexidos {
        a.paint.wetpaint.knobs.set(k, v);
    }
    let base = Parametros::novo(MEIO_DO_PRESET, "Teste", "Metal");
    let (prm, texturas) = a.wet_preset_capturar(&base);
    let mut b = ferramenta();
    b.wet_preset_aplicar(&pelo_ficheiro(prm.clone(), texturas.clone()));
    for (k, v) in mexidos {
        assert_eq!(b.paint.wetpaint.knobs.get(k), v, "{k:?}");
    }
    let mut sem = prm;
    for (k, _) in mexidos {
        sem.engine
            .remove(ph2d_wet_paint::tuning::KNOB_DEFS[k as usize].key);
    }
    b.wet_preset_aplicar(&pelo_ficheiro(sem, texturas));
    for (k, _) in mexidos {
        assert_eq!(
            b.paint.wetpaint.knobs.get(k),
            ph2d_wet_paint::tuning::KNOB_DEFS[k as usize].default,
            "{k:?}"
        );
    }
}

/// ⭐ O preset gravado ANTES da W3 (as chaves velhas do material) abre: `roughnessMul` m → Roughness `0,5·m`,
/// `translucency` → Subsurface, os renomeados com o mesmo valor, e os que saíram (`specularTint`, `anisotropy`)
/// ignorados.
#[test]
fn o_preset_de_antes_da_w3_abre_com_o_material_convertido() {
    let a = ferramenta();
    let base = Parametros::novo(MEIO_DO_PRESET, "Teste", "Antigo");
    let (mut prm, texturas) = a.wet_preset_capturar(&base);
    for k in [
        Knob::Roughness,
        Knob::IorLevel,
        Knob::Coat,
        Knob::CoatRoughness,
        Knob::Subsurface,
    ] {
        prm.engine
            .remove(ph2d_wet_paint::tuning::KNOB_DEFS[k as usize].key);
    }
    for (chave, v) in [
        ("roughnessMul", 1.4),
        ("specular", 0.8),
        ("specularTint", 0.3),
        ("clearcoat", 0.6),
        ("clearcoatRoughness", 0.2),
        ("anisotropy", 0.7),
        ("translucency", 0.25),
    ] {
        prm.engine.insert(chave.to_owned(), v);
    }
    let mut b = ferramenta();
    b.wet_preset_aplicar(&pelo_ficheiro(prm, texturas));
    let k = &b.paint.wetpaint.knobs;
    assert_eq!(k.get(Knob::Roughness), 0.5 * 1.4);
    assert_eq!(k.get(Knob::IorLevel), 0.8);
    assert_eq!(k.get(Knob::Coat), 0.6);
    assert_eq!(k.get(Knob::CoatRoughness), 0.2);
    assert_eq!(k.get(Knob::Subsurface), 0.25);
}
