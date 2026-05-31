//! SpritePipeline — the wgpu RenderPipeline + bind group layouts for
//! the sprite shader (`shaders/sprite.wgsl`).
//!
//! Per LLM1 audit §10.5: `PipelineLayout` is **always** explicit. Two
//! BindGroupLayouts (frame/material) match the shader's `@group(0)`
//! and `@group(1)` declarations. Per-instance data is via the second
//! vertex buffer slot (instance step mode) — cheaper than a third
//! bind group for streaming data. Single triangle-strip draw call per
//! batch.
//!
//! W3 §8 ClipChildren (ADR-0070-amendment-7 / ADR-0074-amendment-1):
//! beside the `normal` pipeline (no stencil) there are two stencil
//! variants sharing the same layout + shader module:
//! - `mark` — writes the clip-parent silhouette into the stencil
//!   buffer (`Replace` ref where the texel alpha clears the cutoff),
//!   color writes MASKED OFF. Reads the per-instance `clip_meta`
//!   (cutoff) via `@location(5)` — the freed `tint` slot repurposed,
//!   since the 16-attribute device limit (`@location` 0..15) is full.
//! - `test` — draws the clipped descendants with stencil `Equal ref`
//!   (read-only), so they only appear inside the marked silhouette.
//!   Reuses `vs_main`/`fs_main` (the normal color path).

use crate::sprite::{QuadVertex, RenderInstance};
use ph2d_gpu::GpuContext;

/// Stencil attachment format for the ClipChildren mark/test passes
/// (ADR-0074-amendment-1). `Stencil8` is a WebGPU-required, stencil-only
/// format (no depth aspect → the render pass needs no depth ops), keeping
/// the clip pass minimal. Chosen over `Depth24PlusStencil8` because we
/// never use depth here.
pub const STENCIL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Stencil8;

pub struct SpritePipeline {
    /// Normal pass (no stencil): every non-clipped sprite, exactly as
    /// before ClipChildren existed (zero-regression path).
    pub pipeline: wgpu::RenderPipeline,
    /// Stencil-mark pass: writes the clip-parent / Mask2D silhouette, no
    /// color. Shared by ClipChildren and Mask2D (both mark by alpha cutoff).
    pub mark_pipeline: wgpu::RenderPipeline,
    /// Stencil-test pass: draws where `stencil == ref` (ClipChildren
    /// members + MaskInteraction `VisibleInside` responders).
    pub test_pipeline: wgpu::RenderPipeline,
    /// Inverse stencil-test pass: draws where `stencil != ref`
    /// (MaskInteraction `VisibleOutside` responders).
    pub test_outside_pipeline: wgpu::RenderPipeline,
    pub frame_bgl: wgpu::BindGroupLayout,
    pub material_bgl: wgpu::BindGroupLayout,
}

/// The parts of a sprite render pipeline that vary between the normal /
/// mark / test variants. Everything else (primitive state, multisample,
/// pipeline layout, shader module) is shared.
struct Variant<'a> {
    label: &'a str,
    vs_entry: &'a str,
    fs_entry: &'a str,
    buffers: &'a [wgpu::VertexBufferLayout<'a>],
    color_write: wgpu::ColorWrites,
    blend: Option<wgpu::BlendState>,
    depth_stencil: Option<wgpu::DepthStencilState>,
}

