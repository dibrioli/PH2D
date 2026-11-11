//! **Prancha B16c** (B16.A5 + B20.12): o véu de humidade, antes e depois, em 3 fundos × 1080p/4K.
//! Escreve `docs/Painter/missao_4k/pranchas/B16c/`. O «antes» é o caminho antigo da ponte: recorte
//! com origem em `lo - 8` (a fase segue a câmera) e o filtro `Low` (vizinho mais próximo) a 400 %.
//! O filtro `Medium` do Vello é emulado aqui por bilinear na CPU.

use super::build_veil;
use crate::tool::PainterTool;

const TINT: [f32; 3] = [34.0, 31.0, 28.0];

fn fundo(w: u32, h: u32, qual: u8) -> Vec<u8> {
    if qual == 2 {
        let mut t = PainterTool::default();
        t.agenda_papel_texturizado(7);
        t.bind_document(7, vec![255u8; (w * h * 4) as usize], w, h);
        t.invalidate_composite();
        return t.take_preview_arc().expect("o preview").0.to_vec();
    }
    let cores = [
        [200u8, 60, 50],
        [60, 150, 210],
        [240, 200, 60],
        [90, 170, 90],
    ];
    let mut v = vec![255u8; (w * h * 4) as usize];
    if qual == 1 {
        for (i, p) in v.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let (x, y) = (i as u32 % w, i as u32 / w);
            let c = cores[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)];
            p[..3].copy_from_slice(&c);
        }
    }
    v
}

/// Uma mancha molhada com franja ruidosa de grão fino (o que «borbulha» quando a fase anda).
fn humidade(w: usize, h: usize) -> Vec<u8> {
    let mut wet = vec![0u8; w * h];
    let mut s = 0x9e37_79b9_u32;
    let (cx, cy, r) = (w as f32 / 2.0, h as f32 / 2.0, h as f32 * 0.3);
    for y in 0..h {
        for x in 0..w {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt() / r;
            let base = (1.4 - d).clamp(0.0, 1.0) * 200.0;
            if base > 0.0 {
                wet[y * w + x] = (base + (s & 63) as f32).min(255.0) as u8;
            }
        }
    }
    wet
}

fn mistura(bg: &[u8], i: usize, a: f32) -> [u8; 4] {
    let c = |k: usize| (f32::from(bg[i + k]) * (1.0 - a) + TINT[k] * a).round() as u8;
    [c(0), c(1), c(2), 255]
}

/// A vista a 25 % (passo 4) do quarto central da tela, com a câmera `pan` px de imagem à direita; o
/// recorte como a ponte o fazia (`alinhado = false`) ou na grelha absoluta (B16.A5). Cada px de tela
/// é amostrado no MESMO px de imagem em todos os `pan`, para se ver só o que o véu muda.
fn fase(
    w: u32,
    h: u32,
    wet: &[u8],
    bg: &[u8],
    pan: u32,
    alinhado: bool,
) -> (Vec<f32>, Vec<[u8; 4]>, u32, u32) {
    let step = 4u32;
    let (ox, oy) = (w / 4 + pan, h / 4);
    let (vw, vh) = (w / 2 / step, h / 2 / step);
    let (mut x0, mut y0) = (ox - 8, oy - 8);
    let (mut x1, mut y1) = (ox + vw * step + 8, oy + vh * step + 8);
    if alinhado {
        (x0, y0) = (x0 / step * step, y0 / step * step);
        (x1, y1) = (x1.div_ceil(step) * step, y1.div_ceil(step) * step);
    }
    let (veil, tw, _) = build_veil(
        wet,
        w as usize,
        (x0 as usize, y0 as usize),
        ((x1 - x0) as usize, (y1 - y0) as usize),
        0.55,
        step as usize,
    );
    let (mut alfa, mut px) = (Vec::new(), Vec::new());
    for sy in 0..vh {
        for sx in 0..vw {
            let (ix, iy) = (w / 4 + sx * step, oy + sy * step);
            let t = ((iy - y0) / step) as usize * tw + ((ix - x0) / step) as usize;
            let a = f32::from(veil[t * 4 + 3]) / 255.0;
            alfa.push(a);
            px.push(mistura(bg, ((iy * w + ix) * 4) as usize, a));
        }
    }
    (alfa, px, vw, vh)
}

