//! O apoio dos gates: os fixtures, a descompressão, e o corte que fabrica bordo.
//!
//! ⚠️ **Os fixtures são DADOS, e a proveniência deles está escrita ao lado deles**
//! (`docs/3D/cleanroom/fixtures/README.md`): a malha e o campo são nossos, o mapa foi
//! calculado por um programa independente corrido **fora da árvore**, e a saída de um
//! programa não é coberta pela licença do programa.

#![allow(dead_code)]

use ph2d_quadextract::mapa::Mapa;

/// A pasta dos fixtures, a partir desta crate.
pub fn fixture_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/3D/cleanroom/fixtures")
}

/// **DESCOMPRIME um `.gz`** sem sair da árvore.
///
/// ⚠️ O cabeçalho do gzip é de tamanho variável (nome e comentário são opcionais), e
/// o rabo são oito bytes de CRC e tamanho que o inflate não quer ver. *Assumir dez
/// bytes fixos funciona até ao primeiro ficheiro gravado com o nome lá dentro* — e
/// os nossos são.
pub fn gunzip(raw: &[u8]) -> Vec<u8> {
    assert!(raw.len() > 18, "ficheiro curto demais para ser gzip");
    assert_eq!(&raw[..2], &[0x1f, 0x8b], "nao e' gzip");
    let flg = raw[3];
    let mut off = 10usize;
    if flg & 0b0000_0100 != 0 {
        let n = usize::from(u16::from_le_bytes([raw[off], raw[off + 1]]));
        off += 2 + n;
    }
    for bit in [0b0000_1000u8, 0b0001_0000] {
        if flg & bit != 0 {
            while raw[off] != 0 {
                off += 1;
            }
            off += 1;
        }
    }
    if flg & 0b0000_0010 != 0 {
        off += 2;
    }
    miniz_oxide::inflate::decompress_to_vec(&raw[off..raw.len() - 8]).expect("o fixture nao inflou")
}

/// Lê um fixture pelo nome do ficheiro `.mapa.gz`.
pub fn load(name: &str) -> Mapa {
    let raw = std::fs::read(fixture_dir().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"));
    let text = String::from_utf8(gunzip(&raw)).expect("o fixture nao e' UTF-8");
    Mapa::parse(&text).unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// O toro — **género 1**, e a peça que já expôs uma perda de asa nesta linha.
pub fn torus() -> Mapa {
    load("torus_64x32.mapa.gz")
}

/// O gancho orgânico, fechado — o caso realista.
pub fn hooked() -> Mapa {
    load("sculpt_hooked.mapa.gz")
}

/// ⭐ **ABRE UM BURACO EM DISCO** — o fixture de BORDO, que a espec diz não ter
/// oráculo (a integração de referência cai com falha de segmentação em malha com
/// bordo, medido em duas peças).
///
/// ⚠️ **Cresce por adjacência a partir de uma semente**, e não por índice: um
/// conjunto arbitrário de faces abre buracos espalhados e pode partir o leque de um
/// vértice em dois, e aí o que se mede deixa de ser *bordo*.
pub fn with_hole(src: &Mapa, seed: usize, count: usize) -> Mapa {
    use std::collections::{BTreeMap, BTreeSet, VecDeque};
    let mut side: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
    for (f, t) in src.tris.iter().enumerate() {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            side.entry(if a < b { (a, b) } else { (b, a) })
                .or_default()
                .push(f);
        }
    }
    let mut nb: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
    for l in side.values() {
        if l.len() == 2 {
            nb.entry(l[0]).or_default().insert(l[1]);
            nb.entry(l[1]).or_default().insert(l[0]);
        }
    }
    let mut rm: BTreeSet<usize> = BTreeSet::new();
    rm.insert(seed);
    let mut q = VecDeque::from([seed]);
    while rm.len() < count {
        let Some(cur) = q.pop_front() else { break };
        for &g in nb.get(&cur).into_iter().flatten() {
            if rm.len() >= count {
                break;
            }
            if rm.insert(g) {
                q.push_back(g);
            }
        }
    }
    let mut out = Mapa {
        pos: src.pos.clone(),
        ..Mapa::default()
    };
    for (f, t) in src.tris.iter().enumerate() {
        if rm.contains(&f) {
            continue;
        }
        out.tris.push(*t);
        out.uv.push(src.uv[f]);
    }
    out
}

/// ⭐⭐ **RE-CALIBRA o mapa por translações INTEIRAS, uma por carta** — a liberdade
/// que um mapa de grade inteira de facto tem.
///
/// ⚠️ **Ela não muda o mapa, muda a REPRESENTAÇÃO dele:** as transições passam de
/// `t` para `t + o_g − R(r)·o_f`, que continua inteiro, e as isolinhas inteiras são
/// exactamente as mesmas. ⇒ *a extracção tem de devolver o mesmo*, e o que muda é só
/// a magnitude dos números — que é precisamente o que a lei da precisão (§2.3)
/// afirma ser o problema.
///
/// O gerador é um LCG fixo: uma fixtura que muda entre corridas não é uma fixtura.
pub fn regauge(src: &Mapa, spread: i64) -> Mapa {
    let mut out = src.clone();
    let mut state: u64 = 0x2545_F491_4F6C_DD1D;
    for tri in &mut out.uv {
        let mut next = || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            #[allow(clippy::cast_possible_wrap)]
            let v = (state >> 33) as i64;
            v % (2 * spread + 1) - spread
        };
        let o = [next(), next()];
        #[allow(clippy::cast_precision_loss)]
        let o = [o[0] as f64, o[1] as f64];
        for c in tri.iter_mut() {
            c[0] += o[0];
            c[1] += o[1];
        }
    }
    out
}
