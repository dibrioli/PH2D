//! SONDA (auditoria do `fx.filter`, 10/10): os defeitos do dono medidos na placa — a sombra
//! interna de um disco CERCADO na camada contra o mesmo disco SOZINHO, e o bevel de um disco
//! semeado pelo raster (sem geometria, como o Motion o corre) contra o pé exato, sob movimento
//! sub-pixel. Só medição: imprime, não reprova.
//!
//! `PH2D_GPU=1 cargo test -p ph2d-render --test it fx_stack_camada_sonda -- --ignored --nocapture`

use ph2d_fx_op::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};

use crate::fx_stack_common::{make_src, op, readback, try_headless_gpu};

const R: f64 = 7.0;

/// Cobertura de um disco de raio `R` em `(cx, cy)` no texel `(x, y)`, por 8×8 sub-amostras.
fn cobertura(x: u32, y: u32, cx: f64, cy: f64) -> f64 {
    let mut n = 0;
    for sy in 0..8 {
        for sx in 0..8 {
            let px = f64::from(x) + (f64::from(sx) + 0.5) / 8.0;
            let py = f64::from(y) + (f64::from(sy) + 0.5) / 8.0;
            if (px - cx).powi(2) + (py - cy).powi(2) <= R * R {
                n += 1;
            }
        }
    }
    f64::from(n) / 64.0
}

/// Uma camada branca de discos (alfa reto).
fn camada(w: u32, h: u32, centros: &[(f64, f64)]) -> Vec<u8> {
    let mut b = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let a = centros
                .iter()
                .map(|&(cx, cy)| cobertura(x, y, cx, cy))
                .fold(0.0f64, f64::max);
            let o = ((y * w + x) * 4) as usize;
            b[o..o + 4].copy_from_slice(&[255, 255, 255, (a * 255.0).round() as u8]);
        }
    }
    b
}

fn corre(w: u32, h: u32, bytes: &[u8], ops: &[FxOpGpu], geom: &[[f32; 4]]) -> Vec<u8> {
    let gpu = try_headless_gpu().unwrap();
    let src = make_src(&gpu, w, h, bytes);
    let dst = make_output_texture(&gpu, w, h);
    let mut pass = FxStackPass::new(&gpu);
    pass.run(&gpu, &src, &dst, w, h, ops, geom);
    readback(&gpu, &dst, w, h)
}

/// Brilho médio (0..255, canal R) dos texels de alfa cheio dentro do disco `(cx, cy)`.
fn miolo(out: &[u8], w: u32, cx: f64, cy: f64) -> f64 {
    let (mut s, mut n) = (0.0, 0.0);
    for y in (cy - R) as u32..=(cy + R) as u32 {
        for x in (cx - R) as u32..=(cx + R) as u32 {
            if cobertura(x, y, cx, cy) >= 1.0 {
                s += f64::from(out[((y * w + x) * 4) as usize]);
                n += 1.0;
            }
        }
    }
    s / n
}

#[test]
#[ignore = "sonda de GPU"]
fn sonda_sombra_interna_cercado_contra_sozinho() {
    if try_headless_gpu().is_none() {
        return;
    }
    let (w, h) = (160u32, 160u32);
    for (passo, nome) in [(16.0, "vão 2 px"), (14.0, "a tocar"), (20.0, "vão 6 px")] {
        let c = 80.0;
        let mut grelha = Vec::new();
        for j in -4..=4 {
            for i in -4..=4 {
                grelha.push((c + f64::from(i) * passo, c + f64::from(j) * passo));
            }
        }
        for modo in [FxOp::MODE_PROXIMITY, FxOp::MODE_CONTOUR] {
            let mut s = op(FxOp::INNER_SHADOW, 5.3, [0.0, 0.0, 0.0, 1.0]);
            s.offset_px = [6, 6];
            s.mode = modo;
            let cercado = corre(w, h, &camada(w, h, &grelha), &[s], &[]);
            let sozinho = corre(w, h, &camada(w, h, &[(c, c)]), &[s], &[]);
            eprintln!(
                "[sonda inner] {nome} modo {modo}: miolo do disco central — cercado {:.1} · sozinho {:.1} (255 = sem sombra)",
                miolo(&cercado, w, c, c),
                miolo(&sozinho, w, c, c)
            );
        }
    }
}

