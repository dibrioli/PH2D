//! GPU `LayerCompositor` implementation, split by responsibility (blindagem
//! Fase 3.2 — was a single 2396-LOC file). This file holds pipeline setup
//! (`new`) + accessors; the rest fan out into the sibling modules below. Every
//! method is on the same `LayerCompositor` (defined in the parent
//! `layer_compositor` module); the submodules reach the parent's items via
//! `use super::super::*`. `ShTonal` lives here (private) so both `dispatch`
//! (constructs it) and `effects` (reads it) see it as descendant modules.

use super::*;

mod api; // public composite entry points
mod buffers; // texture/buffer (re)allocation + uploads
mod dispatch; // dispatch + segmented (grouped) compositing
mod effects; // bloom / shadows-highlights / combine / encode kernels
mod pass; // work-texture setup + segment/blur/chroma passes

/// The 6 Shadows/Highlights tonal scalars `cs_combine_sh` reads (the two radii
/// drive the blurs instead, so they live in the `BlurStage::Sh` weights).
/// Pulled from `SpatialAdjustment.params` by `resolve_kernel`.
#[derive(Copy, Clone)]
struct ShTonal {
    shadows_amount: f32,
    highlights_amount: f32,
    shadows_tonal_width: f32,
    highlights_tonal_width: f32,
    color_correction: f32,
    midtone_contrast: f32,
}

