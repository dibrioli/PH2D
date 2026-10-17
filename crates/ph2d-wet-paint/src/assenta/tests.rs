//! Os portões de unidade de [`super`] (a lei do ecrã e as transferências verticais) — fora do ficheiro pelo tecto de
//! LOC do workspace (prompt 42: o Glaze do pincel levou-o a 721).
use super::*;

const AZ: [f64; 3] = [30.0, 100.0, 220.0];
const AM: [f64; 3] = [245.0, 205.0, 25.0];

fn perto(a: [f64; 3], b: [f64; 3], tol: f64) -> bool {
    (0..3).all(|c| (a[c] - b[c]).abs() <= tol)
}

/// Assentar e re-molhar não mudam o ecrã, nas duas leis, de filme grosso a quase seco (≤ 1,5 níveis: onde
/// o ecrã quase não vê a cor que muda — sensibilidade < 0,1 —, o palpite físico fica sem a correcção
/// fina; medido 1,4 no filme a 10 %).
#[test]
fn as_transferencias_verticais_nao_mudam_o_ecra() {
    for e in [Ecra::PorCima(0.20), Ecra::PorCima(0.35), Ecra::Camada] {
        let cor = |e, sc, uc, a_s, a_f| sobre_branco(e, sc, uc, a_s, a_f);
        for (a_s, a_f, d) in [
            (0.9, 0.95, 0.05),
            (0.999, 0.99, 0.02),
            (0.6, 0.5, 0.2),
            (0.99, 0.3, 0.29),
            (0.2, 0.1, 0.05),
            (0.01, 0.9, 0.1),
            (0.95, 1.0, 0.0),
        ] {
            let antes = cor(e, AZ, AM, a_s, a_f);
            let a_s2 = 1.0 - (1.0 - a_s) * (1.0 - d);
            let a_f2 = 1.0 - (1.0 - a_f) / (1.0 - d);
            let s2 = assenta(e, AZ, AM, a_s, a_f, a_s2, a_f2);
            let depois = cor(e, s2, AM, a_s2, a_f2);
            assert!(
                perto(antes, depois, 1.5),
                "{e:?} assenta {a_s},{a_f}: {antes:?} → {depois:?}"
            );
            let u2 = levanta(e, AZ, AM, a_s2, a_f2, a_s, a_f);
            let ida = cor(e, AZ, AM, a_s2, a_f2);
            let volta = cor(e, AZ, u2, a_s, a_f);
            assert!(
                perto(ida, volta, 1.5),
                "{e:?} levanta {a_s},{a_f}: {ida:?} → {volta:?}"
            );
        }
    }
}

/// A MESMA tinta por cima de si mesma fica a mesma cor (o traço de uma cor não muda ao secar). Na lei do
/// Rebelle, duas camadas da mesma tinta são UMA com a massa das duas: o tapar a mais só pesa quando o
/// pigmento muda (mutação que sangra: tapar a mais sem a diferença de pigmento clareia `0,80 → 0,58`).
#[test]
fn a_mesma_tinta_nao_muda() {
    let e = Ecra::PorCima(0.20);
    let s2 = assenta(e, AM, AM, 0.9, 0.8, 0.95, 0.6);
    assert!(perto(s2, AM, 0.01), "{s2:?}");
    assert_eq!(assenta(e, [0.0; 3], AM, 0.0, 0.8, 0.3, 0.7), AM);
    let cobre = Ecra::Cobre {
        vela: 0.0,
        filme: 0.2,
        esconde: 3.0,
        molhada: 1.0,
    };
    let (duas, uma) = (
        sobre_branco(cobre, AM, AM, 0.6, 0.5),
        sobre_branco(cobre, AM, AM, 0.8, 0.0),
    );
    assert!(perto(duas, uma, 0.5), "duas camadas {duas:?} ≠ uma {uma:?}");
}

/// Tudo assentado = a cor que se via.
#[test]
fn secar_de_vez_fica_com_a_cor_que_se_via() {
    for e in [Ecra::PorCima(0.20), Ecra::Camada] {
        let antes = cor(e, AZ, AM, 0.9, 0.7);
        assert_eq!(assenta(e, AZ, AM, 0.9, 0.7, 0.97, 0.0), antes);
    }
}

