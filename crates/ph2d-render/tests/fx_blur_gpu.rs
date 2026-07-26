//! **`FxBlurPass` na GPU** (o borrão do FX raster vetorial, plano 24) — o par GPU do que era o
//! gate de blur na CPU (removido com o readback).
//!
//! `#[ignore]`d: precisa de um dispositivo de verdade, então roda na máquina do dev / na lane de
//! GPU do CI, não nos runners CPU headless (`None` de `try_headless_gpu`). Rodar:
//! `cargo test -p ph2d-render --test fx_blur_gpu -- --ignored`.

use std::sync::OnceLock;

use ph2d_gpu::GpuContext;
use ph2d_render::{FxBlurPass, make_output_texture};

fn try_headless_gpu() -> Option<GpuContext> {
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// Uma textura `Rgba8Unorm` de entrada (`TEXTURE_BINDING | COPY_DST`) com `bytes` (premultiplicado).
fn make_src(gpu: &GpuContext, w: u32, h: u32, bytes: &[u8]) -> wgpu::Texture {
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("test fx_blur src"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
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
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * 4),
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

fn readback(gpu: &GpuContext, tex: &wgpu::Texture, w: u32, h: u32) -> Vec<u8> {
    let unpadded = w * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test fx_blur readback"),
        size: (padded as u64) * (h as u64),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([enc.finish()]);
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();
    let view = slice.get_mapped_range();
    let mut out = Vec::with_capacity((unpadded as usize) * (h as usize));
    for row in 0..h as usize {
        let s = row * padded as usize;
        out.extend_from_slice(&view[s..s + unpadded as usize]);
    }
    drop(view);
    staging.unmap();
    out
}

/// Um degrau opaco→transparente vira uma RAMPA monótona de alfa centrada na fronteira, e ela
/// ALARGA com o sigma — a propriedade que separa um borrão de um simples corte de alfa (a queixa
/// que o produto existe para responder). No shader, na GPU, sem CPU no caminho.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_step_edge_becomes_a_monotone_alpha_ramp_that_widens_with_sigma() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_blur_gpu] sem adapter — skip");
        return;
    };
    let (w, h) = (64u32, 8u32);
    // Premultiplicado: metade esquerda branca opaca, direita transparente.
    let mut src_bytes = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w / 2 {
            let o = ((y * w + x) * 4) as usize;
            src_bytes[o..o + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    let src = make_src(&gpu, w, h, &src_bytes);
    let mut pass = FxBlurPass::new(&gpu);

    // Lê a fileira do meio (alfa por coluna) para um dado sigma.
    let alpha_row = |pass: &mut FxBlurPass, sigma: f32| -> Vec<u8> {
        let dst = make_output_texture(&gpu, w, h);
        pass.run(&gpu, &src, &dst, w, h, sigma, 0, [0.0, 0.0, 0.0, 1.0], 1.0);
        let bytes = readback(&gpu, &dst, w, h);
        let y = h / 2;
        (0..w)
            .map(|x| bytes[(((y * w + x) * 4) + 3) as usize])
            .collect()
    };
    let ramp_width = |a: &[u8]| a.iter().filter(|&&v| v > 25 && v < 230).count();

    let narrow = alpha_row(&mut pass, 2.0);
    let wide = alpha_row(&mut pass, 6.0);

    for a in [&narrow, &wide] {
        // Monótona não-crescente (opaca → transparente). i32 para não estourar o u8.
        for pair in a.windows(2) {
            assert!(
                i32::from(pair[1]) <= i32::from(pair[0]) + 4,
                "ramp de alfa nao e monotona: {a:?}"
            );
        }
        // Centrada: ~127 na fronteira (col 31/32).
        let mid = i32::from(a[(w / 2 - 1) as usize]);
        assert!(
            (mid - 127).abs() < 40,
            "fronteira nao esta em ~0.5 (deu {mid})"
        );
    }
    assert!(
        ramp_width(&wide) > ramp_width(&narrow),
        "sigma 6 ({}) deveria alargar mais que sigma 2 ({})",
        ramp_width(&wide),
        ramp_width(&narrow),
    );
}

/// O Glow/Drop Shadow (`kind != 0`) descartam a COR da forma e pintam a do EFEITO com o alfa
/// borrado da silhueta — a diferença de MODO. Fixture: forma BRANCA, tint VERMELHO.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn glow_paints_the_effect_colour_not_the_shape_colour() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx_blur_gpu] sem adapter — skip");
        return;
    };
    let (w, h) = (16u32, 16u32);
    let mut src_bytes = vec![0u8; (w * h * 4) as usize];
    for y in 4..12 {
        for x in 4..12 {
            let o = ((y * w + x) * 4) as usize;
            src_bytes[o..o + 4].copy_from_slice(&[255, 255, 255, 255]); // forma branca
        }
    }
    let src = make_src(&gpu, w, h, &src_bytes);
    let mut pass = FxBlurPass::new(&gpu);
    let dst = make_output_texture(&gpu, w, h);
    // Glow vermelho, sigma pequeno.
    pass.run(&gpu, &src, &dst, w, h, 1.0, 1, [1.0, 0.0, 0.0, 1.0], 1.0);
    let bytes = readback(&gpu, &dst, w, h);
    // No miolo coberto: cor do EFEITO (vermelho), não a branca da forma.
    let o = (((8 * w) + 8) * 4) as usize;
    assert!(
        bytes[o] > 200 && bytes[o + 1] < 40 && bytes[o + 2] < 40,
        "glow nao pintou a cor do efeito (deu {:?})",
        &bytes[o..o + 4]
    );
    assert!(bytes[o + 3] > 180, "glow perdeu a cobertura da silhueta");
}
