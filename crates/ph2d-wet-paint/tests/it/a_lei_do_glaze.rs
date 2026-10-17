//! **O Glaze do PINCEL** (prompt 42, doc `42_medicao_a_lei_da_tinta.md`): a lei da camada que filtra a de baixo.

use ph2d_wet_paint::assenta::{Ecra, assenta, sobre_branco};
use ph2d_wet_paint::colorops::{outra_tinta, vela_da_camada};
use ph2d_wet_paint::opacity::alpha_of_mass;

const COBRE: Ecra = Ecra::Cobre {
    filme: 0.0,
    esconde: 5.0,
    molhada: 1.0,
    vela: 0.0,
};

/// A fracção multiplicada é a MEDIDA no Rebelle (16/10, sRGB por canal): a mesma tinta 0,48–0,51, amarelo sobre
/// laranja 0,61–0,89, vermelho sobre azul 0,98.
#[test]
fn a_fraccao_da_camada_e_a_medida_no_rebelle() {
    let azul = [23.0, 101.0, 179.0];
    let mesma = vela_da_camada(1.0, azul, azul);
    assert!((0.48..=0.51).contains(&mesma), "a mesma tinta: {mesma:.3}");
    let laranja = vela_da_camada(1.0, [240.0, 120.0, 20.0], [245.0, 205.0, 25.0]);
    assert!(
        (0.61..=0.89).contains(&laranja),
        "amarelo sobre laranja: {laranja:.3}"
    );
    let vermelho = vela_da_camada(1.0, azul, [191.0, 0.0, 18.0]);
    assert!(vermelho >= 0.95, "vermelho sobre azul: {vermelho:.3}");
    assert_eq!(vela_da_camada(0.0, azul, [191.0, 0.0, 18.0]), 0.0);
    // Os carimbos de um traço (a mesma tinta, diluída) não se empilham em camadas.
    assert_eq!(
        outra_tinta(azul, azul.map(|v| 255.0 - 0.4 * (255.0 - v))),
        0.0
    );
    assert!(outra_tinta(azul, [191.0, 0.0, 18.0]) >= 0.99);
}

/// Glaze 0 é a lei da sessão AO BIT — a porta que todo pincel sem Glaze atravessa.
#[test]
fn glaze_zero_e_a_lei_da_sessao() {
    assert_eq!(COBRE.com_vela(0.0), COBRE);
    assert_eq!(Ecra::Porto.com_vela(1.0), Ecra::Porto);
    let (sc, uc) = ([23.0, 101.0, 179.0], [191.0, 0.0, 18.0]);
    assert_eq!(
        sobre_branco(COBRE.com_vela(0.0), sc, uc, 0.8, 0.9),
        sobre_branco(COBRE, sc, uc, 0.8, 0.9)
    );
}

/// O vermelho de Glaze 1 sobre o azul assentado mostra o PRODUTO das duas camadas (o marcador do Rebelle:
/// (20,0,15) contra (191,0,18)⊙(23,101,179)/255 = (17,0,13)); sem Glaze, o vermelho tapa.
#[test]
fn glaze_um_mostra_o_produto_das_camadas() {
    let (sc, uc) = ([23.0, 101.0, 179.0], [191.0, 0.0, 18.0]);
    let tapa = sobre_branco(COBRE, sc, uc, 1.0, 1.0);
    let filtra = sobre_branco(COBRE.com_vela(1.0), sc, uc, 1.0, 1.0);
    assert!(tapa[0] > 180.0, "sem Glaze o vermelho tapa: {tapa:?}");
    for k in 0..3 {
        let produto = sc[k] * uc[k] / 255.0;
        assert!(
            (filtra[k] - produto).abs() < 1.0,
            "canal {k}: {filtra:?} contra {produto:.1}"
        );
    }
}