/// Na lei do Rebelle o assentar preserva o ecrã SECO em transferências reais (≤ 5 % da massa por passe)
/// e a MESMA tinta não muda ao trocar de camada (re-molhar). Re-molhar pigmento DIFERENTE muda o tapar
/// (a camada de cima tapa mais) — sem invariante aí.
#[test]
fn na_lei_do_rebelle_assentar_preserva_o_ecra_seco() {
    let e = Ecra::Cobre {
        vela: 0.0,
        filme: 0.2,
        esconde: 3.0,
        molhada: 1.6,
    };
    for (a_s, a_f, d) in [
        (0.9, 0.95, 0.05),
        (0.6, 0.5, 0.05),
        (0.99, 0.3, 0.03),
        (0.2, 0.1, 0.05),
    ] {
        let a_s2 = 1.0 - (1.0 - a_s) * (1.0 - d);
        let a_f2 = 1.0 - (1.0 - a_f) / (1.0 - d);
        let antes = sobre_branco(e.seco(), AZ, AM, a_s, a_f);
        let s2 = assenta(e, AZ, AM, a_s, a_f, a_s2, a_f2);
        let depois = sobre_branco(e.seco(), s2, AM, a_s2, a_f2);
        assert!(
            perto(antes, depois, 1.5),
            "assenta {a_s},{a_f}: {antes:?} → {depois:?}"
        );
    }
    // A massa conserva-se: (1 − 0,95)(1 − 0,2) = (1 − 0,9)(1 − 0,6).
    let u2 = levanta(e, AM, AM, 0.95, 0.2, 0.9, 0.6);
    assert!(perto(u2, AM, 0.5), "a mesma tinta: {u2:?}");
}

/// **Bug #49 — azul escuro a assentar sobre azul claro seco não vai a verde.** O assentar corre em MUITOS
/// passes (2–5 % da massa cada) e o ecrã tem de ficar o mesmo em todos: 4 azuis claros × 4 escuros, `a_s`
/// 0,2–0,99, `a_f` 0,5–0,95, 60 passes. Antes (cada canal com a derivada do ecrã do início da volta) o
/// pior subia o verde +15 a `d = 0,05` e o assentado ficava verde-água (83,178,211).
#[test]
fn assentar_azul_sobre_azul_nao_muda_o_tom() {
    let e = Ecra::Cobre {
        vela: 0.0,
        filme: 0.0,
        esconde: 5.0,
        molhada: 1.0,
    };
    let claros = [
        [120.0, 180.0, 235.0],
        [90.0, 190.0, 240.0],
        [150.0, 200.0, 255.0],
        [170.0, 210.0, 240.0],
    ];
    let escuros = [
        [20.0, 40.0, 140.0],
        [10.0, 30.0, 110.0],
        [0.0, 0.0, 128.0],
        [40.0, 60.0, 160.0],
    ];
    let mut pior = 0.0f64;
    for sc0 in claros {
        for uc in escuros {
            for a_s0 in [0.2, 0.5, 0.8, 0.95, 0.99] {
                for a_f0 in [0.5, 0.8, 0.95] {
                    for d in [0.02, 0.05] {
                        let (mut sc, mut a_s, mut a_f) = (sc0, a_s0, a_f0);
                        let v0 = sobre_branco(e, sc, uc, a_s, a_f);
                        for _ in 0..60 {
                            let a_s2 = 1.0 - (1.0 - a_s) * (1.0 - d * a_f);
                            let a_f2 = (1.0 - (1.0 - a_f) / (1.0 - d * a_f)).max(0.0);
                            sc = assenta(e, sc, uc, a_s, a_f, a_s2, a_f2);
                            (a_s, a_f) = (a_s2, a_f2);
                            let v = sobre_branco(e, sc, uc, a_s, a_f);
                            pior = pior.max((v[1] - v[2]) - (v0[1] - v0[2]));
                        }
                    }
                }
            }
        }
    }
    assert!(pior <= 3.0, "o assentar subiu o verde {pior:+.1} níveis");
}

/// **Bug #50 — o ecrã fica na caixa das tintas** (dono 12/10: a orla «esbranquiçada» do azul escuro fino sobre
/// o azul claro seco, (71,129,184) entre (35,125,207) e (19,74,125)). O que se vê sobre o branco fica, por
/// canal, entre o assentado visto (`t_s`) e a tinta de cima — o tapar extra não traz o branco da diluição.
/// 6 × 6 tintas (o par do dono primeiro), `a_s` 0,2–0,99, `a_f` 0,02–0,95, tapar do produto e da velatura.
/// Antes da caixa: 61 níveis (vermelho sob azul a `a_f` 0,3 → (179,161,197)).
#[test]
fn o_ecra_fica_na_caixa_das_tintas() {
    let tintas = [
        [35.0, 125.0, 207.0],
        [19.0, 74.0, 125.0],
        [220.0, 60.0, 60.0],
        AM,
        AZ,
        [248.0, 246.0, 240.0],
    ];
    let mut pior = (0.0f64, String::new());
    for esconde in [5.0, 2.0] {
        let e = Ecra::Cobre {
            vela: 0.0,
            filme: 0.0,
            esconde,
            molhada: 1.0,
        };
        for sc in tintas {
            for uc in tintas {
                for a_s in [0.2, 0.6, 0.9, 0.99] {
                    for a_f in [0.02, 0.1, 0.2, 0.3, 0.5, 0.8, 0.95] {
                        let t_s = sc.map(|v| 255.0 - a_s * (255.0 - v));
                        let v = sobre_branco(e, sc, uc, a_s, a_f);
                        let fora = fora_da_caixa(v, t_s, uc);
                        if fora > pior.0 {
                            pior = (
                                fora,
                                format!("{sc:?} sob {uc:?} a_s {a_s} a_f {a_f} → {v:.0?}"),
                            );
                        }
                    }
                }
            }
        }
    }
    assert!(
        pior.0 <= 0.5,
        "o ecrã saiu da caixa {:.1} níveis: {}",
        pior.0,
        pior.1
    );
}