impl LayerCompositor {
    /// Build the compute pipeline. Cheap — no GPU textures until the first
    /// [`Self::composite`].
    #[must_use]
    pub fn new(gpu: &GpuContext) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ph2d-render layer_composite shader"),
                source: wgpu::ShaderSource::Wgsl(composite_source().into()),
            });

        let bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render layer_composite bgl"),
                entries: &[
                    // 0: ops storage (read)
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                core::mem::size_of::<GpuOp>() as u64
                            ),
                        },
                        count: None,
                    },
                    // 1: globals uniform
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                core::mem::size_of::<GpuGlobals>() as u64,
                            ),
                        },
                        count: None,
                    },
                    // 2: layer texture array (sampled via textureLoad — non-filterable)
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // 3: output storage texture (write)
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    // 4: sRGB→linear decode LUT (256 × f32 storage, read)
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(SRGB_LUT_LEN as u64 * 4),
                        },
                        count: None,
                    },
                    // 5: adjustment params storage (read; ≥1 element always bound)
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                core::mem::size_of::<AdjParamsGpu>() as u64,
                            ),
                        },
                        count: None,
                    },
                    // 6: display-space transfer LUTs (Curves/Levels); ≥1 f32 bound
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(4),
                        },
                        count: None,
                    },
                ],
            });

        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ph2d-render layer_composite layout"),
                bind_group_layouts: &[&bgl],
                immediate_size: 0,
            });

        let make_pipeline = |entry: &str, label: &str| {
            gpu.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(label),
                    layout: Some(&layout),
                    module: &shader,
                    entry_point: Some(entry),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                })
        };
        let pipeline_flat = make_pipeline("cs_flat", "ph2d-render layer_composite flat");
        let pipeline_grouped = make_pipeline("cs_grouped", "ph2d-render layer_composite grouped");

        // ── Segmented spatial pass-graph BGLs + pipelines (bindings 7–20) ─────
        let storage_ro = |binding: u32, min: u64| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(min),
            },
            count: None,
        };
        let uniform = |binding: u32, min: u64| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(min),
            },
            count: None,
        };
        let sampled = |binding: u32, dim: wgpu::TextureViewDimension| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: dim,
                multisampled: false,
            },
            count: None,
        };
        let storage_tex = |binding: u32, format: wgpu::TextureFormat| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        };
        let f32_lin = wgpu::TextureFormat::Rgba32Float;
        let op_sz = core::mem::size_of::<GpuOp>() as u64;
        let adj_sz = core::mem::size_of::<AdjParamsGpu>() as u64;

        let bgl_segment = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render layer_composite seg bgl"),
                entries: &[
                    storage_ro(0, op_sz),                            // ops
                    sampled(2, wgpu::TextureViewDimension::D2Array), // layers
                    storage_ro(4, SRGB_LUT_LEN as u64 * 4),          // srgb lut
                    storage_ro(5, adj_sz),                           // adj params
                    storage_ro(6, 4),                                // adj luts
                    uniform(7, core::mem::size_of::<SegGlobals>() as u64),
                    storage_tex(8, f32_lin), // seg_out (linear)
                    sampled(9, wgpu::TextureViewDimension::D2), // base_in
                ],
            });
        let bgl_blur = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render layer_composite blur bgl"),
                entries: &[
                    uniform(10, core::mem::size_of::<BlurGlobals>() as u64),
                    sampled(11, wgpu::TextureViewDimension::D2), // src
                    storage_tex(12, f32_lin),                    // dst
                    storage_ro(13, 4),                           // weights (≥1 f32)
                ],
            });
        let bgl_combine = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render layer_composite combine bgl"),
                entries: &[
                    uniform(14, core::mem::size_of::<CombineGlobals>() as u64),
                    sampled(15, wgpu::TextureViewDimension::D2), // base
                    sampled(16, wgpu::TextureViewDimension::D2), // blurred
                    storage_tex(17, f32_lin),                    // dst
                ],
            });
        let bgl_encode = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render layer_composite encode bgl"),
                entries: &[
                    uniform(18, core::mem::size_of::<EncodeGlobals>() as u64),
                    sampled(19, wgpu::TextureViewDimension::D2), // src (linear)
                    storage_tex(20, wgpu::TextureFormat::Rgba8Unorm), // out (sRGB8)
                ],
            });
        let bgl_chroma = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render layer_composite chroma bgl"),
                entries: &[
                    uniform(21, core::mem::size_of::<ChromaGlobals>() as u64),
                    sampled(22, wgpu::TextureViewDimension::D2), // src (base, linear)
                    storage_tex(23, f32_lin),                    // dst (linear)
                ],
            });
        let bgl_bloom = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render layer_composite bloom bgl"),
                entries: &[
                    uniform(24, core::mem::size_of::<BloomGlobals>() as u64),
                    sampled(25, wgpu::TextureViewDimension::D2), // base (linear)
                    storage_tex(26, f32_lin),                    // glow (premultiplied)
                ],
            });
        let bgl_bloom_mip = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render layer_composite bloom mip bgl"),
                entries: &[
                    uniform(27, core::mem::size_of::<BloomMipGlobals>() as u64),
                    sampled(28, wgpu::TextureViewDimension::D2), // src
                    storage_tex(29, f32_lin),                    // dst
                ],
            });
        let sh_sz = core::mem::size_of::<ShGlobals>() as u64;
        let bgl_sh_luma = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render layer_composite sh luma bgl"),
                entries: &[
                    uniform(27, sh_sz),
                    sampled(28, wgpu::TextureViewDimension::D2), // base (linear)
                    storage_tex(29, f32_lin),                    // luma field (.r)
                ],
            });
        let bgl_sh_combine =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("ph2d-render layer_composite sh combine bgl"),
                    entries: &[
                        uniform(27, sh_sz),
                        sampled(28, wgpu::TextureViewDimension::D2), // base (linear)
                        sampled(30, wgpu::TextureViewDimension::D2), // local_lo (shadows)
                        sampled(31, wgpu::TextureViewDimension::D2), // local_hi (highlights)
                        storage_tex(32, f32_lin),                    // dst (linear)
                    ],
                });

        let make_pipeline_for = |bgl: &wgpu::BindGroupLayout, entry: &str, label: &str| {
            let layout = gpu
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(label),
                    bind_group_layouts: &[bgl],
                    immediate_size: 0,
                });
            gpu.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(label),
                    layout: Some(&layout),
                    module: &shader,
                    entry_point: Some(entry),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                })
        };
        let pipeline_segment = make_pipeline_for(
            &bgl_segment,
            "cs_segment",
            "ph2d-render layer_composite segment",
        );
        let pipeline_blur_h =
            make_pipeline_for(&bgl_blur, "cs_blur_h", "ph2d-render layer_composite blur_h");
        let pipeline_blur_v =
            make_pipeline_for(&bgl_blur, "cs_blur_v", "ph2d-render layer_composite blur_v");
        let pipeline_blur_dir = make_pipeline_for(
            &bgl_blur,
            "cs_blur_dir",
            "ph2d-render layer_composite blur_dir",
        );
        let pipeline_combine = make_pipeline_for(
            &bgl_combine,
            "cs_combine",
            "ph2d-render layer_composite combine",
        );
        let pipeline_encode = make_pipeline_for(
            &bgl_encode,
            "cs_encode",
            "ph2d-render layer_composite encode",
        );
        let pipeline_chroma = make_pipeline_for(
            &bgl_chroma,
            "cs_chroma",
            "ph2d-render layer_composite chroma",
        );
        let pipeline_bloom_bright = make_pipeline_for(
            &bgl_bloom,
            "cs_bloom_bright",
            "ph2d-render layer_composite bloom_bright",
        );
        let pipeline_bloom_down = make_pipeline_for(
            &bgl_bloom_mip,
            "cs_bloom_down",
            "ph2d-render layer_composite bloom_down",
        );
        let pipeline_bloom_up = make_pipeline_for(
            &bgl_bloom_mip,
            "cs_bloom_up",
            "ph2d-render layer_composite bloom_up",
        );
        let pipeline_sh_luma = make_pipeline_for(
            &bgl_sh_luma,
            "cs_sh_luma",
            "ph2d-render layer_composite sh_luma",
        );
        let pipeline_sh_combine = make_pipeline_for(
            &bgl_sh_combine,
            "cs_combine_sh",
            "ph2d-render layer_composite sh_combine",
        );

        let make_uniform_buf = |size: u64, label: &str| {
            gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let seg_globals_buffer = make_uniform_buf(
            core::mem::size_of::<SegGlobals>() as u64,
            "ph2d-render layer_composite seg globals",
        );
        let blur_globals_buffer = make_uniform_buf(
            core::mem::size_of::<BlurGlobals>() as u64,
            "ph2d-render layer_composite blur globals",
        );
        let combine_globals_buffer = make_uniform_buf(
            core::mem::size_of::<CombineGlobals>() as u64,
            "ph2d-render layer_composite combine globals",
        );
        let encode_globals_buffer = make_uniform_buf(
            core::mem::size_of::<EncodeGlobals>() as u64,
            "ph2d-render layer_composite encode globals",
        );
        let chroma_globals_buffer = make_uniform_buf(
            core::mem::size_of::<ChromaGlobals>() as u64,
            "ph2d-render layer_composite chroma globals",
        );
        let bloom_globals_buffer = make_uniform_buf(
            core::mem::size_of::<BloomGlobals>() as u64,
            "ph2d-render layer_composite bloom globals",
        );
        let bloom_mip_globals_buffer = make_uniform_buf(
            core::mem::size_of::<BloomMipGlobals>() as u64,
            "ph2d-render layer_composite bloom mip globals",
        );
        let sh_globals_buffer = make_uniform_buf(
            core::mem::size_of::<ShGlobals>() as u64,
            "ph2d-render layer_composite sh globals",
        );

        // 1×1 linear dummy bound as base_in for the first (start-from-zero) segment.
        let dummy_tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-render layer_composite seg base dummy"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: f32_lin,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let seg_base_dummy = dummy_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let globals_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render layer_composite globals"),
            size: core::mem::size_of::<GpuGlobals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let lut = build_srgb_lut();
        let srgb_lut_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render layer_composite srgb lut"),
            size: (SRGB_LUT_LEN * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue
            .write_buffer(&srgb_lut_buffer, 0, bytemuck::cast_slice(&lut));

        Self {
            pipeline_flat,
            pipeline_grouped,
            bgl,
            device_max_layers: gpu.device.limits().max_texture_array_layers,
            // Read ONCE at construction: `get_info()` round-trips to the backend, and
            // the answer cannot change for the life of a device.
            cache_budget_bytes: layer_cache_budget(gpu.adapter.get_info().device_type),
            array: None,
            out: None,
            cache: BTreeMap::new(),
            clock: 0,
            scratch_ops: GpuOpScratch::new(),
            op_buffer: None,
            globals_buffer,
            srgb_lut_buffer,
            adj_params_buffer: None,
            adj_luts_buffer: None,
            pipeline_segment,
            pipeline_blur_h,
            pipeline_blur_v,
            pipeline_blur_dir,
            pipeline_chroma,
            pipeline_bloom_bright,
            pipeline_bloom_down,
            pipeline_bloom_up,
            pipeline_sh_luma,
            pipeline_sh_combine,
            pipeline_combine,
            pipeline_encode,
            bgl_segment,
            bgl_blur,
            bgl_combine,
            bgl_encode,
            bgl_chroma,
            bgl_bloom,
            bgl_bloom_mip,
            bgl_sh_luma,
            bgl_sh_combine,
            seg_globals_buffer,
            blur_globals_buffer,
            combine_globals_buffer,
            encode_globals_buffer,
            chroma_globals_buffer,
            bloom_globals_buffer,
            bloom_mip_globals_buffer,
            sh_globals_buffer,
            blur_weights_buffer: None,
            seg_base_dummy,
            work: None,
        }
    }

    /// Cache cap for `width × height` on this device: the smaller of the
    /// per-budget cap and the device's `max_texture_array_layers`.
    #[must_use]
    pub fn cache_cap(&self, width: u32, height: u32) -> u32 {
        max_layers_for_budget(width, height, self.cache_budget_bytes).min(self.device_max_layers)
    }

    /// The VRAM budget this compositor resolved for its device — exposed so a
    /// measurement (and a caller sizing a document) READS the budget in force
    /// instead of re-deriving it from a const that may not be the one chosen.
    #[must_use]
    pub fn cache_budget_bytes(&self) -> u64 {
        self.cache_budget_bytes
    }

    /// Number of cached layer slices currently resident.
    #[must_use]
    pub fn cached_len(&self) -> usize {
        self.cache.len()
    }

    /// **Does this compositor still hold a slice for `key`?** — asked by a producer that wants to
    /// SKIP re-rendering a layer whose pixels have not changed.
    ///
    /// ⚠️ **The `false` half is the one that matters, and there are two ways to get it:** the slice
    /// may have been **evicted** (`alloc_slice`'s LRU, when layers outnumber the cap) or **cleared**
    /// wholesale by an array rebuild (a resize, or a wider op-list). A producer that trusted its own
    /// memo instead of asking would keep showing art it no longer owns, in exactly those two cases —
    /// the ADR-0124 lesson (ask the OWNER, never your own copy) at the slice level.
    ///
    /// ⚠️ **Read-only ON PURPOSE — it does not touch `last_used`.** A skipping producer therefore
    /// lets its stable layers grow cold, so in a scene with more layers than `cache_cap` they can be
    /// evicted and re-rendered. That degrades to *doing the work* (today's behaviour), never to
    /// showing stale pixels — and a query that silently counted as a use would be a lie about being
    /// read-only.
    #[must_use]
    pub fn has_slice(&self, key: u64) -> bool {
        self.cache.contains_key(&key)
    }

    /// The output texture of the last [`Self::composite`] (region-sized,
    /// straight sRGB8 `rgba8unorm`). The shell blits this onto the sprite;
    /// `None` before the first composite.
    #[must_use]
    pub fn output_texture(&self) -> Option<&wgpu::Texture> {
        self.out.as_ref().map(|o| &o.texture)
    }
}
