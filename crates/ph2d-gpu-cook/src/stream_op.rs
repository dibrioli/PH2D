//! **The structural stream operations** (ADR-0136): the count-changing family's
//! machinery — filter, birth-gather, concatenate, project — living ONCE in the
//! sequencer, driven by [`StreamOp`] side-metadata exactly like the grid is
//! driven by a `GridSpec`.
//!
//! The per-element MAP is the [`GpuKernel`]'s job and stays in `encode.rs`; what
//! lives here is everything whose OUTPUT is not shaped like its input:
//!
//! - [`GpuCook::encode_compact`] — order-preserving filter: a predicate kernel
//!   (a plain [`GpuKernel`] writing [`KEEP_FLAG_COL`]) → exclusive scan (the
//!   counting-sort's own [`Scan`]) → survivors scatter their row → every column
//!   gathered dense. **The one readback on a frame path**: the survivor count is
//!   8 bytes, read at the compaction seam by splitting the submit — the bounded
//!   kind (`debug_read`'s measured rule), constant in N. Order is preserved
//!   because the CPU preserves it on purpose (`sim.lifetime`: reshuffling per
//!   tick would flicker every index-based consumer).
//! - [`GpuCook::encode_source_gather`] — a [`StreamOp::SourceRows`] kernel wrote
//!   [`ROWS_COL`] (+ its own columns); the template's remaining columns are
//!   gathered at those rows, so a newborn inherits whatever the template
//!   carries without the kernel enumerating columns.
//! - [`GpuCook::encode_concat`] — `motion.combine` is `copy_buffer_to_buffer` +
//!   `clear_buffer` per column region: no shader, no readback, the count is a
//!   host-side sum.
//! - [`GpuCook::encode_project`] — `value.attribute`'s column NAME is a text
//!   param, dynamic by design, so it is resolved against the stream's column
//!   map here rather than pretending to be a static binding.

use crate::plan::resolve_param;
use crate::scan::{Scan, ScanScratch};
use crate::{
    GpuColumn, GpuCook, GpuCookError, GpuStream, codegen, create_pipeline, gather, stream,
};
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::gpu::{GpuKernel, KEEP_FLAG_COL, ROWS_COL, SourceWindow};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::NodeManifest;
use ph2d_nodegraph::port::Dim;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Cache salt for a compaction predicate's pipelines — see
/// [`GpuCook::encode_kernel_stage`]'s `cache_salt`: the predicate shares the
/// node's type id with its epilogue kernel, and presence signatures of two
/// DIFFERENT binding lists can collide.
const PREDICATE_SALT: u64 = 0x5354_5245_414d_4f50; // "STREAMOP"

const WG: u32 = codegen::WORKGROUP_SIZE;

/// The fixed pipelines of this module, compiled on first use and reused every
/// cook (the tap/grid pattern — `GpuCook` is `Default` and has no device).
pub(crate) struct StreamOpPipes {
    scan: Scan,
    /// `scan_data[i] = u32(flags[i])` — the predicate writes its verdict as an
    /// ordinary `f32` column (so it IS an ordinary kernel); the scan wants ints.
    convert: wgpu::ComputePipeline,
    /// `if flags[i] ≥ 0.5 { rows[scan[i]] = i }` — the order-preserving scatter:
    /// survivor `i`'s dense position is the count of survivors before it.
    rows: wgpu::ComputePipeline,
    /// `dst[j·stride..] = src[rows[j]·stride..]` — the shared column gather, in
    /// `f32` words so ONE pipeline serves every dim (stride is a uniform; the
    /// stride comes from [`stream::element_stride`], the same door the uploader
    /// and binder use).
    gather_u32: wgpu::ComputePipeline,
    /// The same, with `rows` as an `f32` column (a [`ROWS_COL`] a kernel wrote —
    /// exact below `ID_WRAP`, which is the id model's own ceiling).
    gather_f32: wgpu::ComputePipeline,
    /// `dst[i] = sqrt(x² + y²)` over a vec2 column — `value.attribute`'s length
    /// mode, the same expression as the CPU's (not WGSL `length()`, which
    /// carries no cross-vendor bit guarantee).
    length: wgpu::ComputePipeline,
}

