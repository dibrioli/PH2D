use super::super::*;

impl LayerCompositor {
    /// Ensure the linear work intermediates are at least `w × h` (grow-only —
    /// passes are bounded by `work_region`, so a larger texture just has an
    /// unused border; this avoids reallocating when the dirty rect shrinks).
    pub(super) fn ensure_work_textures(&mut self, gpu: &GpuContext, w: u32, h: u32) {
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
            sh: [
                make("ph2d-render layer_composite work sh0"),
                make("ph2d-render layer_composite work sh1"),
            ],
        });
    }

    /// Upload the separable Gaussian weights (binding 13). Grows like the other
    /// persistent buffers; writes ≥1 f32 so the storage binding is never empty.
    pub(super) fn upload_blur_weights(&mut self, gpu: &GpuContext, weights: &[f32]) {
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
    pub(super) fn dispatch_pass(
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
                timestamp_writes: ph2d_gpu::pass_profiler::compute_writes("render.layer_comp"),
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
    pub(super) fn run_segment(
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
    pub(super) fn run_blur(
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
        // Write the blur uniforms with the pass's `premul_read`. Per-pass write
        // (not once up front): the queue applies writes/submits in order, so the
        // H/dir pass reads premul=1 (premultiply on tap) and V reads premul=0
        // (source already premultiplied). See the WGSL premul note.
        let write_globals = |premul_read: f32| {
            let g = BlurGlobals {
                width: work.w,
                height: work.h,
                half,
                _pad0: 0,
                dir_x: dir[0],
                dir_y: dir[1],
                premul_read,
                _pad2: 0.0,
            };
            gpu.queue
                .write_buffer(&self.blur_globals_buffer, 0, bytemuck::bytes_of(&g));
        };
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
            // arbitrary direction). Premultiplies on read.
            write_globals(1.0);
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
        write_globals(1.0); // H premultiplies the straight base on read
        let bg_h = blur_bg(&work_tex.base[base_idx].view, &work_tex.blur[0].view);
        self.dispatch_pass(
            gpu,
            &self.pipeline_blur_h,
            &bg_h,
            work.w,
            work.h,
            "ph2d-render layer_composite blur_h pass",
        );
        write_globals(0.0); // V reads already-premultiplied data
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

    /// Chromatic-aberration gather over `base[base_idx]` → `blur[1]` (the combine
    /// reads `blur[1]`). The radial centre + per-channel scales are precomputed
    /// here so the GPU does no per-pixel `sqrt` (parity-robust nearest sampling).
    pub(super) fn run_chroma(
        &self,
        gpu: &GpuContext,
        base_idx: usize,
        shifts: [f32; 3],
        work: Region,
        canvas_w: u32,
        canvas_h: u32,
    ) {
        let Some(work_tex) = &self.work else {
            return;
        };
        let cw = canvas_w as f32;
        let ch = canvas_h as f32;
        // Half the canvas diagonal — the max distance from centre, so a shift of
        // `shift_c` px at the corner is `scale_c = shift_c / half_diag` per unit
        // of `dir = local − centre`.
        let half_diag = 0.5 * (cw * cw + ch * ch).sqrt();
        let inv = if half_diag > 0.0 {
            1.0 / half_diag
        } else {
            0.0
        };
        let g = ChromaGlobals {
            width: work.w,
            height: work.h,
            // Canvas centre expressed in work_region-local coords.
            center_x: cw * 0.5 - work.x as f32,
            center_y: ch * 0.5 - work.y as f32,
            scale_r: shifts[0] * inv,
            scale_g: shifts[1] * inv,
            scale_b: shifts[2] * inv,
            _pad: 0.0,
        };
        gpu.queue
            .write_buffer(&self.chroma_globals_buffer, 0, bytemuck::bytes_of(&g));
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render layer_composite chroma bg"),
            layout: &self.bgl_chroma,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 21,
                    resource: self.chroma_globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 22,
                    resource: wgpu::BindingResource::TextureView(&work_tex.base[base_idx].view),
                },
                wgpu::BindGroupEntry {
                    binding: 23,
                    resource: wgpu::BindingResource::TextureView(&work_tex.blur[1].view),
                },
            ],
        });
        self.dispatch_pass(
            gpu,
            &self.pipeline_chroma,
            &bind_group,
            work.w,
            work.h,
            "ph2d-render layer_composite chroma pass",
        );
    }
}
