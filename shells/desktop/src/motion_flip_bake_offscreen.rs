//! **O maquinário de desenhar FORA da tela** — o provider mudo que o compositor
//! exige e a volta dos pixels do dispositivo para a CPU. Irmão de `motion_flip_bake`,
//! cortado por ASSUNTO no teto de LOC da shell (HR-18): o pai responde *o que uma tile
//! de Flip É e quando ela existe*, este responde *como se tiram os bytes do device*.
//!
//! FILHO via `#[path]`, então `super` é o módulo do bake.

use ph2d_gpu::GpuContext;
use ph2d_render::layer_compositor::{LayerPixelProvider, LayerPixels};

/// Provider that returns the zeroed dummy for ANY key at version `0` — the SAME version
/// `inject_slice_from_texture` records, so the compositor finds every slice "clean" and
/// never uploads the transparent dummy over the injected art (the frame pass's trick).
pub(super) struct DummyProvider<'a> {
    pub(super) pixels: &'a [u8],
}

impl LayerPixelProvider for DummyProvider<'_> {
    fn layer_pixels(&self, _key: u64) -> Option<LayerPixels<'_>> {
        Some(LayerPixels {
            version: 0,
            rgba8: self.pixels,
            dirty: None,
        })
    }
}

/// Read a straight `Rgba8Unorm` texture back to CPU RGBA (`w·h·4`, row padding stripped)
/// — the standard `copy_texture_to_buffer` + `map_async` + poll dance (mirrors
/// `fx_dump::readback`). Slow, but it runs only on a content change (cached).
pub(super) fn readback(gpu: &GpuContext, tex: &wgpu::Texture, w: u32, h: u32) -> Vec<u8> {
    let unpadded = w * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("motion flip bake readback"),
        size: u64::from(padded) * u64::from(h),
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
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let _ = rx.recv();
    let view = slice.get_mapped_range();
    let mut out = Vec::with_capacity((unpadded * h) as usize);
    for row in 0..h as usize {
        let s = row * padded as usize;
        out.extend_from_slice(&view[s..s + unpadded as usize]);
    }
    drop(view);
    staging.unmap();
    out
}
