//! GPU compute driver for the inpaint EM loop (W2, ADR-0102). The pyramid,
//! region classification and cross-level upsampling stay on the CPU (cheap); the
//! per-level hot loop — NNF init, cost refresh, jump-flood propagation, random
//! search, gather-voting — runs on the GPU via the WGSL in [`wgsl`]. One thread
//! per pixel; the kernels mirror the CPU reference op-for-op, so the two
//! reconcile within ε (`tests`). Runtime picks GPU, falling back to CPU when no
//! adapter is available ([`crate::inpaint`]).
//!
//! Each dispatch bakes its `step`/`pass` into its OWN `create_buffer_init`
//! uniform + bind group: a queued `write_buffer` can't vary a uniform between
//! dispatches in one encoder (all writes land before the submit), so per-dispatch
//! uniform buffers are the portable way to sequence the jump-flood steps.

mod wgsl;

#[cfg(test)]
mod tests;

use crate::mask::{Mask, Regions};
use crate::plane::Plane;
use bytemuck::{Pod, Zeroable};
use ph2d_gpu::GpuContext;
use std::borrow::Cow;
use wgpu::util::DeviceExt;

const WORKGROUP: u32 = 64;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    w: u32,
    h: u32,
    r: i32,
    step: i32,
    em_pass: u32,
    seed: u32,
    n_src: u32,
    max_r: i32,
}

/// Compiled pipelines for the five inpaint kernels + their shared bind-group
/// layout. Build once (`new`), reuse across levels and calls.
pub struct GpuInpainter {
    bgl: wgpu::BindGroupLayout,
    init: wgpu::ComputePipeline,
    cost_refresh: wgpu::ComputePipeline,
    propagate: wgpu::ComputePipeline,
    random_search: wgpu::ComputePipeline,
    vote: wgpu::ComputePipeline,
}

/// The per-level storage buffers (all live on the GPU for the level's EM loop).
/// `flags` packs the three per-pixel masks (bit0 source, bit1 target, bit2 hole)
/// so the kernel stays within the 8-storage-buffer device floor.
struct LevelBuffers {
    content: wgpu::Buffer,
    src: wgpu::Buffer,
    flags: wgpu::Buffer,
    sources: wgpu::Buffer,
    off_a: wgpu::Buffer,
    cost_a: wgpu::Buffer,
    off_b: wgpu::Buffer,
    cost_b: wgpu::Buffer,
}

fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

