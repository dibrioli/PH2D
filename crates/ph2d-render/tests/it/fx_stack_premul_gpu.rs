//! **A PILHA SOBRE A CAMADA DO MOTION** (ciclo 20 W3, doc 129 §3.3) — a fonte pré-multiplicada no
//! espaço do desenhista (`FxStackPass::run_premultiplied`).
//!
//! Dois gates: a mesma imagem dada pré-multiplicada sai IGUAL à dada reta (a des-pré-multiplicação
//! do ingest é a única diferença), e o borrão sobre a cena do oráculo bate com o ImageMagick
//! (`ph2d-lens/tests/fixtures/blur_linear_s3.txt`, σ = 3 px em luz linear).
//!
//! Rodar: `PH2D_GPU=1 cargo test -p ph2d-render --test it fx_stack_premul_gpu -- --ignored`.

use ph2d_fx_op::FxOp;
use ph2d_render::{FxStackPass, make_output_texture};

use crate::fx_stack_common::{make_src, op, readback, try_headless_gpu};

/// `f32` → meio-float (arredondado ao mais próximo), para escrever a camada `Rgba16Float`.
fn f16(v: f32) -> u16 {
    let b = v.to_bits();
    let sinal = ((b >> 16) & 0x8000) as u16;
    let exp = ((b >> 23) & 0xff) as i32 - 127 + 15;
    let man = b & 0x007f_ffff;
    if exp <= 0 {
        if exp < -10 {
            return sinal;
        }
        let m = (man | 0x0080_0000) >> (14 - exp);
        return sinal | m as u16;
    }
    let h = (exp as u32) << 10 | man >> 13;
    sinal | (h + ((man >> 12) & 1)) as u16
}

/// A CAMADA como o Motion a monta: `Rgba16Float`, sRGB codificado e PRÉ-multiplicado.
fn camada(gpu: &ph2d_gpu::GpuContext, w: u32, h: u32, reto: &[[f32; 4]]) -> wgpu::Texture {
    let bytes: Vec<u8> = reto
        .iter()
        .flat_map(|&[r, g, b, a]| [r * a, g * a, b * a, a])
        .flat_map(|v| f16(v).to_le_bytes())
        .collect();
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("test camada do motion"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * 8),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    tex
}

/// ⭐ **A mesma imagem, reta ou pré-multiplicada, sai igual** — sob a pilha vazia, um borrão, uma
/// sombra e um ajuste de cor, numa rampa de cobertura colorida (onde des-pré-multiplicar no sítio
/// errado lavaria a borda). E a des-pré-multiplicação TEM de acontecer: sem ela a borda escurece.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_premultiplied_layer_filters_like_the_straight_image() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_premul] sem adapter — skip");
        return;
    };
    let (w, h) = (48u32, 24u32);
    let mut reto = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let a = (f32::from(u8::try_from(x).unwrap()) / (w - 1) as f32).powf(0.7);
            let cor = [
                0.92,
                0.69 * (y as f32 / h as f32),
                0.24,
                if y < 4 { 0.0 } else { a },
            ];
            reto.push(cor);
        }
    }
    let bytes: Vec<u8> = reto
        .iter()
        .flat_map(|c| c.map(|v| (v * 255.0).round() as u8))
        .collect();
    // A reta é a quantização a 8 bits da MESMA imagem; a camada leva os mesmos 8 bits.
    let reto8: Vec<[f32; 4]> = bytes
        .chunks(4)
        .map(|c| [0, 1, 2, 3].map(|i| f32::from(c[i]) / 255.0))
        .collect();
    let src = make_src(&gpu, w, h, &bytes);
    let pre = camada(&gpu, w, h, &reto8);
    let mut sombra = op(FxOp::DROP_SHADOW, 2.0, [0.0, 0.0, 0.0, 1.0]);
    sombra.offset_px = [2, 3];
    let mut ajuste = op(FxOp::COLOR_ADJUST, 0.0, [0.0; 4]);
    ajuste.hue = 0.2;
    ajuste.sat = 0.3;
    let pilhas: [(&str, Vec<ph2d_render::FxOpGpu>); 4] = [
        ("vazia", vec![]),
        ("blur", vec![op(FxOp::BLUR, 2.5, [0.0; 4])]),
        ("sombra", vec![sombra]),
        ("ajuste", vec![ajuste]),
    ];
    let mut pass = FxStackPass::new(&gpu);
    for (nome, ops) in pilhas {
        let a = make_output_texture(&gpu, w, h);
        let b = make_output_texture(&gpu, w, h);
        pass.run(&gpu, &src, &a, w, h, &ops, &[]);
        pass.run_premultiplied(&gpu, &pre, &b, w, h, &ops);
        let (ra, rb) = (readback(&gpu, &a, w, h), readback(&gpu, &b, w, h));
        let pior = ra
            .iter()
            .zip(&rb)
            .map(|(x, y)| (i32::from(*x) - i32::from(*y)).abs())
            .max()
            .unwrap_or(0);
        eprintln!("[fx_stack_premul] {nome}: pior desvio {pior} níveis");
        assert!(
            pior <= 1,
            "{nome}: a camada pré-multiplicada desvia {pior} níveis da reta"
        );
        // O controlo: a MESMA camada lida como reta (o ingest antigo) escurece a borda.
        if nome == "vazia" {
            let c = make_output_texture(&gpu, w, h);
            pass.run(&gpu, &pre, &c, w, h, &ops, &[]);
            let rc = readback(&gpu, &c, w, h);
            let controlo = ra
                .iter()
                .zip(&rc)
                .map(|(x, y)| (i32::from(*x) - i32::from(*y)).abs())
                .max()
                .unwrap_or(0);
            assert!(
                controlo > 20,
                "o controlo não reprova ({controlo}): o gate não mede nada"
            );
        }
    }
}

