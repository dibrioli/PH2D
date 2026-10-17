//! `GpuStream` — the GPU-resident form of [`ph2d_nodegraph::attr::Stream`]:
//! named, typed columns living in `wgpu::Buffer`s, plus the pool that recycles
//! them frame-to-frame.
//!
//! Columns are **immutable once written**: a kernel that rewrites a column
//! writes a FRESH buffer from the pool, and untouched columns are shared by
//! `Arc` between the input and output streams. That is the implicit ping-pong
//! — no in-place hazard, no barrier bookkeeping — and it mirrors the CPU cook,
//! where every node emits a new `Stream` and clones through what it doesn't
//! change (there the clone copies; here it is a refcount).

use ph2d_gpu::GpuContext;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::port::Dim;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Bytes one element of a column occupies in its GPU buffer. `Vec3` pads to 16
/// (WGSL `array<vec3<f32>>` has a 16-byte stride — a tight 12-byte upload
/// would misindex from element 1 on); everything else is tight.
pub fn element_stride(dim: Dim) -> u64 {
    match dim {
        Dim::Scalar => 4,
        Dim::Vec2 => 8,
        Dim::Vec3 => 16,
        Dim::Vec4 => 16,
        // Matrix columns don't exist in the instance-stream convention; sized
        // conservatively should one ever appear.
        Dim::Mat2 => 16,
        Dim::Mat3 => 48,
        Dim::Mat4 => 64,
    }
}

/// One GPU-resident column: a storage buffer of `count` elements of `dim`.
#[derive(Clone)]
pub struct GpuColumn {
    pub buffer: Arc<wgpu::Buffer>,
    pub dim: Dim,
}

/// A GPU-resident stream: `count` elements with named typed columns — the
/// wgpu mirror of [`Stream`]. Deterministic iteration (`BTreeMap`, ADR-0022).
#[derive(Clone, Default)]
pub struct GpuStream {
    pub count: u32,
    pub cols: BTreeMap<String, GpuColumn>,
}

impl GpuStream {
    pub fn get(&self, name: &str) -> Option<&GpuColumn> {
        self.cols.get(name)
    }
}

/// Recycling allocator for the per-frame column buffers. Buffers are handed
/// out as `Arc`s sized to power-of-two byte classes; [`BufferPool::reclaim`]
/// (call at the top of each cook) sweeps every previously handed-out buffer
/// whose refcount fell back to 1 into the free lists, so a steady scene
/// allocates NOTHING after its first frame — the GPU analogue of the pump's
/// reused `Vec<RenderInstance>` (M0.T12).
#[derive(Default)]
pub struct BufferPool {
    /// Free buffers keyed by their exact allocated size (a pow2 class).
    free: BTreeMap<u64, Vec<Arc<wgpu::Buffer>>>,
    /// Everything handed out since the last reclaim (still referenced by the
    /// frame's `GpuStream`s until they drop).
    handed_out: Vec<Arc<wgpu::Buffer>>,
    /// Buffers this pool has ever created — the number a "steady scene
    /// allocates NOTHING" claim is about. Declaring that property without a way
    /// to observe it is how a budget rule goes years without firing
    /// ([[feedback_a_rule_that_never_observes_cannot_fire]]).
    allocations: usize,
}

impl BufferPool {
    /// Storage-column usage: compute passes read/write them, uploads seed the
    /// boundary stream, and COPY_SRC keeps every intermediate inspectable by
    /// the parity gates (the deliberate, off-hot-path readback).
    const USAGE: wgpu::BufferUsages = wgpu::BufferUsages::STORAGE
        .union(wgpu::BufferUsages::COPY_DST)
        .union(wgpu::BufferUsages::COPY_SRC);

    /// Take (or create) a storage buffer of at least `bytes`.
    pub fn acquire(&mut self, gpu: &GpuContext, bytes: u64) -> Arc<wgpu::Buffer> {
        let class = bytes.max(16).next_power_of_two();
        let buf = match self.free.get_mut(&class).and_then(Vec::pop) {
            Some(b) => b,
            None => {
                self.allocations += 1;
                Arc::new(gpu.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("ph2d-gpu-cook column"),
                    size: class,
                    usage: Self::USAGE,
                    mapped_at_creation: false,
                }))
            }
        };
        self.handed_out.push(Arc::clone(&buf));
        buf
    }

    /// How many buffers this pool has ever created — flat across a steady
    /// scene's frames, by construction.
    pub fn allocations(&self) -> usize {
        self.allocations
    }

    /// Buffers handed out that something still holds — for a sim, exactly last
    /// tick's state columns.
    pub fn retained(&self) -> usize {
        self.handed_out
            .iter()
            .filter(|b| Arc::strong_count(b) > 1)
            .count()
    }

    /// Return every no-longer-referenced buffer to the free lists. Call after
    /// the frame's `GpuStream`s have been dropped (top of the next cook).
    pub fn reclaim(&mut self) {
        for buf in std::mem::take(&mut self.handed_out) {
            if Arc::strong_count(&buf) == 1 {
                self.free.entry(buf.size()).or_default().push(buf);
            } else {
                // Still referenced — a simulation's state, held across the tick
                // by `GpuCook::prev` (ADR-0127 D1). Keep TRACKING it: dropping
                // our `Arc` here would forget the buffer, and when the sim
                // finally released it the allocation would be freed instead of
                // pooled — i.e. the steady-state sim would allocate its whole
                // state every frame, which is the one thing this pool exists to
                // prevent.
                self.handed_out.push(buf);
            }
        }
    }
}

/// Upload a CPU [`Stream`] into a [`GpuStream`] — the **explicit CPU→GPU
/// boundary** (one crossing per frame, at the seam between the CPU-cooked
/// prefix and the GPU suffix; ADR-0126 "fronteira explícita").
pub fn upload_stream(gpu: &GpuContext, pool: &mut BufferPool, stream: &Stream) -> GpuStream {
    let count = stream.count() as u32;
    let mut cols = BTreeMap::new();
    for (name, col) in stream.columns() {
        let (dim, bytes): (Dim, Vec<u8>) = match col {
            Column::Scalar(v) => (Dim::Scalar, bytemuck::cast_slice(v).to_vec()),
            Column::Vec2(v) => (Dim::Vec2, bytemuck::cast_slice(v).to_vec()),
            // Pad each element to 16 B (see [`element_stride`]).
            Column::Vec3(v) => {
                let mut padded = Vec::with_capacity(v.len() * 16);
                for e in v {
                    padded.extend_from_slice(bytemuck::cast_slice(e));
                    padded.extend_from_slice(&[0u8; 4]);
                }
                (Dim::Vec3, padded)
            }
            Column::Vec4(v) => (Dim::Vec4, bytemuck::cast_slice(v).to_vec()),
        };
        let buffer = pool.acquire(gpu, bytes.len().max(1) as u64);
        if !bytes.is_empty() {
            gpu.queue.write_buffer(&buffer, 0, &bytes);
        }
        cols.insert(name.clone(), GpuColumn { buffer, dim });
    }
    GpuStream { count, cols }
}
