//! ⭐⭐⭐ **O CHÃO QUE TAPA visto DE BAIXO** — o report do dono (03/10): o reflexo do cromo via a base
//! oculta das vizinhas (o chão debaixo de uma peça pousada lia-se ACESO), e com a câmara abaixo do chão
//! o chão pintava-se por cima das peças (uma cruz clara onde elas encostam). O oráculo é o de
//! `tests_chao_tapa` com a câmara por baixo (`oraculo_chao_tapa_blender.py -- … baixo`): o Cycles vê a
//! base das peças através do chão invisível à câmara.

use crate::tests_chao_tapa::{PECAS, desenha_em, difusa, metal, oraculo_de};
use crate::tests_contacto::{LADO, bordas, camera_do_blender, desenhista_de, linear};

const ORACULO: &str = include_str!("../fixtures/oraculo_chao_tapa_baixo.csv");
const DE: [f32; 3] = [0.5, -0.5, 0.8];
const ALVO: [f32; 3] = [0.0, 0.1, 0.15];
const MEIA: f32 = 0.8;

/// ⭐⭐⭐ **De baixo, a base das peças é a do Cycles** — a difusa (razão contra o quadro sem chão nem
/// grelhas) e o espelho (razão com/sem chão), pixel a pixel longe das silhuetas; e à parte a BASE
/// (`n.y < −0,6`), onde a peça olha para o chão que ela própria tapa.
#[test]
#[ignore = "precisa de aparelho"]
fn o_chao_visto_de_baixo_e_o_do_cycles() {
    let (Some(mut grades), Some(mut nua)) =
        (desenhista_de(&PECAS, true), desenhista_de(&PECAS, false))
    else {
        eprintln!("sem aparelho — o gate não corre aqui");
        return;
    };
    let cam = camera_do_blender(DE, ALVO, MEIA);
    let (pontos, linhas) = oraculo_de(ORACULO);
    let base = desenha_em(&mut nua, false, difusa(), cam);
    let borda = bordas(&pontos, &base);
    let difusa_com = desenha_em(&mut grades, true, difusa(), cam);
    // O CONTROLO: as grelhas sem o chão — o que a base via antes da lei.
    let difusa_sem = desenha_em(&mut grades, false, difusa(), cam);
    let (mut ctl, mut n_ctl) = (0.0f32, 0usize);
    // O espelho na razão com/sem chão (a régua de `o_reflexo_do_chao_e_o_do_cycles`).
    let (espelho_com, espelho_sem) = (
        desenha_em(&mut grades, true, metal(0.0), cam),
        desenha_em(&mut grades, false, metal(0.0), cam),
    );
    // (soma, n, pior) para: a difusa toda · a base · o espelho todo · o espelho na base.
    let mut m = [(0.0f32, 0usize, 0.0f32); 4];
    let mut junta = |k: usize, e: f32| {
        m[k].0 += e;
        m[k].1 += 1;
        m[k].2 = m[k].2.max(e);
    };
    for (p, l) in pontos.iter().zip(&linhas) {
        if borda[(p.j * LADO + p.i) as usize] {
            continue;
        }
        let i = ((p.j * LADO + p.i) * 4 + 1) as usize;
        let na_base = p.n[1] < -0.6;
        let lb = linear(base[i]);
        if lb >= 0.05 {
            let e = (linear(difusa_com[i]) / lb - l.com).abs();
            junta(0, e);
            if na_base {
                junta(1, e);
                ctl += (linear(difusa_sem[i]) / lb - l.com).abs();
                n_ctl += 1;
            }
        }
        let (cs, cc, ls) = (l.brilho[0], l.brilho[1], linear(espelho_sem[i]));
        if cs >= 0.05 && ls >= 0.05 {
            let e = (linear(espelho_com[i]) / ls - cc / cs).abs();
            junta(2, e);
            if na_base {
                junta(3, e);
            }
        }
    }
    let media = |k: usize| m[k].0 / m[k].1 as f32;
    eprintln!(
        "de baixo · difusa {} px |Δ| médio {:.4} máx {:.3} · na base {} px {:.4} máx {:.3} (SEM a lei {:.4}) · \
         espelho {} px {:.4} máx {:.3} · na base {} px {:.4} máx {:.3}",
        m[0].1,
        media(0),
        m[0].2,
        m[1].1,
        media(1),
        m[1].2,
        ctl / n_ctl as f32,
        m[2].1,
        media(2),
        m[2].2,
        m[3].1,
        media(3),
        m[3].2
    );
    assert!(
        m[0].1 > 5000 && m[1].1 > 500 && m[3].1 > 300,
        "a fixtura encolheu"
    );
    assert!(
        ctl / n_ctl as f32 > 0.1,
        "CONTROLO: sem a lei a base vê o céu de baixo e erra"
    );
    // Medido (03/10): difusa `0,0133` (máx `0,098`), na base `0,0107` (máx `0,074`); espelho `0,0027`, na
    // base `0,0055`. Com o chão debaixo das peças lido ACESO (antes do report do dono): `0,1306` e na base
    // `0,3701`, máx `1,0`.
    assert!(
        media(0) < 0.018 && media(1) < 0.015,
        "a difusa de baixo afastou-se do Cycles"
    );
    assert!(
        media(2) < 0.005 && media(3) < 0.009,
        "o espelho de baixo afastou-se do Cycles"
    );
}
