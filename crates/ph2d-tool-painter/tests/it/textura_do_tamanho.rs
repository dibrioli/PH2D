//! ⭐ **A textura acompanha o TAMANHO do pincel** (Óleo Real W3 K2, `docs/Painter/Pinceis/W3_K2_textura.md`): o
//! grão `Tiled` cresce `(D ÷ ref)^γ` com o diâmetro do pincel (`textureSizeExp`/`textureSizeRef`), e a marca pequena
//! do spray pousa (era cortada do tapete de cerdas e o portão do papel comia-a). O neutro (`γ = 0`, sem spray) é o
//! gate da fábrica inteira, `presets_de_fabrica::a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`.

use ph2d_brush_presets::{Biblioteca, Preset};
use ph2d_tool_painter::miniatura_do_preset;
use serde_json::{Value, json};
use std::path::PathBuf;

pub(crate) fn dabber() -> Preset {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    Biblioteca::carregar(&raiz, None)
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == "Dabber")
        .expect("o Dabber de fábrica")
        .preset
}

/// O Dabber com o grão Stucci `Tiled` (o do candidato Dabber 3), ao diâmetro `d` e com o expoente `gama`.
fn com_grao(d: f64, gama: Option<f64>) -> Preset {
    let mut p = dabber();
    p.parametros.volumes.size_px = Some(d);
    p.parametros.brush.insert(
        "texture".into(),
        json!({"angle_deg": 0, "flow": false, "kind": "Stucci", "mapping": "Tiled", "offset": [0.0, 0.0],
               "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [45.0, 45.0],
               "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}),
    );
    if let Some(g) = gama {
        p.parametros.engine.insert("textureSizeExp".into(), g);
        p.parametros.engine.insert("textureSizeRef".into(), 24.0);
    }
    p
}

/// A tinta de um píxel (`0` = papel branco, `1` = preto).
pub(crate) fn tinta(px: &[u8]) -> f64 {
    1.0 - f64::from(u16::from(px[0]) + u16::from(px[1]) + u16::from(px[2])) / 765.0
}

/// O TAMANHO do grão (px): os píxeis de tinta ÷ as transições tinta↔papel ao longo das linhas — o inverso da
/// densidade de bordas (a orla do traço entra igual nas duas leis, por isso compara-se a RAZÃO entre tamanhos).
pub(crate) fn tamanho_do_grao(p: &Preset) -> f64 {
    let m = miniatura_do_preset(p);
    let lado = m.largura as usize;
    let t: Vec<bool> = m.rgba.chunks(4).map(|px| tinta(px) >= 0.25).collect();
    let pintados = t.iter().filter(|&&v| v).count();
    let bordas = t
        .chunks(lado)
        .map(|linha| linha.windows(2).filter(|w| w[0] != w[1]).count())
        .sum::<usize>();
    assert!(bordas > 0, "a miniatura não tem traço");
    pintados as f64 / bordas as f64
}

/// O grão `Tiled` com `γ = 1` cresce com o pincel: de 24 para 96 px (×4) o tamanho do grão cresce 1,5× mais do que
/// com o grão preso aos px da tela (`γ = 0`). E `γ = 0` escrito é o preset sem a chave, ao byte.
#[test]
fn o_grao_tiled_cresce_com_o_pincel() {
    assert_eq!(
        miniatura_do_preset(&com_grao(96.0, Some(0.0))),
        miniatura_do_preset(&com_grao(96.0, None)),
        "γ = 0 tem de ser o grão de sempre, ao byte"
    );
    let fixo = tamanho_do_grao(&com_grao(96.0, None)) / tamanho_do_grao(&com_grao(24.0, None));
    let segue =
        tamanho_do_grao(&com_grao(96.0, Some(1.0))) / tamanho_do_grao(&com_grao(24.0, Some(1.0)));
    assert!(
        segue > 1.5 * fixo,
        "o tamanho do grão 96 ÷ 24: γ = 1 {segue:.2}, γ = 0 {fixo:.2} (esperado ≥ 1,5×; medido 1,85×)"
    );
}

/// A marca pequena do SPRAY (`particle` 0,1) de um pincel de CERDAS de tapete fraco (o candidato Metallic Splats:
/// `bristleStrength` 0, `felt` 0,05) pousa: dez marcas por ponto cobrem pelo menos um quarto da tinta do pincel
/// sem spray (antes da cura, na bancada: 54 contra 4 719, ~1 %).
#[test]
fn a_marca_pequena_do_spray_pousa() {
    let soma = |p: &Preset| -> f64 { miniatura_do_preset(p).rgba.chunks(4).map(tinta).sum() };
    let mut sem = dabber();
    sem.parametros.volumes.size_px = Some(96.0);
    for (k, v) in [
        ("bristleStrength", 0.0),
        ("felt", 0.05),
        ("pigmentPerDab", 2000.0),
    ] {
        sem.parametros.engine.insert(k.into(), v);
    }
    let mut com = sem.clone();
    com.parametros
        .brush
        .insert("spray_count".into(), Value::from(10));
    let mut jitter = com
        .parametros
        .brush
        .get("dab_jitter")
        .cloned()
        .unwrap_or_else(|| json!({}));
    jitter["particle"] = Value::from(0.1);
    com.parametros.brush.insert("dab_jitter".into(), jitter);
    let (a, b) = (soma(&sem), soma(&com));
    assert!(
        b > 0.25 * a,
        "a tinta com spray (particle 0,1) {b:.0} contra {a:.0} sem spray"
    );
}

/// O PÓ solto cresce com o pincel (o Metallic Splats do Rebelle: os pontos crescem a 200 px): com `γ = 1` e a
/// referência a 24 px, a 96 px os grãos são maiores que os de `γ = 0` (o corpo do dab não pousa: pigmento 50).
#[test]
fn o_po_cresce_com_o_pincel() {
    let po = |gama: Option<f64>| -> f64 {
        let mut p = dabber();
        p.parametros.volumes.size_px = Some(96.0);
        p.parametros.engine.insert("pigmentPerDab".into(), 50.0);
        let mut j = p
            .parametros
            .brush
            .get("dab_jitter")
            .cloned()
            .unwrap_or_else(|| json!({}));
        j["po"] = Value::from(3.0);
        j["po_px"] = Value::from(1.5);
        j["po_opacidade"] = Value::from(1.0);
        p.parametros.brush.insert("dab_jitter".into(), j);
        if let Some(g) = gama {
            p.parametros.engine.insert("textureSizeExp".into(), g);
            p.parametros.engine.insert("textureSizeRef".into(), 24.0);
        }
        tamanho_do_grao(&p)
    };
    let (fixo, segue) = (po(None), po(Some(1.0)));
    assert!(
        segue > 1.5 * fixo,
        "o tamanho do grão do pó a 96 px: γ = 1 {segue:.2}, γ = 0 {fixo:.2}"
    );
}
