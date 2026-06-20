use super::super::*;

impl LayerCompositor {
    // ── internals ────────────────────────────────────────────────────────

    /// Ensure the texture array exists with the right dims + enough capacity
    /// for the distinct layers in `ops`.
    pub(super) fn ensure_array(
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
    pub(super) fn ensure_slice(
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
    pub(super) fn alloc_slice(
        &mut self,
        array_cap: u32,
        epoch: u64,
    ) -> Result<u32, LayerCompositeError> {
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
    pub(super) fn upload_slice(
        &self,
        gpu: &GpuContext,
        slice: u32,
        width: u32,
        height: u32,
        rgba8: &[u8],
    ) {
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
    pub(super) fn ensure_out(&mut self, gpu: &GpuContext, w: u32, h: u32) {
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
    pub(super) fn upload_op_buffer(&mut self, gpu: &GpuContext) {
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
    pub(super) fn upload_adj_buffer(&mut self, gpu: &GpuContext) {
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
    pub(super) fn upload_adj_luts_buffer(&mut self, gpu: &GpuContext, adj_luts: &[f32]) {
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

    pub(super) fn write_globals(
        &self,
        gpu: &GpuContext,
        canvas_w: u32,
        canvas_h: u32,
        region: Region,
        out_canvas_coords: bool,
    ) {
        let g = GpuGlobals {
            canvas_width: canvas_w,
            canvas_height: canvas_h,
            region_x: region.x,
            region_y: region.y,
            region_w: region.w,
            region_h: region.h,
            op_count: self.scratch_ops.len() as u32,
            out_canvas_coords: u32::from(out_canvas_coords),
        };
        gpu.queue
            .write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&g));
    }
}