/// Quanto `x` sai, por canal, da caixa de `a` e `b` (níveis; `0` dentro).
fn fora_da_caixa(x: [f64; 3], a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3)
        .map(|c| (x[c] - a[c].max(b[c])).max(a[c].min(b[c]) - x[c]))
        .fold(0.0, f64::max)
}

/// **O re-molhar fica na caixa das tintas** (prompt 16, Fase E — o adversário da Fase D′): em 2 240
/// estados (4 tintas × 4, `a_s` 1e-4–0,95, `a_f` 0,05–0,999, `d` 0,5–5 %), com o tapar do produto e o do
/// «Glaze layering», o filme que a água leva fica, por canal, entre as duas tintas que se juntam. O estado
/// pior do adversário (azul escuro sob quase branco, `a_s` 0,2, `a_f` 0,3, `d` 0,05) entra por nome.
/// CONTROLO: sem a caixa reprova.
#[test]
fn as_transferencias_verticais_ficam_na_caixa_das_tintas() {
    const AZUL: [f64; 3] = [20.0, 50.0, 110.0];
    const VERMELHO: [f64; 3] = [220.0, 60.0, 60.0];
    const QUASE_BRANCO: [f64; 3] = [248.0, 246.0, 240.0];
    let tintas = [AZUL, AM, VERMELHO, QUASE_BRANCO];
    let ecra = |esconde| Ecra::Cobre {
        vela: 0.0,
        filme: crate::colorops::VELATURA_DO_FILME,
        esconde,
        molhada: crate::colorops::TINTA_MOLHADA_ESCURECE,
    };
    let (mut n, mut pior, mut livre) = (0usize, 0.0f64, 0.0f64);
    for esconde in [
        crate::colorops::TINTA_ESCONDE,
        crate::colorops::TINTA_ESCONDE_EM_VELATURA,
    ] {
        let e = ecra(esconde);
        for sc in tintas {
            for uc in tintas {
                for a_s in [1e-4, 1e-3, 1e-2, 0.05, 0.2, 0.6, 0.95] {
                    for a_f in [0.05, 0.3, 0.6, 0.9, 0.96, 0.99, 0.999] {
                        for d in [0.005, 0.02, 0.05] {
                            if a_f <= d + 1e-6 {
                                continue;
                            }
                            let a_s2 = 1.0 - (1.0 - a_s) * (1.0 - d);
                            let a_f2 = 1.0 - (1.0 - a_f) / (1.0 - d);
                            let u2 = levanta(e, sc, uc, a_s2, a_f2, a_s, a_f);
                            let solto = levanta_cobre(e, sc, uc, a_s2, a_f2, a_s, a_f);
                            n += 1;
                            pior = pior.max(fora_da_caixa(u2, sc, uc));
                            livre = livre.max(fora_da_caixa(solto, sc, uc));
                        }
                    }
                }
            }
        }
    }
    assert_eq!(n, 2 * 2240);
    assert!(pior <= 1e-9, "o filme saiu da caixa das tintas: {pior:.3}");
    assert!(
        livre > 1.0,
        "CONTROLO: sem a caixa o filme nunca saiu dela ({livre:.3}) — a varredura não discrimina"
    );
    let e = ecra(crate::colorops::TINTA_ESCONDE);
    let (a_s, a_f, d) = (0.2, 0.3, 0.05);
    let (a_s2, a_f2) = (1.0 - (1.0 - a_s) * (1.0 - d), 1.0 - (1.0 - a_f) / (1.0 - d));
    let u2 = levanta(e, AZUL, QUASE_BRANCO, a_s2, a_f2, a_s, a_f);
    assert!(fora_da_caixa(u2, AZUL, QUASE_BRANCO) <= 1e-9, "{u2:?}");
}

/// Re-molhar tinta SECA (filme vazio) leva a cor do assentado: a cor guardada de um filme sem massa é a de
/// uma tinta que já saiu. CONTROLO: a cor velha do vazio é outra tinta, longe do assentado.
#[test]
fn re_molhar_tinta_seca_leva_a_cor_do_assentado() {
    let velha = [220.0, 60.0, 60.0];
    for esconde in [
        crate::colorops::TINTA_ESCONDE,
        crate::colorops::TINTA_ESCONDE_EM_VELATURA,
    ] {
        let e = Ecra::Cobre {
            vela: 0.0,
            filme: crate::colorops::VELATURA_DO_FILME,
            esconde,
            molhada: crate::colorops::TINTA_MOLHADA_ESCURECE,
        };
        let u2 = levanta(e, AZ, velha, 0.6, 0.0, 0.55, 0.1);
        assert!(
            perto(u2, AZ, 1.0),
            "esconde {esconde}: {u2:?} ≠ o assentado {AZ:?}"
        );
        assert!(
            !perto(velha, AZ, 1.0),
            "CONTROLO: a cor velha não discrimina"
        );
    }
}
