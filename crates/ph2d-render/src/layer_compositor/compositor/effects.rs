use super::super::*;
use super::ShTonal;

impl LayerCompositor {
    /// Bloom — RADIUS-INDEPENDENT (O(1)) glow, leaving the result in `blur[1]` for
    /// `run_combine`'s `COMBINE_BLOOM` (additive) step. All premultiplied. The passes:
    /// `cs_bloom_bright` (`base[base_idx]` → `blur[1]`, full-res glow); `cs_bloom_down`
    /// (box-downsample `blur[1]` full → `blur[0]` low = work/factor); a separable blur
    /// of the LOW-res glow (`low_half`, premul_read 0: `blur[0]` → `blur[1]` H →
    /// `blur[0]` V, at the low dims); then `cs_bloom_up` (bilinear-upsample `blur[0]`
    /// low → `blur[1]` full). The only kernel work is the bounded low-res blur, so the
    /// cost is ~constant at any radius. For `factor == 1` the down/up are 1:1 (the
    /// direct blur for small radii — the parity gate's degenerate case).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn run_bloom(
        &self,
        gpu: &GpuContext,
        base_idx: usize,
        threshold: f32,
        falloff: f32,
        factor: u32,
        low_half: u32,
        work: Region,
    ) {
        let (Some(work_tex), Some((weights_buffer, _))) = (&self.work, &self.blur_weights_buffer)
        else {
            return;
        };
        let low_w = work.w.div_ceil(factor);
        let low_h = work.h.div_ceil(factor);

        // (1) bright-pass → blur[1] (full-res premultiplied glow).
        let g = BloomGlobals {
            width: work.w,
            height: work.h,
            threshold,
            falloff,
        };
        gpu.queue
            .write_buffer(&self.bloom_globals_buffer, 0, bytemuck::bytes_of(&g));
        let bright_bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render layer_composite bloom bright bg"),
            layout: &self.bgl_bloom,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 24,
                    resource: self.bloom_globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 25,
                    resource: wgpu::BindingResource::TextureView(&work_tex.base[base_idx].view),
                },
                wgpu::BindGroupEntry {
                    binding: 26,
                    resource: wgpu::BindingResource::TextureView(&work_tex.blur[1].view),
                },
            ],
        });
        self.dispatch_pass(
            gpu,
            &self.pipeline_bloom_bright,
            &bright_bg,
            work.w,
            work.h,
            "ph2d-render layer_composite bloom bright pass",
        );

        // (2) box-downsample blur[1] (full) → blur[0] (low).
        let mip_bg = |src: &wgpu::TextureView, dst: &wgpu::TextureView| {
            gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-render layer_composite bloom mip bg"),
                layout: &self.bgl_bloom_mip,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 27,
                        resource: self.bloom_mip_globals_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 28,
                        resource: wgpu::BindingResource::TextureView(src),
                    },
                    wgpu::BindGroupEntry {
                        binding: 29,
                        resource: wgpu::BindingResource::TextureView(dst),
                    },
                ],
            })
        };
        let write_mip_globals = |src_w: u32, src_h: u32, dst_w: u32, dst_h: u32| {
            let g = BloomMipGlobals {
                src_w,
                src_h,
                dst_w,
                dst_h,
                factor,
                _pad0: 0,
                _pad1: 0,
                _pad2: 0,
            };
            gpu.queue
                .write_buffer(&self.bloom_mip_globals_buffer, 0, bytemuck::bytes_of(&g));
        };
        write_mip_globals(work.w, work.h, low_w, low_h);
        let down_bg = mip_bg(&work_tex.blur[1].view, &work_tex.blur[0].view);
        self.dispatch_pass(
            gpu,
            &self.pipeline_bloom_down,
            &down_bg,
            low_w,
            low_h,
            "ph2d-render layer_composite bloom down pass",
        );

        // (3) separable blur the LOW-res glow (premul_read = 0): blur[0] → blur[1] (H)
        //     → blur[0] (V), at the low dims.
        let bg_blur = BlurGlobals {
            width: low_w,
            height: low_h,
            half: low_half,
            _pad0: 0,
            dir_x: 0.0,
            dir_y: 0.0,
            premul_read: 0.0,
            _pad2: 0.0,
        };
        gpu.queue
            .write_buffer(&self.blur_globals_buffer, 0, bytemuck::bytes_of(&bg_blur));
        let blur_bg = |src: &wgpu::TextureView, dst: &wgpu::TextureView| {
            gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-render layer_composite bloom blur bg"),
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
        let bg_h = blur_bg(&work_tex.blur[0].view, &work_tex.blur[1].view);
        self.dispatch_pass(
            gpu,
            &self.pipeline_blur_h,
            &bg_h,
            low_w,
            low_h,
            "ph2d-render layer_composite bloom low blur_h pass",
        );
        let bg_v = blur_bg(&work_tex.blur[1].view, &work_tex.blur[0].view);
        self.dispatch_pass(
            gpu,
            &self.pipeline_blur_v,
            &bg_v,
            low_w,
            low_h,
            "ph2d-render layer_composite bloom low blur_v pass",
        );

        // (4) bilinear-upsample blur[0] (low) → blur[1] (full).
        write_mip_globals(low_w, low_h, work.w, work.h);
        let up_bg = mip_bg(&work_tex.blur[0].view, &work_tex.blur[1].view);
        self.dispatch_pass(
            gpu,
            &self.pipeline_bloom_up,
            &up_bg,
            work.w,
            work.h,
            "ph2d-render layer_composite bloom up pass",
        );
    }

    /// Shadows/Highlights sub-graph: `cs_sh_luma` extracts the display luma into
    /// `sh[0]`, two scalar blurs build the local tone maps (shadows radius → `sh[1]`,
    /// highlights radius → `blur[1]`), then `cs_combine_sh` applies the tonal
    /// correction (`base[base_idx]` + the two maps → `base[dst_idx]`), coverage
    /// preserved. Mirror of `apply_shadows_highlights`. `&mut self` because the two
    /// blurs upload DIFFERENT weights between their dispatches.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn run_shadows_highlights(
        &mut self,
        gpu: &GpuContext,
        base_idx: usize,
        dst_idx: usize,
        lo_weights: &[f32],
        lo_half: u32,
        hi_weights: &[f32],
        hi_half: u32,
        tonal: ShTonal,
        blend: u8,
        opacity: f32,
        work: Region,
    ) {
        // ShGlobals — written once; the luma pass + the combine both read it.
        let g = ShGlobals {
            width: work.w,
            height: work.h,
            shadows_amount: tonal.shadows_amount,
            highlights_amount: tonal.highlights_amount,
            shadows_tonal_width: tonal.shadows_tonal_width,
            highlights_tonal_width: tonal.highlights_tonal_width,
            color_correction: tonal.color_correction,
            midtone_contrast: tonal.midtone_contrast,
            blend_mode: u32::from(blend),
            opacity,
            _pad0: 0,
            _pad1: 0,
        };
        gpu.queue
            .write_buffer(&self.sh_globals_buffer, 0, bytemuck::bytes_of(&g));
        // (1) luma extract: base[base_idx] → sh[0].
        if let Some(work_tex) = &self.work {
            let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-render layer_composite sh luma bg"),
                layout: &self.bgl_sh_luma,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 27,
                        resource: self.sh_globals_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 28,
                        resource: wgpu::BindingResource::TextureView(&work_tex.base[base_idx].view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 29,
                        resource: wgpu::BindingResource::TextureView(&work_tex.sh[0].view),
                    },
                ],
            });
            self.dispatch_pass(
                gpu,
                &self.pipeline_sh_luma,
                &bg,
                work.w,
                work.h,
                "ph2d-render layer_composite sh luma pass",
            );
        }
        // (2) shadows tone map: blur sh[0] → sh[1].
        self.upload_blur_weights(gpu, lo_weights);
        self.sh_scalar_blur(gpu, lo_half, true, work);
        // (3) highlights tone map: blur sh[0] → blur[1].
        self.upload_blur_weights(gpu, hi_weights);
        self.sh_scalar_blur(gpu, hi_half, false, work);
        // (4) tonal combine: base[base_idx] + sh[1] (lo) + blur[1] (hi) → base[dst_idx].
        if let Some(work_tex) = &self.work {
            let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-render layer_composite sh combine bg"),
                layout: &self.bgl_sh_combine,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 27,
                        resource: self.sh_globals_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 28,
                        resource: wgpu::BindingResource::TextureView(&work_tex.base[base_idx].view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 30,
                        resource: wgpu::BindingResource::TextureView(&work_tex.sh[1].view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 31,
                        resource: wgpu::BindingResource::TextureView(&work_tex.blur[1].view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 32,
                        resource: wgpu::BindingResource::TextureView(&work_tex.base[dst_idx].view),
                    },
                ],
            });
            self.dispatch_pass(
                gpu,
                &self.pipeline_sh_combine,
                &bg,
                work.w,
                work.h,
                "ph2d-render layer_composite sh combine pass",
            );
        }
    }

    /// One scalar separable blur of the luma field `sh[0]` → `dst` (`sh[1]` when
    /// `dst_sh1`, else `blur[1]`), via `blur[0]` as the H temp. `premul_read = 0`
    /// (a scalar field, not premultiplied colour). Weights already uploaded by the
    /// caller. Mirror of `separable_blur_scalar`.
    pub(super) fn sh_scalar_blur(&self, gpu: &GpuContext, half: u32, dst_sh1: bool, work: Region) {
        let (Some(work_tex), Some((weights_buffer, _))) = (&self.work, &self.blur_weights_buffer)
        else {
            return;
        };
        let bg_blur = BlurGlobals {
            width: work.w,
            height: work.h,
            half,
            _pad0: 0,
            dir_x: 0.0,
            dir_y: 0.0,
            premul_read: 0.0,
            _pad2: 0.0,
        };
        gpu.queue
            .write_buffer(&self.blur_globals_buffer, 0, bytemuck::bytes_of(&bg_blur));
        let blur_bg = |src: &wgpu::TextureView, dst: &wgpu::TextureView| {
            gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-render layer_composite sh blur bg"),
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
        let bg_h = blur_bg(&work_tex.sh[0].view, &work_tex.blur[0].view);
        self.dispatch_pass(
            gpu,
            &self.pipeline_blur_h,
            &bg_h,
            work.w,
            work.h,
            "ph2d-render layer_composite sh blur_h pass",
        );
        let dst_view = if dst_sh1 {
            &work_tex.sh[1].view
        } else {
            &work_tex.blur[1].view
        };
        let bg_v = blur_bg(&work_tex.blur[0].view, dst_view);
        self.dispatch_pass(
            gpu,
            &self.pipeline_blur_v,
            &bg_v,
            work.w,
            work.h,
            "ph2d-render layer_composite sh blur_v pass",
        );
    }

    /// Derive the kernel result from `base[base_idx]` + `blur[1]` (per
    /// `combine_mode`/`amount`) and blend it over `base[base_idx]` by
    /// `blend`/`opacity` into `base[dst_idx]`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn run_combine(
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
    pub(super) fn run_encode(
        &self,
        gpu: &GpuContext,
        base_idx: usize,
        region: Region,
        work: Region,
    ) {
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
