//! **The bounded tap** — a fixed, small subsample of a GPU-resident frame,
//! cheap enough to take every frame.
//!
//! ## Why this exists, and why it is not `debug_read`
//!
//! A GPU cook does not feed the CPU memo, so the graph panel's readouts, change
//! digest, postage stamps and probe all go blank exactly when the numbers get
//! interesting — which is what kept `PH2D_GPU_COOK` opt-in. The obvious answer,
//! "read the results back", was ruled out by measurement: `readback_tap_cost_probe`
//! reports **297 ms** to pull 4,19 M instances, worse than cooking the whole
//! thing on the CPU.
//!
//! That measurement is right and its conclusion was over-applied. It is a
//! statement about a SIZE. The panel does not want the buffer — it wants **48
//! samples**, because the CPU path it mirrors already subsamples to 48
//! (`DIGEST_SAMPLES`, `PREVIEW_POINTS`). `bounded_readback_cost_probe` measures
//! the difference: 48 elements costs **0,022-0,023 ms flat at every window size**,
//! and **+0,075 ms taken in flight** on top of a 6,989 ms cook of 4,19 M — 1% of
//! it, 0,5% of a 60 fps frame. The cost is bandwidth, not the map+poll stall
//! ([[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]]).
//!
//! So: `debug_read` pulls WHOLE columns and needs `retain_streams_for_debug`,
//! which pins buffers past [`crate::BufferPool::reclaim`] — gates only, forever.
//! This module pulls a bounded sample, pins nothing, and belongs on the frame
//! path.
//!
//! ## Strided, never a prefix
//!
//! The samples are taken by **stride**, not off the front. The panel's stamp
//! shows the SHAPE of what a node emits, and the first 48 points of a 5 000-point
//! spiral are an arc — the CPU-side docstring says exactly this and its sampler
//! strides for exactly this reason. A prefix copy would be one
//! `copy_buffer_to_buffer` and no shader; it would also be the wrong picture, so
//! this is a gather pass.
//!
//! ## One submit, one map
//!
//! Every requested column is gathered by one dispatch into a **single** shared
//! output buffer, all in one encoder — so the tap costs one submit and one map
//! however many nodes the panel is looking at. Tapping per node would be N device
//! syncs, and the sync is the part that does not amortise.
//!
//! ## What the tap does NOT answer
//!
//! **How many elements a node carried.** That comes from [`crate::shape::CookShape`],
//! exactly, because it is what the host SIZED the dispatch with. Counting the
//! tapped samples instead would report `48 inst` for a stream of four million —
//! a subsample is not a census, and the one number here that must be exact is the
//! one the tap cannot give.

use crate::{GpuCook, GpuStream};
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::NodeId;
use ph2d_nodegraph::port::Dim;
use std::collections::BTreeMap;

/// How many elements a tap takes from each column.
///
/// It mirrors the CPU readout path's `DIGEST_SAMPLES` / `PREVIEW_POINTS` (both
/// 48) — the tap exists to feed those, and a different number here would mean the
/// two paths subsample differently and their digests would disagree on a document
/// that had not changed.
pub const TAP_SAMPLES: u32 = 48;

/// The gather kernel: `out[i] = src[i * stride_elems + first]`, clamped.
///
/// `lanes` is the column's element width in f32s (from
/// [`crate::stream::element_stride`] — never re-derived here, or a `Vec3`'s
/// 16-byte padding would be re-guessed as 12 and every sample past the first
/// would misindex).
const TAP_WGSL: &str = r#"
struct TapParams {
    src_count: u32,
    lanes: u32,
    samples: u32,
    dst_offset: u32,
};
@group(0) @binding(0) var<uniform> p: TapParams;
@group(0) @binding(1) var<storage, read> src: array<f32>;
@group(0) @binding(2) var<storage, read_write> dst: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= p.samples) { return; }
    // Stride so the SHAPE survives: sample i of n maps onto element
    // `i * count / samples`, which walks the whole stream instead of its front.
    var e = i;
    if (p.samples > 0u) {
        e = (i * p.src_count) / p.samples;
    }
    if (e >= p.src_count) { e = p.src_count - 1u; }
    for (var l = 0u; l < p.lanes; l = l + 1u) {
        dst[p.dst_offset + i * p.lanes + l] = src[e * p.lanes + l];
    }
}
"#;

/// One column's slot in the packed output buffer.
struct Slot {
    node: NodeId,
    column: String,
    dim: Dim,
    /// Elements actually sampled — `min(TAP_SAMPLES, count)`, so a stream of 3
    /// yields 3 and not 48 copies of the same row.
    samples: u32,
    /// Offset into the shared `f32` output buffer.
    offset: u32,
}

/// The tap's persistent GPU resources — created once, reused every frame.
pub(crate) struct TapPipeline {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
}

