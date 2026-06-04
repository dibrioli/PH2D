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
    pub fn composite(
        &mut self,
        gpu: &GpuContext,
        ops: &[LayerOp],
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
        let (Some(array), Some(out), Some((op_buffer, _)), Some((adj_buffer, _))) = (
            &self.array,
            &self.out,
            &self.op_buffer,
            &self.adj_params_buffer,
        ) else {
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
}
