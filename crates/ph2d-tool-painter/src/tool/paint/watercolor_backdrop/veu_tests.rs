use super::{VeuDeHumidade, build_veil};

/// O `build_veil` de antes do B16.10, VERBATIM (`ph2d-app-painter/src/painter_bridge_wetness.rs`,
/// a8dbe3320f): o vertical do desfoque percorria por COLUNAS. Oráculo da paridade ao bit.
mod oraculo {
    pub fn build_veil(
        wet: &[u8],
        canvas_w: usize,
        (rx0, ry0): (usize, usize),
        (rw, rh): (usize, usize),
        max_alpha: f32,
        step: usize,
    ) -> (Vec<u8>, usize, usize) {
        const TINT: [u8; 3] = [34, 31, 28];
        const BLUR_R: usize = 4;
        let step = step.max(1);
        let (vw, vh) = (rw.div_ceil(step), rh.div_ceil(step));
        if vw == 0 || vh == 0 {
            return (Vec::new(), 0, 0);
        }
        let mut alpha = vec![0.0f32; vw * vh];
        for vy in 0..vh {
            for vx in 0..vw {
                let (y0, x0) = (vy * step, vx * step);
                let (y1, x1) = ((y0 + step).min(rh), (x0 + step).min(rw));
                let mut acc = 0.0f32;
                for y in y0..y1 {
                    let src = (ry0 + y) * canvas_w + rx0;
                    for x in x0..x1 {
                        acc += f32::from(wet[src + x]);
                    }
                }
                let n = ((y1 - y0) * (x1 - x0)).max(1) as f32;
                alpha[vy * vw + vx] = acc / n / 255.0 * max_alpha;
            }
        }
        let alpha = box_blur_f32(&alpha, vw, vh, (BLUR_R / step).max(1));
        let mut veil = vec![0u8; vw * vh * 4];
        for (i, &a) in alpha.iter().enumerate() {
            if a > 0.002 {
                let p = i * 4;
                veil[p] = TINT[0];
                veil[p + 1] = TINT[1];
                veil[p + 2] = TINT[2];
                veil[p + 3] = (a * 255.0).clamp(0.0, 255.0) as u8;
            }
        }
        (veil, vw, vh)
    }

    fn box_blur_f32(src: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
        if r == 0 || w == 0 || h == 0 {
            return src.to_vec();
        }
        let win = (2 * r + 1) as f32;
        let mut tmp = vec![0.0f32; w * h];
        for y in 0..h {
            let base = y * w;
            let mut acc = 0.0f32;
            for x in 0..(r + 1).min(w) {
                acc += src[base + x];
            }
            for x in 0..w {
                tmp[base + x] = acc / win;
                if x + r + 1 < w {
                    acc += src[base + x + r + 1];
                }
                if x >= r {
                    acc -= src[base + x - r];
                }
            }
        }
        let mut out = vec![0.0f32; w * h];
        for x in 0..w {
            let mut acc = 0.0f32;
            for y in 0..(r + 1).min(h) {
                acc += tmp[y * w + x];
            }
            for y in 0..h {
                out[y * w + x] = acc / win;
                if y + r + 1 < h {
                    acc += tmp[(y + r + 1) * w + x];
                }
                if y >= r {
                    acc -= tmp[(y - r) * w + x];
                }
            }
        }
        out
    }
}

/// Mapa PLAUSÍVEL (miolo molhado, bordas secas, ruído determinístico): um mapa chapado deixaria o
/// `if a > 0.002` decidir tudo por um ramo só.
fn mapa(cw: usize, ch: usize) -> Vec<u8> {
    let mut wet = vec![0u8; cw * ch];
    let mut s = 0x9e37_79b9_u32;
    for y in 0..ch {
        for x in 0..cw {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            let d = ((x as f32 / cw as f32) - 0.5).abs() + ((y as f32 / ch as f32) - 0.5).abs();
            let base = (1.0 - 1.6 * d).clamp(0.0, 1.0) * 230.0;
            wet[y * cw + x] = (base as u32 + (s & 15)).min(255) as u8 * u8::from(base > 0.0);
        }
    }
    wet
}

/// **O véu no tool é o véu da ponte de antes, ao bit**, em toda a combinação de recorte, passo e
/// força. Mutação que sangra: trocar a ordem de `+=`/`-=` no desfoque.
#[test]
fn o_veu_no_tool_da_o_bit_do_veu_da_ponte() {
    let (cw, ch) = (301usize, 207usize);
    let wet = mapa(cw, ch);
    for (rx0, ry0, rw, rh) in [
        (0usize, 0usize, 301usize, 207usize),
        (13, 7, 250, 180),
        (100, 50, 3, 2),
        (0, 0, 1, 1),
    ] {
        for step in [1usize, 2, 3, 4, 7, 8] {
            for max_alpha in [0.55f32, 0.165, 0.0] {
                let novo = build_veil(&wet, cw, (rx0, ry0), (rw, rh), max_alpha, step);
                let velho = oraculo::build_veil(&wet, cw, (rx0, ry0), (rw, rh), max_alpha, step);
                assert!(
                    novo == velho,
                    "o véu por linhas divergiu do de antes: rect ({rx0},{ry0}) {rw}×{rh}, \
                     passo {step}, força {max_alpha}"
                );
            }
        }
    }
}