/// A CENA do oráculo (`docs/Motion Nodes/ferramentas/lens_oracle/entrada.py`), 192×128 opaca.
fn cena(x: u32, y: u32) -> [u8; 3] {
    const W: u32 = 192;
    const H: u32 = 128;
    let discos: [(i64, i64, i64, [u8; 3]); 3] = [
        (48, 40, 22, [230, 40, 30]),
        (120, 70, 28, [20, 180, 60]),
        (160, 30, 16, [40, 60, 230]),
    ];
    let mut c = [
        (x * 255 / (W - 1)) as u8,
        (y * 255 / (H - 1)) as u8,
        ((x + y) * 255 / (W + H - 2)) as u8,
    ];
    for (cx, cy, r, cor) in discos {
        let (dx, dy) = (i64::from(x) - cx, i64::from(y) - cy);
        if dx * dx + dy * dy <= r * r {
            c = cor;
        }
    }
    if (20..172).contains(&x) && (100..108).contains(&y) {
        c = [255, 255, 255];
    }
    c
}

/// ⭐ **O borrão da camada é o do ImageMagick** (σ = 3 px em luz linear) no MIOLO — a pilha tem
/// borda TRANSPARENTE e o oráculo borda REPLICADA, logo só os pontos a mais de `3σ` (o suporte do
/// kernel) da borda são a mesma pergunta.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_layer_blur_is_the_oracles() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_stack_premul] sem adapter — skip");
        return;
    };
    let (w, h) = (192u32, 128u32);
    let reto: Vec<[f32; 4]> = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .map(|(x, y)| {
            let c = cena(x, y);
            [
                f32::from(c[0]) / 255.0,
                f32::from(c[1]) / 255.0,
                f32::from(c[2]) / 255.0,
                1.0,
            ]
        })
        .collect();
    let pre = camada(&gpu, w, h, &reto);
    let mut pass = FxStackPass::new(&gpu);
    let fixture = include_str!("../../../ph2d-lens/tests/fixtures/blur_linear_s3.txt");
    let mut desvio = |sigma: f32| {
        let dst = make_output_texture(&gpu, w, h);
        pass.run_premultiplied(&gpu, &pre, &dst, w, h, &[op(FxOp::BLUR, sigma, [0.0; 4])]);
        let out = readback(&gpu, &dst, w, h);
        let margem = 10;
        let (mut pior, mut n, mut pior_em) = (0.0f64, 0usize, (0, 0));
        for linha in fixture.lines().filter(|l| !l.starts_with('#')) {
            let v: Vec<f64> = linha
                .split_whitespace()
                .map(|s| s.parse().unwrap())
                .collect();
            let (x, y) = (v[0] as u32, v[1] as u32);
            if x < margem || y < margem || x >= w - margem || y >= h - margem {
                continue;
            }
            n += 1;
            let o = ((y * w + x) * 4) as usize;
            for c in 0..3 {
                let d = (f64::from(out[o + c]) / 255.0 - v[2 + c]).abs();
                if d > pior {
                    pior = d;
                    pior_em = (x, y);
                }
            }
        }
        (pior, n, pior_em)
    };
    let (pior, n, pior_em) = desvio(3.0);
    // O controlo: um σ errado por meio pixel tem de reprovar.
    let (controlo, _, _) = desvio(2.5);
    eprintln!(
        "[fx_stack_premul] blur σ=3 contra o ImageMagick: {n} pontos, pior {:.4} ({:.2} níveis) em \
         {pior_em:?}; controlo σ=2,5: {:.2} níveis",
        pior,
        pior * 255.0,
        controlo * 255.0
    );
    assert!(
        controlo > 1.5 / 255.0,
        "o controlo σ=2,5 não reprova: o gate não mede nada"
    );
    assert!(n > 900, "só {n} pontos no miolo");
    assert!(
        pior <= 1.5 / 255.0,
        "o borrão da camada desvia {pior:.4} do oráculo em {pior_em:?}"
    );
}
