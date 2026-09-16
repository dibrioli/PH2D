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
use ph2d_nodegraph::gpu::{
    ConcatFill, DerivedUniform, GpuKernel, KEEP_FLAG_COL, ROWS_COL, ReduceSpec, SourceWindow,
};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::NodeManifest;
use ph2d_nodegraph::port::Dim;
use std::collections::BTreeMap;

#[path = "stream_op_project.rs"]
mod project;

/// **O que um predicado de compactação vê além das colunas** — vazio num `Compact` (ver o
/// porquê em [`GpuCook::encode_compact`]) e cheio num `Carry` (ciclo 7), cujo predicado pergunta
/// ao estado INTEIRO se há um eco na faixa do espaçamento e deriva a janela da contagem viva.
pub(crate) struct PredicateExtras<'a> {
    /// As reduções dobradas antes do predicado, e os buffers delas.
    pub reduces: (&'static [ReduceSpec], &'a [wgpu::Buffer]),
    /// O WGSL que o predicado e as reduções veem os dois.
    pub shared: &'static str,
    /// Os uniforms derivados, com as contagens CRUAS.
    pub derived: (&'static [DerivedUniform], &'a [u32]),
}

impl PredicateExtras<'_> {
    /// Nada — o predicado de um `Compact`.
    pub const NONE: PredicateExtras<'static> = PredicateExtras {
        reduces: (&[], &[]),
        shared: "",
        derived: (&[], &[]),
    };
}

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
    /// `dst[i] = src[i·stride + lane]` — `value.attribute`'s COMPONENT mode. One
    /// pipeline serves every width because the width rides the `stride` uniform
    /// the gather already carries: asking for lane `k` of a `Vec2`, a `Vec3` or a
    /// `Vec4` is the same strided read, which is exactly why the CPU ladder has
    /// one arm for it too.
    component: wgpu::ComputePipeline,
    /// `dst[i] = degrees(atan2(y, x))` over a vec2 column — `value.attribute`'s ANGLE
    /// mode, a direcao. GRAUS porque e o que a coluna `rot` fala do outro lado da
    /// cadeia; ver `MODE_ANGLE` no crate do no.
    ///
    /// ⚠️ Este e o UNICO bracao transcendental da escada, e por isso o unico cuja
    /// paridade e por EPSILON e nao por bit: a CPU corre `libm::atan2f` (MUSL) e o
    /// device o `atan2` do vendedor. O bit-a-bit nao e a politica deste projeto
    /// (o compositor ja o declara), e o gate de paridade carrega o numero.
    angle: wgpu::ComputePipeline,
    /// `dst[(first + i)·words + w] = value[w]` — a IDENTIDADE de uma coluna na região de uma
    /// fonte que não a tem (a junção de um `Carry`, ciclo 7). Uniform próprio: ele leva um
    /// `vec4`, que o `U` dos outros não tem.
    fill: wgpu::ComputePipeline,
}

/// A região que um [`StreamOpPipes::fill`] escreve.
struct FillRegion<'a> {
    dst: &'a wgpu::Buffer,
    /// Quantos elementos.
    n: u32,
    /// Palavras `f32` por elemento (a passada da dimensão).
    words: u32,
    /// O primeiro elemento da região.
    first: u32,
    /// O valor de cada faixa (as faixas acima de 4 — a folga de um `Vec3` — levam `0`).
    value: [f32; 4],
}

