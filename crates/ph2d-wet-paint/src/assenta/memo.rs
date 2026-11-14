//! **MEMO do [`assenta`] — ao bit** (31/10, rodada 4, `docs/Painter/49_realshader/MEDIDA_31_10_RODADA4.md`).
//!
//! O `assenta` é uma função PURA das suas entradas, e elas são quantizadas: as quatro coberturas saem da tabela
//! de [`crate::opacity::alpha_of_mass`] (índice inteiro da massa) e as cores são `f32` guardadas. Num wash ~95 %
//! das células pedem uma conta que outra célula do mesmo fio já fez; uma tabela de endereço directo por fio, com a
//! chave = os bits de TODAS as entradas, devolve o resultado guardado. Igualdade de bits ⇒ o mesmo resultado.
//! A entrada cabe numa linha de cache (64 B): a régua mediu que o custo de uma consulta é a falha de memória.
use std::cell::RefCell;
use std::sync::atomic::{AtomicU8, AtomicU32, AtomicU64, Ordering::Relaxed};

use super::{Ecra, assenta};

/// A rota do assentar na secagem — `1` = com o memo (o produto), `0` = sem (a régua A/B e os portões de
/// identidade): os dois dão o mesmo grid ao bit. `2` = com o memo, a contar ([`MEMO_CONTA`]); `3` = SÓ a régua do
/// chão (devolve `sc` sem conta — resultado errado, mede o que a secagem custa sem as transferências). O `levanta` pela
/// mesma tabela foi medido e não ganha (rodada 4).
#[doc(hidden)]
pub static MEMO_ASSENTA: AtomicU8 = AtomicU8::new(1);

/// A geração das tabelas: [`memo_esquece`] sobe-a e toda entrada guardada deixa de valer.
static GERACAO: AtomicU32 = AtomicU32::new(1);

/// Esquece as tabelas de todos os fios (a régua mede a tabela FRIA em cada corrida).
#[doc(hidden)]
pub fn memo_esquece() {
    GERACAO.fetch_add(1, Relaxed);
}

/// Acertos e falhas, contados só com `MEMO_ASSENTA = 2` (a régua).
#[doc(hidden)]
pub static MEMO_CONTA: [AtomicU64; 2] = [AtomicU64::new(0), AtomicU64::new(0)];

/// Os bits do índice da tabela (≤ [`BITS_MAX`]) — a régua varre-os no mesmo processo.
#[doc(hidden)]
pub static MEMO_BITS: AtomicU8 = AtomicU8::new(15);

const BITS_MAX: u8 = 16;
const N: usize = 1 << BITS_MAX;
/// Os bits da tabela de leis (endereço directo): o Glaze, o óleo e a opaca variam por célula, e uma lei nova toma o
/// lugar de outra em vez de desligar o memo (B13.8: com uma lista de 64 a 65.ª lei do fio deixava-o morto para sempre).
const LEI_BITS: u32 = 10;
/// As vias de cada balde da tabela de leis (associativa: leis cujo endereço colide não se expulsam umas às outras).
const VIAS: usize = 4;

#[derive(Clone, Copy)]
#[repr(C, align(64))]
struct Entrada {
    /// `sc` e `uc` em `f32` (exactas: ver [`f32_exacta`]).
    cor: [u32; 6],
    a: [u64; 4],
    ok: u32,
    lei: u32,
}

/// A entrada de UMA linha (rodada 5): as coberturas são `f32` exactas (a tabela de `alpha_of_mass` é `f32`) e o
/// resultado guarda-se já em `f32`, que é o que a secagem escreve — chave e resultado na MESMA linha de cache.
#[derive(Clone, Copy)]
#[repr(C, align(64))]
struct Linha {
    cor: [u32; 6],
    a: [u32; 4],
    val: [f32; 3],
    ok: u32,
    lei: u32,
}

