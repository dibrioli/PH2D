use super::*;

impl LayerCompositor {
    /// Build the compute pipeline. Cheap — no GPU textures until the first
    /// [`Self::composite`].
    #[must_use]
    pub fn new(gpu: &GpuContext) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ph2d-render layer_composite shader"),
                source: wgpu::ShaderSource::Wgsl(LAYER_COMPOSITE_WGSL.into()),
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
            pipeline_combine,
            pipeline_encode,
            bgl_segment,
            bgl_blur,
            bgl_combine,
            bgl_encode,
            seg_globals_buffer,
            blur_globals_buffer,
            combine_globals_buffer,
            encode_globals_buffer,
            blur_weights_buffer: None,
            seg_base_dummy,
            work: None,
        }
    }

    /// Cache cap for `width × height` on this device: the smaller of the
    /// per-budget cap and the device's `max_texture_array_layers`.
    #[must_use]
    pub fn cache_cap(&self, width: u32, height: u32) -> u32 {
        max_layers_for_budget(width, height, LAYER_CACHE_BUDGET_BYTES).min(self.device_max_layers)
    }

    /// Number of cached layer slices currently resident.
    #[must_use]
    pub fn cached_len(&self) -> usize {
        self.cache.len()
    }

    /// The output texture of the last [`Self::composite`] (region-sized,
    /// straight sRGB8 `rgba8unorm`). The shell blits this onto the sprite;
    /// `None` before the first composite.
    #[must_use]
    pub fn output_texture(&self) -> Option<&wgpu::Texture> {
        self.out.as_ref().map(|o| &o.texture)
    }

    /// Composite `ops` into the output texture, covering `region` of the
    /// `canvas_w × canvas_h` canvas. Uploads only layers whose version changed
    /// since the last call. Encodes + submits one compute dispatch.
    /// Composite with no display-space transfer LUTs — the common case (the
    /// op-list has no Curves/Levels adjustment). Thin wrapper over
    /// [`Self::composite_with_luts`] with an empty `adj_luts`.
    pub fn composite(
        &mut self,
        gpu: &GpuContext,
        ops: &[LayerOp],
        src: &impl LayerPixelProvider,
        canvas_w: u32,
        canvas_h: u32,
        region: Region,
    ) -> Result<(), LayerCompositeError> {
        self.composite_with_luts(gpu, ops, &[], src, canvas_w, canvas_h, region)
    }

    /// Composite an op-list, with `adj_luts` carrying the concatenated display-
    /// space transfer tables for any `ADJ_CURVES`(7)/`ADJ_LEVELS`(8) op (each
    /// such op's `params[0]` is its base float offset into `adj_luts`; Curves =
    /// 3×256 R/G/B, Levels = 1×256). Built CPU-side from `curves_display_luts`/
    /// `levels_display_lut` so the GPU reads the SAME table the CPU compositor
    /// does (parity gate). `adj_luts` may be empty when no such op is present.
    #[allow(clippy::too_many_arguments)] // gpu+ops+luts+src+dims+region are all intrinsic
    pub fn composite_with_luts(
        &mut self,
        gpu: &GpuContext,
        ops: &[LayerOp],
        adj_luts: &[f32],
        src: &impl LayerPixelProvider,
        canvas_w: u32,
        canvas_h: u32,
        region: Region,
    ) -> Result<(), LayerCompositeError> {
        if canvas_w == 0
            || canvas_h == 0
            || canvas_w > gpu.device.limits().max_texture_dimension_2d
            || canvas_h > gpu.device.limits().max_texture_dimension_2d
        {
            return Err(LayerCompositeError::InvalidCanvas {
                width: canvas_w,
                height: canvas_h,
            });
        }
        validate_op_list(ops)?;

        self.clock += 1;
        let epoch = self.clock;

        // Distinct layer keys referenced this frame, in first-seen order.
        let cap = self.cache_cap(canvas_w, canvas_h).max(1);
        self.ensure_array(gpu, canvas_w, canvas_h, ops, cap)?;

        // Resolve every referenced layer to a slice (uploading dirty pixels),
        // then flatten ops into the reusable GPU scratch.
        for op in ops {
            if let LayerOp::Layer { key, .. } = op {
                self.ensure_slice(gpu, *key, src, canvas_w, canvas_h, epoch, cap)?;
            }
        }
        let cache = &self.cache;
        flatten_layer_ops(
            ops,
            |k| cache.get(&k).map_or(0, |c| c.slice),
            &mut self.scratch_ops,
        );

        let region = region.clamped(canvas_w, canvas_h);
        if region.w == 0 || region.h == 0 {
            return Ok(()); // nothing to recomposite
        }
        self.ensure_out(gpu, region.w, region.h);
        self.upload_op_buffer(gpu);
        self.upload_adj_buffer(gpu);
        self.upload_adj_luts_buffer(gpu, adj_luts);

        // Spatial adjustment present → take the segmented pass-graph (materialise
        // → blur → combine → continue → encode). The single-pass path below stays
        // bit-identical for the common (no-spatial) case.
        if has_spatial(ops) {
            return self.composite_segmented(gpu, ops, canvas_w, canvas_h, region);
        }

        self.write_globals(gpu, canvas_w, canvas_h, region);
        let has_groups = ops.iter().any(|o| matches!(o, LayerOp::PushGroup));
        self.dispatch(gpu, region, has_groups);
        Ok(())
    }

    /// Read the output texture back to a region-sized straight-sRGB8 `Vec<u8>`.
    /// Test/verification path only (blocks on `device.poll`); the shell blits
    /// the texture directly. Returns `None` before the first composite.
    #[must_use]
    pub fn read_output(&self, gpu: &GpuContext) -> Option<Vec<u8>> {
        let out = self.out.as_ref()?;
        Some(readback_rgba8(gpu, &out.texture, out.width, out.height))
    }

    // ── internals ────────────────────────────────────────────────────────

    /// Ensure the texture array exists with the right dims + enough capacity
    /// for the distinct layers in `ops`.
    fn ensure_array(
        &mut self,
        gpu: &GpuContext,
        width: u32,
        height: u32,
        ops: &[LayerOp],
        cap: u32,
    ) -> Result<(), LayerCompositeError> {
        let distinct = distinct_layer_count(ops);
        if distinct > cap {
            return Err(LayerCompositeError::TooManyLayers {
                requested: distinct,
                cap,
            });
        }
        let needs_rebuild = match &self.array {
            Some(a) => a.width != width || a.height != height || a.capacity < distinct.max(1),
            None => true,
        };
        if !needs_rebuild {
            return Ok(());
        }
        // Grow capacity to the cap (bounded by distinct need) so subsequent
        // frames rarely rebuild; dims change → cache invalidated.
        let capacity = distinct
            .max(1)
            .max(self.array.as_ref().map_or(0, |a| a.capacity))
            .min(cap);
        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-render layer_composite array"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: capacity,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("ph2d-render layer_composite array view"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        self.array = Some(LayerArray {
            texture,
            view,
            width,
            height,
            capacity,
        });
        // Dims/capacity changed → existing slice assignments are invalid.
        self.cache.clear();
        Ok(())
    }

    /// Resolve `key` to a slice, uploading its pixels if newly assigned or its
    /// version changed. Evicts the LRU slice if the array is full.
    #[allow(clippy::too_many_arguments)]
    fn ensure_slice(
        &mut self,
        gpu: &GpuContext,
        key: u64,
        src: &impl LayerPixelProvider,
        width: u32,
        height: u32,
        epoch: u64,
        cap: u32,
    ) -> Result<(), LayerCompositeError> {
        let pixels = src
            .layer_pixels(key)
            .filter(|p| p.rgba8.len() == (width as usize) * (height as usize) * 4)
            .ok_or(LayerCompositeError::MissingOrMalformedLayer { key })?;

        if let Some(existing) = self.cache.get_mut(&key) {
            existing.last_used = epoch;
            if existing.version == pixels.version {
                return Ok(()); // clean — already resident
            }
            let slice = existing.slice;
            existing.version = pixels.version;
            self.upload_slice(gpu, slice, width, height, pixels.rgba8);
            return Ok(());
        }

        // New key — find a free slice or evict the LRU.
        let array_cap = self.array.as_ref().map_or(0, |a| a.capacity).min(cap);
        let slice = self.alloc_slice(array_cap, epoch)?;
        self.upload_slice(gpu, slice, width, height, pixels.rgba8);
        self.cache.insert(
            key,
            CachedSlice {
                slice,
                version: pixels.version,
                last_used: epoch,
            },
        );
        Ok(())
    }

    /// Pick a slice index for a new key: the first unused index, else evict the
    /// least-recently-used slice not touched this `epoch`.
    fn alloc_slice(&mut self, array_cap: u32, epoch: u64) -> Result<u32, LayerCompositeError> {
        // Allocation-free (HR-3): scan for the first slice no cached layer
        // holds. O(array_cap × cache_len), array_cap ≤ HARD_CAP_LAYERS, and
        // this runs only on a NEW-key resolution (not the steady-state path)
        // — cheaper than collecting a BTreeSet per new key (audit LOW).
        for s in 0..array_cap {
            if !self.cache.values().any(|c| c.slice == s) {
                return Ok(s);
            }
        }
        // Full — evict the LRU slice whose layer was not used this frame.
        let victim = self
            .cache
            .iter()
            .filter(|(_, c)| c.last_used < epoch)
            .min_by_key(|(_, c)| c.last_used)
            .map(|(k, c)| (*k, c.slice));
        match victim {
            Some((k, slice)) => {
                self.cache.remove(&k);
                Ok(slice)
            }
            // Every slice is live this frame and we still need one more — the
            // caller's distinct count exceeded capacity (should have been
            // caught by ensure_array, but guard defensively).
            None => Err(LayerCompositeError::TooManyLayers {
                requested: array_cap + 1,
                cap: array_cap,
            }),
        }
    }

    /// Upload one canvas-sized straight-sRGB8 layer into array `slice`.
    fn upload_slice(&self, gpu: &GpuContext, slice: u32, width: u32, height: u32, rgba8: &[u8]) {
        let Some(array) = &self.array else { return };
        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &array.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: slice,
                },
                aspect: wgpu::TextureAspect::All,
            },
            rgba8,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
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
            label: Some("ph2d-render layer_composite out"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
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

    /// (Re)upload the flattened op-list into the persistent storage buffer,
    /// growing it only when the op count exceeds the current capacity.
    fn upload_op_buffer(&mut self, gpu: &GpuContext) {
        let bytes: &[u8] = bytemuck::cast_slice(&self.scratch_ops.ops);
        let needed = bytes.len().max(core::mem::size_of::<GpuOp>()) as u64;
        let grow = match &self.op_buffer {
            Some((_, cap)) => *cap < needed,
            None => true,
        };
        if grow {
            let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-render layer_composite ops"),
                size: needed,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.op_buffer = Some((buffer, needed));
        }
        if let Some((buffer, _)) = &self.op_buffer {
            gpu.queue.write_buffer(buffer, 0, bytes);
        }
    }

    /// Upload the flattened adjustment params (binding 5). Always writes ≥1
    /// element (a zero-filled dummy when there are no adjustments) so the storage
    /// binding is never zero-sized. Grows like the op buffer (HR-3: no realloc
    /// once warm).
    fn upload_adj_buffer(&mut self, gpu: &GpuContext) {
        let one = core::mem::size_of::<AdjParamsGpu>() as u64;
        let dummy = [AdjParamsGpu {
            kind: 0,
            p0: 0.0,
            p1: 0.0,
            p2: 0.0,
        }];
        let src: &[AdjParamsGpu] = if self.scratch_ops.adj.is_empty() {
            &dummy
        } else {
            &self.scratch_ops.adj
        };
        let bytes: &[u8] = bytemuck::cast_slice(src);
        let needed = (bytes.len() as u64).max(one);
        let grow = match &self.adj_params_buffer {
            Some((_, cap)) => *cap < needed,
            None => true,
        };
        if grow {
            let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-render layer_composite adj params"),
                size: needed,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.adj_params_buffer = Some((buffer, needed));
        }
        if let Some((buffer, _)) = &self.adj_params_buffer {
            gpu.queue.write_buffer(buffer, 0, bytes);
        }
    }

    /// Upload the concatenated display-space transfer LUTs (binding 6). Writes
    /// ≥1 f32 (a single `0.0` when there are no Curves/Levels ops) so the storage
    /// binding is never zero-sized. Grows like the other buffers (HR-3: no
    /// realloc once warm).
    fn upload_adj_luts_buffer(&mut self, gpu: &GpuContext, adj_luts: &[f32]) {
        let dummy = [0.0f32];
        let src: &[f32] = if adj_luts.is_empty() {
            &dummy
        } else {
            adj_luts
        };
        let bytes: &[u8] = bytemuck::cast_slice(src);
        let needed = (bytes.len() as u64).max(4);
        let grow = match &self.adj_luts_buffer {
            Some((_, cap)) => *cap < needed,
            None => true,
        };
        if grow {
            let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-render layer_composite adj luts"),
                size: needed,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.adj_luts_buffer = Some((buffer, needed));
        }
        if let Some((buffer, _)) = &self.adj_luts_buffer {
            gpu.queue.write_buffer(buffer, 0, bytes);
        }
    }

    fn write_globals(&self, gpu: &GpuContext, canvas_w: u32, canvas_h: u32, region: Region) {
        let g = GpuGlobals {
            canvas_width: canvas_w,
            canvas_height: canvas_h,
            region_x: region.x,
            region_y: region.y,
            region_w: region.w,
            region_h: region.h,
            op_count: self.scratch_ops.len() as u32,
            _pad: 0,
        };
        gpu.queue
            .write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&g));
    }

    fn dispatch(&self, gpu: &GpuContext, region: Region, has_groups: bool) {
        let (
            Some(array),
            Some(out),
            Some((op_buffer, _)),
            Some((adj_buffer, _)),
            Some((luts_buffer, _)),
        ) = (
            &self.array,
            &self.out,
            &self.op_buffer,
            &self.adj_params_buffer,
            &self.adj_luts_buffer,
        )
        else {
            return;
        };
        let pipeline = if has_groups {
            &self.pipeline_grouped
        } else {
            &self.pipeline_flat
        };
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render layer_composite bg"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: op_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&array.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&out.view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.srgb_lut_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: adj_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: luts_buffer.as_entire_binding(),
                },
            ],
        });
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render layer_composite encoder"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ph2d-render layer_composite pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(
                region.w.div_ceil(WORKGROUP_EDGE),
                region.h.div_ceil(WORKGROUP_EDGE),
                1,
            );
        }
        gpu.queue.submit([encoder.finish()]);
    }

    // ── Segmented spatial pass-graph (W4) ────────────────────────────────────

    /// Composite an op-list containing ≥1 root-level spatial adjustment via the
    /// pass-graph: split the op-list at each depth-0 spatial op into segments;
    /// materialise the below-composite into a linear intermediate, run the
    /// separable kernel through the ping-pong pair, blend back, continue the
    /// layers above, and finally encode the requested region to straight sRGB8.
    /// All intermediates are linear `Rgba32Float`; the only sRGB encode is the
    /// final `cs_encode`. Each pass operates over `work_region` = the requested
    /// dirty rect dilated by the total blur halo so the kernel has valid
    /// neighbours up to the region edge.
    ///
    /// Op/adj/luts buffers + the layer array are already uploaded by the caller;
    /// `out` is sized to `region`.
    fn composite_segmented(
        &mut self,
        gpu: &GpuContext,
        ops: &[LayerOp],
        canvas_w: u32,
        canvas_h: u32,
        region: Region,
    ) -> Result<(), LayerCompositeError> {
        /// One root-level spatial pass break: where it sits in the op-list, its
        /// (provisional) blur kernel (weights + half + separable-vs-directional),
        /// the combine mode + amount, and its blend/opacity.
        struct Break {
            idx: usize,
            weights: Vec<f32>,
            half: u32,
            directional: bool,
            dir: [f32; 2],
            combine_mode: u32,
            amount: f32,
            blend: u8,
            opacity: f32,
        }

        /// The blur kernel + combine a spatial op resolves to.
        struct KernelPlan {
            weights: Vec<f32>,
            half: u32,
            directional: bool,
            dir: [f32; 2],
            combine_mode: u32,
            amount: f32,
        }

        // Resolve a spatial kernel to its blur kernel + combine. GAUSSIAN:
        // params[0]=radius, separable, passthrough. SHARPEN: params[0]=amount,
        // params[1]=blur radius, separable, unsharp combine. MOTION:
        // params[0]=distance, params[1]=angle (rad), directional box, passthrough.
        fn resolve_kernel(kernel: u8, params: &[f32; 4]) -> Option<KernelPlan> {
            match kernel {
                k if k == SPATIAL_GAUSSIAN => {
                    let (weights, half) = gaussian_weights(params[0]);
                    Some(KernelPlan {
                        weights,
                        half,
                        directional: false,
                        dir: [0.0, 0.0],
                        combine_mode: COMBINE_GAUSSIAN,
                        amount: 0.0,
                    })
                }
                k if k == SPATIAL_SHARPEN => {
                    let (weights, half) = gaussian_weights(params[1]);
                    Some(KernelPlan {
                        weights,
                        half,
                        directional: false,
                        dir: [0.0, 0.0],
                        combine_mode: COMBINE_SHARPEN,
                        amount: params[0],
                    })
                }
                k if k == SPATIAL_MOTION => {
                    let (weights, half) = motion_weights(params[0]);
                    let angle = params[1];
                    Some(KernelPlan {
                        weights,
                        half,
                        directional: true,
                        // Direction computed CPU-side so the GPU does no sin/cos
                        // (no transcendental parity drift).
                        dir: [angle.cos(), angle.sin()],
                        combine_mode: COMBINE_GAUSSIAN,
                        amount: 0.0,
                    })
                }
                _ => None,
            }
        }

        let mut breaks: Vec<Break> = Vec::new();
        let mut depth: i32 = 0;
        let mut total_halo: u32 = 0;
        for (i, op) in ops.iter().enumerate() {
            match op {
                LayerOp::PushGroup => depth += 1,
                LayerOp::PopGroup { .. } => depth -= 1,
                LayerOp::SpatialAdjustment {
                    kernel,
                    params,
                    blend_mode,
                    opacity,
                } if depth == 0 => {
                    let Some(plan) = resolve_kernel(*kernel, params) else {
                        continue; // unknown kernel → identity (segment loop no-ops it)
                    };
                    total_halo = total_halo.saturating_add(plan.half);
                    breaks.push(Break {
                        idx: i,
                        weights: plan.weights,
                        half: plan.half,
                        directional: plan.directional,
                        dir: plan.dir,
                        combine_mode: plan.combine_mode,
                        amount: plan.amount,
                        blend: *blend_mode,
                        opacity: *opacity,
                    });
                }
                _ => {}
            }
        }

        // No effective break (every spatial op is nested in a group or an
        // unknown kernel) → fall back to the single pass; the segment loops /
        // `cs_flat` no-op the `OP_SPATIAL` placeholders, so the effect is simply
        // skipped rather than corrupting the composite (documented limitation:
        // spatial-inside-group is a follow-up).
        if breaks.is_empty() {
            self.write_globals(gpu, canvas_w, canvas_h, region);
            let has_groups = ops.iter().any(|o| matches!(o, LayerOp::PushGroup));
            self.dispatch(gpu, region, has_groups);
            return Ok(());
        }

        // work_region = region dilated by the total halo, clamped to the canvas.
        let x0 = region.x.saturating_sub(total_halo);
        let y0 = region.y.saturating_sub(total_halo);
        let x1 = (region.x + region.w)
            .saturating_add(total_halo)
            .min(canvas_w);
        let y1 = (region.y + region.h)
            .saturating_add(total_halo)
            .min(canvas_h);
        let work = Region {
            x: x0,
            y: y0,
            w: x1 - x0,
            h: y1 - y0,
        };
        self.ensure_work_textures(gpu, work.w, work.h);

        // Running base alternates between base[0]/base[1] across segments +
        // combines; the blur ping-pong uses blur[0]/blur[1]. No pass ever reads
        // and writes the same texture.
        let mut cur: usize = 0;
        let mut seg_start: u32 = 0;
        let mut from_base = false;
        for b in &breaks {
            let dst = if from_base { cur ^ 1 } else { cur };
            self.run_segment(
                gpu,
                seg_start,
                b.idx as u32,
                from_base,
                cur,
                dst,
                work,
                canvas_w,
                canvas_h,
            );
            cur = dst;
            self.upload_blur_weights(gpu, &b.weights);
            self.run_blur(gpu, cur, b.half, b.directional, b.dir, work);
            self.run_combine(
                gpu,
                cur,
                cur ^ 1,
                b.blend,
                b.opacity,
                b.combine_mode,
                b.amount,
                work,
            );
            cur ^= 1;
            seg_start = b.idx as u32 + 1;
            from_base = true;
        }
        // Layers above the last spatial break (if any).
        if (seg_start as usize) < ops.len() {
            self.run_segment(
                gpu,
                seg_start,
                ops.len() as u32,
                true,
                cur,
                cur ^ 1,
                work,
                canvas_w,
                canvas_h,
            );
            cur ^= 1;
        }
        self.run_encode(gpu, cur, region, work);
        Ok(())
    }

    /// Ensure the linear work intermediates are at least `w × h` (grow-only —
    /// passes are bounded by `work_region`, so a larger texture just has an
    /// unused border; this avoids reallocating when the dirty rect shrinks).
    fn ensure_work_textures(&mut self, gpu: &GpuContext, w: u32, h: u32) {
        if let Some(work) = &self.work
            && work.width >= w
            && work.height >= h
        {
            return;
        }
        let nw = w.max(self.work.as_ref().map_or(0, |t| t.width)).max(1);
        let nh = h.max(self.work.as_ref().map_or(0, |t| t.height)).max(1);
        let make = |label: &str| {
            let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: nw,
                    height: nh,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            WorkTex { texture, view }
        };
        self.work = Some(WorkTextures {
            width: nw,
            height: nh,
            base: [
                make("ph2d-render layer_composite work base0"),
                make("ph2d-render layer_composite work base1"),
            ],
            blur: [
                make("ph2d-render layer_composite work blur0"),
                make("ph2d-render layer_composite work blur1"),
            ],
        });
    }

    /// Upload the separable Gaussian weights (binding 13). Grows like the other
    /// persistent buffers; writes ≥1 f32 so the storage binding is never empty.
    fn upload_blur_weights(&mut self, gpu: &GpuContext, weights: &[f32]) {
        let dummy = [0.0f32];
        let src: &[f32] = if weights.is_empty() { &dummy } else { weights };
        let bytes: &[u8] = bytemuck::cast_slice(src);
        let needed = (bytes.len() as u64).max(4);
        let grow = match &self.blur_weights_buffer {
            Some((_, cap)) => *cap < needed,
            None => true,
        };
        if grow {
            let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-render layer_composite blur weights"),
                size: needed,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.blur_weights_buffer = Some((buffer, needed));
        }
        if let Some((buffer, _)) = &self.blur_weights_buffer {
            gpu.queue.write_buffer(buffer, 0, bytes);
        }
    }

    /// Run one compute pass (encoder → pass → submit) over a `w × h` grid.
    fn dispatch_pass(
        &self,
        gpu: &GpuContext,
        pipeline: &wgpu::ComputePipeline,
        bind_group: &wgpu::BindGroup,
        w: u32,
        h: u32,
        label: &str,
    ) {
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(label),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, bind_group, &[]);
            pass.dispatch_workgroups(w.div_ceil(WORKGROUP_EDGE), h.div_ceil(WORKGROUP_EDGE), 1);
        }
        gpu.queue.submit([encoder.finish()]);
    }

    /// Composite `ops[op_start..op_end]` over `work` into `base[dst_idx]`,
    /// starting from `base[src_idx]` when `from_base` (else from zero).
    #[allow(clippy::too_many_arguments)]
    fn run_segment(
        &self,
        gpu: &GpuContext,
        op_start: u32,
        op_end: u32,
        from_base: bool,
        src_idx: usize,
        dst_idx: usize,
        work: Region,
        canvas_w: u32,
        canvas_h: u32,
    ) {
        let (
            Some(work_tex),
            Some(array),
            Some((op_buffer, _)),
            Some((adj_buffer, _)),
            Some((luts_buffer, _)),
        ) = (
            &self.work,
            &self.array,
            &self.op_buffer,
            &self.adj_params_buffer,
            &self.adj_luts_buffer,
        )
        else {
            return;
        };
        let g = SegGlobals {
            canvas_width: canvas_w,
            canvas_height: canvas_h,
            region_x: work.x,
            region_y: work.y,
            region_w: work.w,
            region_h: work.h,
            op_start,
            op_end,
            seg_from_base: u32::from(from_base),
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
        };
        gpu.queue
            .write_buffer(&self.seg_globals_buffer, 0, bytemuck::bytes_of(&g));
        let base_in_view = if from_base {
            &work_tex.base[src_idx].view
        } else {
            &self.seg_base_dummy
        };
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render layer_composite seg bg"),
            layout: &self.bgl_segment,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: op_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&array.view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.srgb_lut_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: adj_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: luts_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: self.seg_globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(&work_tex.base[dst_idx].view),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::TextureView(base_in_view),
                },
            ],
        });
        self.dispatch_pass(
            gpu,
            &self.pipeline_segment,
            &bind_group,
            work.w,
            work.h,
            "ph2d-render layer_composite seg pass",
        );
    }

    /// Blur `base[base_idx]` into `blur[1]` (the combine reads `blur[1]`).
    /// Separable: H into `blur[0]` then V into `blur[1]` (2 passes). Directional
    /// (motion): one `cs_blur_dir` pass along `dir`. Weights already uploaded.
    fn run_blur(
        &self,
        gpu: &GpuContext,
        base_idx: usize,
        half: u32,
        directional: bool,
        dir: [f32; 2],
        work: Region,
    ) {
        let (Some(work_tex), Some((weights_buffer, _))) = (&self.work, &self.blur_weights_buffer)
        else {
            return;
        };
        let g = BlurGlobals {
            width: work.w,
            height: work.h,
            half,
            _pad0: 0,
            dir_x: dir[0],
            dir_y: dir[1],
            _pad1: 0.0,
            _pad2: 0.0,
        };
        gpu.queue
            .write_buffer(&self.blur_globals_buffer, 0, bytemuck::bytes_of(&g));
        let blur_bg = |src: &wgpu::TextureView, dst: &wgpu::TextureView| {
            gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-render layer_composite blur bg"),
                layout: &self.bgl_blur,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 10,
                        resource: self.blur_globals_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 11,
                        resource: wgpu::BindingResource::TextureView(src),
                    },
                    wgpu::BindGroupEntry {
                        binding: 12,
                        resource: wgpu::BindingResource::TextureView(dst),
                    },
                    wgpu::BindGroupEntry {
                        binding: 13,
                        resource: weights_buffer.as_entire_binding(),
                    },
                ],
            })
        };
        if directional {
            // One 1-D pass straight into blur[1] (no separability for an
            // arbitrary direction).
            let bg = blur_bg(&work_tex.base[base_idx].view, &work_tex.blur[1].view);
            self.dispatch_pass(
                gpu,
                &self.pipeline_blur_dir,
                &bg,
                work.w,
                work.h,
                "ph2d-render layer_composite blur_dir pass",
            );
            return;
        }
        let bg_h = blur_bg(&work_tex.base[base_idx].view, &work_tex.blur[0].view);
        self.dispatch_pass(
            gpu,
            &self.pipeline_blur_h,
            &bg_h,
            work.w,
            work.h,
            "ph2d-render layer_composite blur_h pass",
        );
        let bg_v = blur_bg(&work_tex.blur[0].view, &work_tex.blur[1].view);
        self.dispatch_pass(
            gpu,
            &self.pipeline_blur_v,
            &bg_v,
            work.w,
            work.h,
            "ph2d-render layer_composite blur_v pass",
        );
    }

    /// Derive the kernel result from `base[base_idx]` + `blur[1]` (per
    /// `combine_mode`/`amount`) and blend it over `base[base_idx]` by
    /// `blend`/`opacity` into `base[dst_idx]`.
    #[allow(clippy::too_many_arguments)]
    fn run_combine(
        &self,
        gpu: &GpuContext,
        base_idx: usize,
        dst_idx: usize,
        blend: u8,
        opacity: f32,
        combine_mode: u32,
        amount: f32,
        work: Region,
    ) {
        let Some(work_tex) = &self.work else {
            return;
        };
        let g = CombineGlobals {
            width: work.w,
            height: work.h,
            blend_mode: u32::from(blend),
            combine_mode,
            opacity,
            amount,
            _pad2: 0.0,
            _pad3: 0.0,
        };
        gpu.queue
            .write_buffer(&self.combine_globals_buffer, 0, bytemuck::bytes_of(&g));
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render layer_composite combine bg"),
            layout: &self.bgl_combine,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 14,
                    resource: self.combine_globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 15,
                    resource: wgpu::BindingResource::TextureView(&work_tex.base[base_idx].view),
                },
                wgpu::BindGroupEntry {
                    binding: 16,
                    resource: wgpu::BindingResource::TextureView(&work_tex.blur[1].view),
                },
                wgpu::BindGroupEntry {
                    binding: 17,
                    resource: wgpu::BindingResource::TextureView(&work_tex.base[dst_idx].view),
                },
            ],
        });
        self.dispatch_pass(
            gpu,
            &self.pipeline_combine,
            &bind_group,
            work.w,
            work.h,
            "ph2d-render layer_composite combine pass",
        );
    }

    /// Encode `base[base_idx]` (linear, work_region-sized) cropped to `region`
    /// into the straight-sRGB8 `out` texture.
    fn run_encode(&self, gpu: &GpuContext, base_idx: usize, region: Region, work: Region) {
        let (Some(work_tex), Some(out)) = (&self.work, &self.out) else {
            return;
        };
        let g = EncodeGlobals {
            out_w: region.w,
            out_h: region.h,
            src_off_x: region.x - work.x,
            src_off_y: region.y - work.y,
        };
        gpu.queue
            .write_buffer(&self.encode_globals_buffer, 0, bytemuck::bytes_of(&g));
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render layer_composite encode bg"),
            layout: &self.bgl_encode,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 18,
                    resource: self.encode_globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 19,
                    resource: wgpu::BindingResource::TextureView(&work_tex.base[base_idx].view),
                },
                wgpu::BindGroupEntry {
                    binding: 20,
                    resource: wgpu::BindingResource::TextureView(&out.view),
                },
            ],
        });
        self.dispatch_pass(
            gpu,
            &self.pipeline_encode,
            &bind_group,
            region.w,
            region.h,
            "ph2d-render layer_composite encode pass",
        );
    }
}
