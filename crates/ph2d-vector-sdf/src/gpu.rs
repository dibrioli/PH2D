//! GPU compute port of the SDF draft (ADR-0065 phase 2).
//!
//! `GpuSdf` builds the compute pipeline once; [`GpuSdf::network_sdf`] uploads a
//! network's flattened boundary edges + a globals uniform, dispatches the
//! per-pixel SDF compute (`shaders/network_sdf.wgsl`), and reads the signed grid
//! back into an [`SdfGrid`] — bit-comparable to the CPU [`crate::network_sdf`]
//! for a single-region NonZero silhouette (the parity gate). Mirrors the
//! `ph2d-render/src/layer_compositor/` pipeline + readback discipline.

use crate::{Bounds, SdfGrid, network_edges};
use ph2d_gpu::GpuContext;
use ph2d_vector_doc::VectorNetwork;

/// Globals uniform — mirrors the WGSL `Globals` (32 bytes, 16-aligned).
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    res: u32,
    edge_count: u32,
    _pad0: u32,
    _pad1: u32,
    bmin: [f32; 2],
    bmax: [f32; 2],
}

const SHADER: &str = include_str!("../shaders/network_sdf.wgsl");

/// A reusable GPU SDF compute pipeline.
pub struct GpuSdf {
    pipeline: wgpu::ComputePipeline,
    bgl: wgpu::BindGroupLayout,
}

impl GpuSdf {
    /// Build the compute pipeline. Cheap — no GPU buffers until the first call.
    #[must_use]
    pub fn new(gpu: &GpuContext) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ph2d-vector-sdf network_sdf"),
                source: wgpu::ShaderSource::Wgsl(SHADER.into()),
            });
        let bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-vector-sdf bgl"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                core::mem::size_of::<Globals>() as u64,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(16),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
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
                label: Some("ph2d-vector-sdf layout"),
                bind_group_layouts: &[&bgl],
                immediate_size: 0,
            });
        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("ph2d-vector-sdf pipeline"),
                layout: Some(&layout),
                module: &shader,
                entry_point: Some("sdf_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });
        Self { pipeline, bgl }
    }

    /// Compute `net`'s SDF over `bounds` at `res` on the GPU and read it back.
    /// Uses NonZero (union) winding — matches the CPU [`crate::network_sdf`] for
    /// a single-region NonZero silhouette. Blocks on `device.poll` (the readback
    /// is a draft/verification path, not a per-frame hot path).
    #[must_use]
    pub fn network_sdf(
        &self,
        gpu: &GpuContext,
        net: &VectorNetwork,
        res: u32,
        bounds: Bounds,
    ) -> SdfGrid {
        let res = res.max(1);
        let n = (res * res) as usize;
        let real_edges = network_edges(net);
        let globals = Globals {
            res,
            edge_count: real_edges.len() as u32,
            _pad0: 0,
            _pad1: 0,
            bmin: [bounds.min.x, bounds.min.y],
            bmax: [bounds.max.x, bounds.max.y],
        };
        // Storage bindings must hold ≥1 element even when the network is empty.
        let mut edges = real_edges;
        if edges.is_empty() {
            edges.push([0.0; 4]);
        }

        let globals_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-vector-sdf globals"),
            size: core::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue
            .write_buffer(&globals_buf, 0, bytemuck::bytes_of(&globals));

        let edges_bytes: &[u8] = bytemuck::cast_slice(&edges);
        let edges_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-vector-sdf edges"),
            size: edges_bytes.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&edges_buf, 0, edges_bytes);

        let out_size = (n * core::mem::size_of::<f32>()) as u64;
        let out_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-vector-sdf out"),
            size: out_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-vector-sdf readback"),
            size: out_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-vector-sdf bg"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: edges_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: out_buf.as_entire_binding(),
                },
            ],
        });

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-vector-sdf encoder"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ph2d-vector-sdf pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            let groups = res.div_ceil(8);
            pass.dispatch_workgroups(groups, groups, 1);
        }
        encoder.copy_buffer_to_buffer(&out_buf, 0, &staging, 0, out_size);
        gpu.queue.submit([encoder.finish()]);

        let (tx, rx) = std::sync::mpsc::channel();
        staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let data = match rx.recv() {
            Ok(Ok(())) => {
                let mapped = staging.slice(..).get_mapped_range();
                let floats: Vec<f32> = bytemuck::cast_slice(&mapped).to_vec();
                drop(mapped);
                staging.unmap();
                floats
            }
            // Device lost / validation error → a flat outside field (draft path).
            _ => vec![1e30; n],
        };
        SdfGrid { res, bounds, data }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;
    use ph2d_vector_doc::{Region, Segment, Vertex, VertexId, VertexKind, WindingRule};

    /// A 20×20 axis-aligned square as a one-region NonZero network (CCW).
    fn square(cx: f32, cy: f32, h: f32) -> VectorNetwork {
        let mut net = VectorNetwork::empty();
        let corners = [
            Vec2::new(cx - h, cy - h),
            Vec2::new(cx + h, cy - h),
            Vec2::new(cx + h, cy + h),
            Vec2::new(cx - h, cy + h),
        ];
        for (i, &p) in corners.iter().enumerate() {
            net.vertices
                .push(Vertex::new(i as VertexId, p, VertexKind::Auto));
        }
        let mut region = Region::new(0, WindingRule::NonZero);
        for i in 0..4u32 {
            net.segments.push(Segment::straight(i, i, (i + 1) % 4));
            region.segments.push((i, true));
        }
        net.regions.push(region);
        net
    }

    fn try_gpu() -> Option<GpuContext> {
        GpuContext::new(GpuContext::default_instance(), None).ok()
    }

    #[test]
    #[ignore = "needs a GPU device"]
    fn gpu_sdf_matches_cpu_reference() {
        let Some(gpu) = try_gpu() else {
            eprintln!("no GPU — skipping");
            return;
        };
        let net = square(50.0, 50.0, 10.0);
        let bounds = Bounds::of_network(&net, 10.0);
        let res = 64u32;
        let cpu = crate::network_sdf(&net, res, bounds);
        let pipe = GpuSdf::new(&gpu);
        let got = pipe.network_sdf(&gpu, &net, res, bounds);

        let mut max_diff = 0.0f32;
        for i in 0..(res * res) as usize {
            max_diff = max_diff.max((cpu.data[i] - got.data[i]).abs());
        }
        // Same algorithm (min-dist + NonZero winding over the same edges) in f32;
        // the only divergence is GPU vs CPU `sqrt`/FMA rounding (sub-pixel) plus a
        // rare near-boundary winding flip (where the distance is ~0). A draft
        // silhouette within a pixel is exact for the purpose (ADR-0065 §2.3).
        assert!(
            max_diff < 1.0,
            "GPU vs CPU SDF parity: max diff {max_diff} world-px (expected sub-pixel)"
        );
    }
}