impl TapPipeline {
    /// Built through the crate's own [`crate::create_pipeline`] with an AUTO
    /// layout, like every other pipeline here — a hand-written
    /// `BindGroupLayout` would be a second description of a binding set the
    /// shader already declares, and the two would drift.
    fn new(gpu: &GpuContext) -> Self {
        let pipeline = crate::create_pipeline(gpu, TAP_WGSL, "ph2d-gpu-cook tap");
        let layout = pipeline.get_bind_group_layout(0);
        Self { pipeline, layout }
    }
}

impl GpuCook {
    /// **Sample every staged node's output** into small CPU-side streams — the
    /// frame-path tap.
    ///
    /// Returns one [`Stream`] per node whose `count` is the number of SAMPLES
    /// taken, never the node's element count. Ask [`GpuCook::shape`] for the
    /// count; see the module docs for why the tap must not be asked.
    ///
    /// `None` when there is nothing to sample (no streams retained for this
    /// frame, or every node empty) — distinct from an empty map, which would say
    /// "I sampled and found nothing".
    ///
    /// Requires the cook's streams to still be alive, which is why the shell
    /// calls this inside the same frame as the cook and before the next
    /// [`crate::BufferPool::reclaim`].
    pub fn tap(&mut self, gpu: &GpuContext, samples: u32) -> Option<BTreeMap<NodeId, Stream>> {
        let streams: Vec<(NodeId, &GpuStream)> =
            self.tap_streams.iter().map(|(n, s)| (*n, s)).collect();
        if streams.is_empty() {
            return None;
        }
        // Lay out the packed destination: every (node, column) gets a contiguous
        // run of `samples * lanes` floats.
        let mut slots: Vec<Slot> = Vec::new();
        let mut total: u32 = 0;
        for (node, stream) in &streams {
            if stream.count == 0 {
                continue;
            }
            let take = samples.min(stream.count);
            for (name, col) in &stream.cols {
                let lanes = crate::stream::element_stride(col.dim) as u32 / 4;
                slots.push(Slot {
                    node: *node,
                    column: name.clone(),
                    dim: col.dim,
                    samples: take,
                    offset: total,
                });
                total += take * lanes;
            }
        }
        if total == 0 {
            return None;
        }

        let pipe = self
            .tap_pipeline
            .get_or_insert_with(|| TapPipeline::new(gpu));
        let bytes = u64::from(total) * 4;
        let dst = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-gpu-cook tap dst"),
            size: bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-gpu-cook tap"),
            });
        // One pass, one dispatch per column — all inside a single encoder, so the
        // whole tap is one submit no matter how many nodes are being watched.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ph2d-gpu-cook tap pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipe.pipeline);
            for slot in &slots {
                let stream = streams
                    .iter()
                    .find(|(n, _)| *n == slot.node)
                    .map(|(_, s)| *s)
                    .expect("slot came from this list");
                let col = stream
                    .get(&slot.column)
                    .expect("slot came from this stream");
                let lanes = crate::stream::element_stride(col.dim) as u32 / 4;
                let params = [stream.count, lanes, slot.samples, slot.offset];
                let ub = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("ph2d-gpu-cook tap params"),
                    size: 16,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: true,
                });
                ub.slice(..)
                    .get_mapped_range_mut()
                    .copy_from_slice(bytemuck::cast_slice(&params));
                ub.unmap();
                let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("ph2d-gpu-cook tap bg"),
                    layout: &pipe.layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: ub.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: col.buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: dst.as_entire_binding(),
                        },
                    ],
                });
                pass.set_bind_group(0, &bg, &[]);
                pass.dispatch_workgroups(slot.samples.div_ceil(64), 1, 1);
            }
        }
        let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-gpu-cook tap staging"),
            size: bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(&dst, 0, &staging, 0, bytes);
        gpu.queue.submit(Some(encoder.finish()));

        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv().ok()?.ok()?;
        let flat: Vec<f32> = bytemuck::cast_slice(&slice.get_mapped_range()).to_vec();
        staging.unmap();

        let mut out: BTreeMap<NodeId, Stream> = BTreeMap::new();
        for slot in &slots {
            let lanes = crate::stream::element_stride(slot.dim) as usize / 4;
            let n = slot.samples as usize;
            let base = slot.offset as usize;
            let run = &flat[base..base + n * lanes];
            let col = match slot.dim {
                Dim::Scalar => Column::Scalar(run.to_vec()),
                Dim::Vec2 => Column::Vec2(run.chunks_exact(2).map(|c| [c[0], c[1]]).collect()),
                // Vec3 pads to 16 bytes (4 lanes) — take the first three.
                Dim::Vec3 => Column::Vec3(
                    run.chunks_exact(lanes)
                        .map(|c| [c[0], c[1], c[2]])
                        .collect(),
                ),
                _ => Column::Vec4(
                    run.chunks_exact(4)
                        .map(|c| [c[0], c[1], c[2], c[3]])
                        .collect(),
                ),
            };
            out.entry(slot.node)
                .or_insert_with(|| Stream::new(n))
                .set(slot.column.clone(), col);
        }
        Some(out)
    }
}
