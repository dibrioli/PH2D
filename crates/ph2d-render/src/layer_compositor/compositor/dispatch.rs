use super::super::*;
use super::ShTonal;

impl LayerCompositor {
    pub(super) fn dispatch(&self, gpu: &GpuContext, region: Region, has_groups: bool) {
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
                timestamp_writes: ph2d_gpu::pass_profiler::compute_writes("render.layer_comp"),
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
    pub(super) fn composite_segmented(
        &mut self,
        gpu: &GpuContext,
        ops: &[LayerOp],
        canvas_w: u32,
        canvas_h: u32,
        region: Region,
    ) -> Result<(), LayerCompositeError> {
        /// How a spatial op's blur stage produces the adjusted texture (into
        /// `blur[1]`, which the combine reads).
        enum BlurStage {
            /// Separable box: `cs_blur_h` then `cs_blur_v` (uses `weights`/`half`).
            Separable,
            /// Directional motion blur: one `cs_blur_dir` pass along `dir`.
            Directional([f32; 2]),
            /// Chromatic-aberration gather: `cs_chroma`, per-channel shift `[r,g,b]`
            /// (px at the canvas corner). No weights.
            Chroma([f32; 3]),
            /// Bloom: `cs_bloom_bright` (threshold, falloff) extracts the premultiplied
            /// glow; it is downsampled by `factor`, blurred at LOW res (`weights`/
            /// `low_half`, the bounded kernel), then bilinear-upsampled — so the blur
            /// is radius-independent (O(1)). `COMBINE_BLOOM` adds `intensity·glow` back.
            Bloom {
                threshold: f32,
                falloff: f32,
                factor: u32,
                low_half: u32,
            },
            /// Shadows/Highlights: `cs_sh_luma` extracts the display luma, two scalar
            /// blurs (shadows / highlights radii) build local tone maps, and
            /// `cs_combine_sh` applies the tonal correction (its OWN combine — the
            /// shared `run_combine` is skipped). Carries the 6 tonal scalars + the
            /// two radii's weights.
            Sh {
                lo_weights: Vec<f32>,
                lo_half: u32,
                hi_weights: Vec<f32>,
                hi_half: u32,
                tonal: ShTonal,
            },
        }

        /// One root-level spatial pass break: where it sits in the op-list, its
        /// (provisional) blur stage (+ weights/half), the combine mode + amount,
        /// and its blend/opacity.
        struct Break {
            idx: usize,
            weights: Vec<f32>,
            half: u32,
            stage: BlurStage,
            combine_mode: u32,
            amount: f32,
            blend: u8,
            opacity: f32,
        }

        /// The blur stage + combine a spatial op resolves to.
        struct KernelPlan {
            weights: Vec<f32>,
            half: u32,
            stage: BlurStage,
            combine_mode: u32,
            amount: f32,
        }

        // Resolve a spatial kernel to its blur stage + combine. GAUSSIAN:
        // params[0]=radius, separable, passthrough. SHARPEN: params[0]=amount,
        // params[1]=blur radius, separable, unsharp combine. MOTION:
        // params[0]=distance, params[1]=angle (rad), directional box, passthrough.
        // CHROMA: params[0..3]=R/G/B shift, directional gather, passthrough.
        fn resolve_kernel(kernel: u8, params: &[f32; 8]) -> Option<KernelPlan> {
            match kernel {
                k if k == SPATIAL_GAUSSIAN => {
                    let (weights, half) = gaussian_weights(params[0]);
                    Some(KernelPlan {
                        weights,
                        half,
                        stage: BlurStage::Separable,
                        combine_mode: COMBINE_GAUSSIAN,
                        amount: 0.0,
                    })
                }
                k if k == SPATIAL_SHARPEN => {
                    let (weights, half) = gaussian_weights(params[1]);
                    Some(KernelPlan {
                        weights,
                        half,
                        stage: BlurStage::Separable,
                        combine_mode: COMBINE_SHARPEN,
                        amount: params[0],
                    })
                }
                k if k == SPATIAL_MOTION => {
                    let (weights, half) = motion_weights(params[0]);
                    let angle = params[1];
                    // Direction computed CPU-side so the GPU does no sin/cos.
                    let stage = BlurStage::Directional([angle.cos(), angle.sin()]);
                    Some(KernelPlan {
                        weights,
                        half,
                        stage,
                        combine_mode: COMBINE_GAUSSIAN,
                        amount: 0.0,
                    })
                }
                k if k == SPATIAL_CHROMA => {
                    let shifts = [params[0], params[1], params[2]];
                    // Halo = the largest per-channel shift (px at the corner).
                    let max_shift = shifts.iter().fold(0.0f32, |m, s| m.max(s.abs()));
                    let half = (max_shift.ceil() as u32).clamp(1, MAX_BLUR_HALF);
                    Some(KernelPlan {
                        weights: Vec::new(),
                        half,
                        stage: BlurStage::Chroma(shifts),
                        combine_mode: COMBINE_GAUSSIAN,
                        amount: 0.0,
                    })
                }
                k if k == SPATIAL_BLOOM => {
                    // params: [threshold, intensity, radius, falloff]. Radius-
                    // independent blur: downsample by `factor`, blur at LOW res with
                    // the bounded `low_radius = radius/factor` kernel, upsample. The
                    // halo is still the FULL radius (the glow spreads that far); the
                    // weights/half are the LOW-res kernel.
                    let radius = params[2];
                    let factor = bloom_downsample_factor(radius);
                    let low_radius = radius / factor as f32;
                    let (low_weights, low_half) = gaussian_weights(low_radius);
                    let halo = (radius.ceil() as u32).clamp(1, MAX_BLUR_HALF);
                    Some(KernelPlan {
                        weights: low_weights,
                        half: halo,
                        stage: BlurStage::Bloom {
                            threshold: params[0],
                            falloff: params[3],
                            factor,
                            low_half,
                        },
                        combine_mode: COMBINE_BLOOM,
                        amount: params[1], // intensity
                    })
                }
                k if k == SPATIAL_SHADOWS_HIGHLIGHTS => {
                    // params: [shad_amount, shad_tonal_width, shad_radius,
                    //          high_amount, high_tonal_width, high_radius,
                    //          color_correction, midtone_contrast].
                    let (lo_weights, lo_half) = gaussian_weights(params[2]);
                    let (hi_weights, hi_half) = gaussian_weights(params[5]);
                    Some(KernelPlan {
                        weights: Vec::new(),
                        half: lo_half.max(hi_half), // halo = the larger blur radius
                        stage: BlurStage::Sh {
                            lo_weights,
                            lo_half,
                            hi_weights,
                            hi_half,
                            tonal: ShTonal {
                                shadows_amount: params[0],
                                shadows_tonal_width: params[1],
                                highlights_amount: params[3],
                                highlights_tonal_width: params[4],
                                color_correction: params[6],
                                midtone_contrast: params[7],
                            },
                        },
                        combine_mode: COMBINE_GAUSSIAN, // unused (S/H has its own combine)
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
                        stage: plan.stage,
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
            self.write_globals(gpu, canvas_w, canvas_h, region, false);
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
            // Stage → writes the adjusted texture into blur[1] for the SHARED
            // combine; or, for S/H, runs its OWN sub-graph + combine into base[dst].
            let shared_combine = match &b.stage {
                BlurStage::Separable => {
                    self.upload_blur_weights(gpu, &b.weights);
                    self.run_blur(gpu, cur, b.half, false, [0.0, 0.0], work);
                    true
                }
                BlurStage::Directional(dir) => {
                    self.upload_blur_weights(gpu, &b.weights);
                    self.run_blur(gpu, cur, b.half, true, *dir, work);
                    true
                }
                BlurStage::Chroma(shifts) => {
                    self.run_chroma(gpu, cur, *shifts, work, canvas_w, canvas_h);
                    true
                }
                BlurStage::Bloom {
                    threshold,
                    falloff,
                    factor,
                    low_half,
                } => {
                    self.upload_blur_weights(gpu, &b.weights); // the low-res kernel
                    self.run_bloom(gpu, cur, *threshold, *falloff, *factor, *low_half, work);
                    true
                }
                BlurStage::Sh {
                    lo_weights,
                    lo_half,
                    hi_weights,
                    hi_half,
                    tonal,
                } => {
                    self.run_shadows_highlights(
                        gpu,
                        cur,
                        cur ^ 1,
                        lo_weights,
                        *lo_half,
                        hi_weights,
                        *hi_half,
                        *tonal,
                        b.blend,
                        b.opacity,
                        work,
                    );
                    false // S/H wrote base[cur ^ 1] itself
                }
            };
            if shared_combine {
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
            }
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
}
