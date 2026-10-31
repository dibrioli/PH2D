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
/// Leis de célula distintas que uma tabela guarda (o Glaze, o óleo e a opaca variam por célula); acima, sem memo.
const LEIS: usize = 64;

#[derive(Clone, Copy)]
#[repr(C, align(64))]
struct Entrada {
    /// `sc` e `uc` em `f32` (exactas: ver [`f32_exacta`]).
    cor: [u32; 6],
    a: [u64; 4],
    ok: u32,
    lei: u32,
}

struct Tabela {
    e: Vec<Entrada>,
    val: Vec<[f64; 3]>,
    leis: Vec<[u64; 6]>,
    geracao: u32,
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
        let t = t[qual].get_or_insert_with(|| Tabela {
            e: vec![
                Entrada {
                    cor: [0; 6],
                    a: [0; 4],
                    ok: 0,
                    lei: 0,
                };
                N
            ],
            val: vec![[0.0; 3]; N],
            leis: Vec::with_capacity(LEIS),
            geracao: g,
        });
        if t.geracao != g {
            t.leis.clear();
            t.geracao = g;
        }
        let lei = match t.leis.iter().position(|l| *l == lei_k) {
            Some(p) => p,
            None if t.leis.len() < LEIS => {
                t.leis.push(lei_k);
                t.leis.len() - 1
            }
            None => return conta(),
        } as u32;
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
