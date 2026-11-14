//! SONDA J10 — o chão (o óleo assentado) como FORÇA no momento × a ladeira no nivelamento clampado: Dabber seco +
//! Wet Drip diluído por cima (a cena de `escorre_contorna_tests`), variantes no MESMO processo, intercaladas. Mede a
//! força da ladeira contra a gravidade na orla (`MEDIDA_DO_CHAO`) e abaixo/miolo/acima. Medição, nunca portão.

use super::agua_sobre_oleo_tests::relato;
use super::escorre_contorna_tests::{ObstaculoA, SUFIXO, linha};
use ph2d_wet_paint::solver::regua::Regua;
use ph2d_wet_paint::solver::{CHAO_FORCA, CHAO_MEDE, CHAO_ROTA, CHAO_SECO, LADEIRA_ROTA};

/// `(nome, ganho, rota do chão, film seco)`; ganho `< 0` = sem a força (o produto de antes).
const VARIANTES: [(&str, f32, u8, f32); 6] = [
    ("antes", -1.0, 0, 0.1),
    ("g0.05s0.1", 0.05, 0, 0.1),
    ("g0.125s0.1", 0.125, 0, 0.1),
    ("g0.125s0.5", 0.125, 0, 0.5),
    ("g0.125s0.02", 0.125, 0, 0.02),
    ("g0.5s0.1", 0.5, 0, 0.1),
];

fn bloco(v: (&str, f32, u8, f32), casos: &[(ObstaculoA, u32, f64)], vezes: u32) {
    let _f = Regua::poe(&CHAO_FORCA, v.1.to_bits());
    let _r = Regua::poe(&CHAO_ROTA, v.2);
    let _s = Regua::poe(&CHAO_SECO, v.3.to_bits());
    *SUFIXO.lock().expect("o sufixo") = format!("_{}", v.0);
    let linhas: Vec<String> = std::thread::scope(|s| {
        let h: Vec<_> = casos
            .iter()
            .map(|&(a, e, m)| {
                std::thread::Builder::new()
                    .stack_size(64 << 20)
                    .spawn_scoped(s, move || linha(a, e, m, 0, vezes))
                    .expect("a thread")
            })
            .collect();
        h.into_iter()
            .map(|h| h.join().unwrap_or_else(|_| "REBENTOU".into()))
            .collect()
    });
    for l in linhas {
        relato(format!("{:<11}| {l}", v.0));
    }
}

/// O volume: B passa 1, 3 e 6 vezes contra o Dabber seco; com volume a água tem de passar por cima.
#[test]
#[ignore = "sonda J10"]
fn diag_o_volume_transborda() {
    let _m = Regua::poe(&CHAO_MEDE, 1);
    for v in [VARIANTES[0], VARIANTES[2]] {
        for n in [1u32, 3, 6] {
            bloco(
                (
                    Box::leak(format!("{}x{n}", v.0).into_boxed_str()),
                    v.1,
                    v.2,
                    v.3,
                ),
                &[
                    (ObstaculoA::Dabber, 1200, 0.0),
                    (ObstaculoA::DabberCurto, 1200, 0.0),
                ],
                n,
            );
        }
    }
}

#[test]
#[ignore = "sonda J10"]
fn diag_o_chao_como_forca() {
    let _m = Regua::poe(&CHAO_MEDE, 1);
    let _l = Regua::poe(&LADEIRA_ROTA, 0);
    let casos = [
        (ObstaculoA::Dabber, 1200, 0.0),
        (ObstaculoA::Dabber, 1200, 1.0),
        (ObstaculoA::DabberDiluido, 1200, 0.0),
        (ObstaculoA::WetRound, 1200, 0.0),
        (ObstaculoA::DabberCurto, 1200, 0.0),
        (ObstaculoA::DabberCurto, 1200, 1.0),
    ];
    // Duas rodadas, a ordem das variantes invertida na segunda (intercaladas).
    for r in 0..1 {
        let mut vs = VARIANTES.to_vec();
        if r == 1 {
            vs.reverse();
        }
        for v in vs {
            bloco(v, &casos, 1);
        }
    }
}