struct Tabela {
    l: Vec<Linha>,
    e: Vec<Entrada>,
    val: Vec<[f64; 3]>,
    /// `(lei, id)` por endereço directo; `id = 0` é o lugar vazio. Os ids nunca se repetem na tabela ([`Tabela::proxima`]),
    /// por isso uma entrada guardada com a lei expulsa nunca casa.
    leis: Vec<([u64; 6], u32)>,
    proxima: u32,
    geracao: u32,
    /// A última lei pedida e o seu id (`0` = nenhuma): a secagem pede a MESMA lei célula após célula.
    ultima: ([u64; 6], u32),
}

impl Tabela {
    fn nova(g: u32) -> Self {
        Self {
            l: Vec::new(),
            e: Vec::new(),
            val: Vec::new(),
            leis: Vec::new(),
            proxima: 1,
            geracao: g,
            ultima: ([0; 6], 0),
        }
    }

    /// O id da lei `k` nesta tabela (dá-lhe um novo se não estiver). Uma geração nova esquece leis e entradas.
    fn id_da_lei(&mut self, k: [u64; 6], g: u32) -> u32 {
        if self.proxima == u32::MAX {
            // Os ids voltariam a repetir-se: as entradas guardadas com eles saem também.
            self.l.clear();
            self.e.clear();
            self.val.clear();
            self.geracao = g.wrapping_sub(1);
        }
        if self.geracao != g {
            // As entradas da geração velha já não casam (`ok == g`); os ids podem recomeçar.
            self.leis.clear();
            self.ultima.1 = 0;
            self.proxima = 1;
            self.geracao = g;
        }
        // Sem o atalho a procura pela tabela custava +15 % à secagem de uma lei só (B13.8, `N10_agua_wet.md`).
        if self.ultima.1 != 0 && self.ultima.0 == k {
            return self.ultima.1;
        }
        if self.leis.is_empty() {
            self.leis = vec![([0; 6], 0); 1 << LEI_BITS];
        }
        let mut h = 0u64;
        for (j, &v) in k.iter().enumerate() {
            h = h.wrapping_add((v ^ (v >> 29)).wrapping_mul(MULT[j]));
        }
        let i = ((h ^ (h >> 32)).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> (64 - LEI_BITS)) as usize
            & !(VIAS - 1);
        let balde = &mut self.leis[i..i + VIAS];
        if let Some(l) = balde.iter().find(|l| l.1 != 0 && l.0 == k) {
            self.ultima = (k, l.1);
            return l.1;
        }
        // Expulsa a mais velha do balde (o id mais baixo; o vazio tem 0).
        let velha = balde.iter_mut().min_by_key(|l| l.1).expect("VIAS > 0");
        *velha = (k, self.proxima);
        self.proxima += 1;
        self.ultima = *velha;
        velha.1
    }
}

thread_local! {
    static TABELAS: RefCell<[Option<Tabela>; 1]> = const { RefCell::new([None]) };
}

#[inline]
fn f32_exacta(v: f64) -> Option<u32> {
    let f = v as f32;
    (f64::from(f).to_bits() == v.to_bits()).then_some(f.to_bits())
}

/// [`assenta`] pela tabela do fio (ao bit). Só a [`Ecra::Cobre`] passa pelo memo — as outras leis são baratas.
#[inline]
#[must_use]
pub fn assenta_memo(
    e: Ecra,
    sc: [f64; 3],
    uc: [f64; 3],
    a_s: f64,
    a_f: f64,
    a_s2: f64,
    a_f2: f64,
) -> [f64; 3] {
    pela_tabela(0, assenta, e, sc, uc, [a_s, a_f, a_s2, a_f2])
}