/// O pior erro e quantos estados mudam o ecrã > 1,5 níveis ao assentar, na grelha de estados (cores, massas e a
/// massa que desce — as coberturas pela conta do motor).
fn erro_ao_assentar(vela: f32) -> (f64, usize, String) {
    let cores = [
        [23.0, 101.0, 179.0],
        [191.0, 0.0, 18.0],
        [245.0, 205.0, 25.0],
        [240.0, 120.0, 20.0],
        [40.0, 160.0, 60.0],
    ];
    let e = COBRE.com_vela(vela);
    let (mut pior, mut acima, mut onde) = (0.0f64, 0, String::new());
    for sc in cores {
        for uc in cores {
            for (ms, mf) in [
                (200.0, 400.0),
                (800.0, 2000.0),
                (3000.0, 600.0),
                (500.0, 5000.0),
            ] {
                for desce in [0.1, 0.3, 0.6] {
                    let dm = mf * desce;
                    let (a_s, a_f) = (alpha_of_mass(ms), alpha_of_mass(mf));
                    let (a_s2, a_f2) = (alpha_of_mass(ms + dm), alpha_of_mass(mf - dm));
                    let alvo = sobre_branco(e, sc, uc, a_s, a_f);
                    let novo = assenta(e, sc, uc, a_s, a_f, a_s2, a_f2);
                    let depois = sobre_branco(e, novo, uc, a_s2, a_f2);
                    let d = (0..3)
                        .map(|k| (depois[k] - alvo[k]).abs())
                        .fold(0.0, f64::max);
                    if d > 1.5 {
                        acima += 1;
                    }
                    if d > pior {
                        pior = d;
                        onde = format!("sc {sc:?} uc {uc:?} massas {ms}/{mf} desce {desce}");
                    }
                }
            }
        }
    }
    (pior, acima, onde)
}

/// ⭐ **Assentar não muda o que se vê — e o Glaze não o faz pior que a lei da obra** (dono 10/10). O CONTROLO é a
/// mesma grelha com Glaze 0 (a lei de hoje: 41 de 300 estados > 1,5, o pior 42,6 — o aberto (b) do 05d).
#[test]
fn assentar_guarda_o_ecra_com_o_glaze() {
    let (p0, n0, _) = erro_ao_assentar(0.0);
    for vela in [0.5, 1.0] {
        let (p, n, onde) = erro_ao_assentar(vela);
        assert!(
            p <= p0 && n <= n0,
            "Glaze {vela}: {n} estados > 1,5 e o pior {p:.2} ({onde}) — a lei da obra: {n0} e {p0:.2}"
        );
    }
}

#[test]
/// SONDA: existe uma cor de assentado que guarda o ecrã? (força bruta na grelha 3³ do RGB) — prompt 42.
#[ignore = "sonda: o alcance da inversa do Glaze"]
fn sonda_alcance() {
    let e = COBRE.com_vela(1.0);
    for (sc, uc, ms, mf, desce) in [
        (
            [245.0, 205.0, 25.0],
            [40.0, 160.0, 60.0],
            500.0,
            5000.0,
            0.6,
        ),
        ([23.0, 101.0, 179.0], [191.0, 0.0, 18.0], 500.0, 5000.0, 0.6),
        ([23.0, 101.0, 179.0], [191.0, 0.0, 18.0], 3000.0, 600.0, 0.3),
    ] {
        let dm = mf * desce;
        let (a_s, a_f) = (alpha_of_mass(ms), alpha_of_mass(mf));
        let (a_s2, a_f2) = (alpha_of_mass(ms + dm), alpha_of_mass(mf - dm));
        let alvo = sobre_branco(e, sc, uc, a_s, a_f);
        let novo = assenta(e, sc, uc, a_s, a_f, a_s2, a_f2);
        let mut melhor = (f64::MAX, [0.0; 3]);
        for r in (0..=255).step_by(3) {
            for g in (0..=255).step_by(3) {
                for b in (0..=255).step_by(3) {
                    let x = [r as f64, g as f64, b as f64];
                    let y = sobre_branco(e, x, uc, a_s2, a_f2);
                    let d = (0..3).map(|k| (y[k] - alvo[k]).abs()).fold(0.0, f64::max);
                    if d < melhor.0 {
                        melhor = (d, x);
                    }
                }
            }
        }
        let y = sobre_branco(e, novo, uc, a_s2, a_f2);
        println!(
            "alvo {alvo:?} · assenta {novo:?} → {y:?} · melhor da grelha {:.2} em {:?} · a {a_s:.3}/{a_f:.3} → {a_s2:.3}/{a_f2:.3}",
            melhor.0, melhor.1
        );
    }
}