fn simple_module(bindings: &str, body: &str) -> String {
    format!(
        "struct U {{ n: u32, stride: u32 }}\n\
         @group(0) @binding(0) var<uniform> u: U;\n\
         {bindings}\n\
         @compute @workgroup_size({WG})\n\
         fn main(@builtin(global_invocation_id) gid: vec3<u32>) {{\n\
         \x20   let i = gid.x;\n\
         \x20   if (i >= u.n) {{ return; }}\n\
         {body}\n\
         }}\n"
    )
}

fn gather_module(rows_ty: &str, row_expr: &str) -> String {
    simple_module(
        &format!(
            "@group(0) @binding(1) var<storage, read> rows: array<{rows_ty}>;\n\
             @group(0) @binding(2) var<storage, read> src: array<f32>;\n\
             @group(0) @binding(3) var<storage, read_write> dst: array<f32>;"
        ),
        &format!(
            "\x20   let r = {row_expr};\n\
             \x20   for (var w = 0u; w < u.stride; w = w + 1u) {{\n\
             \x20       dst[i * u.stride + w] = src[r * u.stride + w];\n\
             \x20   }}"
        ),
    )
}

impl StreamOpPipes {
    fn new(gpu: &GpuContext) -> Self {
        let convert = simple_module(
            "@group(0) @binding(1) var<storage, read> flags: array<f32>;\n\
             @group(0) @binding(2) var<storage, read_write> scan_data: array<u32>;",
            "\x20   scan_data[i] = u32(flags[i]);",
        );
        let rows = simple_module(
            "@group(0) @binding(1) var<storage, read> flags: array<f32>;\n\
             @group(0) @binding(2) var<storage, read> scan_data: array<u32>;\n\
             @group(0) @binding(3) var<storage, read_write> rows: array<u32>;",
            "\x20   if (flags[i] >= 0.5) { rows[scan_data[i]] = i; }",
        );
        let length = simple_module(
            "@group(0) @binding(1) var<storage, read> src: array<f32>;\n\
             @group(0) @binding(2) var<storage, read_write> dst: array<f32>;",
            "\x20   let x = src[2u * i];\n\
             \x20   let y = src[2u * i + 1u];\n\
             \x20   dst[i] = sqrt(x * x + y * y);",
        );
        StreamOpPipes {
            scan: Scan::new(gpu),
            convert: create_pipeline(gpu, &convert, "ph2d-stream-op convert"),
            rows: create_pipeline(gpu, &rows, "ph2d-stream-op rows"),
            gather_u32: create_pipeline(
                gpu,
                &gather_module("u32", "rows[i]"),
                "ph2d-stream-op gather",
            ),
            gather_f32: create_pipeline(
                gpu,
                &gather_module("f32", "u32(max(rows[i], 0.0))"),
                "ph2d-stream-op gather-f32",
            ),
            length: create_pipeline(gpu, &length, "ph2d-stream-op length"),
        }
    }

    /// Encode one fixed pass: `{u: (n, stride)} + buffers`, dispatched over `n`.
    /// The uniform is a fresh 16-byte buffer per dispatch (the scan's own
    /// pattern); it lands in `hold` so it outlives the submit.
    #[allow(clippy::too_many_arguments)] // private seam, mirrors `encode_kernel_stage`
    fn pass(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &wgpu::ComputePipeline,
        n: u32,
        stride: u32,
        buffers: &[&wgpu::Buffer],
        hold: &mut Vec<wgpu::Buffer>,
    ) {
        let uni = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-stream-op u"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut bytes = [0u8; 8];
        bytes[0..4].copy_from_slice(&n.to_le_bytes());
        bytes[4..8].copy_from_slice(&stride.to_le_bytes());
        gpu.queue.write_buffer(&uni, 0, &bytes);
        let mut entries = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: uni.as_entire_binding(),
        }];
        for (k, b) in buffers.iter().enumerate() {
            entries.push(wgpu::BindGroupEntry {
                binding: (k + 1) as u32,
                resource: b.as_entire_binding(),
            });
        }
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-stream-op"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &entries,
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ph2d-stream-op"),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(n.div_ceil(WG), 1, 1);
        }
        hold.push(uni);
    }
}

