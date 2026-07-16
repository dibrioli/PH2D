//! The **lowering**: the last compute pass of a cook, which gathers the stream's
//! columns straight into a buffer laid out as [`ph2d_render::RenderInstance`] —
//! the sprite renderer's instance vertex buffer.
//!
//! This is where the GPU cook stops being a stream engine and becomes a frame:
//! it is the only stage that is not a node, the only one that applies the
//! caller's `default_uv_rect`/`default_size`, and the only one that writes a
//! PERSISTENT (grow-only) buffer rather than a pooled column. Kept beside the
//! sequencer rather than in it because it answers a different question — the
//! sequencer asks "what does this node compute?", this asks "what does the
//! renderer need?".

use crate::stream::GpuStream;
use crate::{CachedPipeline, GpuCook, UNIFORM_BYTES, codegen, create_pipeline, lower};
use ph2d_gpu::GpuContext;

/// The GPU-resident instance output of a cook: a buffer laid out as
/// `[RenderInstance; len]`, usable directly as the sprite renderer's instance
/// vertex buffer (usage VERTEX) and mappable for the parity gates (COPY_SRC).
pub struct GpuInstances {
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) len: u32,
    pub(crate) capacity: u32,
}

impl GpuInstances {
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
    pub fn len(&self) -> u32 {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl GpuCook {
    /// Encode the final lowering pass into the persistent instance buffer.
    pub(crate) fn encode_lowering(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        uniform_slot: usize,
        stream: &GpuStream,
        default_uv_rect: [f32; 4],
        default_size: [f32; 2],
    ) {
        let count = stream.count;
        self.ensure_instance_capacity(gpu, count.max(1));
        let instances = self.instances.as_mut().expect("just ensured");
        instances.len = count;
        if count == 0 {
            return;
        }

        let present: [bool; 5] = std::array::from_fn(|i| {
            stream
                .cols
                .get(lower::LOWER_COLUMNS[i])
                .is_some_and(|c| c.dim == expected_lower_dim(i))
        });
        let sig = lower::lower_signature(present);
        self.lower_pipelines.entry(sig).or_insert_with(|| {
            let src = lower::lower_module(present);
            CachedPipeline {
                pipeline: create_pipeline(gpu, &src, "ph2d-gpu-cook lowering"),
            }
        });

        // Uniform: count, pad, default_size (vec2 @ 8), default_uv (vec4 @ 16).
        let mut uni = [0u8; 32];
        uni[0..4].copy_from_slice(&count.to_le_bytes());
        uni[8..12].copy_from_slice(&default_size[0].to_le_bytes());
        uni[12..16].copy_from_slice(&default_size[1].to_le_bytes());
        for (k, v) in default_uv_rect.iter().enumerate() {
            uni[16 + k * 4..20 + k * 4].copy_from_slice(&v.to_le_bytes());
        }
        let uniform = self.uniform_slot(gpu, uniform_slot);
        gpu.queue.write_buffer(uniform, 0, &uni);
        let uniform = &self.uniforms[uniform_slot];

        let instances = self.instances.as_ref().expect("ensured above");
        let mut entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: instances.buffer.as_entire_binding(),
            },
        ];
        let mut slot = 2u32;
        for (i, name) in lower::LOWER_COLUMNS.iter().enumerate() {
            if present[i] {
                let col = stream.cols.get(*name).expect("presence checked");
                entries.push(wgpu::BindGroupEntry {
                    binding: slot,
                    resource: col.buffer.as_entire_binding(),
                });
                slot += 1;
            }
        }
        let pipeline = &self
            .lower_pipelines
            .get(&sig)
            .expect("inserted above")
            .pipeline;
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-gpu-cook lowering"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &entries,
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ph2d-gpu-cook lowering"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(count.div_ceil(codegen::WORKGROUP_SIZE), 1, 1);
    }

    /// The persistent uniform buffer for stage slot `idx` (created on demand).
    pub(crate) fn uniform_slot(&mut self, gpu: &GpuContext, idx: usize) -> &wgpu::Buffer {
        while self.uniforms.len() <= idx {
            self.uniforms
                .push(gpu.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("ph2d-gpu-cook uniforms"),
                    size: UNIFORM_BYTES,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
        }
        &self.uniforms[idx]
    }

    /// Grow (never shrink) the instance output to hold `count` instances —
    /// `InstanceBuffer`'s policy, plus STORAGE (the lowering writes it) and
    /// COPY_SRC (the parity gates read it back, deliberately off-path).
    fn ensure_instance_capacity(&mut self, gpu: &GpuContext, count: u32) {
        let needs_grow = match &self.instances {
            Some(gi) => gi.capacity < count,
            None => true,
        };
        if !needs_grow {
            return;
        }
        let mut capacity = self
            .instances
            .as_ref()
            .map(|gi| gi.capacity.max(1))
            .unwrap_or(1);
        while capacity < count {
            capacity = capacity.saturating_mul(2);
        }
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-gpu-cook instances"),
            size: u64::from(capacity) * u64::from(lower::INSTANCE_WORDS) * 4,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        self.instances = Some(GpuInstances {
            buffer,
            len: 0,
            capacity,
        });
    }
}

/// The dim each lowering column must have to be gathered (a column with the
/// wrong type is ignored, exactly like the CPU's typed `*_at` accessors).
fn expected_lower_dim(i: usize) -> ph2d_nodegraph::port::Dim {
    use ph2d_nodegraph::port::Dim;
    match i {
        0 | 1 => Dim::Vec2, // P, size
        2 => Dim::Scalar,   // rot
        _ => Dim::Vec4,     // tint, uv_rect
    }
}
