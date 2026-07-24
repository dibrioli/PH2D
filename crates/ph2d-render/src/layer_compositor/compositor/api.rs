use super::super::*;

impl LayerCompositor {
    /// Make every key `ops` references resident: layer pixels **and** masks.
    ///
    /// The two are resolved by the same `ensure_slice` and live in the same
    /// texture array — a mask is a layer key like any other — but they differ in
    /// how a failure is handled, and that difference is the whole reason this is
    /// a function instead of a loop written twice:
    ///
    /// * a **layer** the provider cannot serve is an error. The op-list named a
    ///   layer that does not exist; compositing it would silently draw slice 0.
    /// * a **mask** it cannot serve is **not** an error — it degrades to *no
    ///   mask*, which is exactly what the CPU reference does (`mrgba.len() >= …`
    ///   falls through to "fully visible"). The `flatten`'s `mask_slot_of` then
    ///   finds no cache entry and emits `NO_MASK_SLOT`.
    ///
    /// Getting that backwards would hand a whole document to the other producer
    /// over one malformed mask buffer.
    // gpu + ops + src + dims + epoch + cap are all intrinsic to "make these keys
    // resident"; bundling them would be a struct that exists only to satisfy a lint.
    #[allow(clippy::too_many_arguments)]
    fn resolve_keys(
        &mut self,
        gpu: &GpuContext,
        ops: &[LayerOp],
        src: &impl LayerPixelProvider,
        canvas_w: u32,
        canvas_h: u32,
        epoch: u64,
        cap: u32,
    ) -> Result<(), LayerCompositeError> {
        for op in ops {
            if let LayerOp::Layer { key, .. } = op {
                self.ensure_slice(gpu, *key, src, canvas_w, canvas_h, epoch, cap)?;
            }
            if let Some(m) = op_mask(op) {
                let _ = self.ensure_slice(gpu, m.key, src, canvas_w, canvas_h, epoch, cap);
            }
        }
        Ok(())
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
        self.resolve_keys(gpu, ops, src, canvas_w, canvas_h, epoch, cap)?;
        let cache = &self.cache;
        flatten_layer_ops(
            ops,
            |k| cache.get(&k).map_or(0, |c| c.slice),
            |k| cache.get(&k).map(|c| c.slice),
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

        self.write_globals(gpu, canvas_w, canvas_h, region, false);
        let has_groups = ops.iter().any(|o| matches!(o, LayerOp::PushGroup));
        self.dispatch(gpu, region, has_groups);
        Ok(())
    }

    /// **E5 (ADR-0078 S2 perf): region-scoped live-stroke recomposite into a
    /// CANVAS-SIZED, PERSISTENT output.** Same composite as [`Self::composite_with_luts`]
    /// but writes at CANVAS coords (not region-local), so a dispatch over `region`
    /// refreshes ONLY that dirty rect and leaves the rest of `out` intact from the
    /// prior frame. This converts the per-frame cost of a watercolor stroke's layer
    /// recomposite from `O(canvas × layers)` to `O(wet-envelope × layers)` — the 4K
    /// multi-layer FPS fix — while the full-canvas `out` stays valid for a
    /// full-canvas premul + a region slot copy.
    ///
    /// **Contract the caller MUST honour:** seed a FULL composite (`region ==
    /// Region::full`) on the FIRST frame of a stroke so the persistent `out` holds a
    /// valid backdrop everywhere; pass the growing (monotonic) wet envelope on the
    /// following frames. Because `out` persists across calls, out-of-region texels
    /// are whatever the last call wrote there — correct only under that monotonic
    /// discipline. A root spatial adjustment falls back to a full composite (the
    /// segmented pass-graph has no region-into-canvas mode).
    #[allow(clippy::too_many_arguments)] // gpu+ops+luts+src+dims+region are all intrinsic
    pub fn composite_region_into_canvas(
        &mut self,
        gpu: &GpuContext,
        ops: &[LayerOp],
        adj_luts: &[f32],
        src: &impl LayerPixelProvider,
        canvas_w: u32,
        canvas_h: u32,
        region: Region,
    ) -> Result<(), LayerCompositeError> {
        // Spatial stacks: the segmented pass-graph isn't region-into-canvas, but a
        // full composite over the whole canvas writes the (canvas-sized) `out` at
        // canvas coords anyway (region origin 0 ⇒ region-local == canvas). Correct,
        // just not region-scoped — spatial adjustments mid-stroke are uncommon.
        if has_spatial(ops) {
            return self.composite_with_luts(
                gpu,
                ops,
                adj_luts,
                src,
                canvas_w,
                canvas_h,
                Region::full(canvas_w, canvas_h),
            );
        }
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
        let cap = self.cache_cap(canvas_w, canvas_h).max(1);
        self.ensure_array(gpu, canvas_w, canvas_h, ops, cap)?;
        self.resolve_keys(gpu, ops, src, canvas_w, canvas_h, epoch, cap)?;
        let cache = &self.cache;
        flatten_layer_ops(
            ops,
            |k| cache.get(&k).map_or(0, |c| c.slice),
            |k| cache.get(&k).map(|c| c.slice),
            &mut self.scratch_ops,
        );

        let region = region.clamped(canvas_w, canvas_h);
        if region.w == 0 || region.h == 0 {
            return Ok(()); // nothing to recomposite
        }
        // PERSISTENT full-canvas out (no per-frame resize churn as the envelope grows).
        self.ensure_out(gpu, canvas_w, canvas_h);
        self.upload_op_buffer(gpu);
        self.upload_adj_buffer(gpu);
        self.upload_adj_luts_buffer(gpu, adj_luts);
        self.write_globals(gpu, canvas_w, canvas_h, region, true);
        let has_groups = ops.iter().any(|o| matches!(o, LayerOp::PushGroup));
        self.dispatch(gpu, region, has_groups);
        Ok(())
    }

    /// **Inject an externally-rendered STRAIGHT-sRGB8 texture into the cached
    /// slice for layer `key` — GPU→GPU, zero CPU bytes** (Painter fluid E5,
    /// ADR-0078 S2: the live wet-field composite feeds the layer chain
    /// mid-stroke without the per-frame readback→re-upload round-trip).
    ///
    /// `ops` is the SAME op-list the subsequent [`Self::composite`] will use:
    /// it sizes the texture array exactly like the composite (so the injection
    /// never triggers a later capacity rebuild that would discard the slice).
    /// `src` must be a `≥ width × height` texture whose format is copy-
    /// compatible with the array's `Rgba8Unorm` (same format family modulo the
    /// sRGB suffix) and carry `COPY_SRC`.
    ///
    /// ## Version invariant (inject vs CPU upload)
    ///
    /// The slice cache re-uploads from the [`LayerPixelProvider`] iff the
    /// provider's version **differs** from the cached one, and the tool's pixel
    /// versions are monotonic. Callers MUST pass `version` = the provider's
    /// CURRENT version for `key` (NOT bumped):
    /// - a provider pass while the CPU pixels are intentionally stale
    ///   (mid-stroke — same version) does NOT clobber the injection;
    /// - the first real pixel change (e.g. the pointer-up readback applying
    ///   bands bumps the version) makes the provider version differ → the CPU
    ///   upload wins, retiring the injected content exactly when `canvas_rgba`
    ///   has caught up.
    ///
    /// `region` (`x, y, w, h`, canvas coords) scopes the GPU→GPU copy to the wet
    /// envelope: only that sub-rect of `src` is copied into the slice (at the same
    /// canvas offset), leaving the rest of the slice from the previous frame /
    /// `ensure_slice` upload intact. Pass the FULL canvas rect to copy everything.
    /// Because the wet envelope grows monotonically and `src` outside it holds the
    /// straight backdrop (= the slice's pre-stroke base), a rect copy is exact.
    #[allow(clippy::too_many_arguments)] // gpu+ops+key+src+dims+region+version are all intrinsic
    pub fn inject_slice_from_texture(
        &mut self,
        gpu: &GpuContext,
        ops: &[LayerOp],
        key: u64,
        src: &wgpu::Texture,
        width: u32,
        height: u32,
        region: (u32, u32, u32, u32),
        version: u64,
    ) -> Result<(), LayerCompositeError> {
        if width == 0
            || height == 0
            || width > gpu.device.limits().max_texture_dimension_2d
            || height > gpu.device.limits().max_texture_dimension_2d
        {
            return Err(LayerCompositeError::InvalidCanvas { width, height });
        }
        validate_op_list(ops)?;
        // Clamp the copy rect to the canvas (defensive — a stale envelope must not
        // read/write past the textures); an empty rect after clamping is a no-op.
        let (rx, ry, rx_hi, ry_hi) = (
            region.0.min(width),
            region.1.min(height),
            (region.0 + region.2).min(width),
            (region.1 + region.3).min(height),
        );
        if rx_hi <= rx || ry_hi <= ry {
            return Ok(());
        }
        let (rw, rh) = (rx_hi - rx, ry_hi - ry);
        // Copy-compatibility: `copy_texture_to_texture` requires the same format
        // modulo the sRGB suffix (the trick `individual.rs` relies on), and the
        // source must cover the canvas.
        if src.format().remove_srgb_suffix() != wgpu::TextureFormat::Rgba8Unorm
            || src.width() < width
            || src.height() < height
        {
            return Err(LayerCompositeError::MissingOrMalformedLayer { key });
        }
        self.clock += 1;
        let epoch = self.clock;
        let cap = self.cache_cap(width, height).max(1);
        // Size the array for the REAL op-list (not just this key) so the
        // following composite never rebuilds (a rebuild clears the cache,
        // including this injection).
        self.ensure_array(gpu, width, height, ops, cap)?;
        let slice = match self.cache.get_mut(&key) {
            Some(existing) => {
                existing.last_used = epoch;
                existing.version = version;
                existing.slice
            }
            None => {
                let array_cap = self.array.as_ref().map_or(0, |a| a.capacity).min(cap);
                let slice = self.alloc_slice(array_cap, epoch)?;
                self.cache.insert(
                    key,
                    CachedSlice {
                        slice,
                        version,
                        last_used: epoch,
                    },
                );
                slice
            }
        };
        let Some(array) = &self.array else {
            // Unreachable after ensure_array, but fail honestly rather than panic.
            return Err(LayerCompositeError::MissingOrMalformedLayer { key });
        };
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render layer_composite inject slice"),
            });
        enc.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: src,
                mip_level: 0,
                origin: wgpu::Origin3d { x: rx, y: ry, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &array.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: rx,
                    y: ry,
                    z: slice,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: rw,
                height: rh,
                depth_or_array_layers: 1,
            },
        );
        gpu.queue.submit([enc.finish()]);
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
}
