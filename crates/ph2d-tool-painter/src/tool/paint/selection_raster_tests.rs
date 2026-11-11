//! **B40.A2** — o Feather da seleção: a soma deslizante na janela caixa ⊕ 3r dá os três `box_blur` de
//! antes ao bit (oráculo verbatim abaixo), e a sonda 4096² r=64 mede o custo.

use super::FEATHER_MAX_PX;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::RasterEditTool as _;
use std::sync::Arc;
use std::time::Instant;

/// O `box_blur` de antes do B40.A2, VERBATIM (O(r) por texel, passe V contra o stride).
fn box_blur_antigo(src: &[u8], w: usize, h: usize, r: usize) -> Vec<u8> {
    if r == 0 || w == 0 || h == 0 || src.len() != w * h {
        return src.to_vec();
    }
    let win = (2 * r + 1) as u32;
    let mut tmp = vec![0u8; w * h];
    for y in 0..h {
        let row = y * w;
        for x in 0..w {
            let mut acc = 0u32;
            for k in 0..(2 * r + 1) {
                let xx = (x + k).saturating_sub(r).min(w - 1);
                acc += u32::from(src[row + xx]);
            }
            tmp[row + x] = (acc / win) as u8;
        }
    }
    let mut out = vec![0u8; w * h];
    for x in 0..w {
        for y in 0..h {
            let mut acc = 0u32;
            for k in 0..(2 * r + 1) {
                let yy = (y + k).saturating_sub(r).min(h - 1);
                acc += u32::from(tmp[yy * w + x]);
            }
            out[y * w + x] = (acc / win) as u8;
        }
    }
    out
}

fn feather_antigo(crisp: &[u8], w: usize, h: usize, r: usize) -> Vec<u8> {
    let m = box_blur_antigo(crisp, w, h, r);
    let m = box_blur_antigo(&m, w, h, r);
    box_blur_antigo(&m, w, h, r)
}

/// Ruído determinístico (xorshift) num retângulo `[x0,x1)×[y0,y1)`, zero fora dele.
fn mancha(w: usize, h: usize, x0: usize, y0: usize, x1: usize, y1: usize, semente: u32) -> Vec<u8> {
    let mut s = semente | 1;
    let mut v = vec![0u8; w * h];
    for y in y0..y1 {
        for x in x0..x1 {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            v[y * w + x] = if s.is_multiple_of(3) {
                255
            } else {
                (s >> 8) as u8
            };
        }
    }
    v
}

#[test]
fn a_soma_deslizante_da_o_box_blur_ao_bit() {
    // Janela no meio, a tocar cada borda, maior que a tela, e r maior que a tela.
    let casos: [(usize, usize, [usize; 4], usize); 7] = [
        (64, 48, [20, 15, 30, 25], 3),
        (64, 48, [0, 0, 10, 48], 5),
        (64, 48, [50, 40, 64, 48], 2),
        (64, 48, [0, 0, 64, 48], 7),
        (37, 23, [5, 4, 9, 7], 40),
        (200, 9, [90, 2, 120, 5], 13),
        (1, 1, [0, 0, 1, 1], 4),
    ];
    for (i, (w, h, [x0, y0, x1, y1], r)) in casos.into_iter().enumerate() {
        let crisp = mancha(w, h, x0, y0, x1, y1, 0x9E37_79B9 ^ i as u32);
        let oraculo = feather_antigo(&crisp, w, h, r);
        for fios in [1usize, 2, 3, 7, 16] {
            crate::compositor::FIOS_FORCADOS.set(Some(fios));
            let hoje = super::feather(&crisp, w, h, r);
            crate::compositor::FIOS_FORCADOS.set(None);
            assert_eq!(
                hoje, oraculo,
                "caso {i}: {w}×{h} caixa {x0},{y0}..{x1},{y1} r={r} fios={fios}"
            );
        }
    }
    assert_eq!(
        super::feather(&[0u8; 64], 8, 8, 3),
        vec![0u8; 64],
        "seleção vazia"
    );
}

/// Sonda: `derive_effective` numa tela 4096² com uma elipse grande e Feather no máximo (r=64), o
/// MÍNIMO de 3 corridas. Tecto: um quadro de arrasto a 30 Hz.
#[test]
#[ignore = "sonda B40.A2: custo do Feather 4096² r=64"]
fn sonda_feather_4096_r64() {
    let n = 4096u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (n * n * 4) as usize], n, n);
    let crisp = t.raster_marquee([512.0, 512.0], [3584.0, 3584.0], true);
    t.paint.selection_crisp = Arc::new(crisp);
    t.paint.selection_feather = 1.0;
    assert_eq!(FEATHER_MAX_PX, 64);
    // Intercalado no MESMO processo: o caminho antigo (oráculo) × o `derive_effective` de hoje.
    let crisp = Arc::clone(&t.paint.selection_crisp);
    let (mut melhor, mut antigo) = (f64::MAX, f64::MAX);
    for volta in 0..4 {
        if volta % 2 == 0 {
            let t0 = Instant::now();
            let m = feather_antigo(&crisp, n as usize, n as usize, FEATHER_MAX_PX);
            antigo = antigo.min(t0.elapsed().as_secs_f64() * 1e3);
            if volta == 0 {
                t.derive_effective();
                assert_eq!(*t.paint.selection_mask, m, "o mesmo byte a 4096²");
            }
        }
        let t0 = Instant::now();
        t.derive_effective();
        melhor = melhor.min(t0.elapsed().as_secs_f64() * 1e3);
    }
    eprintln!(
        "B40.A2 derive_effective 4096² r=64: antigo {antigo:.1} ms → hoje {melhor:.1} ms (mínimos)"
    );
    assert!(
        melhor <= 33.0,
        "Feather 4096² r=64 custa {melhor:.1} ms (tecto 33 ms)"
    );
}
