//! Corte F37 (teto de LOC) de `motion_fx.rs`: a escrita da tabela do halo na textura e o passe de
//! triângulo de ecrã inteiro que todas as etapas usam.

use super::{HALO_LUT_TEXELS, MotionFx};
use ph2d_gpu::GpuContext;

impl MotionFx {
    /// Escreve a tabela na textura, convertendo para meio-float — o formato que a torna
    /// **filtrável** (ver [`make_lut`]).
    pub(super) fn upload_lut(&self, gpu: &GpuContext, lut: &[[f32; 4]]) {
        let mut bytes = Vec::with_capacity(HALO_LUT_TEXELS * 8);
        for texel in lut {
            for c in texel {
                bytes.extend_from_slice(&ph2d_color::f32_to_half(*c).to_le_bytes());
            }
        }
        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.lut.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((HALO_LUT_TEXELS * 8) as u32),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: HALO_LUT_TEXELS as u32,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
    }
}

/// One fullscreen-triangle pass into `view`.
pub(super) fn fullscreen(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &wgpu::RenderPipeline,
    bind_group: &wgpu::BindGroup,
    view: &wgpu::TextureView,
    load: wgpu::LoadOp<wgpu::Color>,
    profile: &'static str,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("ph2d-render motion-fx pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: ph2d_gpu::pass_profiler::render_writes(profile)
            .as_ref()
            .map(ph2d_gpu::pass_profiler::PassTimestamps::render),
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.draw(0..3, 0..1);
}
