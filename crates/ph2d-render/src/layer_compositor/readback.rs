//! A **leitura de volta bloqueante** da saída do compositor — só teste e verificação —, irmã de
//! `layer_compositor/mod.rs` por tecto de LOC.
//!
//! Corte mecânico: a função saiu inteira, verbatim; abriu-se só a visibilidade para o módulo pai.

use super::*;

/// Block-on-GPU readback of an `rgba8unorm` texture to a tight `w*h*4` buffer
/// (strips the 256-byte row padding). Test/verification only.
pub(super) fn readback_rgba8(
    gpu: &GpuContext,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> Vec<u8> {
    if width == 0 || height == 0 {
        return Vec::new();
    }
    let unpadded_bpr = width * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bpr = unpadded_bpr.div_ceil(align) * align;
    let buffer_size = (padded_bpr as u64) * (height as u64);
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d-render layer_composite readback"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ph2d-render layer_composite readback encoder"),
        });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([encoder.finish()]);

    let (tx, rx) = std::sync::mpsc::channel();
    staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    // Check the map result: on failure (device lost / validation) return empty
    // rather than letting `get_mapped_range` panic with an opaque "not mapped"
    // message that hides the real cause (audit 2026-06-01 LOW; test-path only).
    match rx.recv() {
        Ok(Ok(())) => {}
        _ => return Vec::new(),
    }

    let mapped = staging.slice(..).get_mapped_range();
    let mut out = Vec::with_capacity((unpadded_bpr as usize) * (height as usize));
    for row in 0..height as usize {
        let start = row * padded_bpr as usize;
        out.extend_from_slice(&mapped[start..start + unpadded_bpr as usize]);
    }
    drop(mapped);
    staging.unmap();
    out
}
