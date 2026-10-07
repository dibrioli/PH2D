//! ⛔ **A cerca da linha Wet Painter: Digital, Watercolor e Impasto não mudam um byte** (decisão do
//! dono, 2026-10-07: *«não tenho intenção de mudar os modos Impasto, Digital ou Watercolor. Apenas
//! aperfeiçoar Wet Paint»*).
//!
//! A cena é a da bancada (`sonda_gestos`, tamanho 28: os cinco gestos medidos contra o Rebelle) e os
//! hashes são a saída de `main` 2a925859d (iguais nos perfis `test` e `smoke`), gravados antes do
//! 1.º commit de produto da linha; as
//! imagens estão em `docs/Painter/NewFeaturesWetPaint/fixtures/sonda_28/`. Um hash que muda aqui é um
//! meio que não devia ter mudado: a cura é desfazer, nunca regravar o número.

use super::sonda_gestos::{fnv1a, pinta};
use ph2d_tool_painter::PaintMedia;

const SIZE: f32 = 28.0;

const ESPERADO: [(&str, PaintMedia, u64); 3] = [
    ("digital", PaintMedia::Digital, 0x9b4c_55cf_f565_67bd),
    ("watercolor", PaintMedia::Watercolor, 0xc4fd_b261_ca6e_97af),
    ("impasto", PaintMedia::Impasto, 0xd6f7_2b02_2a53_0144),
];

#[test]
fn digital_watercolor_e_impasto_dao_a_cena_da_bancada_ao_byte() {
    let mut mudou = Vec::new();
    for (nome, meio, esperado) in ESPERADO {
        let (px, _, _) = pinta(meio, SIZE);
        let h = fnv1a(&px);
        if h != esperado {
            mudou.push(format!("{nome}: {h:#018x} (esperado {esperado:#018x})"));
        }
    }
    assert!(
        mudou.is_empty(),
        "um meio fora do Wet Paint mudou:\n{}",
        mudou.join("\n")
    );
}