impl GpuInpainter {
    /// Compile the shader + pipelines against `gpu`.
    pub fn new(gpu: &GpuContext) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("inpaint.shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(wgsl::INPAINT_WGSL)),
            });

        let mut entries = vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<Uniforms>() as u64),
            },
            count: None,
        }];
        // read-only: src(2) flags(3) sources(4)
        for b in [2, 3, 4] {
            entries.push(storage_entry(b, true));
        }
        // read-write: content(1) off_a(5) cost_a(6) off_b(7) cost_b(8)
        for b in [1, 5, 6, 7, 8] {
            entries.push(storage_entry(b, false));
        }
        entries.sort_by_key(|e| e.binding);
        let bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("inpaint.bgl"),
                entries: &entries,
            });
        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("inpaint.layout"),
                bind_group_layouts: &[&bgl],
                immediate_size: 0,
            });
        let mk = |entry: &str| {
            gpu.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("inpaint.pipeline"),
                    layout: Some(&layout),
                    module: &shader,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    cache: None,
                })
        };
        Self {
            init: mk("init_nnf"),
            cost_refresh: mk("cost_refresh"),
            propagate: mk("propagate"),
            random_search: mk("random_search"),
            vote: mk("vote"),
            bgl,
        }
    }

    /// Run the full EM loop for one pyramid level on the GPU, overwriting the
    /// hole pixels of `content` in place. `reg.sources` must be non-empty.
    #[allow(clippy::too_many_arguments)] // mirrors the backend-closure param list
    pub fn run_level(
        &self,
        gpu: &GpuContext,
        content: &mut Plane,
        src: &Plane,
        mask: &Mask,
        reg: &Regions,
        r: i32,
        em_iters: u32,
        seed: u32,
    ) {
        let (w, h) = (content.w, content.h);
        let n = w * h;
        let n_src = reg.sources.len() as u32;
        debug_assert!(n_src > 0, "run_level requires at least one source");
        let bufs = self.upload(gpu, content, src, mask, reg);
        let base = Uniforms {
            w: w as u32,
            h: h as u32,
            r,
            step: 0,
            em_pass: 0,
            seed,
            n_src,
            max_r: w.max(h) as i32,
        };
        let groups = (n as u32).div_ceil(WORKGROUP);

        // Keep every per-dispatch uniform buffer + bind group alive until submit.
        let mut keep: Vec<(wgpu::Buffer, wgpu::BindGroup)> = Vec::new();
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("inpaint.enc"),
            });

        self.pass(gpu, &mut enc, &bufs, &mut keep, &self.init, base, groups);
        for p in 0..em_iters {
            self.pass(
                gpu,
                &mut enc,
                &bufs,
                &mut keep,
                &self.cost_refresh,
                base,
                groups,
            );
            // Jump-flood propagation: steps n/2 … 1, copying b→a after each.
            let mut step = (w.max(h).next_power_of_two() / 2).max(1) as i32;
            loop {
                let u = Uniforms { step, ..base };
                self.pass(gpu, &mut enc, &bufs, &mut keep, &self.propagate, u, groups);
                enc.copy_buffer_to_buffer(&bufs.off_b, 0, &bufs.off_a, 0, bufs.off_a.size());
                enc.copy_buffer_to_buffer(&bufs.cost_b, 0, &bufs.cost_a, 0, bufs.cost_a.size());
                if step == 1 {
                    break;
                }
                step /= 2;
            }
            let us = Uniforms { em_pass: p, ..base };
            self.pass(
                gpu,
                &mut enc,
                &bufs,
                &mut keep,
                &self.random_search,
                us,
                groups,
            );
            self.pass(gpu, &mut enc, &bufs, &mut keep, &self.vote, base, groups);
        }

        // Copy the reconstructed content back and decode into the Plane.
        let bytes = (n * 4 * 4) as u64; // vec4<f32> per pixel
        let readback = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("inpaint.readback"),
            size: bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        enc.copy_buffer_to_buffer(&bufs.content, 0, &readback, 0, bytes);
        gpu.queue.submit(std::iter::once(enc.finish()));

        let slice = readback.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        gpu.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("poll inpaint readback");
        rx.recv().expect("readback channel").expect("map_async");
        let data = slice.get_mapped_range();
        let floats: &[f32] = bytemuck::cast_slice(&data);
        for i in 0..n {
            let o = i * 4;
            content.px[i * 3] = floats[o];
            content.px[i * 3 + 1] = floats[o + 1];
            content.px[i * 3 + 2] = floats[o + 2];
        }
        drop(data);
        readback.unmap();
    }

    /// Encode one kernel dispatch with its own baked uniform buffer + bind group.
    #[allow(clippy::too_many_arguments)] // shared buffers + encoder + keepalive
    fn pass(
        &self,
        gpu: &GpuContext,
        enc: &mut wgpu::CommandEncoder,
        bufs: &LevelBuffers,
        keep: &mut Vec<(wgpu::Buffer, wgpu::BindGroup)>,
        pipeline: &wgpu::ComputePipeline,
        u: Uniforms,
        groups: u32,
    ) {
        let ub = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("inpaint.uniforms"),
                contents: bytemuck::bytes_of(&u),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("inpaint.bg"),
            layout: &self.bgl,
            entries: &[
                bind(0, &ub),
                bind(1, &bufs.content),
                bind(2, &bufs.src),
                bind(3, &bufs.flags),
                bind(4, &bufs.sources),
                bind(5, &bufs.off_a),
                bind(6, &bufs.cost_a),
                bind(7, &bufs.off_b),
                bind(8, &bufs.cost_b),
            ],
        });
        {
            let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("inpaint.pass"),
                timestamp_writes: None,
            });
            cp.set_pipeline(pipeline);
            cp.set_bind_group(0, &bg, &[]);
            cp.dispatch_workgroups(groups, 1, 1);
        }
        keep.push((ub, bg));
    }

    /// Build + upload every per-level buffer from the CPU-side host arrays.
    fn upload(
        &self,
        gpu: &GpuContext,
        content: &Plane,
        src: &Plane,
        mask: &Mask,
        reg: &Regions,
    ) -> LevelBuffers {
        let n = content.w * content.h;
        let content_v4 = pack_vec4(content);
        let src_v4 = pack_vec4(src);
        // Pack the three masks into one flag word per pixel: bit0 source, bit1
        // target, bit2 hole (keeps the kernel within the 8-storage-buffer floor).
        let mut flags = vec![0u32; n];
        for (f, &s) in flags.iter_mut().zip(reg.is_source.iter()) {
            *f |= u32::from(s);
        }
        for &t in &reg.targets {
            flags[t as usize] |= 2;
        }
        for (f, &hole) in flags.iter_mut().zip(mask.hole.iter()) {
            *f |= u32::from(hole) << 2;
        }

        let init = |label: &str, bytes: &[u8], extra: wgpu::BufferUsages| {
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(label),
                    contents: bytes,
                    usage: wgpu::BufferUsages::STORAGE | extra,
                })
        };
        let empty = |label: &str, size: u64| {
            gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        LevelBuffers {
            content: init(
                "inpaint.content",
                bytemuck::cast_slice(&content_v4),
                wgpu::BufferUsages::COPY_SRC,
            ),
            src: init(
                "inpaint.src",
                bytemuck::cast_slice(&src_v4),
                wgpu::BufferUsages::empty(),
            ),
            flags: init(
                "inpaint.flags",
                bytemuck::cast_slice(&flags),
                wgpu::BufferUsages::empty(),
            ),
            sources: init(
                "inpaint.sources",
                bytemuck::cast_slice(&reg.sources),
                wgpu::BufferUsages::empty(),
            ),
            off_a: empty("inpaint.off_a", (n * 8) as u64),
            cost_a: empty("inpaint.cost_a", (n * 4) as u64),
            off_b: empty("inpaint.off_b", (n * 8) as u64),
            cost_b: empty("inpaint.cost_b", (n * 4) as u64),
        }
    }
}

fn bind(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

/// Pack an RGB [`Plane`] into `w*h` `vec4<f32>` (a = 1) for a storage buffer.
fn pack_vec4(p: &Plane) -> Vec<f32> {
    let mut v = vec![0.0f32; p.w * p.h * 4];
    for (dst, srcpx) in v.chunks_exact_mut(4).zip(p.px.chunks_exact(3)) {
        dst[0] = srcpx[0];
        dst[1] = srcpx[1];
        dst[2] = srcpx[2];
        dst[3] = 1.0;
    }
    v
}
