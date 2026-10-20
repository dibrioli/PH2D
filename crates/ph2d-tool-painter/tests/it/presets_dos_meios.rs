//! **Os presets nos QUATRO meios** (L1) e **a mão nos três antigos** (L2) — `docs/Painter/Pinceis/03_MOTORES.md`
//! §4. Cada portão pinta a cena `gestos` da BATERIA (`exame_bancada::pinta_cena`, a fonte única dos gestos: a
//! rampa de pressão 0→1 está nela) e compara-a AO BYTE.

use crate::exame_bancada::{Fonte, Pincel, RECEITAS_DE_MEIO, bateria, pinta_cena};
use ph2d_tool_painter::PaintMedia;
use serde_json::Value;

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// A cena `gestos` da bateria, a foto A (molhada), com `pincel`.
pub fn gestos(pincel: &Pincel) -> Vec<u8> {
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    let cena: &Value = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == "gestos")
        .unwrap();
    pinta_cena(pincel, None, cena, tela, papel, 0).1
}

fn meio(m: PaintMedia) -> Pincel {
    let r = RECEITAS_DE_MEIO.iter().find(|r| r.meio == m).unwrap();
    Pincel {
        nome: r.nome.to_owned(),
        fonte: Fonte::Meio(r),
    }
}

/// A cena `gestos` (a rampa 0→1 incluída) nos três meios antigos ANTES da L2 — `pressureSize 0,9` (o
/// `size_min 0,1` de sempre), sem pressão na opacidade, a inclinação de pé. Medido em `0797d99072`.
const NEUTRO_DE_HOJE: [(PaintMedia, u64); 3] = [
    (PaintMedia::Digital, 0x84f5_0263_d09d_3843),
    (PaintMedia::Watercolor, 0x2a62_0899_d312_8981),
    (PaintMedia::Impasto, 0xf545_285d_938d_8abf),
];

/// ⭐ **O neutro da mão é o traço de hoje**: com a mão de fábrica, os três meios antigos pintam a cena da bateria
/// ao byte do que pintavam antes de a mão ser um campo do pincel.
#[test]
fn o_neutro_da_mao_e_o_traco_de_hoje() {
    let mut erros = Vec::new();
    for (m, esperado) in NEUTRO_DE_HOJE {
        let h = fnv1a(&gestos(&meio(m)));
        if h != esperado {
            erros.push(format!("{m:?}: 0x{h:016x} (esperado 0x{esperado:016x})"));
        }
    }
    assert!(erros.is_empty(), "o neutro mudou o traço: {erros:#?}");
}
