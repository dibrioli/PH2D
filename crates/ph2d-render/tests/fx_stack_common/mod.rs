//! Os utilitários partilhados dos gates de GPU do `FxStackPass` (o dispositivo, a textura de
//! entrada, o readback e o construtor de degrau).
//!
//! Mora num SUBDIRETÓRIO de propósito: um arquivo solto em `tests/` vira outro binário de teste.
//! Uma cópia por arquivo de gate divergiria — e o readback é exatamente o tipo de código onde uma
//! divergência (o padding de linha) faz um gate medir a coisa errada em silêncio.

#![allow(dead_code)]

use std::sync::OnceLock;

use ph2d_gpu::GpuContext;
use ph2d_render::FxOpGpu;

/// Um degrau, resolvido em pixels.
pub fn op(kind: u8, sigma_px: f32, tint: [f32; 4]) -> FxOpGpu {
    FxOpGpu {
        kind,
        sigma_px,
        offset_px: [0, 0],
        tint,
        opacity: 1.0,
        mode: 0,
        blend: 0,
        noise_scale_px: 0.0,
        detail: 1,
        seed: 0,
        grow_px: 0.0,
        hue: 0.0,
        sat: 0.0,
        bright: 0.0,
    }
}

pub const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
pub const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

pub fn try_headless_gpu() -> Option<GpuContext> {
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// Uma textura `Rgba8Unorm` de entrada com `bytes` — alfa **RETO**, como o Vello entrega.
pub fn make_src(gpu: &GpuContext, w: u32, h: u32, bytes: &[u8]) -> wgpu::Texture {
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("test fx_stack src"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        // ⚠️ `COPY_SRC` para a sonda poder FOTOGRAFAR a fonte: sem ele o `0_src` da
        // `fx_look_probe` era erro de validação, e o modo analítico dela nunca rodava — só o
        // `PH2D_FX_VELLO=1` passava, o que fazia a comparação entre rasterizadores impossível.
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
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

pub fn readback(gpu: &GpuContext, tex: &wgpu::Texture, w: u32, h: u32) -> Vec<u8> {
    let unpadded = w * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test fx_stack readback"),
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
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
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

// ── A FIXTURE OBLÍQUA ANTIALIASADA ────────────────────────────────────────────────────────────
//
// ⚠️ **A inclinação é IRRACIONAL de propósito, e é o coração destes gates.** Numa normal racional
// — o gate antigo do bevel usava `(2,5)/√29` — todo texel à mesma distância da aresta é translação
// de rede de outro, então a FASE da rasterização é a mesma em todos e o artefato mede **zero por
// construção**. Foi assim que um pente de dezenas de níveis atravessou 13 gates verdes.
//
// E é rasa: a 23,7° a escada de rasterização tem passo longo, que é onde a estimativa da fronteira
// pela cobertura erra mais (um estêncil 3×3 não resolve a inclinação).

/// A inclinação da aresta (irracional).
pub const SLOPE: f64 = 0.438_74;
/// Onde ela cruza o centro da textura.
pub const EDGE_C: f64 = 48.0;
/// A cor da forma, em sRGB reto.
pub const INK: [f64; 3] = [235.0, 175.0, 60.0];

/// Distância COM SINAL do centro do texel `(x, y)` à aresta — positiva DENTRO.
#[must_use]
pub fn oblique_signed(x: u32, y: u32) -> f64 {
    let px = f64::from(x) + 0.5;
    let py = f64::from(y) + 0.5;
    (py - (SLOPE * (px - EDGE_C) + EDGE_C)) / (1.0 + SLOPE * SLOPE).sqrt()
}

/// O meio-plano antialiasado — cobertura por supersampling 8×8, **alfa RETO**.
///
/// ⚠️ **RETO, e isso foi MEDIDO no rasterizador real, não suposto.** O `render_to_intermediate` do
/// Vello escreve a cor CHEIA com o alfa ao lado: censo de uma estrela, **1696 de 1696** texels de
/// cobertura parcial trazem `(235,175,60)`. A premissa "a fonte é premultiplicada" atravessou o
/// módulo inteiro porque num texel OPACO e num VAZIO as duas convenções coincidem — e toda fixture
/// de cobertura parcial foi escrita pela mesma mão que escreveu a premissa.
pub fn oblique_source(gpu: &GpuContext, w: u32, h: u32) -> wgpu::Texture {
    let mut bytes = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let mut acc = 0u32;
            for sy in 0..8u32 {
                for sx in 0..8u32 {
                    let px = f64::from(x) + (f64::from(sx) + 0.5) / 8.0;
                    let py = f64::from(y) + (f64::from(sy) + 0.5) / 8.0;
                    if py > SLOPE * (px - EDGE_C) + EDGE_C {
                        acc += 1;
                    }
                }
            }
            let cov = f64::from(acc) / 64.0;
            let o = ((y * w + x) * 4) as usize;
            for c in 0..3 {
                bytes[o + c] = INK[c].round() as u8;
            }
            bytes[o + 3] = (cov * 255.0).round() as u8;
        }
    }
    make_src(gpu, w, h, &bytes)
}

/// A MESMA aresta, como segmento — a geometria que o campo exato consome. Um só, longo o
/// bastante para cobrir a textura pelos dois lados.
#[must_use]
pub fn oblique_segments(w: u32) -> Vec<[f32; 4]> {
    let far = f64::from(w) * 4.0;
    let y = |x: f64| SLOPE * (x - EDGE_C) + EDGE_C;
    vec![[(-far) as f32, y(-far) as f32, far as f32, y(far) as f32]]
}