impl SpritePipeline {
    pub fn new(gpu: &GpuContext, color_format: wgpu::TextureFormat) -> Self {
        let frame_bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render frame bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let material_bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render material bgl"),
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
                label: Some("ph2d-render sprite layout"),
                bind_group_layouts: &[&frame_bgl, &material_bgl],
                immediate_size: 0,
            });

        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ph2d-render sprite shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/sprite.wgsl").into()),
            });

        // Normal-pass vertex buffers: quad + the 12-attr instance layout.
        let normal_buffers = [QuadVertex::buffer_layout(), RenderInstance::buffer_layout()];

        // Mark-pass instance layout. The 16-attribute device limit
        // (`@location` 0..15) is already FULL (quad 0..1 + instance 2..15),
        // so `clip_meta` can't be a 17th attribute. The mark vertex shader
        // only needs the geometry + UV inputs, so we build a minimal layout
        // over the SAME instance buffer (explicit per-field offsets) and
        // REPURPOSE the freed `@location(5)` (normally `tint`, which the mark
        // shader ignores) to carry `clip_meta` from its real CPU-tail offset.
        // wgpu interprets each pipeline's own layout, so this aliases nothing.
        let off = |f| f as wgpu::BufferAddress;
        let mark_attrs = [
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: off(std::mem::offset_of!(RenderInstance, world_pos)),
                shader_location: 2,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: off(std::mem::offset_of!(RenderInstance, size)),
                shader_location: 3,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: off(std::mem::offset_of!(RenderInstance, atlas_uv)),
                shader_location: 4,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: off(std::mem::offset_of!(RenderInstance, basis)),
                shader_location: 6,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: off(std::mem::offset_of!(RenderInstance, anchor)),
                shader_location: 8,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint32,
                offset: off(std::mem::offset_of!(RenderInstance, flip_uv)),
                shader_location: 14,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: off(std::mem::offset_of!(RenderInstance, uv_xform)),
                shader_location: 15,
            },
            // Repurposed: clip_meta (cutoff bits) at @location(5).
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint32,
                offset: off(std::mem::offset_of!(RenderInstance, clip_meta)),
                shader_location: 5,
            },
        ];
        let mark_buffers = [
            QuadVertex::buffer_layout(),
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<RenderInstance>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &mark_attrs,
            },
        ];

        // Stencil mark: write `ref` (Replace) wherever the fragment survives
        // the alpha cutoff; color writes off. front == back (cull_mode None).
        let mark_face = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Always,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::Replace,
        };
        let mark_ds = wgpu::DepthStencilState {
            format: STENCIL_FORMAT,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            stencil: wgpu::StencilState {
                front: mark_face,
                back: mark_face,
                read_mask: 0xff,
                write_mask: 0xff,
            },
            bias: wgpu::DepthBiasState::default(),
        };
        // Stencil test: draw only where `stencil == ref`; never write stencil.
        let test_face = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Equal,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::Keep,
        };
        let test_ds = wgpu::DepthStencilState {
            format: STENCIL_FORMAT,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            stencil: wgpu::StencilState {
                front: test_face,
                back: test_face,
                read_mask: 0xff,
                write_mask: 0x00,
            },
            bias: wgpu::DepthBiasState::default(),
        };
        // Inverse test (Mask2D VisibleOutside): draw where stencil != ref.
        let test_outside_face = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::NotEqual,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::Keep,
        };
        let test_outside_ds = wgpu::DepthStencilState {
            format: STENCIL_FORMAT,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            stencil: wgpu::StencilState {
                front: test_outside_face,
                back: test_outside_face,
                read_mask: 0xff,
                write_mask: 0x00,
            },
            bias: wgpu::DepthBiasState::default(),
        };

        let pipeline = build_variant(
            &gpu.device,
            &layout,
            &shader,
            color_format,
            Variant {
                label: "ph2d-render sprite pipeline",
                vs_entry: "vs_main",
                fs_entry: "fs_main",
                buffers: &normal_buffers,
                color_write: wgpu::ColorWrites::ALL,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                depth_stencil: None,
            },
        );
        let mark_pipeline = build_variant(
            &gpu.device,
            &layout,
            &shader,
            color_format,
            Variant {
                label: "ph2d-render clip stencil-mark pipeline",
                vs_entry: "vs_stencil_mark",
                fs_entry: "fs_stencil_mark",
                buffers: &mark_buffers,
                // Mark writes ONLY the stencil — no color (the silhouette
                // is a mould, not pixels). spec §6.2 step 1.
                color_write: wgpu::ColorWrites::empty(),
                blend: None,
                depth_stencil: Some(mark_ds),
            },
        );
        let test_pipeline = build_variant(
            &gpu.device,
            &layout,
            &shader,
            color_format,
            Variant {
                label: "ph2d-render clip stencil-test pipeline",
                vs_entry: "vs_main",
                fs_entry: "fs_main",
                buffers: &normal_buffers,
                color_write: wgpu::ColorWrites::ALL,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                depth_stencil: Some(test_ds),
            },
        );

        let test_outside_pipeline = build_variant(
            &gpu.device,
            &layout,
            &shader,
            color_format,
            Variant {
                label: "ph2d-render mask stencil-test-outside pipeline",
                vs_entry: "vs_main",
                fs_entry: "fs_main",
                buffers: &normal_buffers,
                color_write: wgpu::ColorWrites::ALL,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                depth_stencil: Some(test_outside_ds),
            },
        );

        Self {
            pipeline,
            mark_pipeline,
            test_pipeline,
            test_outside_pipeline,
            frame_bgl,
            material_bgl,
        }
    }
}

