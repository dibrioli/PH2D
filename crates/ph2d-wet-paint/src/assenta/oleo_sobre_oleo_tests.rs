//! **Óleo molhado sobre óleo com o Real Oil na mão** (dono 31/10, `smoke_31_10_oleo/dono_5.png`: «o branco ocorre no
//! óleo sobre óleo»). A célula pousada com o Real Oil (`pincel_de_oleo`, E10e1) guarda o tapar extra (a cobertura do óleo sobre o óleo, `a4c8c80e3d`), mas a
//! [`na_caixa_das_tintas`] fazia dele transparência: o papel creme aparecia por óleo grosso e a secagem guardava bege.
//! Sobre o óleo a tinta é de CORPO: cobre e mistura pigmento, como a opaca ([`Ecra::com_opaca`]).
use super::*;
use crate::colorops::{ks_of_srgb255, reflectance_of_ks, srgb255_of_linear};
use crate::grid::oleo_celula::OleoCelula;

const AZ: [f64; 3] = [40.0, 90.0, 220.0];
const AM: [f64; 3] = [245.0, 205.0, 25.0];
const CREME: [f64; 3] = [236.0, 226.0, 206.0];

/// A distância (níveis) de `x` à curva Kubelka–Munk entre `a` e `b` (a mistura de pigmento das duas tintas).
fn fora_da_mistura(x: [f64; 3], a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..=200)
        .map(|i| {
            let c = f64::from(i) / 200.0;
            (0..3)
                .map(|k| {
                    let ks = (1.0 - c) * ks_of_srgb255(a[k]) + c * ks_of_srgb255(b[k]);
                    (x[k] - srgb255_of_linear(reflectance_of_ks(ks))).powi(2)
                })
                .sum::<f64>()
                .sqrt()
        })
        .fold(f64::INFINITY, f64::min)
}

fn lei(oleo: f64) -> Ecra {
    Ecra::Cobre {
        opaca: 0.0,
        oleo,
        vela: 0.0,
        filme: 0.0,
        esconde: 5.0,
        molhada: 1.0,
    }
}

const OLEO: OleoCelula = OleoCelula {
    oleo: 1.0,
    ..OleoCelula::NEUTRA
};

/// O óleo pousado com o Real Oil na mão (a última mão desta célula).
const OLEO_DO_PINCEL: OleoCelula = OleoCelula {
    pincel_de_oleo: true,
    ..OLEO
};

/// O papel não aparece pelo óleo: a cobertura que o render compõe é a do ponto, e sobre o creme o píxel fica na
/// mistura de pigmento das duas tintas (azul sob amarelo e o contrário, filme fino a grosso).
#[test]
fn o_oleo_sobre_o_oleo_nao_abre_o_papel_nem_clareia() {
    let e = lei(0.0).com_oleo(OLEO_DO_PINCEL);
    let mut pior = (0.0f64, String::new());
    for (sc, uc) in [(AZ, AM), (AM, AZ)] {
        for a_s in [1.0, 0.9] {
            for a_f in [0.15, 0.3, 0.5, 0.8] {
                let (c, la) = cor_e_alfa(e, sc, uc, a_s, a_f);
                let (c2, a2) = na_caixa_das_tintas(e, (c, la), sc, uc, a_s, a_f);
                assert!(
                    a2 >= la - 1e-9,
                    "o papel aparece pelo óleo: cobertura {la:.3} → {a2:.3} ({sc:?} sob {uc:?}, {a_s},{a_f})"
                );
                let t_s = sc.map(|v| 255.0 - a_s * (255.0 - v));
                let px = [0, 1, 2].map(|k| c2[k] * a2 + CREME[k] * (1.0 - a2));
                let m = fora_da_mistura(px, t_s, uc);
                if m > pior.0 {
                    pior = (m, format!("{sc:?} sob {uc:?} {a_s},{a_f}: {px:.0?}"));
                }
            }
        }
    }
    assert!(
        pior.0 <= 20.0,
        "o óleo sobre o óleo saiu da mistura das tintas {:.1} níveis: {}",
        pior.0,
        pior.1
    );
}

/// A SECAGEM guarda a mistura, não o bege: o filme todo assenta no óleo de baixo.
#[test]
fn assentar_o_oleo_sobre_o_oleo_guarda_a_mistura() {
    let e = lei(0.0).com_oleo(OLEO_DO_PINCEL);
    for (sc, uc) in [(AZ, AM), (AM, AZ)] {
        for a_f in [0.15, 0.3, 0.5] {
            let s2 = assenta(e, sc, uc, 1.0, a_f, 1.0, 0.0);
            let m = fora_da_mistura(s2, sc, uc);
            assert!(
                m <= 20.0,
                "assentar {uc:?} a {a_f} sobre {sc:?} guardou {s2:.0?}, {m:.1} níveis fora da mistura"
            );
        }
    }
}

/// CONTROLOS: a primeira pincelada (sem tinta assentada) e o óleo sem o Real Oil na mão ficam ao bit.
#[test]
fn a_pincelada_unica_e_a_lei_sem_o_pincel_de_oleo_nao_mudam() {
    for o in [0.0, 1.0] {
        let e = lei(o);
        assert_eq!(e.com_oleo(OLEO), lei(if o == 0.0 { 1.0 } else { o }));
    }
    let e = lei(0.0);
    for a_f in [0.15, 0.5, 1.0] {
        let antes = cor_e_alfa(e, AZ, AM, 0.0, a_f);
        let depois = cor_e_alfa(e.com_oleo(OLEO_DO_PINCEL), AZ, AM, 0.0, a_f);
        assert_eq!(antes, depois);
    }
}

/// ⭐ **A orla FINA não enferruja** (dono 31/10, pressão baixa: «se eu passar o amarelo no azul com pouca pressão
/// aparecem bordas ferrugem»). A mistura opaca fazia-se com o assentado já diluído em BRANCO e re-exprimia-se pela
/// cobertura: o canal que o amarelo absorve ia a `0` — cor da camada (181,48,0) a `a_s = a_f = 0,02`. A cor da camada
/// fica na mistura de pigmento das duas tintas, fina ou grossa.
#[test]
fn a_orla_fina_do_oleo_sobre_o_oleo_nao_enferruja() {
    let e = lei(0.0).com_oleo(OLEO_DO_PINCEL);
    let mut pior = (0.0f64, String::new());
    for (sc, uc) in [(AZ, AM), (AM, AZ)] {
        for a_s in [0.02, 0.05, 0.1, 0.3, 0.7] {
            for a_f in [0.02, 0.05, 0.1, 0.3] {
                let (c, la) = cor_e_alfa(e, sc, uc, a_s, a_f);
                let (c2, _) = na_caixa_das_tintas(e, (c, la), sc, uc, a_s, a_f);
                for (nome, x) in [("camada", c), ("caixa", c2)] {
                    let m = fora_da_mistura(x, sc, uc);
                    if m > pior.0 {
                        pior = (m, format!("{nome} {sc:?} sob {uc:?} {a_s},{a_f}: {x:.0?}"));
                    }
                }
            }
        }
    }
    assert!(
        pior.0 <= 20.0,
        "a orla fina saiu da mistura {:.1} níveis: {}",
        pior.0,
        pior.1
    );
}