fn simple_module(bindings: &str, body: &str) -> String {
    format!(
        "struct U {{ n: u32, stride: u32, lane: u32 }}\n\
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
        let angle = simple_module(
            "@group(0) @binding(1) var<storage, read> src: array<f32>;\n\
             @group(0) @binding(2) var<storage, read_write> dst: array<f32>;",
            "\x20   let x = src[2u * i];\n\
             \x20   let y = src[2u * i + 1u];\n\
             \x20   dst[i] = degrees(atan2(y, x));",
        );
        let component = simple_module(
            "@group(0) @binding(1) var<storage, read> src: array<f32>;\n\
             @group(0) @binding(2) var<storage, read_write> dst: array<f32>;",
            "\x20   dst[i] = src[i * u.stride + u.lane];",
        );
        let fill = format!(
            "struct F {{ n: u32, words: u32, first: u32, _pad: u32, value: vec4<f32> }}\n\
             @group(0) @binding(0) var<uniform> u: F;\n\
             @group(0) @binding(1) var<storage, read_write> dst: array<f32>;\n\
             @compute @workgroup_size({WG})\n\
             fn main(@builtin(global_invocation_id) gid: vec3<u32>) {{\n\
             \x20   let i = gid.x;\n\
             \x20   if (i >= u.n) {{ return; }}\n\
             \x20   for (var w = 0u; w < u.words; w = w + 1u) {{\n\
             \x20       var v = 0.0;\n\
             \x20       if (w < 4u) {{ v = u.value[w]; }}\n\
             \x20       dst[(u.first + i) * u.words + w] = v;\n\
             \x20   }}\n\
             }}\n"
        );
        StreamOpPipes {
            fill: create_pipeline(gpu, &fill, "ph2d-stream-op fill"),
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
            component: create_pipeline(gpu, &component, "ph2d-stream-op component"),
            angle: create_pipeline(gpu, &angle, "ph2d-stream-op angle"),
        }
    }

    /// Encode one identity FILL over a region ([`FillRegion`]).
    fn fill(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        r: FillRegion<'_>,
        hold: &mut Vec<wgpu::Buffer>,
    ) {
        let uni = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-stream-op fill u"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut bytes = [0u8; 32];
        bytes[0..4].copy_from_slice(&r.n.to_le_bytes());
        bytes[4..8].copy_from_slice(&r.words.to_le_bytes());
        bytes[8..12].copy_from_slice(&r.first.to_le_bytes());
        for (k, v) in r.value.iter().enumerate() {
            bytes[16 + 4 * k..20 + 4 * k].copy_from_slice(&v.to_le_bytes());
        }
        gpu.queue.write_buffer(&uni, 0, &bytes);
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-stream-op fill bg"),
            layout: &self.fill.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uni.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: r.dst.as_entire_binding(),
                },
            ],
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ph2d-stream-op fill"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.fill);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(r.n.div_ceil(WG), 1, 1);
        }
        hold.push(uni);
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
        lane: u32,
        buffers: &[&wgpu::Buffer],
        hold: &mut Vec<wgpu::Buffer>,
    ) {
        let uni = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-stream-op u"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut bytes = [0u8; 12];
        bytes[0..4].copy_from_slice(&n.to_le_bytes());
        bytes[4..8].copy_from_slice(&stride.to_le_bytes());
        bytes[8..12].copy_from_slice(&lane.to_le_bytes());
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
        dt: f64,
        inputs: &[GpuStream],
        port: usize,
        extras: PredicateExtras<'_>,
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
            .resolve(&|p| resolve_param(graph, node, manifest, p, &self.driven))
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
        // ⚠️ O orçamento conta as reduções que o predicado lê (um `Carry`), como o do estágio.
        let needed = codegen::storage_buffers(
            bindings,
            |b| gather::column_present(None, n, inputs, b),
            codegen::ExtraBuffers {
                grid: None,
                reduces: extras.reduces.0,
                luts: &[],
            },
        );
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
            dt,
            inputs,
            src.clone(),
            None,
            // A `Compact` predicate reads no whole-stream reduction: it decides per
            // element whether the element survives, and a number about the whole stream
            // would be a number about a stream that is changing (`PredicateExtras::NONE`).
            // ⚠️ Um `Carry` (ciclo 7) pede-as DE PROPÓSITO, dobradas sobre o estado CRU que
            // este predicado filtra — e com elas o canal partilhado e os derivados.
            extras.reduces,
            // Nor a LUT — a predicate samples no authored curve (A1-gpu).
            (&[], &[]),
            extras.shared,
            extras.derived,
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
                0,
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
                0,
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
                    0,
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
                    0,
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
        let fontes: Vec<&GpuStream> = ports.iter().filter_map(|p| inputs.get(*p)).collect();
        self.encode_join(gpu, encoder, &fontes, &[])
    }

    /// **A junção** — as fontes, pela ordem, ponta a ponta: a união das colunas, a dimensão
    /// da PRIMEIRA fonte não-vazia que a tem como protótipo, e onde uma fonte não a tem (ou a
    /// tem noutra dimensão) a IDENTIDADE que as `fills` dão — zeros quando nenhuma a nomeia
    /// (o `motion.combine`), o `default_for` da CPU num `Carry` (ciclo 7: um `size`/`tint` a
    /// zero apagava o eco). Fontes vazias saltam-se (a regra de snapshot da CPU).
    pub(crate) fn encode_join(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        fontes: &[&GpuStream],
        fills: &[ConcatFill],
    ) -> GpuStream {
        let live: Vec<&GpuStream> = fontes.iter().copied().filter(|s| s.count > 0).collect();
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
        let mut hold: Vec<wgpu::Buffer> = Vec::new();
        for (name, dim) in protos {
            let stride = stream::element_stride(dim);
            let dst = self.pool.acquire(gpu, u64::from(total) * stride);
            let identidade = ConcatFill::of(fills, &name, dim);
            let mut off: u64 = 0;
            for s in &live {
                let bytes = u64::from(s.count) * stride;
                match s.cols.get(&name) {
                    Some(c) if c.dim == dim => {
                        encoder.copy_buffer_to_buffer(&c.buffer, 0, &dst, off, bytes);
                    }
                    _ if identidade == [0.0; 4] => encoder.clear_buffer(&dst, off, Some(bytes)),
                    _ => {
                        let pipes = self
                            .stream_op_pipes
                            .get_or_insert_with(|| StreamOpPipes::new(gpu));
                        pipes.fill(
                            gpu,
                            encoder,
                            FillRegion {
                                dst: &dst,
                                n: s.count,
                                words: (stride / 4) as u32,
                                first: (off / stride) as u32,
                                value: identidade,
                            },
                            &mut hold,
                        );
                    }
                }
                off += bytes;
            }
            out.cols.insert(name, GpuColumn { buffer: dst, dim });
        }
        self.stream_op_hold.append(&mut hold);
        out
    }
}