/// O polígono do círculo (64 lados) no espaço do texel.
fn circulo(cx: f64, cy: f64) -> Vec<[f32; 4]> {
    (0..64)
        .map(|i| {
            let a0 = f64::from(i) / 64.0 * std::f64::consts::TAU;
            let a1 = f64::from(i + 1) / 64.0 * std::f64::consts::TAU;
            [
                (cx + R * a0.cos()) as f32,
                (cy + R * a0.sin()) as f32,
                (cx + R * a1.cos()) as f32,
                (cy + R * a1.sin()) as f32,
            ]
        })
        .collect()
}

/// Amostra bilinear do canal R.
fn bil(out: &[u8], w: u32, x: f64, y: f64) -> f64 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let p = |xx: f64, yy: f64| f64::from(out[(((yy as u32) * w + xx as u32) * 4) as usize]);
    p(x0, y0) * (1.0 - fx) * (1.0 - fy)
        + p(x0 + 1.0, y0) * fx * (1.0 - fy)
        + p(x0, y0 + 1.0) * (1.0 - fx) * fy
        + p(x0 + 1.0, y0 + 1.0) * fx * fy
}

/// O anel do relevo a `r` px do centro, 64 ângulos, menos a 1.ª harmónica (a luz): o que sobra é
/// RUÍDO (um disco exacto dá um relevo que só depende do ângulo da luz).
fn ondulacao(out: &[u8], w: u32, cx: f64, cy: f64, r: f64) -> (f64, Vec<f64>) {
    let v: Vec<f64> = (0..64)
        .map(|i| {
            let a = f64::from(i) / 64.0 * std::f64::consts::TAU;
            bil(out, w, cx + r * a.cos() - 0.5, cy + r * a.sin() - 0.5)
        })
        .collect();
    let n = 64.0;
    let a0 = v.iter().sum::<f64>() / n;
    let (mut a1, mut b1) = (0.0, 0.0);
    for (i, x) in v.iter().enumerate() {
        let a = i as f64 / n * std::f64::consts::TAU;
        a1 += x * a.cos() * 2.0 / n;
        b1 += x * a.sin() * 2.0 / n;
    }
    let res: Vec<f64> = v
        .iter()
        .enumerate()
        .map(|(i, x)| {
            let a = i as f64 / n * std::f64::consts::TAU;
            x - (a0 + a1 * a.cos() + b1 * a.sin())
        })
        .collect();
    let max = res.iter().fold(0.0f64, |m, r| m.max(r.abs()));
    (max, v)
}

#[test]
#[ignore = "sonda de GPU"]
fn sonda_bevel_raster_contra_exacto_sob_movimento() {
    if try_headless_gpu().is_none() {
        return;
    }
    let (w, h) = (48u32, 48u32);
    let mut b = op(FxOp::BEVEL, 5.3, [0.0, 0.0, 0.0, 1.0]);
    b.offset_px = [-6, 6];
    b.opacity = 0.9;
    let mut anteriores: Vec<(String, Vec<f64>)> = Vec::new();
    for dx in [0.0, 0.25, 0.5, 0.75] {
        let (cx, cy) = (24.0 + dx, 24.0 + dx * 0.5);
        let src = camada(w, h, &[(cx, cy)]);
        for (rota, geom) in [("raster", Vec::new()), ("exacto", circulo(cx, cy))] {
            let out = corre(w, h, &src, &[b], &geom);
            for r in [2.5, 4.5, 6.0] {
                let (onda, anel) = ondulacao(&out, w, cx, cy, r);
                eprintln!(
                    "[sonda bevel] {rota} dx {dx:.2}: anel r={r} — ondulação máx {onda:.1} níveis"
                );
                let chave = format!("{rota} r={r}");
                if let Some((_, a)) = anteriores.iter().find(|(k, _)| *k == chave) {
                    let d = a.iter().zip(&anel).fold(0.0f64, |m, (x, y)| m.max((x - y).abs()));
                    eprintln!("[sonda bevel]   {chave}: muda {d:.1} níveis contra dx 0 (o mesmo ponto do disco)");
                } else {
                    anteriores.push((chave, anel));
                }
            }
        }
    }
}