/// [`assenta_memo`] já em `f32` (o que a secagem guarda): a tabela de uma linha por entrada ([`Linha`]). Os bits
/// são os de `assenta(..).map(|v| v as f32)`.
#[inline]
#[must_use]
pub fn assenta_memo_f32(
    e: Ecra,
    sc: [f32; 3],
    uc: [f32; 3],
    a_s: f64,
    a_f: f64,
    a_s2: f64,
    a_f2: f64,
) -> [f32; 3] {
    let conta = || {
        let r = assenta(
            e,
            [sc[0] as f64, sc[1] as f64, sc[2] as f64],
            [uc[0] as f64, uc[1] as f64, uc[2] as f64],
            a_s,
            a_f,
            a_s2,
            a_f2,
        );
        [r[0] as f32, r[1] as f32, r[2] as f32]
    };
    let Ecra::Cobre {
        filme,
        esconde,
        molhada,
        vela,
        oleo,
        opaca,
    } = e
    else {
        return conta();
    };
    let modo = MEMO_ASSENTA.load(Relaxed);
    match modo {
        0 => return conta(),
        3 => return sc,
        _ => {}
    }
    let (Some(a0), Some(a1), Some(a2), Some(a3)) = (
        f32_exacta(a_s),
        f32_exacta(a_f),
        f32_exacta(a_s2),
        f32_exacta(a_f2),
    ) else {
        return conta();
    };
    let a = [a0, a1, a2, a3];
    let cor: [u32; 6] = [
        sc[0].to_bits(),
        sc[1].to_bits(),
        sc[2].to_bits(),
        uc[0].to_bits(),
        uc[1].to_bits(),
        uc[2].to_bits(),
    ];
    let lei_k = [
        filme.to_bits(),
        esconde.to_bits(),
        molhada.to_bits(),
        vela.to_bits(),
        oleo.to_bits(),
        opaca.to_bits(),
    ];
    let g = GERACAO.load(Relaxed);
    let bits = u32::from(MEMO_BITS.load(Relaxed).min(BITS_MAX));
    TABELAS.with(|t| {
        let mut t = t.borrow_mut();
        let t = t[0].get_or_insert_with(|| Tabela::nova(g));
        let lei = t.id_da_lei(lei_k, g);
        if t.l.is_empty() {
            t.l = vec![
                Linha {
                    cor: [0; 6],
                    a: [0; 4],
                    val: [0.0; 3],
                    ok: 0,
                    lei: 0,
                };
                N
            ];
        }
        let mut h = u64::from(lei).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        for (k, &v) in cor.iter().chain(a.iter()).enumerate() {
            h = h.wrapping_add(u64::from(v).wrapping_mul(MULT[k]));
        }
        h = (h ^ (h >> 32)).wrapping_mul(0xD6E8_FEB8_6659_FD93);
        let i = (h >> (64 - bits)) as usize;
        let en = &t.l[i];
        let acerta = en.ok == g && en.lei == lei && en.cor == cor && en.a == a;
        if modo == 2 {
            MEMO_CONTA[usize::from(!acerta)].fetch_add(1, Relaxed);
        }
        if acerta {
            return en.val;
        }
        let r = conta();
        t.l[i] = Linha {
            cor,
            a,
            val: r,
            ok: g,
            lei,
        };
        r
    })
}

/// B13.12: `potencia` pela tabela do fio (`1`, o produto) ou o `pow` direto (`0`, o oráculo A×B).
#[doc(hidden)]
pub static MEMO_POTENCIA: AtomicU8 = AtomicU8::new(1);

const POT_BITS: u32 = 12;

thread_local! {
    static POTENCIAS: RefCell<Vec<(u64, u64, f64)>> = const { RefCell::new(Vec::new()) };
}

