//! **Reading a mid-chain column back — for gates, never for the hot path.**
//!
//! `read_instances` is the deliberate boundary for a chain that ends in the
//! instance lowering. A chain that ends in a **VALUE** stream has no lowering, so
//! there was no way to compare its numbers against the CPU at all — and the
//! `value.*` family's gates could only ever check the stream's SHAPE. Shape
//! without numbers is a false green waiting to happen: the luma weights could be
//! transposed and every column-set assertion would still pass.
//!
//! **Why this is opt-in.** Answering it means keeping the cook's output streams
//! alive, and a `GpuStream` is `Arc<wgpu::Buffer>` columns — holding them pins the
//! buffers and [`crate::BufferPool::reclaim`] skips anything still referenced. In
//! production that is a leak of the whole graph's intermediates, every frame. So
//! retention is **off by default** and a gate turns it on for itself.
//!
//! And it stays a *gate* tool on purpose: the measured cost of pulling real data
//! back is the reason this engine is built the way it is (`readback_tap_cost_probe`
//! — 268 ms for 4,19 M, worse than computing the whole thing on the CPU). Nothing
//! on a frame path may call this.

use crate::{GpuCook, GpuStream};
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::graph::NodeId;

impl GpuCook {
    /// Keep each staged node's output stream after a cook so [`Self::read_column`]
    /// can read it. **Gates only** — see the module docs: this pins buffers.
    pub fn retain_streams_for_debug(&mut self, on: bool) {
        self.debug_retain = on;
        if !on {
            self.debug_streams.clear();
        }
    }

    /// Read `node`'s scalar `column` back — `None` unless
    /// [`Self::retain_streams_for_debug`] was on for the last cook and that node
    /// staged and carried the column. This is the VALUE-stream reader.
    pub fn read_column(&self, gpu: &GpuContext, node: NodeId, column: &str) -> Option<Vec<f32>> {
        self.read_column_lanes(gpu, node, column)
    }

    /// The same, for a `Dim::Vec2` column (`P`, `size`, …).
    pub fn read_column_vec2(
        &self,
        gpu: &GpuContext,
        node: NodeId,
        column: &str,
    ) -> Option<Vec<[f32; 2]>> {
        let flat: Vec<f32> = self.read_column_lanes(gpu, node, column)?;
        Some(flat.chunks_exact(2).map(|c| [c[0], c[1]]).collect())
    }

    /// The one reader both wrap. **The width comes from the COLUMN's own `dim`**,
    /// through [`crate::stream::element_stride`] — the same function the uploader
    /// and the binder use, so this cannot disagree with them about how wide an
    /// element is (a `Vec3` pads to 16 bytes, and a reader that assumed 12 would
    /// misindex from element 1 on). Asking the caller for the width would be a
    /// second answer to a question the crate already answers, and a wrong one
    /// would read as a parity failure rather than a mis-read.
    fn read_column_lanes(&self, gpu: &GpuContext, node: NodeId, column: &str) -> Option<Vec<f32>> {
        let stream: &GpuStream = self.debug_streams.get(&node)?;
        let col = stream.get(column)?;
        let lanes = crate::stream::element_stride(col.dim) as usize / 4;
        let n = stream.count as usize * lanes;
        if n == 0 {
            return Some(Vec::new());
        }
        let bytes = (n * std::mem::size_of::<f32>()) as u64;
        let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-gpu-cook debug column readback"),
            size: bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        encoder.copy_buffer_to_buffer(&col.buffer, 0, &staging, 0, bytes);
        gpu.queue.submit(Some(encoder.finish()));
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let data = slice.get_mapped_range();
        let out = bytemuck::cast_slice::<u8, f32>(&data)[..n].to_vec();
        drop(data);
        staging.unmap();
        Some(out)
    }
}
