//! **Prancha B33**: uma aguada plana clara (25 %) assentada no dente do Cold Press, sobre 3 fundos × 1080p/4K,
//! antes (uma leitura de 256 px, [`paper_tile::sample_uma_leitura`]) e depois ([`paper_tile::sample`]). Grava PPM em
//! `PH2D_PRANCHA_B33=<pasta>` e escreve em `autocorrelacao.txt` a autocorrelação da luminância a 256 px em x e y.
//! `PH2D_PRANCHA_B33=/tmp/b33 cargo test -p ph2d-painter-brush prancha_b33 -- --ignored --nocapture`

use super::paper_tile;

const AGUADA: [f32; 3] = [40.0, 90.0, 200.0];
const PAPEL: [f32; 3] = [246.0, 241.0, 230.0];
const BLOCOS: [[f32; 3]; 4] = [
    [200.0, 60.0, 50.0],
    [60.0, 150.0, 210.0],
    [240.0, 200.0, 60.0],
    [90.0, 170.0, 90.0],
];

/// A imagem `w×h` (RGB `f32`) da aguada no fundo `qual` (0 branco · 1 blocos · 2 papel com relevo).
fn cena(w: usize, h: usize, qual: u8, dente: &dyn Fn(f32, f32) -> f32) -> Vec<[f32; 3]> {
    let t = |x: usize, y: usize| dente(x as f32 / 256.0, y as f32 / 256.0);
    (0..w * h)
        .map(|i| {
            let (x, y) = (i % w, i / w);
            let d = t(x, y);
            let fundo = match qual {
                0 => [255.0; 3],
                1 => BLOCOS[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)],
                _ => {
                    let g = t(x.saturating_sub(1), y) - t(x + 1, y) + t(x, y.saturating_sub(1))
                        - t(x, y + 1);
                    PAPEL.map(|c| (c * (1.0 + 1.5 * g)).clamp(0.0, 255.0))
                }
            };
            // O pigmento assenta nos vales: cobertura 25 % no meio do dente.
            let a = (0.25 * (1.5 - d)).clamp(0.0, 1.0);
            [0, 1, 2].map(|c| fundo[c] * (1.0 - a) + AGUADA[c] * a * (fundo[c] / 255.0))
        })
        .collect()
}

fn luma(p: &[f32; 3]) -> f64 {
    f64::from(0.299 * p[0] + 0.587 * p[1] + 0.114 * p[2])
}

fn autocorr(img: &[[f32; 3]], w: usize, h: usize, lag: usize, eixo: u8) -> f64 {
    let (mut sa, mut sb, mut saa, mut sbb, mut sab, mut n) = (0f64, 0f64, 0f64, 0f64, 0f64, 0f64);
    // Só o quadrante de cima à esquerda: os blocos de cor não podem somar correlação própria.
    let (w2, h2) = (w / 2, h / 2);
    for y in 0..h2 - if eixo == 1 { lag } else { 0 } {
        for x in 0..w2 - if eixo == 0 { lag } else { 0 } {
            let a = luma(&img[y * w + x]);
            let b = luma(
                &img[if eixo == 0 {
                    y * w + x + lag
                } else {
                    (y + lag) * w + x
                }],
            );
            (sa, sb, saa, sbb, sab, n) = (
                sa + a,
                sb + b,
                saa + a * a,
                sbb + b * b,
                sab + a * b,
                n + 1.0,
            );
        }
    }
    (sab / n - sa / n * (sb / n))
        / ((saa / n - (sa / n).powi(2)).sqrt() * (sbb / n - (sb / n).powi(2)).sqrt())
}

/// Lado a lado, antes | depois, reduzido `red`× por média de caixa.
fn grava(path: &str, antes: &[[f32; 3]], depois: &[[f32; 3]], w: usize, h: usize, red: usize) {
    let (ow, oh) = (w / red, h / red);
    let mut out = format!("P6\n{} {}\n255\n", 2 * ow, oh).into_bytes();
    for y in 0..oh {
        for img in [antes, depois] {
            for x in 0..ow {
                let mut s = [0f32; 3];
                for dy in 0..red {
                    for dx in 0..red {
                        let p = img[(y * red + dy) * w + x * red + dx];
                        (0..3).for_each(|c| s[c] += p[c]);
                    }
                }
                out.extend(s.map(|v| (v / (red * red) as f32).round().clamp(0.0, 255.0) as u8));
            }
        }
    }
    std::fs::write(path, out).expect("ppm");
}

#[test]
#[ignore = "prancha: PH2D_PRANCHA_B33=<pasta>"]
fn prancha_b33() {
    let Ok(dir) = std::env::var("PH2D_PRANCHA_B33") else {
        return;
    };
    let mut relato = String::new();
    for (res, w, h, red) in [("1080p", 1920, 1080, 1), ("4k", 3840, 2160, 2)] {
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            let antes = cena(w, h, qual, &|u, v| paper_tile::sample_uma_leitura(0, u, v));
            let depois = cena(w, h, qual, &|u, v| paper_tile::sample(0, u, v));
            for eixo in [0u8, 1] {
                relato += &format!(
                    "{res} {nome} eixo {eixo}: autocorrelação a 256 px {:.3} → {:.3}",
                    autocorr(&antes, w, h, 256, eixo),
                    autocorr(&depois, w, h, 256, eixo)
                );
                relato.push('\n');
            }
            grava(
                &format!("{dir}/{res}_{nome}_antes_depois.ppm"),
                &antes,
                &depois,
                w,
                h,
                red,
            );
        }
    }
    std::fs::write(format!("{dir}/autocorrelacao.txt"), relato).expect("relato");
}