/// `x^e` (`libm::pow`, a do [`super::tapa`]) por uma tabela de endereço directo por fio com a chave = os bits de `x`
/// e de `e`: as coberturas saem da tabela de `alpha_of_mass` e o `a_dm` de pares dela, e repetem-se — o mesmo
/// argumento dá o mesmo resultado ao bit. A chave inválida é `x = NaN` com todos os bits (nenhum `1 − a` a dá).
#[inline]
pub(super) fn potencia(x: f64, e: f64) -> f64 {
    use ph2d_det_math::DetMath as _;
    if MEMO_POTENCIA.load(Relaxed) == 0 {
        return x.det_powf(e);
    }
    let (kx, ke) = (x.to_bits(), e.to_bits());
    let h = (kx ^ ke.rotate_left(29)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let i = (h >> (64 - POT_BITS)) as usize;
    POTENCIAS.with(|t| {
        let mut t = t.borrow_mut();
        if t.is_empty() {
            *t = vec![(u64::MAX, 0, 0.0); 1 << POT_BITS];
        }
        let en = t[i];
        if en.0 == kx && en.1 == ke {
            return en.2;
        }
        let v = x.det_powf(e);
        t[i] = (kx, ke, v);
        v
    })
}

type Porta = fn(Ecra, [f64; 3], [f64; 3], f64, f64, f64, f64) -> [f64; 3];

/// A tabela `qual` do fio (só a `0`, o assentar: o `levanta` memoizado foi medido sem ganho).
#[inline]
fn pela_tabela(
    qual: usize,
    porta: Porta,
    e: Ecra,
    sc: [f64; 3],
    uc: [f64; 3],
    al: [f64; 4],
) -> [f64; 3] {
    let [a_s, a_f, a_s2, a_f2] = al;
    let conta = || porta(e, sc, uc, a_s, a_f, a_s2, a_f2);
    let Ecra::Cobre {
        filme,
        esconde,
        molhada,
        vela,
        oleo,
        opaca,
    } = e
    else {
        return conta();
    };
    let modo = MEMO_ASSENTA.load(Relaxed);
    match modo {
        0 => return conta(),
        3 => return sc,
        _ => {}
    }
    let mut cor = [0u32; 6];
    for (k, v) in sc.into_iter().chain(uc).enumerate() {
        match f32_exacta(v) {
            Some(b) => cor[k] = b,
            None => return conta(),
        }
    }
    let a = [a_s, a_f, a_s2, a_f2].map(f64::to_bits);
    let lei_k = [filme, esconde, molhada, vela, oleo, opaca].map(f64::to_bits);
    let g = GERACAO.load(Relaxed);
    let bits = u32::from(MEMO_BITS.load(Relaxed).min(BITS_MAX));
    TABELAS.with(|t| {
        let mut t = t.borrow_mut();
        let t = t[qual].get_or_insert_with(|| Tabela::nova(g));
        let lei = t.id_da_lei(lei_k, g);
        if t.e.is_empty() {
            t.e = vec![
                Entrada {
                    cor: [0; 6],
                    a: [0; 4],
                    ok: 0,
                    lei: 0,
                };
                N
            ];
            t.val = vec![[0.0; 3]; N];
        }
        // Multiplicações independentes e uma mistura final.
        let mut h = u64::from(lei).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        for (k, &v) in cor.iter().enumerate() {
            h = h.wrapping_add(u64::from(v).wrapping_mul(MULT[k]));
        }
        for (k, &v) in a.iter().enumerate() {
            h = h.wrapping_add((v ^ (v >> 31)).wrapping_mul(MULT[6 + k]));
        }
        h = (h ^ (h >> 32)).wrapping_mul(0xD6E8_FEB8_6659_FD93);
        let i = (h >> (64 - bits)) as usize;
        let en = &t.e[i];
        let acerta = en.ok == g && en.lei == lei && en.cor == cor && en.a == a;
        if modo == 2 {
            MEMO_CONTA[usize::from(!acerta)].fetch_add(1, Relaxed);
        }
        if acerta {
            return t.val[i];
        }
        let r = conta();
        t.e[i] = Entrada { cor, a, ok: g, lei };
        t.val[i] = r;
        r
    })
}

const MULT: [u64; 10] = [
    0xBF58_476D_1CE4_E5B9,
    0x94D0_49BB_1331_11EB,
    0xD6E8_FEB8_6659_FD93,
    0xA076_1D64_78BD_642F,
    0xE703_7ED1_A0B4_28DB,
    0x8EBC_6AF0_9C88_C6E3,
    0x5899_65CC_7537_4CC3,
    0x1D8E_4E27_C47D_124F,
    0xC2B2_AE3D_27D4_EB4F,
    0x1656_67B1_9E37_79F9,
];

#[cfg(test)]
mod potencia_tests {
    use super::*;
    use ph2d_det_math::DetMath as _;

    /// B13.12: a tabela do `pow` dá o `libm::pow` ao bit — coberturas da tabela, pares `a_dm`, vários `esconde`
    /// (colisões e reescritas incluídas: 3 001 × 2 voltas numa tabela de 4 096).
    #[test]
    fn a_potencia_pela_tabela_e_o_pow_ao_bit() {
        let al: Vec<f64> = (0..=3000).map(crate::opacity::table_at).collect();
        for e in [5.0, 1.0, 2.5, crate::colorops::TINTA_ESCONDE_EM_VELATURA] {
            for volta in 0..2 {
                for (k, &a) in al.iter().enumerate() {
                    let b = al[(k * 7 + volta) % al.len()];
                    let a_dm = if a < 1.0 {
                        1.0 - (1.0 - b) / (1.0 - a)
                    } else {
                        0.0
                    };
                    for x in [1.0 - a, 1.0 - a_dm] {
                        assert_eq!(
                            potencia(x, e).to_bits(),
                            x.det_powf(e).to_bits(),
                            "x {x} e {e}"
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod leis_tests {
    use super::*;

    /// B13.8 (T1-45): mais de 64 leis de célula no mesmo fio (o Glaze de duas camadas varia por célula) — a segunda
    /// passagem pelas MESMAS contas acerta ≥ 97 % no memo (296 de 300), e o resultado é o `assenta` ao bit. Com a lista de 64 leis
    /// a 65.ª desligava o memo do fio para sempre (192 acertos em 600).
    #[test]
    fn o_memo_acerta_com_mais_de_64_leis() {
        let leis: Vec<Ecra> = (0..100)
            .map(|k| Ecra::Cobre {
                filme: 0.3,
                esconde: 2.5,
                molhada: 1.0,
                vela: f64::from(k as f32 / 100.0),
                oleo: 0.0,
                opaca: 0.0,
            })
            .collect();
        let cores: [([f32; 3], [f32; 3]); 3] = [
            ([0.2, 0.4, 0.6], [0.9, 0.1, 0.3]),
            ([0.5, 0.5, 0.5], [0.1, 0.8, 0.2]),
            ([0.7, 0.3, 0.1], [0.3, 0.3, 0.9]),
        ];
        let al = |m: f64| crate::opacity::alpha_of_mass(m);
        let (a_s, a_f, a_s2, a_f2) = (al(0.4), al(0.2), al(0.6), al(0.05));
        MEMO_ASSENTA.store(2, Relaxed);
        let mut acertos = 0;
        for volta in 0..2 {
            let antes = MEMO_CONTA[0].load(Relaxed);
            for &e in &leis {
                for (sc, uc) in cores {
                    let m = assenta_memo_f32(e, sc, uc, a_s, a_f, a_s2, a_f2);
                    let r = assenta(
                        e,
                        sc.map(f64::from),
                        uc.map(f64::from),
                        a_s,
                        a_f,
                        a_s2,
                        a_f2,
                    );
                    assert_eq!(m.map(f32::to_bits), r.map(|v| (v as f32).to_bits()));
                }
            }
            if volta == 1 {
                acertos = MEMO_CONTA[0].load(Relaxed) - antes;
            }
        }
        MEMO_ASSENTA.store(1, Relaxed);
        let total = (leis.len() * cores.len()) as u64;
        // O resto são colisões da tabela de contas (300 chaves em 32 Ki lugares), não leis expulsas.
        assert!(acertos * 100 >= total * 97, "acertos {acertos} de {total}");
    }
}
