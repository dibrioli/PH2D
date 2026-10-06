//! O [`box_blur`]: a média EXACTA da caixa, o mesmo bit em qualquer janela (BUGS #36), e os borrões
//! de vários canais iguais ao de um.

use super::*;

/// A lei de ANTES (somas de prefixo em `f32` a partir da origem da janela), em série — o controlo
/// vermelho do gate da janela e o lado A da medida de custo.
fn box_blur_f32_de_antes(src: &[f32], w: usize, h: usize, radius: usize) -> Vec<f32> {
    if radius == 0 || w == 0 || h == 0 {
        return src.to_vec();
    }
    let mut tmp = vec![0.0f32; w * h];
    for (trow, srow) in tmp.chunks_mut(w).zip(src.chunks(w)) {
        let mut pref = vec![0.0f32; w + 1];
        for x in 0..w {
            pref[x + 1] = pref[x] + srow[x];
        }
        for (x, t) in trow.iter_mut().enumerate() {
            let lo = x.saturating_sub(radius);
            let hi = (x + radius).min(w - 1);
            *t = (pref[hi + 1] - pref[lo]) / (hi - lo + 1) as f32;
        }
    }
    // A vertical por LINHAS de prefixos (a mesma soma por coluna, pela mesma ordem, e a memória
    // percorrida como o produto a percorre — a medida de custo compara a aritmética, não a arrumação).
    let mut pref = vec![0.0f32; (h + 1) * w];
    for y in 0..h {
        for x in 0..w {
            pref[(y + 1) * w + x] = pref[y * w + x] + tmp[y * w + x];
        }
    }
    let mut out = vec![0.0f32; w * h];
    for y in 0..h {
        let lo = y.saturating_sub(radius);
        let hi = (y + radius).min(h - 1);
        for x in 0..w {
            out[y * w + x] = (pref[(hi + 1) * w + x] - pref[lo * w + x]) / (hi - lo + 1) as f32;
        }
    }
    out
}

/// O `(x0, y0, w, h)` de dentro de `src` (`sw` de largura), copiado.
fn recorte(src: &[f32], sw: usize, (x0, y0, w, h): (usize, usize, usize, usize)) -> Vec<f32> {
    (y0..y0 + h)
        .flat_map(|y| src[y * sw + x0..y * sw + x0 + w].iter().copied())
        .collect()
}

/// Quantos texels do miolo da janela `(x0, y0, w, h)` (a `r` px das bordas dela) o borrão `f` dá
/// diferentes do borrão da tela inteira.
fn diferem_na_janela(
    f: fn(&[f32], usize, usize, usize) -> Vec<f32>,
    src: &[f32],
    (sw, sh): (usize, usize),
    jan: (usize, usize, usize, usize),
    r: usize,
) -> usize {
    let todo = f(src, sw, sh, r);
    let parte = f(&recorte(src, sw, jan), jan.2, jan.3, r);
    let (x0, y0, w, h) = jan;
    let mut n = 0;
    for y in r..h - r {
        for x in r..w - r {
            if parte[y * w + x].to_bits() != todo[(y0 + y) * sw + x0 + x].to_bits() {
                n += 1;
            }
        }
    }
    n
}

/// Um campo com MAGNITUDES misturadas (a soma em `f32` só é sensível à ordem quando os termos têm
/// escalas diferentes — um campo liso não apanharia uma ordem trocada) e reprodutível.
fn campo(w: usize, h: usize, semente: u32) -> Vec<f32> {
    let mut s = semente.wrapping_mul(2_654_435_761).max(1);
    (0..w * h)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            let u = (s >> 8) as f32 / (1u32 << 24) as f32;
            match s % 4 {
                0 => u * 1e-3,
                1 => u * 255.0,
                2 => 0.0,
                _ => u,
            }
        })
        .collect()
}

/// ⭐ **A CAIXA É A MESMA EM QUALQUER JANELA** (BUGS #36, o pixel do Airbrush): o composite borra a
/// janela do quadro, e a recomposição total a tela inteira — o miolo tem de dar o mesmo bit. Janelas
/// que começam em colunas e linhas diferentes, raios do aro ao rewet. CONTROLO: a lei de antes
/// (`f32` a partir da origem) diverge nas mesmas janelas; e a média é a da soma exacta (`f64`
/// directo) a menos de um arredondamento.
#[test]
fn a_caixa_do_borrao_e_a_mesma_em_qualquer_janela() {
    let (sw, sh) = (160usize, 120usize);
    let src = campo(sw, sh, 29);
    let janelas = [
        (0, 0, 160, 120),
        (7, 3, 90, 70),
        (33, 41, 101, 63),
        (64, 17, 80, 90),
    ];
    let (mut novo, mut antes) = (0usize, 0usize);
    for r in [1usize, 2, 5, 12, 20] {
        for jan in janelas {
            novo += diferem_na_janela(box_blur, &src, (sw, sh), jan, r);
            antes += diferem_na_janela(box_blur_f32_de_antes, &src, (sw, sh), jan, r);
        }
        let b = box_blur(&src, sw, sh, r);
        for (k, v) in b.iter().enumerate() {
            let (x, y) = (k % sw, k / sw);
            let (x0, x1) = (x.saturating_sub(r), (x + r).min(sw - 1));
            let (y0, y1) = (y.saturating_sub(r), (y + r).min(sh - 1));
            // A média separável: a média das médias horizontais de cada linha da caixa.
            let exacta = (y0..=y1)
                .map(|yy| {
                    let s: f64 = (x0..=x1).map(|xx| f64::from(src[yy * sw + xx])).sum();
                    f64::from((s / (x1 - x0 + 1) as f64) as f32)
                })
                .sum::<f64>()
                / (y1 - y0 + 1) as f64;
            let erro = (f64::from(*v) - exacta).abs();
            assert!(
                erro <= exacta.abs() * 1e-6 + 1e-9,
                "r{r} texel ({x}, {y}): {v} contra a soma exacta {exacta}"
            );
        }
    }
    assert_eq!(
        novo, 0,
        "o borrão dá {novo} texels do miolo diferentes conforme a janela"
    );
    assert!(
        antes > 0,
        "controlo: a lei f32 de antes tinha de divergir entre janelas"
    );
}

