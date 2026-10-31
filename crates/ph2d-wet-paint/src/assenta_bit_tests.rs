//! **Os cortes da secagem são AO BIT** (31/10, `docs/Painter/49_realshader/SECAGEM_diagnostico.md`): o hash FNV-1a
//! das portas de [`crate::assenta`] sobre uma grelha que cobre os ramos (as quatro leis; na `Cobre` filme, Glaze,
//! vidrado, óleo e o pincel de óleo; a mesma tinta filtrada; branco e preto; o passo da derivada no topo `255`;
//! coberturas nulas, parciais, totais, iguais e diferentes) e de 6 secagens encadeadas (assentar → re-molhar).
use crate::assenta::{
    Ecra, PINCEL_DE_OLEO, assenta, cor, cor_e_alfa, levanta, na_caixa_das_tintas, sobre_branco,
};

const CORES: [[f64; 3]; 8] = [
    [200.0, 40.0, 40.0],
    [151.0, 29.0, 29.0],
    [190.0, 55.0, 55.0],
    [40.0, 60.0, 200.0],
    [245.0, 205.0, 25.0],
    [255.0, 255.0, 255.0],
    [0.0, 0.0, 0.0],
    [254.8, 10.0, 128.0],
];
const COB: [f64; 5] = [0.0, 0.05, 0.3, 0.7, 1.0];

fn ecras() -> Vec<Ecra> {
    let c = |filme, esconde, molhada, vela, oleo| Ecra::Cobre {
        opaca: 0.0,
        filme,
        esconde,
        molhada,
        vela,
        oleo,
    };
    vec![
        Ecra::Porto,
        Ecra::PorCima(0.0),
        Ecra::PorCima(0.4),
        Ecra::Camada,
        c(0.0, 5.0, 1.6, 0.0, 0.0),
        c(0.3, 5.0, 1.6, 0.0, 0.0),
        c(0.0, 5.0, 1.0, 0.6, 0.0),
        c(0.0, 3.0, 1.3, 1.0, 0.0),
        c(0.3, 2.5, 1.0, 0.6, 0.5),
        c(0.2, 5.0, 1.0, 0.0, PINCEL_DE_OLEO),
        c(0.0, 5.0, 1.0, 0.0, 0.0).com_vidrado(0.5),
        c(0.25, 5.0, 1.2, 0.8, 0.0).com_vidrado(1.0),
    ]
}

struct Fnv(u64);
impl Fnv {
    fn come(&mut self, v: &[f64]) {
        for x in v {
            for b in x.to_bits().to_le_bytes() {
                self.0 ^= u64::from(b);
                self.0 = self.0.wrapping_mul(0x0100_0000_01b3);
            }
        }
    }
}

/// As coberturas de depois: iguais, assentar (desce para o papel), re-molhar (sobe), só o filme muda.
fn depois(a_s: f64, a_f: f64) -> [(f64, f64); 4] {
    [
        (a_s, a_f),
        ((a_s + 0.2).min(1.0), a_f * 0.6),
        (a_s * 0.7, (a_f + 0.15).min(1.0)),
        (a_s, (a_f * 0.8 + 0.1).min(1.0)),
    ]
}

fn hash_das_portas() -> u64 {
    let mut h = Fnv(0xcbf2_9ce4_8422_2325);
    for e in ecras() {
        for sc in CORES {
            for uc in CORES {
                for a_s in COB {
                    for a_f in COB {
                        let ca = cor_e_alfa(e, sc, uc, a_s, a_f);
                        h.come(&ca.0);
                        h.come(&[ca.1]);
                        let cx = na_caixa_das_tintas(e, ca, sc, uc, a_s, a_f);
                        h.come(&cx.0);
                        h.come(&[cx.1]);
                        h.come(&sobre_branco(e, sc, uc, a_s, a_f));
                        h.come(&cor(e, sc, uc, a_s, a_f));
                        for (a_s2, a_f2) in depois(a_s, a_f) {
                            h.come(&assenta(e, sc, uc, a_s, a_f, a_s2, a_f2));
                            h.come(&levanta(e, sc, uc, a_s, a_f, a_s2, a_f2));
                        }
                    }
                }
            }
        }
    }
    h.0
}

