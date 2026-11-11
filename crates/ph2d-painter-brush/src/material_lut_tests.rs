//! B31.2 — **o brilho em anéis** (AP-4): a tabela de 256 entradas lida por truncamento, no expoente 96 (Roughness 0),
//! saltava ~30 % de uma entrada para a outra perto do pico — o realce da crista desenhava-se em degraus concêntricos.
use super::*;

/// O maior salto relativo de `f` entre duas amostras vizinhas de `N·H` a 1/4096 (16 por entrada da tabela), na zona
/// onde o realce vale mais de 5 % do pico.
fn maior_salto(level: usize, f: impl Fn(f32) -> f32) -> f32 {
    let mut pior = 0.0f32;
    let mut ant = f(0.0);
    for k in 1..=4096 {
        let v = f(k as f32 / 4096.0);
        if v > 0.05 && ant > 0.0 {
            pior = pior.max((v - ant).abs() / v);
        }
        ant = v;
    }
    pior
}

/// ⭐ O realce é CONTÍNUO em `N·H`: em toda a Roughness, nenhum salto entre amostras a 1/4096 passa de 1,5× o do próprio
/// `powf` (o lobo verdadeiro também sobe depressa: ~2,4 %/amostra no expoente 96). Vermelho antes: 32,2 % no expoente 96
/// (a entrada inteira de uma vez). *Mutação: `spec` sem a interpolação.*
#[test]
fn o_realce_nao_salta_entre_entradas_da_tabela() {
    let lut = SpecLut::get();
    let saltos: Vec<(usize, f32)> = (0..ROUGH_LEVELS)
        .map(|l| {
            let e = Material::exponent(l as f32 / (ROUGH_LEVELS - 1) as f32);
            let verdade = maior_salto(l, |x| x.powf(e));
            (l, maior_salto(l, |x| lut.spec(l, x)) / verdade)
        })
        .collect();
    let (l, pior) = saltos
        .iter()
        .copied()
        .fold((0, 0.0f32), |a, b| if b.1 > a.1 { b } else { a });
    println!(
        "maior salto sobre o do powf: {pior:.2}× no nível {l} (expoente {:.1}); Roughness 0: {:.2}×",
        Material::exponent(l as f32 / (ROUGH_LEVELS - 1) as f32),
        saltos[0].1
    );
    assert!(
        pior <= 1.5,
        "anéis: o realce salta {pior:.2}× o do powf no nível {l}"
    );
}

/// As entradas da tabela continuam EXATAS nos nós: interpolar não move o realce onde a tabela já o sabia.
#[test]
fn nos_nos_o_realce_e_a_entrada_da_tabela() {
    let lut = SpecLut::get();
    for level in [0, ROUGH_LEVELS / 2, ROUGH_LEVELS - 1] {
        for i in 0..SPEC_LUT {
            let x = i as f32 / (SPEC_LUT - 1) as f32;
            let (v, n) = (lut.spec(level, x), lut.row(level)[i]);
            // `i / 255 × 255` pode cair um ulp abaixo de `i`: a fração ≈ 1 dá a entrada a menos de 1e-4 relativo.
            assert!(
                (v - n).abs() <= 1e-4 * n.max(1e-3),
                "nível {level}, entrada {i}: {v} vs {n}"
            );
        }
    }
}