/// Build one sprite render pipeline variant. The primitive state
/// (triangle-strip, CCW, no cull) + multisample are identical across all
/// three; only the entry points / vertex buffers / color-write / blend /
/// depth-stencil differ (the [`Variant`] fields).
fn build_variant(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    color_format: wgpu::TextureFormat,
    v: Variant,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(v.label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some(v.vs_entry),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: v.buffers,
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(v.fs_entry),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                // M14.5: premultiplied alpha throughout the pipeline
                // (game_rt → tonemap → compositor). The fragment emits
                // `color.rgb *= color.a` before return so sprites with α<1
                // composite correctly. The mark variant masks color off, so
                // its blend is None (irrelevant — nothing is written).
                blend: v.blend,
                write_mask: v.color_write,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: v.depth_stencil,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    /// Shared headless GPU (mirrors `game_rt`/`atlas` test helpers). Each
    /// adapter+device pair costs ~30-50 s cold on Apple Silicon, so cache
    /// it per test binary. Returns `None` on adapter-less environments so
    /// the test skips (passes) where wgpu can't spin up a device.
    fn try_headless_gpu() -> Option<GpuContext> {
        static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
        SHARED
            .get_or_init(|| {
                let instance = GpuContext::default_instance();
                GpuContext::new(instance, None).ok()
            })
            .clone()
    }

    #[test]
    fn sprite_pipeline_v4_shader_compiles_and_binds_eleven_attrs() {
        // The ONLY automated coverage of `SpritePipeline::new` (W1.T1.11
        // closes the T1.7a "pipeline unverified by CI" gap). Building the
        // pipeline runs naga's WGSL front-end + validator on
        // `shaders/sprite.wgsl` AND binds `RenderInstance::buffer_layout()`'s
        // 12 vertex attributes (@location 2..15) against the v4
        // `InstanceInput`. A WGSL syntax/type error (e.g. the new
        // per-corner `mix`, the `& Nu` flag decode, `select`, the `flat`
        // interpolation) or a location/format mismatch raises an
        // uncaptured validation error → wgpu panics → this test fails.
        //
        // W3 §8: this now ALSO compiles + validates the stencil-mark /
        // stencil-test pipelines (`vs_stencil_mark` / `fs_stencil_mark` +
        // the `clip_meta` vertex attribute at the repurposed `@location(5)`
        // + the two `DepthStencilState`s on the `Stencil8` format).
        //
        // Skips gracefully (passes) on adapter-less CI; runs on dev Macs +
        // Mac CI, which is where the Enio smoke also runs.
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let _pipeline = SpritePipeline::new(&gpu, wgpu::TextureFormat::Rgba8UnormSrgb);
        // Reaching here means naga validated the v4 shader + both stencil
        // variants and every vertex layout bound without a device error.
    }
}