impl GpuCook {
    /// **The order-preserving compaction** (ADR-0136 §1–2): filter `inputs[port]`
    /// by `predicate`, returning the dense survivor stream. Splits the submit to
    /// read the survivor count back (8 bytes); on return `encoder` is a FRESH
    /// encoder the caller keeps encoding into.
    #[allow(clippy::too_many_arguments)] // private seam of `cook`, mirrors the stage encoder
    pub(crate) fn encode_compact(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        slot_idx: usize,
        predicate: &GpuKernel,
        graph: &Graph,
        node: NodeId,
        manifest: &'static NodeManifest,
        playhead: f64,
        inputs: &[GpuStream],
        port: usize,
    ) -> Result<GpuStream, GpuCookError> {
        let src = inputs.get(port).cloned().unwrap_or_default();
        let n = src.count;
        if n == 0 {
            return Ok(GpuStream::default());
        }
        // The same refusals the main path applies before any dispatch — the
        // predicate is a dispatch like any other, and skipping them here would
        // reopen exactly the divergence the audit closed.
        let bindings = predicate
            .resolve(&|p| resolve_param(graph, node, manifest, p))
            .bindings;
        if let Some((bport, len)) = gather::broadcast_length_mismatch(None, n, bindings, |b| {
            inputs
                .get(b.port)
                .map(|s| (s.count, s.cols.contains_key(b.column)))
        }) {
            return Err(GpuCookError::BroadcastLengthMismatch {
                ty: manifest.id,
                port: bport,
                len,
                count: n,
            });
        }
        let needed =
            codegen::storage_bindings(bindings, |b| gather::column_present(None, n, inputs, b));
        let limit = gpu.device.limits().max_storage_buffers_per_shader_stage;
        if needed > limit {
            return Err(GpuCookError::TooManyBindings(manifest.id, needed, limit));
        }

        // 1. The predicate — an ordinary kernel dispatch whose one visible
        //    product is the KEEP_FLAG_COL it wrote.
        let pred_out = self.encode_kernel_stage(
            gpu,
            encoder,
            slot_idx,
            PREDICATE_SALT,
            predicate,
            graph,
            node,
            manifest,
            SourceWindow::of_count(n as usize),
            playhead,
            inputs,
            src.clone(),
            None,
            // A compaction predicate reads no whole-stream reduction: it decides
            // per element whether the element survives, and a number about the
            // whole stream would be a number about a stream that is changing.
            (&[], &[]),
            // Nor a LUT — a predicate samples no authored curve (A1-gpu).
            (&[], &[]),
        );
        let Some(flags) = pred_out.cols.get(KEEP_FLAG_COL).map(|c| c.buffer.clone()) else {
            // A predicate that does not write the flag is an authoring bug in the
            // node's own crate; refuse the frame (the CPU stays canonical) rather
            // than compact on garbage.
            debug_assert!(false, "compact predicate wrote no {KEEP_FLAG_COL}");
            return Err(GpuCookError::MalformedStreamOp(manifest.id));
        };

        // 2–4. flags → scan(exclusive, in place) → survivors scatter their rows.
        let scan_buf = self.pool.acquire(gpu, u64::from(n) * 4);
        let rows_buf = self.pool.acquire(gpu, u64::from(n) * 4);
        let mut hold: Vec<wgpu::Buffer> = Vec::new();
        let mut scratch = ScanScratch::default();
        {
            let pipes = self
                .stream_op_pipes
                .get_or_insert_with(|| StreamOpPipes::new(gpu));
            pipes.pass(
                gpu,
                encoder,
                &pipes.convert,
                n,
                1,
                &[&flags, &scan_buf],
                &mut hold,
            );
            pipes
                .scan
                .exclusive(gpu, encoder, &scan_buf, n, &mut scratch);
            pipes.pass(
                gpu,
                encoder,
                &pipes.rows,
                n,
                1,
                &[&flags, &scan_buf, &rows_buf],
                &mut hold,
            );
        }

        // 5. The seam: total = scan[n-1] + flag[n-1], read back through a split
        //    submit. 8 bytes — the BOUNDED readback (`debug_read`'s rule); the
        //    cost is the sync, constant in N, measured in the zone scale gate.
        let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-stream-op total"),
            size: 8,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(&scan_buf, u64::from(n - 1) * 4, &staging, 0, 4);
        encoder.copy_buffer_to_buffer(&flags, u64::from(n - 1) * 4, &staging, 4, 4);
        let done = std::mem::replace(
            encoder,
            gpu.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("ph2d-gpu-cook chain (post-compact)"),
                }),
        );
        gpu.queue.submit(Some(done.finish()));
        // `hold`/`scratch`/locals may drop now — wgpu keeps submitted resources
        // alive until the device is done with them.
        drop(hold);
        drop(scratch);
        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv()
            .expect("map_async callback ran")
            .expect("compaction count map succeeded");
        let (scan_last, flag_last) = {
            let data = slice.get_mapped_range();
            (
                u32::from_le_bytes(data[0..4].try_into().expect("4 bytes")),
                f32::from_le_bytes(data[4..8].try_into().expect("4 bytes")),
            )
        };
        staging.unmap();
        let total = scan_last + u32::from(flag_last >= 0.5);
        if total == 0 {
            return Ok(GpuStream::default());
        }

        // 6. Gather EVERY column of the source at the dense rows — the whole
        //    stream survives at a smaller count, in its original order.
        let mut hold: Vec<wgpu::Buffer> = Vec::new();
        let mut out = GpuStream {
            count: total,
            cols: BTreeMap::new(),
        };
        for (name, col) in &src.cols {
            let stride = stream::element_stride(col.dim);
            let dst = self.pool.acquire(gpu, u64::from(total) * stride);
            {
                let pipes = self.stream_op_pipes.as_ref().expect("built above");
                pipes.pass(
                    gpu,
                    encoder,
                    &pipes.gather_u32,
                    total,
                    (stride / 4) as u32,
                    &[&rows_buf, &col.buffer, &dst],
                    &mut hold,
                );
            }
            out.cols.insert(
                name.clone(),
                GpuColumn {
                    buffer: dst,
                    dim: col.dim,
                },
            );
        }
        self.stream_op_hold.append(&mut hold);
        self.stream_op_hold_bufs.push(rows_buf);
        Ok(out)
    }

    /// Gather the template's remaining columns at the [`ROWS_COL`] a
    /// [`ph2d_nodegraph::gpu::StreamOp::SourceRows`] kernel wrote (ADR-0136):
    /// a newborn inherits every template column the kernel did not write itself
    /// (the CPU's `newborns` copies all-but-`id`). No readback — the count was
    /// the count law's.
    pub(crate) fn encode_source_gather(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        out: GpuStream,
        template: Option<&GpuStream>,
        count: u32,
    ) -> GpuStream {
        let Some(rows) = out.cols.get(ROWS_COL).map(|c| c.buffer.clone()) else {
            debug_assert!(false, "SourceRows kernel wrote no {ROWS_COL}");
            return out;
        };
        let mut out = out;
        out.cols.remove(ROWS_COL);
        let Some(tpl) = template.filter(|t| t.count > 0) else {
            return out; // an empty template contributes nothing (the CPU's zeros-by-absence)
        };
        let mut hold: Vec<wgpu::Buffer> = Vec::new();
        let gathered: Vec<(String, GpuColumn)> = tpl
            .cols
            .iter()
            .filter(|(name, _)| !out.cols.contains_key(*name))
            .map(|(name, col)| {
                let stride = stream::element_stride(col.dim);
                let dst = self.pool.acquire(gpu, u64::from(count) * stride);
                (
                    name.clone(),
                    GpuColumn {
                        buffer: dst,
                        dim: col.dim,
                    },
                )
            })
            .collect();
        {
            let pipes = self
                .stream_op_pipes
                .get_or_insert_with(|| StreamOpPipes::new(gpu));
            for (name, dstcol) in &gathered {
                let src = &tpl.cols[name];
                pipes.pass(
                    gpu,
                    encoder,
                    &pipes.gather_f32,
                    count,
                    (stream::element_stride(src.dim) / 4) as u32,
                    &[&rows, &src.buffer, &dstcol.buffer],
                    &mut hold,
                );
            }
        }
        for (name, col) in gathered {
            out.cols.insert(name, col);
        }
        self.stream_op_hold.append(&mut hold);
        out
    }

    /// `motion.combine` (ADR-0136): the listed ports laid end to end — column
    /// union, first-seen dim as the prototype, zeros where an input lacks the
    /// column **or carries it at another dim** (the CPU's variant-match rule).
    /// Pure copies and clears; the count is a host-side sum.
    pub(crate) fn encode_concat(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        ports: &[usize],
        inputs: &[GpuStream],
    ) -> GpuStream {
        // Non-empty inputs in port order — the CPU's snapshot rule.
        let live: Vec<&GpuStream> = ports
            .iter()
            .filter_map(|p| inputs.get(*p))
            .filter(|s| s.count > 0)
            .collect();
        let total64: u64 = live.iter().map(|s| u64::from(s.count)).sum();
        let total = total64.min(u64::from(u32::MAX)) as u32;
        if total == 0 {
            return GpuStream::default();
        }
        // Ordered column union: the prototype dim is the FIRST live input
        // carrying the name (the CPU's `find_map` over snaps).
        let mut protos: Vec<(String, Dim)> = Vec::new();
        for s in &live {
            for (name, col) in &s.cols {
                if !protos.iter().any(|(n, _)| n == name) {
                    protos.push((name.clone(), col.dim));
                }
            }
        }
        let mut out = GpuStream {
            count: total,
            cols: BTreeMap::new(),
        };
        for (name, dim) in protos {
            let stride = stream::element_stride(dim);
            let dst = self.pool.acquire(gpu, u64::from(total) * stride);
            let mut off: u64 = 0;
            for s in &live {
                let bytes = u64::from(s.count) * stride;
                match s.cols.get(&name) {
                    Some(c) if c.dim == dim => {
                        encoder.copy_buffer_to_buffer(&c.buffer, 0, &dst, off, bytes);
                    }
                    _ => encoder.clear_buffer(&dst, off, Some(bytes)),
                }
                off += bytes;
            }
            out.cols.insert(name, GpuColumn { buffer: dst, dim });
        }
        out
    }

    /// `value.attribute` (ADR-0136): project the column NAMED BY A TEXT PARAM as
    /// the value field `v`. The CPU ladder, exactly: a scalar column in scalar
    /// mode is a copy; a vec2 column in length mode is the magnitude kernel;
    /// anything else — missing, mistyped, wrong dim for the mode — is zeros at
    /// full length, never an error and never an empty broadcast.
    #[allow(clippy::too_many_arguments)] // private seam of `cook`
    pub(crate) fn encode_project(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        graph: &Graph,
        node: NodeId,
        manifest: &'static NodeManifest,
        text_param: &str,
        mode_param: &str,
        inputs: &[GpuStream],
    ) -> GpuStream {
        const MODE_LENGTH: i32 = 1; // `value.attribute`'s own constant
        let src_stream = inputs.first().cloned().unwrap_or_default();
        let n = src_stream.count;
        if n == 0 {
            return GpuStream::default();
        }
        let name = graph
            .node_text_param_overrides(node)
            .and_then(|m| m.get(text_param))
            .map(String::as_str)
            .unwrap_or("");
        let mode = resolve_param(graph, node, manifest, mode_param).round() as i32;
        let dst: Arc<wgpu::Buffer> = self.pool.acquire(gpu, u64::from(n) * 4);
        let mut hold: Vec<wgpu::Buffer> = Vec::new();
        match (src_stream.cols.get(name), mode) {
            (Some(c), m) if c.dim == Dim::Scalar && m != MODE_LENGTH => {
                encoder.copy_buffer_to_buffer(&c.buffer, 0, &dst, 0, u64::from(n) * 4);
            }
            (Some(c), MODE_LENGTH) if c.dim == Dim::Vec2 => {
                let pipes = self
                    .stream_op_pipes
                    .get_or_insert_with(|| StreamOpPipes::new(gpu));
                pipes.pass(
                    gpu,
                    encoder,
                    &pipes.length,
                    n,
                    1,
                    &[&c.buffer, &dst],
                    &mut hold,
                );
            }
            _ => encoder.clear_buffer(&dst, 0, Some(u64::from(n) * 4)),
        }
        self.stream_op_hold.append(&mut hold);
        let mut out = GpuStream {
            count: n,
            cols: BTreeMap::new(),
        };
        out.cols.insert(
            "v".to_string(),
            GpuColumn {
                buffer: dst,
                dim: Dim::Scalar,
            },
        );
        out
    }
}
