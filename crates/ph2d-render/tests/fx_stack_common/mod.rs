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

/// Uma textura `Rgba8Unorm` de entrada (`TEXTURE_BINDING | COPY_DST`) com `bytes` (premultiplicado).
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
