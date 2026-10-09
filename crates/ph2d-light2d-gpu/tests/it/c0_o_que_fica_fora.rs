//! **O que a C0 não captura vem DITO**, nunca some em silêncio: o `Emissor::Ponto` (sem área, o
//! caminho analítico C3 da W5), a forma que não cobre o centro de nenhum texel, a forma fora da
//! grade. E na placa: uma cena só com um `Ponto` dá fluência ZERO em toda a grade — é isto que
//! acontece hoje, até a C3 existir.

use ph2d_light2d_ecs::{CenaDeLuz, Emissor};
use ph2d_light2d_gpu::{Config, ForaDaC0, Motivo, Quem, Solver, o_que_fica_fora};

use crate::comum::{disco, gpu, grade, ler_fluencia, oclusor, regular};

fn so(emissores: Vec<Emissor>) -> CenaDeLuz {
    CenaDeLuz {
        emissores,
        oclusores: vec![],
        ceu: [0.0; 3],
        ..Default::default()
    }
}

fn ponto(centro: [f32; 2]) -> Emissor {
    Emissor::Ponto {
        centro,
        intensidade: [1.0; 3],
        alcance_m: f32::INFINITY,
    }
}

#[test]
fn o_ponto_fica_fora_da_c0_e_o_relatorio_o_diz() {
    let g = grade(64, 0.1);
    let fora = o_que_fica_fora(&so(vec![ponto([3.2, 3.2])]), &g).fora;
    assert_eq!(
        fora,
        vec![ForaDaC0 {
            quem: Quem::Emissor(0),
            motivo: Motivo::PontoSemArea
        }]
    );
}

#[test]
fn na_placa_um_ponto_sozinho_nao_acende_nada() {
    let Some(gpu) = gpu() else { return };
    let mut s = Solver::novo(&gpu, Config::padrao(grade(64, 0.1))).expect("solver");
    let rel = s.quadro(&gpu, &so(vec![ponto([3.2, 3.2])]));
    assert_eq!(rel.fora.len(), 1);
    let f = ler_fluencia(&gpu, &s);
    assert!(
        f.iter().all(|t| *t == [0.0; 3]),
        "o Ponto não pode acender pela C0"
    );
}

#[test]
fn a_forma_menor_que_o_texel_e_a_de_fora_da_grade_vem_ditas() {
    let g = grade(64, 0.1);
    // Raio 0,2 texel num CANTO: cada um dos 4 texels fica com π·0,04/4 ≈ 0,03 de cobertura — não
    // possui nenhum. Raio 0,5 texel num CENTRO: π/4 ≈ 0,79 — possui o seu.
    let no_canto = disco([3.0, 3.0], 0.02, [1.0; 3], f32::INFINITY);
    let no_centro = disco([3.05, 3.05], 0.05, [1.0; 3], f32::INFINITY);
    let longe = disco([30.0, 30.0], 0.5, [1.0; 3], f32::INFINITY);
    let cena = CenaDeLuz {
        emissores: vec![no_canto, no_centro, longe],
        // Uma lasca de 0,11 texel (cobertura ≤ 0,11 em qualquer texel), e um oclusor normal.
        oclusores: vec![
            oclusor(
                vec![[1.007, 0.5], [1.018, 0.5], [1.018, 5.5], [1.007, 5.5]],
                1.0,
            ),
            oclusor(regular([4.0, 4.0], 0.5, 5, 0.1), 1.0),
        ],
        ceu: [0.0; 3],
        ..Default::default()
    };
    let fora = o_que_fica_fora(&cena, &g).fora;
    assert_eq!(
        fora,
        vec![
            ForaDaC0 {
                quem: Quem::Oclusor(0),
                motivo: Motivo::MenorQueOTexel
            },
            ForaDaC0 {
                quem: Quem::Emissor(0),
                motivo: Motivo::MenorQueOTexel
            },
            ForaDaC0 {
                quem: Quem::Emissor(2),
                motivo: Motivo::ForaDaGrade
            },
        ]
    );
}