/// 6 secagens encadeadas: cada uma assenta parte do filme e re-molha parte do assentado, com a cor de saída
/// a entrar na seguinte (como a `f32` da grelha).
fn hash_das_secagens() -> u64 {
    let mut h = Fnv(0xcbf2_9ce4_8422_2325);
    for e in ecras() {
        for (i, sc0) in CORES.iter().enumerate() {
            let uc0 = CORES[(i * 3 + 1) % CORES.len()];
            let (mut sc, mut uc, mut a_s, mut a_f) = (*sc0, uc0, 0.3, 0.6);
            for passo in 0..6 {
                let (a_s2, a_f2) = ((a_s + 0.1f64).min(1.0), a_f * 0.7);
                sc = assenta(e, sc, uc, a_s, a_f, a_s2, a_f2).map(|v| f64::from(v as f32));
                (a_s, a_f) = (a_s2, a_f2);
                // Re-molhar: coberturas iguais nos passos pares (o atalho), diferentes nos ímpares.
                let (a_s3, a_f3) = if passo % 2 == 0 {
                    (a_s, a_f)
                } else {
                    (a_s * 0.9, (a_f + 0.05f64).min(1.0))
                };
                uc = levanta(e, sc, uc, a_s, a_f, a_s3, a_f3).map(|v| f64::from(v as f32));
                (a_s, a_f) = (a_s3, a_f3);
                h.come(&sc);
                h.come(&uc);
            }
        }
    }
    h.0
}

/// Gravados em `47d8c1083e`, ANTES dos cortes (LADO · CANAL · MEMO · SALTA-LEVANTA).
const HASH_PORTAS: u64 = 0x34a0_9a1b_154d_6de4;
const HASH_SECAGENS: u64 = 0xfd8b_6ec6_311b_a0bd;

#[test]
fn assenta_bit_as_portas_nao_mudam_um_bit() {
    assert_eq!(hash_das_portas(), HASH_PORTAS, "{:016x}", hash_das_portas());
}

#[test]
fn assenta_bit_as_secagens_nao_mudam_um_bit() {
    assert_eq!(
        hash_das_secagens(),
        HASH_SECAGENS,
        "{:016x}",
        hash_das_secagens()
    );
}

/// O memo do assentar (rodada 4, `assenta/memo.rs`): pela tabela do fio, fria e quente (duas voltas), com cores
/// `f32` (o caminho da tabela) e `f64` (o que a contorna), os MESMOS bits que a porta.
#[test]
fn assenta_bit_o_memo_da_os_bits_da_porta() {
    let f32_de = |c: [f64; 3]| c.map(|v| f64::from(v as f32));
    for _volta in 0..2 {
        for e in ecras() {
            for sc0 in CORES {
                for uc0 in CORES {
                    for (sc, uc) in [(f32_de(sc0), f32_de(uc0)), (sc0, uc0)] {
                        for a_s in COB {
                            for a_f in COB {
                                for (a_s2, a_f2) in depois(a_s, a_f) {
                                    let m = super::assenta_memo(e, sc, uc, a_s, a_f, a_s2, a_f2);
                                    let p = assenta(e, sc, uc, a_s, a_f, a_s2, a_f2);
                                    assert_eq!(
                                        m.map(f64::to_bits),
                                        p.map(f64::to_bits),
                                        "{e:?} {sc:?} {uc:?}"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// O memo de UMA linha (rodada 5, [`super::assenta_memo_f32`]): cores `f32`, coberturas `f32` exactas (o caminho da
/// tabela) e `f64` (o que a contorna), fria e quente — os bits de `assenta(..)` arredondado a `f32`.
#[test]
fn assenta_bit_o_memo_f32_da_os_bits_da_porta() {
    let f32_de = |c: [f64; 3]| c.map(|v| v as f32);
    let cob32 = |a: f64| f64::from(a as f32);
    for _volta in 0..2 {
        for e in ecras() {
            for sc0 in CORES {
                for uc0 in CORES {
                    let (sc, uc) = (f32_de(sc0), f32_de(uc0));
                    for a_s in COB {
                        for a_f in COB {
                            for (a_s2, a_f2) in depois(a_s, a_f) {
                                for al in
                                    [[a_s, a_f, a_s2, a_f2], [a_s, a_f, a_s2, a_f2].map(cob32)]
                                {
                                    let m = super::assenta_memo_f32(
                                        e, sc, uc, al[0], al[1], al[2], al[3],
                                    );
                                    let p = assenta(
                                        e,
                                        sc.map(f64::from),
                                        uc.map(f64::from),
                                        al[0],
                                        al[1],
                                        al[2],
                                        al[3],
                                    )
                                    .map(|v| v as f32);
                                    assert_eq!(
                                        m.map(f32::to_bits),
                                        p.map(f32::to_bits),
                                        "{e:?} {sc:?} {uc:?} {al:?}"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
