//! Straight → premultiplied GPU blit for the Painter live preview
//! (ADR-0045 Phase 3, step 2).
//!
//! # Why this exists
//!
//! The [`crate::layer_compositor::LayerCompositor`] writes a STRAIGHT-sRGB8
//! `rgba8unorm` output (the GPU sibling of the CPU `composite`, which is also
//! straight). The Painter preview is shown by SUPPRESSING the source sprite and
//! sampling a preview texture in its place — and that slot lives in the
//! [`crate::individual::IndividualTextureStore`] as `Rgba8UnormSrgb` holding
//! PREMULTIPLIED bytes (exactly what the CPU path uploads via
//! `premultiply_rgba8`, so the sprite shader's "already premultiplied" branch
//! composites edge texels as `rgb·a`). To wire the GPU compositor into that
//! slot WITHOUT a CPU readback (which would erase the speed win), this pass
//! premultiplies the compositor output into a `COPY_SRC` texture the bridge
//! then `copy_texture_to_texture`s into the preview slot (`rgba8unorm` →
//! `Rgba8UnormSrgb` is copy-compatible — same texel block, sRGB-ness differs
//! only at sample time).
//!
//! # Parity
//!
//! `textureLoad` on the `rgba8unorm` source is `byte / 255` (no sRGB decode),
//! so `rgb * a` in normalized space equals the byte-space `rgb * a / 255` of
//! [`crate::premul::premultiply_rgba8`], to ±1 (store rounding). The
//! `preview_premul_matches_cpu_reference` GPU-lane test pins that bound.

use ph2d_gpu::GpuContext;

pub(crate) const PREVIEW_PREMUL_WGSL: &str = include_str!("shaders/preview_premul.wgsl");

/// Workgroup edge (mirrors `@workgroup_size(8, 8, 1)` in the shader).
const WORKGROUP_EDGE: u32 = 8;

/// The premultiplied output (straight-sRGB8 bytes, `rgba8unorm`, `COPY_SRC`).
struct OutTex {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
}

/// GPU pass that premultiplies a straight-alpha `rgba8unorm` texture into an
/// owned `COPY_SRC` output of the same size. One per painter session, held by
/// the shell bridge alongside the [`crate::layer_compositor::LayerCompositor`].
pub struct PreviewPremul {
    pipeline: wgpu::ComputePipeline,
    bgl: wgpu::BindGroupLayout,
    out: Option<OutTex>,
}

impl PreviewPremul {
    /// Build the compute pipeline. Cheap — no GPU textures until the first
    /// [`Self::run`].
    #[must_use]
    pub fn new(gpu: &GpuContext) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ph2d-render preview_premul shader"),
                source: wgpu::ShaderSource::Wgsl(PREVIEW_PREMUL_WGSL.into()),
            });
        let bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render preview_premul bgl"),
                entries: &[
                    // 0: source texture (straight rgba8unorm, read via textureLoad)
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // 1: destination storage texture (premultiplied rgba8unorm, write)
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });
        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ph2d-render preview_premul layout"),
                bind_group_layouts: &[&bgl],
                immediate_size: 0,
            });
        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("ph2d-render preview_premul pipeline"),
                layout: Some(&layout),
                module: &shader,
                entry_point: Some("cs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });
        Self {
            pipeline,
            bgl,
            out: None,
        }
    }

    /// The premultiplied output of the last [`Self::run`] (`rgba8unorm`,
    /// `COPY_SRC`). `None` before the first run.
    #[must_use]
    pub fn output_texture(&self) -> Option<&wgpu::Texture> {
        self.out.as_ref().map(|o| &o.texture)
    }

    /// Premultiply `src` (`w × h`, straight `rgba8unorm`) into the internal
    /// output texture and return it. Encodes + submits one compute dispatch.
    /// `None` for a zero-area request.
    pub fn run(
        &mut self,
        gpu: &GpuContext,
        src: &wgpu::Texture,
        w: u32,
        h: u32,
    ) -> Option<&wgpu::Texture> {
        if w == 0 || h == 0 {
            return None;
        }
        self.ensure_out(gpu, w, h);
        let src_view = src.create_view(&wgpu::TextureViewDescriptor::default());
        let out = self.out.as_ref()?;
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render preview_premul bg"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&out.view),
                },
            ],
        });
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render preview_premul encoder"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ph2d-render preview_premul pass"),
                timestamp_writes: ph2d_gpu::pass_profiler::compute_writes("render.premul"),
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(w.div_ceil(WORKGROUP_EDGE), h.div_ceil(WORKGROUP_EDGE), 1);
        }
        gpu.queue.submit([encoder.finish()]);
        self.out.as_ref().map(|o| &o.texture)
    }

    /// Ensure the output texture is sized `w × h`.
    fn ensure_out(&mut self, gpu: &GpuContext, w: u32, h: u32) {
        if let Some(o) = &self.out
            && o.width == w
            && o.height == h
        {
            return;
        }
        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-render preview_premul out"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            // STORAGE_BINDING: the compute pass writes it. COPY_SRC: the bridge
            // copies it into the `Rgba8UnormSrgb` preview slot.
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.out = Some(OutTex {
            texture,
            view,
            width: w,
            height: h,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_premul_wgsl_parses_via_naga() {
        let r = naga::front::wgsl::parse_str(PREVIEW_PREMUL_WGSL);
        assert!(
            r.is_ok(),
            "preview_premul.wgsl failed naga parse: {:?}",
            r.err()
        );
    }

    #[test]
    fn preview_premul_wgsl_validates_via_naga() {
        let module = naga::front::wgsl::parse_str(PREVIEW_PREMUL_WGSL).expect("must parse");
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        );
        let r = validator.validate(&module);
        assert!(
            r.is_ok(),
            "preview_premul.wgsl failed naga validation: {:?}",
            r.err()
        );
    }

    #[test]
    fn shader_workgroup_size_is_8x8x1() {
        let module = naga::front::wgsl::parse_str(PREVIEW_PREMUL_WGSL).expect("must parse");
        let ep = module
            .entry_points
            .iter()
            .find(|ep| ep.name == "cs_main")
            .expect("cs_main entry point");
        assert_eq!(ep.workgroup_size, [WORKGROUP_EDGE, WORKGROUP_EDGE, 1]);
    }
}