/// ⭐ O [`box_blur4`] (os quatro campos do rewet numa passagem) dá o byte de QUATRO [`box_blur`]
/// separados — o oráculo é o próprio `box_blur`, que o gate de cima prende à média exacta.
/// Canais diferentes em cada posição (senão um canal trocado com outro passaria), mais o controlo de
/// que o borrão muda o campo.
#[test]
fn quatro_borroes_juntos_dao_o_byte_de_quatro_separados() {
    let casos = [
        (1usize, 1usize, 1usize),
        (37, 1, 3),
        (1, 37, 3),
        (63, 17, 2),
        (65, 40, 7),
        (130, 90, 12),
        (31, 20, 100),
    ];
    let mut mudou = 0usize;
    for (i, &(w, h, r)) in casos.iter().enumerate() {
        let c: [Vec<f32>; 4] = std::array::from_fn(|k| campo(w, h, (i * 4 + k) as u32 + 11));
        let juntos = box_blur4([&c[0], &c[1], &c[2], &c[3]], w, h, r);
        for k in 0..4 {
            let so = box_blur(&c[k], w, h, r);
            assert_eq!(juntos[k].len(), so.len());
            for (t, (a, b)) in juntos[k].iter().zip(&so).enumerate() {
                assert_eq!(
                    a.to_bits(),
                    b.to_bits(),
                    "{w}×{h} r{r}, canal {k}, texel {t}: {a} contra {b}"
                );
            }
        }
        mudou += juntos[0]
            .iter()
            .zip(&c[0])
            .filter(|(a, b)| a.to_bits() != b.to_bits())
            .count();
    }
    // CONTROLO: o borrão de facto mexe no campo (um caso de 1×1 não mexe — por isso a soma).
    assert!(
        mudou > 5_000,
        "o borrão tem de mudar o campo: mudou {mudou}"
    );
}

/// ⭐ O [`box_blur2`] (o campo molhado e a massa dele numa passagem) dá o byte de DOIS [`box_blur`]
/// separados — o mesmo oráculo e os mesmos casos do irmão de quatro canais.
#[test]
fn dois_borroes_juntos_dao_o_byte_de_dois_separados() {
    let casos = [
        (1usize, 1usize, 1usize),
        (37, 1, 3),
        (1, 37, 3),
        (63, 17, 2),
        (65, 40, 7),
        (130, 90, 12),
        (31, 20, 100),
    ];
    let mut mudou = 0usize;
    for (i, &(w, h, r)) in casos.iter().enumerate() {
        let c: [Vec<f32>; 2] = std::array::from_fn(|k| campo(w, h, (i * 2 + k) as u32 + 29));
        let juntos = box_blur2([&c[0], &c[1]], w, h, r);
        for k in 0..2 {
            let so = box_blur(&c[k], w, h, r);
            assert_eq!(juntos[k].len(), so.len());
            for (t, (a, b)) in juntos[k].iter().zip(&so).enumerate() {
                assert_eq!(
                    a.to_bits(),
                    b.to_bits(),
                    "{w}×{h} r{r}, canal {k}, texel {t}: {a} contra {b}"
                );
            }
        }
        mudou += juntos[1]
            .iter()
            .zip(&c[1])
            .filter(|(a, b)| a.to_bits() != b.to_bits())
            .count();
    }
    // CONTROLO: o borrão de facto mexe no campo (um caso de 1×1 não mexe — por isso a soma).
    assert!(
        mudou > 5_000,
        "o borrão tem de mudar o campo: mudou {mudou}"
    );
}

/// SONDA (relógio) — o preço da soma EXACTA contra a lei `f32` de antes, no mesmo processo: as duas
/// num fio só (a aritmética), intercaladas, mínimo e mediana; e o borrão do produto no pool.
/// `cargo test -p ph2d-tool-painter --profile smoke --lib diag_o_preco_do_borrao_exacto -- --ignored --nocapture`
#[test]
#[ignore = "diagnóstico de relógio"]
fn diag_o_preco_do_borrao_exacto() {
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("loadavg {}", carga.trim());
    let um_fio = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .expect("pool");
    for (lado, r) in [(512usize, 4usize), (512, 24), (1024, 8)] {
        let src = campo(lado, lado, 3);
        let mut t: [Vec<f64>; 3] = Default::default();
        for rodada in 0..7 {
            for k in 0..3 {
                let v = (rodada + k) % 3;
                let q = std::time::Instant::now();
                let out = match v {
                    0 => box_blur_f32_de_antes(&src, lado, lado, r),
                    1 => um_fio.install(|| box_blur(&src, lado, lado, r)),
                    _ => box_blur(&src, lado, lado, r),
                };
                t[v].push(q.elapsed().as_secs_f64() * 1e3);
                std::hint::black_box(out);
            }
        }
        let fmt = |v: &mut Vec<f64>| {
            v.sort_by(f64::total_cmp);
            format!("mín {:.3} med {:.3}", v[0], v[v.len() / 2])
        };
        eprintln!(
            "{lado}² r{r}: f32 de antes (1 fio) {} · exacta (1 fio) {} · exacta (pool) {} ms",
            fmt(&mut t[0]),
            fmt(&mut t[1]),
            fmt(&mut t[2])
        );
    }
}