/// **Um quadro parado não reconstrói o véu** (B16.10): a mesma humidade, recorte, passo e força ⇒
/// o MESMO `StableImage` (id estável no Vello). Qualquer um deles muda ⇒ reconstrói. RED com a chave
/// ignorada (reconstrução por quadro, o caminho de antes).
#[test]
fn o_veu_parado_nao_se_reconstroi() {
    let (cw, ch) = (96usize, 64usize);
    let wet = mapa(cw, ch);
    let rect = [4usize, 2, 90, 60];
    let mut v = VeuDeHumidade::default();
    let a = v
        .veu(&wet, cw, 7, rect, 1, 0.3)
        .map(|(i, w, h)| (i.clone(), w, h));
    for _ in 0..5 {
        assert!(v.veu(&wet, cw, 7, rect, 1, 0.3).is_some());
    }
    assert_eq!(
        v.construcoes(),
        1,
        "seis quadros parados construíram o véu {} vezes",
        v.construcoes()
    );
    let (_, w, h) = a.expect("a região molhada dá véu");
    assert_eq!((w, h), (86, 58));
    for (passo, (rev, r, step, alpha)) in [
        (8u64, rect, 1usize, 0.3f32),
        (8, [5, 2, 90, 60], 1, 0.3),
        (8, [5, 2, 90, 60], 2, 0.3),
        (8, [5, 2, 90, 60], 2, 0.31),
    ]
    .into_iter()
    .enumerate()
    {
        assert!(v.veu(&wet, cw, rev, r, step, alpha).is_some());
        assert_eq!(
            v.construcoes(),
            2 + passo as u64,
            "a chave mudou ({rev}, {r:?}, {step}, {alpha}) e o véu não foi refeito"
        );
    }
}

/// **Quanto custa o VÉU a 3840×2160, e quanto custa o quadro PARADO** (B16.10). Variantes no mesmo
/// processo, intercaladas, o MÍNIMO de 5: o laço de antes (por colunas), o de agora (por linhas) e o
/// quadro parado (a chave acerta).
#[test]
#[ignore = "measurement, not a gate"]
fn measure_the_wetness_veil() {
    use std::time::Instant;
    let (cw, ch) = (3840usize, 2160usize);
    let wet = mapa(cw, ch);
    println!("\no VÉU de umidade 3840x2160 — build por quadro (mínimo de 5, intercalado)\n");
    println!(
        "{:<6} {:>13} {:>13} {:>13}",
        "passo", "antes ms", "agora ms", "parado ms"
    );
    for step in [1usize, 2, 4] {
        let (mut antes, mut agora, mut parado) = (f64::MAX, f64::MAX, f64::MAX);
        let mut v = VeuDeHumidade::default();
        for i in 0..5u64 {
            let t = Instant::now();
            let a = oraculo::build_veil(&wet, cw, (0, 0), (cw, ch), 0.165, step);
            antes = antes.min(t.elapsed().as_secs_f64() * 1e3);
            let t = Instant::now();
            let b = build_veil(&wet, cw, (0, 0), (cw, ch), 0.165, step);
            agora = agora.min(t.elapsed().as_secs_f64() * 1e3);
            assert!(a == b && a.0.iter().any(|&x| x > 0));
            let t = Instant::now();
            assert!(v.veu(&wet, cw, 1, [0, 0, cw, ch], step, 0.165).is_some());
            if i > 0 {
                parado = parado.min(t.elapsed().as_secs_f64() * 1e3);
            }
        }
        println!("{step:<6} {antes:>13.3} {agora:>13.3} {parado:>13.4}");
    }
    println!();
}

/// **O véu grosso cobre a MESMA região e continua molhando** — um passo que perdesse a última
/// linha/coluna deixaria uma faixa seca na borda do desenho (vindo de `painter_bridge_wetness.rs`).
#[test]
fn a_coarser_veil_still_covers_the_whole_region() {
    let (cw, rw, rh) = (64usize, 50usize, 30usize);
    let wet = vec![200u8; cw * cw];
    let (fine, fw, fh) = build_veil(&wet, cw, (0, 0), (rw, rh), 0.3, 1);
    assert_eq!((fw, fh), (rw, rh));
    for step in [2usize, 3, 4, 7] {
        let (coarse, vw, vh) = build_veil(&wet, cw, (0, 0), (rw, rh), 0.3, step);
        assert_eq!((vw, vh), (rw.div_ceil(step), rh.div_ceil(step)));
        assert!(
            vw * step >= rw && vh * step >= rh,
            "o véu grosso não cobre o rect"
        );
        assert!(
            coarse.iter().any(|&b| b > 0),
            "o véu grosso saiu SECO no passo {step}"
        );
        assert!(
            coarse.len() < fine.len(),
            "o passo {step} não economizou nada"
        );
    }
}
