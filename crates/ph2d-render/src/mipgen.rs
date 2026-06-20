//! GPU mip-chain generator — fills mip levels `1..mip_count` of a 2D texture by
//! repeatedly box-downsampling the level above with a fullscreen blit
//! ([`shaders/mipgen.wgsl`](shaders/mipgen.wgsl)).
//!
//! ## Why this exists
//!
//! Sprite textures shipped with **no mip chain** (`mip_level_count == 1`), so a
//! `Linear` sampler MINIFYING a high-resolution canvas (the common case — a 2048²
//! paint canvas viewed zoomed-out) undersampled its antialiased edges into
//! jaggies (the "preview looks worse than the engine output" report, 2026-06-17;
//! `image_filter.rs` once carried the literal TODO "no mip chain yet"). Trilinear
//! sampling against a real mip chain fixes minification for EVERY sprite.
//!
//! ## Correctness
//!
//! The blit samples an `Rgba8UnormSrgb` source view, so the 2× average happens in
//! LINEAR light (hardware sRGB decode → average → encode). Alpha is linear and the
//! preview/atlas textures are PREMULTIPLIED, so a straight box average is the
//! correct downsample (no dark-fringe / wrong-alpha artefacts).

use ph2d_gpu::GpuContext;

/// Mip levels for a `width × height` 2D texture: `floor(log2(max)) + 1`.
/// `mip_levels(1) == 1`; `mip_levels(2048) == 12`.
pub fn mip_levels(width: u32, height: u32) -> u32 {
    let max = width.max(height).max(1);
    32 - max.leading_zeros()
}

/// Reusable mip-chain generator for one texture format. One instance per format
/// (sprite textures are all `Rgba8UnormSrgb`, so the renderer keeps a single one).
pub struct MipGenerator {
    pipeline: wgpu::RenderPipeline,
    bgl: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl MipGenerator {
    pub fn new(gpu: &GpuContext, format: wgpu::TextureFormat) -> Self {
        let bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render mipgen bgl"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ph2d-render mipgen layout"),
                bind_group_layouts: &[&bgl],
                immediate_size: 0,
            });
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ph2d-render mipgen shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/mipgen.wgsl").into()),
            });
        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ph2d-render mipgen pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview_mask: None,
                cache: None,
            });
        // Linear min/mag so the fullscreen blit averages the 2×2 source block.
        // ClampToEdge keeps the border rows from wrapping. No mipmap_filter needed
        // (each source view is a single level).
        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ph2d-render mipgen sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        Self {
            pipeline,
            bgl,
            sampler,
        }
    }

    /// Fill mip levels `1..mip_count` of `texture` from level 0. The texture MUST
    /// carry `RENDER_ATTACHMENT | TEXTURE_BINDING` and `mip_level_count ==
    /// mip_count`, with level 0 already populated. Submits its own command buffer
    /// (ordered after any prior submit that wrote level 0). A `mip_count <= 1`
    /// texture is a no-op.
    pub fn run(&self, gpu: &GpuContext, texture: &wgpu::Texture, mip_count: u32) {
        if mip_count <= 1 {
            return;
        }
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render mipgen encoder"),
            });
        for level in 1..mip_count {
            let src = texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("mipgen src"),
                base_mip_level: level - 1,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let dst = texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("mipgen dst"),
                base_mip_level: level,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-render mipgen bg"),
                layout: &self.bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&src),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-render mipgen pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &dst,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.draw(0..3, 0..1);
        }
        gpu.queue.submit([encoder.finish()]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mip_levels_matches_log2() {
        assert_eq!(mip_levels(1, 1), 1);
        assert_eq!(mip_levels(2, 2), 2);
        assert_eq!(mip_levels(8, 8), 4);
        assert_eq!(mip_levels(2048, 2048), 12);
        assert_eq!(mip_levels(2048, 1), 12); // driven by the larger side
        assert_eq!(mip_levels(0, 0), 1); // guarded
    }
}
