//! **RENDER-AND-LOOK da pilha de FX raster** — a sonda, não um gate.
//!
//! Irmã do `push_look_probe` do Painter: um gate afirma um NÚMERO, e há defeitos cujo oráculo é a
//! FOTO (o pente do Bevel, a linha tracejada do Feather, a serrilha do contorno). Ela desenha uma
//! ESTRELA — arestas oblíquas, pontas agudas e reentrâncias, que é onde os três artefatos vivem —
//! e escreve um PPM por cena.
//!
//! ```text
//! cd <worktree> && PH2D_FX_LOOK_DIR=/tmp/fx cargo test -p ph2d-render \
//!     --test fx_look_probe -- --ignored --nocapture
//! ```
//!
//! Converter para ver: `magick /tmp/fx/<cena>.ppm /tmp/fx/<cena>.png`.

use ph2d_ecs::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};

mod fx_stack_common;
use fx_stack_common::{make_src, readback, try_headless_gpu};

const W: u32 = 512;
const H: u32 = 512;
/// Supersampling da estrela — a rampa de AA tem de parecer com a que o Vello produz, senão a sonda
/// mede a própria rasterização em vez do efeito.
const SS: u32 = 4;

/// A cor da estrela do smoke (`fx_raster_smoke.rs`), para a foto ser comparável à do Enio.
const AMBER: [f32; 3] = [235.0, 175.0, 60.0];

/// Cobertura analítica de uma estrela de 5 pontas (raio interno 0.45), por supersampling.
fn star_alpha(w: u32, h: u32) -> Vec<f32> {
    let cx = f64::from(w) * 0.5;
    let cy = f64::from(h) * 0.5;
    let r_out = f64::from(w.min(h)) * 0.40;
    let r_in = r_out * 0.45;
    // Os 10 vértices, alternando externo/interno, começando pela ponta de cima.
    let mut poly = Vec::with_capacity(10);
    for i in 0..10 {
        let a = -std::f64::consts::FRAC_PI_2 + f64::from(i) * std::f64::consts::PI / 5.0;
        let r = if i % 2 == 0 { r_out } else { r_in };
        poly.push((cx + r * a.cos(), cy + r * a.sin()));
    }
    let inside = |px: f64, py: f64| -> bool {
        let mut hit = false;
        let n = poly.len();
        for i in 0..n {
            let (x0, y0) = poly[i];
            let (x1, y1) = poly[(i + 1) % n];
            if (y0 > py) != (y1 > py) {
                let t = (py - y0) / (y1 - y0);
                if px < x0 + t * (x1 - x0) {
                    hit = !hit;
                }
            }
        }
        hit
    };
    let step = 1.0 / f64::from(SS);
    let mut out = vec![0.0f32; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let mut acc = 0u32;
            for sy in 0..SS {
                for sx in 0..SS {
                    let px = f64::from(x) + (f64::from(sx) + 0.5) * step;
                    let py = f64::from(y) + (f64::from(sy) + 0.5) * step;
                    if inside(px, py) {
                        acc += 1;
                    }
                }
            }
            out[(y * w + x) as usize] = acc as f32 / f64::from(SS * SS) as f32;
        }
    }
    out
}

/// A estrela em RGBA PREMULTIPLICADO — a premissa de toda a pilha.
fn star_src(gpu: &ph2d_gpu::GpuContext) -> (wgpu::Texture, Vec<f32>) {
    let a = star_alpha(W, H);
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for (i, &cov) in a.iter().enumerate() {
        let o = i * 4;
        for c in 0..3 {
            bytes[o + c] = (AMBER[c] * cov).round().clamp(0.0, 255.0) as u8;
        }
        bytes[o + 3] = (cov * 255.0).round().clamp(0.0, 255.0) as u8;
    }
    (make_src(gpu, W, H, &bytes), a)
}

/// PPM P6 sobre um fundo cinza-escuro (o do app), para a foto ser lida como o artista a vê.
///
/// ⚠️ A saída do passe é RGBA **RETO** (o `cs_resolve` divide pelo alfa), então o `over` é
/// `a·rgb + (1−a)·bg`. Compor como premultiplicado clareia toda borda parcial — e uma sonda que
/// mente na borda é uma sonda inútil justamente onde estes efeitos vivem.
fn write_ppm(dir: &str, name: &str, px: &[u8]) {
    let bg = [0x2c_u8, 0x2e, 0x33];
    let mut body = Vec::with_capacity((W * H * 3) as usize);
    for i in 0..(W * H) as usize {
        let o = i * 4;
        let a = f32::from(px[o + 3]) / 255.0;
        for c in 0..3 {
            let v = a * f32::from(px[o + c]) + (1.0 - a) * f32::from(bg[c]);
            body.push(v.round().clamp(0.0, 255.0) as u8);
        }
    }
    let path = format!("{dir}/{name}.ppm");
    let mut f = std::fs::File::create(&path).expect("criar ppm");
    use std::io::Write;
    write!(f, "P6\n{W} {H}\n255\n").expect("cabecalho");
    f.write_all(&body).expect("corpo");
    eprintln!("[fx-look] {path}");
}

fn op(kind: u8, sigma_px: f32, tint: [f32; 4], offset_px: [i32; 2]) -> FxOpGpu {
    FxOpGpu {
        kind,
        sigma_px,
        offset_px,
        tint,
        opacity: 1.0,
        mode: if FxOp::spec(kind).modes.is_empty() {
            0
        } else {
            FxOp::new(kind).mode
        },
    }
}

#[test]
#[ignore = "sonda de olho; roda com --ignored e PH2D_FX_LOOK_DIR"]
fn probe_fx_render_and_look() {
    let Some(dir) = std::env::var("PH2D_FX_LOOK_DIR").ok() else {
        eprintln!("[fx-look] defina PH2D_FX_LOOK_DIR=<dir>");
        return;
    };
    std::fs::create_dir_all(&dir).expect("dir");
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx-look] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    let (src, _cov) = star_src(&gpu);

    let white = [1.0, 1.0, 1.0, 1.0];
    let black = [0.0, 0.0, 0.0, 1.0];
    let scenes: [(&str, Vec<FxOpGpu>); 5] = [
        ("00_plain", vec![]),
        ("01_feather", vec![op(FxOp::FEATHER, 24.0, white, [0, 0])]),
        ("02_bevel", vec![op(FxOp::BEVEL, 20.0, black, [-12, 12])]),
        ("03_outline", vec![op(FxOp::OUTLINE, 8.0, white, [0, 0])]),
        (
            "04_inner_shadow",
            vec![op(FxOp::INNER_SHADOW, 20.0, black, [0, 0])],
        ),
    ];
    for (name, ops) in scenes {
        let dst = make_output_texture(&gpu, W, H);
        pass.run(&gpu, &src, &dst, W, H, &ops);
        let px = readback(&gpu, &dst, W, H);
        write_ppm(&dir, name, &px);
    }
}
