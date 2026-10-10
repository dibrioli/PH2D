//! **O que a C0 não captura vem DITO**, nunca some em silêncio: o `Emissor::Ponto`, a forma que não
//! cobre o centro de nenhum texel, a forma fora da grade. ⭐ Desde a C3 (luz DIRETA, W4) as
//! [`LAMPADAS_DIRETAS_MAX`] lâmpadas mais fortes (discos e pontos) NÃO dependem da grade: não são
//! «fora» e acendem; só o que passa do teto volta a ser relatado.

use ph2d_light2d_ecs::{CenaDeLuz, Emissor};
use ph2d_light2d_gpu::{
    Config, ForaDaC0, LAMPADAS_DIRETAS_MAX, Motivo, Quem, Solver, o_que_fica_fora,
};

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

/// O ponto direto não é «fora»; o que passa do teto (o mais fraco) é, e vem dito.
#[test]
fn o_ponto_alem_do_teto_fica_fora_e_o_relatorio_o_diz() {
    let g = grade(64, 0.1);
    let mut pontos: Vec<Emissor> = (0..LAMPADAS_DIRETAS_MAX)
        .map(|k| ponto([0.3 + 0.35 * k as f32, 5.5]))
        .collect();
    assert!(o_que_fica_fora(&so(pontos.clone()), &g).fora.is_empty());
    pontos.push(Emissor::Ponto {
        centro: [3.2, 3.2],
        intensidade: [0.5; 3],
        alcance_m: f32::INFINITY,
    });
    assert_eq!(
        o_que_fica_fora(&so(pontos), &g).fora,
        vec![ForaDaC0 {
            quem: Quem::Emissor(LAMPADAS_DIRETAS_MAX),
            motivo: Motivo::PontoSemArea
        }]
    );
}

/// ⭐ Na placa, um ponto sozinho ACENDE pela luz direta, e é o gabarito (`F = I/r`) — na grade das
/// cascatas e na FINA (W4b: a parte suave é analítica no texel fino).
#[test]
fn na_placa_um_ponto_sozinho_acende_pela_luz_direta() {
    let Some(gpu) = gpu() else { return };
    let cena = so(vec![ponto([3.2, 3.2])]);
    for fina in [1, 4] {
        let cfg = Config {
            fina,
            ..Config::padrao(grade(64, 0.1))
        };
        let mut s = Solver::novo(&gpu, cfg).expect("solver");
        let rel = s.quadro(&gpu, &cena);
        assert!(rel.fora.is_empty(), "o ponto direto não é «fora»");
        let f = ler_fluencia(&gpu, &s);
        let g = s.grade_da_fluencia();
        let r = crate::comum::gabarito(&cena, &g);
        let mut pior = 0.0_f64;
        for (k, (a, b)) in f.iter().zip(&r).enumerate() {
            let c = g.centro(k as u32 % g.largura, k as u32 / g.largura);
            if (c[0] - 3.2).hypot(c[1] - 3.2) < 0.3 {
                continue; // a 1/r explode perto do ponto
            }
            pior = pior.max((f64::from(a[0]) - b[0]).abs() / b[0]);
        }
        eprintln!("ponto direto × gabarito (fina ×{fina}): pior erro relativo {pior:.4}");
        assert!(
            pior < 0.01,
            "o ponto direto não é o I/r do gabarito (fina ×{fina}): {pior:.4}"
        );
    }
}

#[test]
fn a_forma_menor_que_o_texel_e_a_de_fora_da_grade_vem_ditas() {
    let g = grade(64, 0.1);
    // Raio 0,2 texel num CANTO: cada um dos 4 texels fica com π·0,04/4 ≈ 0,03 de cobertura — não
    // possui nenhum. Raio 0,5 texel num CENTRO: π/4 ≈ 0,79 — possui o seu.
    let no_canto = disco([3.0, 3.0], 0.02, [1.0; 3], f32::INFINITY);
    let no_centro = disco([3.05, 3.05], 0.05, [1.0; 3], f32::INFINITY);
    let longe = disco([30.0, 30.0], 0.5, [1.0; 3], f32::INFINITY);
    // ⚠️ O teto de lâmpadas diretas ocupado por discos MAIS FORTES: os três de cima ficam nas
    // cascatas (dependem da grade) e voltam a ser relatados.
    let fortes = (0..LAMPADAS_DIRETAS_MAX)
        .map(|k| disco([0.3 + 0.35 * k as f32, 5.5], 0.1, [100.0; 3], f32::INFINITY));
    let cena = CenaDeLuz {
        emissores: [no_canto, no_centro, longe]
            .into_iter()
            .chain(fortes)
            .collect(),
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

/// ⭐ **Uma lâmpada direta faz SOMBRA na luz de outra** (a lei do gabarito: emissores são opacos).
/// Elas não estão na captura: o cone livre e a marcha testam os discos analiticamente. A região
/// medida é a faixa ATRÁS da lâmpada do meio, onde só a sombra dela muda a resposta.
#[test]
fn uma_lampada_direta_faz_sombra_na_luz_de_outra() {
    let Some(gpu) = gpu() else { return };
    let cena = so(vec![
        disco([1.0, 3.2], 0.3, [4.0; 3], f32::INFINITY),
        disco([3.0, 3.2], 0.25, [0.05; 3], f32::INFINITY),
    ]);
    // Na grade das cascatas e na FINA (W4b: a visibilidade das sondas e o refino pelas bordas).
    for fina in [1, 4] {
        let cfg = Config {
            fina,
            ..Config::padrao(grade(64, 0.1))
        };
        let mut s = Solver::novo(&gpu, cfg).expect("solver");
        s.quadro(&gpu, &cena);
        let f = ler_fluencia(&gpu, &s);
        let g = s.grade_da_fluencia();
        let r = crate::comum::gabarito(&cena, &g);
        let atras = |k: usize| {
            let [x, y] = g.centro(k as u32 % g.largura, k as u32 / g.largura);
            x > 3.6 && x < 6.0 && (y - 3.2).abs() < 0.12
        };
        let e = crate::comum::rmse_rel(&f, &r, atras);
        let sem_sombra = crate::comum::gabarito(&so(vec![cena.emissores[0].clone()]), &g);
        let controlo = crate::comum::rmse_rel(
            &sem_sombra
                .iter()
                .map(|c| c.map(|x| x as f32))
                .collect::<Vec<_>>(),
            &r,
            atras,
        );
        eprintln!(
            "sombra de lâmpada em lâmpada (fina ×{fina}): RMSE {e:.4} (sem a sombra: {controlo:.4})"
        );
        assert!(
            e < 0.1,
            "a lâmpada do meio não faz a sombra do gabarito (fina ×{fina}): {e:.4}"
        );
        assert!(
            controlo > 0.5,
            "controlo: sem a sombra a resposta tinha de ser outra ({controlo:.4})"
        );
    }
}
