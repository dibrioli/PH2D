//! N.º 7 rodada 4 (13/11): o assentar só pelo PALPITE (0 voltas do Newton) contra a conta exacta, numa secagem de
//! 20 passes por célula — o erro no ECRÃ (níveis de 8 bits). `docs/Painter/missao_4k/medicoes/N7_ITEM1_sim6.md`.
use super::*;
use crate::opacity::alpha_of_mass;

const CORES: [[f64; 3]; 6] = [
    [40.0, 70.0, 200.0],
    [200.0, 40.0, 40.0],
    [230.0, 200.0, 30.0],
    [30.0, 150.0, 60.0],
    [120.0, 40.0, 160.0],
    [250.0, 250.0, 245.0],
];

/// (pior, média) da diferença no ecrã seco final entre a secagem por `aprox` e a exacta.
fn erro(aprox: impl Fn(Ecra, [f64; 3], [f64; 3], f64, f64, f64, f64) -> [f64; 3]) -> (f64, f64) {
    let (mut pior, mut soma, mut n) = (0.0f64, 0.0, 0usize);
    for vela in [0.0, 0.5] {
        let e = Ecra::Cobre {
            filme: 0.2,
            esconde: 3.0,
            molhada: 1.6,
            vela,
            oleo: 0.0,
            opaca: 0.0,
        };
        for sc0 in CORES {
            for uc in CORES {
                for (s0, m0) in [
                    (0.0, 40.0),
                    (5.0, 30.0),
                    (30.0, 10.0),
                    (60.0, 80.0),
                    (150.0, 50.0),
                    (400.0, 300.0),
                ] {
                    let mut ecra = [0.0; 3];
                    for (k, exacta) in [true, false].into_iter().enumerate() {
                        let (mut sc, mut s, mut m) = (sc0, s0, m0);
                        for _ in 0..20 {
                            let dm = m * 0.15;
                            let a = [s, m, s + dm, m - dm].map(alpha_of_mass);
                            sc = if exacta {
                                assenta_com_voltas(VOLTAS_EXATAS, e, sc, uc, a[0], a[1], a[2], a[3])
                            } else {
                                aprox(e, sc, uc, a[0], a[1], a[2], a[3])
                            };
                            // A secagem guarda em `f32`.
                            sc = sc.map(|v| f64::from(v as f32));
                            (s, m) = (s + dm, m - dm);
                        }
                        let v = sobre_branco(e, sc, uc, alpha_of_mass(s), alpha_of_mass(m));
                        if k == 0 {
                            ecra = v;
                        } else {
                            for c in 0..3 {
                                let d = (v[c] - ecra[c]).abs();
                                pior = pior.max(d);
                                soma += d;
                                n += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    (pior, soma / n as f64)
}

/// O palpite fica a ≤ `PIOR` níveis da conta exacta no ecrã seco final; o controlo (o assentado guarda a cor de
/// antes) passa do tecto — a régua morde.
#[test]
fn o_palpite_do_assentar_fica_perto_da_conta_exacta() {
    // Medido 13/11: pior 20,37 · média 0,914 (controlo 107,04 · 12,757).
    const PIOR: f64 = 21.0;
    let (pior, media) = erro(|e, sc, uc, a, b, c, d| assenta_com_voltas(0, e, sc, uc, a, b, c, d));
    println!("palpite × exacta: pior {pior:.2} · média {media:.3} níveis");
    let (cp, cm) = erro(|_, sc, _, _, _, _, _| sc);
    println!("controlo (guarda a cor de antes): pior {cp:.2} · média {cm:.3}");
    assert!(pior <= PIOR, "palpite: pior {pior:.2} > {PIOR}");
    assert!(cp > PIOR, "controlo morto: {cp:.2}");
}