/// A 400 % (passo 1): um recorte de 1/16 da tela na franja, cada texel do véu a 4 px de tela,
/// vizinho mais próximo (`Low`, antes) ou bilinear (`Medium`, depois).
fn zoom(w: u32, h: u32, wet: &[u8], bg: &[u8], bilinear: bool) -> (Vec<[u8; 4]>, u32, u32) {
    let (rw, rh) = (w / 8, h / 8);
    let (ox, oy) = (w / 2 + (h as f32 * 0.42) as u32 - rw / 2, h / 2 - rh / 2);
    let (veil, tw, th) = build_veil(
        wet,
        w as usize,
        (ox as usize, oy as usize),
        (rw as usize, rh as usize),
        0.55,
        1,
    );
    let a_em =
        |x: usize, y: usize| f32::from(veil[(y.min(th - 1) * tw + x.min(tw - 1)) * 4 + 3]) / 255.0;
    let mut px = Vec::new();
    for sy in 0..rh * 4 {
        for sx in 0..rw * 4 {
            let (fx, fy) = ((sx as f32 + 0.5) / 4.0, (sy as f32 + 0.5) / 4.0);
            let a = if bilinear {
                let (gx, gy) = ((fx - 0.5).max(0.0), (fy - 0.5).max(0.0));
                let (x, y) = (gx as usize, gy as usize);
                let (u, v) = (gx.fract(), gy.fract());
                let top = a_em(x, y) * (1.0 - u) + a_em(x + 1, y) * u;
                let bot = a_em(x, y + 1) * (1.0 - u) + a_em(x + 1, y + 1) * u;
                top * (1.0 - v) + bot * v
            } else {
                a_em(fx as usize, fy as usize)
            };
            let (ix, iy) = (ox + fx as u32, oy + fy as u32);
            px.push(mistura(bg, ((iy * w + ix) * 4) as usize, a));
        }
    }
    (px, rw * 4, rh * 4)
}

#[test]
#[ignore = "prancha B16c: escreve PNGs em docs/Painter/missao_4k/pranchas/B16c/"]
fn prancha_b16c_o_veu_de_humidade() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B16c"
    );
    std::fs::create_dir_all(dir).expect("pasta");
    let cinza = image::Rgba([40u8, 40, 40, 255]);
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        let wet = humidade(w as usize, h as usize);
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            let bg = fundo(w, h, qual);
            // B16.A5: linhas antes/depois; colunas pan 0, 1, 2 px e a diferença ×8 entre eles.
            let (gap, mag) = (6u32, 2u32);
            let mut folha: Option<image::RgbaImage> = None;
            for (linha, alinhado) in [(0u32, false), (1, true)] {
                let vistas: Vec<_> = (0..3).map(|p| fase(w, h, &wet, &bg, p, alinhado)).collect();
                let (vw, vh) = (vistas[0].2, vistas[0].3);
                let f = folha.get_or_insert_with(|| {
                    image::RgbaImage::from_pixel((vw * mag + gap) * 4, (vh * mag + gap) * 2, cinza)
                });
                let mut dif = 0usize;
                for i in 0..(vw * vh) as usize {
                    let d = vistas
                        .iter()
                        .map(|v| (v.0[i] - vistas[0].0[i]).abs())
                        .fold(0.0f32, f32::max);
                    dif += usize::from(d > 0.0);
                    let (sx, sy) = (i as u32 % vw, i as u32 / vw);
                    for (col, v) in vistas.iter().enumerate() {
                        for (dx, dy) in (0..mag).flat_map(|a| (0..mag).map(move |b| (a, b))) {
                            let p = image::Rgba(v.1[i]);
                            f.put_pixel(
                                col as u32 * (vw * mag + gap) + sx * mag + dx,
                                linha * (vh * mag + gap) + sy * mag + dy,
                                p,
                            );
                        }
                    }
                    let g = (255.0 - (d * 8.0 * 255.0).min(255.0)) as u8;
                    for (dx, dy) in (0..mag).flat_map(|a| (0..mag).map(move |b| (a, b))) {
                        f.put_pixel(
                            3 * (vw * mag + gap) + sx * mag + dx,
                            linha * (vh * mag + gap) + sy * mag + dy,
                            image::Rgba([255, g, g, 255]),
                        );
                    }
                }
                eprintln!(
                    "PRANCHA {res} {nome} fase {}: px de tela que mudam com o pan {dif}",
                    ["antes", "depois"][linha as usize]
                );
            }
            folha
                .expect("folha")
                .save(format!("{dir}/{res}_{nome}_pan_antes_depois.png"))
                .expect("png");
            // B20.12: esquerda antes (Low), direita depois (Medium).
            let (a, zw, zh) = zoom(w, h, &wet, &bg, false);
            let (d, _, _) = zoom(w, h, &wet, &bg, true);
            let mut par = image::RgbaImage::from_pixel(zw * 2 + gap, zh, cinza);
            for (i, (pa, pd)) in a.iter().zip(&d).enumerate() {
                let (x, y) = (i as u32 % zw, i as u32 / zw);
                par.put_pixel(x, y, image::Rgba(*pa));
                par.put_pixel(x + zw + gap, y, image::Rgba(*pd));
            }
            par.save(format!("{dir}/{res}_{nome}_zoom400_antes_depois.png"))
                .expect("png");
        }
    }
}
