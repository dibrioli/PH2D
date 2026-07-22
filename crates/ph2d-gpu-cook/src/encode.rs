//! **Encoding one kernel stage** — pipeline, uniform, bind group, dispatch.
//!
//! The sibling of [`crate::GpuCook::cook`], which is the WALK: it decides which
//! stages run and in what order, resolves each one's inputs, and asks the count
//! law how wide the stage is. Everything that happens INSIDE one stage lives
//! here, because the two answer different questions and only the walk's is about
//! the graph.
//!
//! The seam between them is deliberately narrow: `cook` hands over the resolved
//! input streams, the base, and the stage's [`SourceWindow`] — one struct
//! carrying both the dispatch size and (for a generator) the identity window, so
//! the two numbers cannot be derived twice and disagree.

use crate::gather::{column_present, gather_key_port, gather_prev_n};
use crate::grid::{Grid, GridBuffers};
use crate::plan::resolve_param;
use crate::{
    CachedPipeline, GpuColumn, GpuCook, GpuStream, UNIFORM_BYTES, codegen, create_pipeline, stream,
};
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::gpu::{ColumnBinding, GpuKernel, GridSpec, ReduceSpec, SourceWindow};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::NodeManifest;

impl GpuCook {
    /// Encode one kernel stage: resolve/compile the pipeline for the inputs'
    /// column sets, allocate output columns, write the uniform, dispatch, and
    /// return the node's output stream.
    /// `stage_idx` doubles as the uniform-slot index, so two dispatches of one
    /// stage (a compaction predicate before the node's own kernel, ADR-0136)
    /// must come with DISTINCT indices — `queue.write_buffer` is staged at
    /// submit, so a shared slot would hand both dispatches the second uniform.
    /// `cache_salt` keeps their pipelines apart for the same reason: the cache
    /// is keyed `(type, presence signature)`, and a predicate whose presence
    /// bits happen to match the kernel's would otherwise dispatch the wrong
    /// module. `0` for a node's own kernel.
    #[allow(clippy::too_many_arguments)] // private seam of `cook`
    pub(crate) fn encode_kernel_stage(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        stage_idx: usize,
        cache_salt: u64,
        kernel: &GpuKernel,
        graph: &Graph,
        node: NodeId,
        manifest: &'static NodeManifest,
        window: SourceWindow,
        playhead: f64,
        inputs: &[GpuStream],
        base: GpuStream,
        grid: Option<(&GridSpec, &GridBuffers)>,
        reduces: (&'static [ReduceSpec], &[wgpu::Buffer]),
    ) -> GpuStream {
        use codegen::BindingPlan;

        // One answer, carried: `cook` already asked the count law, and the count
        // it dispatches must be the same number the kernel is told about.
        let count = window.count.min(u32::MAX as usize) as u32;
        // **The variant, resolved once for this dispatch.** Everything below — the
        // module text, the bind group, the cache key it is stored under — must
        // describe the SAME kernel, so it is asked for once and threaded through
        // rather than re-resolved per site (`GpuKernel::resolve`).
        let kernel = kernel.resolve(&|name| resolve_param(graph, node, manifest, name));
        let bindings = kernel.bindings;

        // Is the binding's column readable off its port? The CPU's re-seed rule
        // (a state whose count no longer matches the live set was rebuilt, so
        // pair nothing — `motion.integrate`'s `pairing`: `_ if sn == n`),
        // expressed once and generally by [`column_present`]. Under an active
        // id-gather (ADR-0130) the state ports are length-decoupled (paired by
        // id, not position); everywhere else it is the dispatch-length test the
        // Fase 1/2 kernels already had.
        let gather_port = gather_key_port(bindings, inputs, count);
        let present = |b: &ColumnBinding| column_present(gather_port, count, inputs, b);
        let port_names: Vec<&str> = manifest.inputs.iter().map(|p| p.name).collect();

        let ty_key = manifest.id.0 ^ cache_salt;
        let sig = codegen::presence_signature(bindings, present);
        self.kernel_pipelines
            .entry((ty_key, sig))
            .or_insert_with(|| {
                let src = codegen::kernel_module(
                    kernel,
                    bindings,
                    &port_names,
                    grid.map(|(s, _)| s),
                    reduces.0,
                    present,
                );
                CachedPipeline {
                    pipeline: create_pipeline(gpu, &src, manifest.name),
                }
            });

        // Uniform: [count, playhead, params…, gather_prev_n?] — the layout
        // `kernel_module` declared, zero-padded to the slot size. `gather_prev_n`
        // is appended AFTER the params (matching the generated struct) only when
        // the gather is active (ADR-0130).
        let mut uni = [0u8; UNIFORM_BYTES as usize];
        uni[0..4].copy_from_slice(&count.to_le_bytes());
        uni[4..8].copy_from_slice(&(playhead as f32).to_le_bytes());
        for (k, name) in kernel.params.iter().enumerate() {
            let v = resolve_param(graph, node, manifest, name);
            let at = 8 + k * 4;
            uni[at..at + 4].copy_from_slice(&v.to_le_bytes());
        }
        if let Some(key_port) = gather_port {
            let at = 8 + kernel.params.len() * 4;
            uni[at..at + 4].copy_from_slice(&gather_prev_n(inputs, key_port).to_le_bytes());
        }
        // The generator's window, in the layout `kernel_module` declared — the
        // SAME question, asked of the same function (`codegen::declares_window`).
        // A node is never both a gatherer and a generator (a generator has no
        // input to gather FROM), but the offset accounts for both so the layout
        // is total rather than true-by-luck.
        let has_window = codegen::declares_window(kernel.count_law.is_some(), &port_names);
        let has_src_n = codegen::declares_src_n(kernel.count_law.is_some(), &port_names);
        let window_at = 8 + kernel.params.len() * 4 + usize::from(gather_port.is_some()) * 4;
        if has_window {
            uni[window_at..window_at + 4].copy_from_slice(&window.first.to_le_bytes());
            uni[window_at + 4..window_at + 8].copy_from_slice(&window.age_first.to_le_bytes());
        }
        // Input port 0's length, for a minted-window kernel with inputs
        // (ADR-0136 — `sim.spawn`'s template arithmetic).
        if has_src_n {
            let src_n = inputs.first().map_or(0, |s| s.count);
            let at = window_at + usize::from(has_window) * 8;
            uni[at..at + 4].copy_from_slice(&src_n.to_le_bytes());
        }
        // Which input ports carry exactly one element — the broadcast bits, last
        // in the layout so nothing above them ever moves.
        if codegen::broadcasts_anything(bindings) {
            let counts: Vec<u32> = inputs.iter().map(|s| s.count).collect();
            let at = window_at + usize::from(has_window) * 8 + usize::from(has_src_n) * 4;
            uni[at..at + 4].copy_from_slice(&codegen::broadcast_mask(&counts).to_le_bytes());
        }
        // The grid's bucket count + cell, LAST in the layout (ADR-0140). `cell`
        // is re-resolved here from the SAME param the build read, so the body's
        // `grid_bucket_of` bins exactly as the build did.
        if let Some((spec, gb)) = grid {
            let grid_at = window_at
                + usize::from(has_window) * 8
                + usize::from(has_src_n) * 4
                + usize::from(codegen::broadcasts_anything(bindings)) * 4;
            let cell = resolve_param(graph, node, manifest, spec.cell_param);
            uni[grid_at..grid_at + 4].copy_from_slice(&gb.num_buckets.to_le_bytes());
            uni[grid_at + 4..grid_at + 8].copy_from_slice(&cell.to_le_bytes());
        }
        let uniform = self.uniform_slot(gpu, stage_idx);
        gpu.queue.write_buffer(uniform, 0, &uni);

        // Bind group: uniform, then read buffers, then fresh write buffers —
        // the exact order `codegen::kernel_module` assigned.
        let plans = codegen::plan_bindings(bindings, present);
        let mut new_cols: Vec<(String, GpuColumn)> = Vec::new();
        let mut entries: Vec<wgpu::BindGroupEntry> = Vec::new();
        // (entries borrow buffers; collect the write buffers first)
        let mut write_buffers: Vec<(usize, GpuColumn)> = Vec::new();
        for (i, (b, (_, write))) in bindings.iter().zip(&plans).enumerate() {
            if matches!(write, Some(BindingPlan::WriteBuffer)) {
                let bytes = u64::from(count) * stream::element_stride(b.dim);
                let buffer = self.pool.acquire(gpu, bytes);
                write_buffers.push((i, GpuColumn { buffer, dim: b.dim }));
            }
        }
        let uniform = &self.uniforms[stage_idx];
        entries.push(wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform.as_entire_binding(),
        });
        let mut slot = 1u32;
        for (b, (read, _)) in bindings.iter().zip(&plans) {
            if matches!(read, Some(BindingPlan::ReadBuffer)) {
                let col = inputs[b.port].cols.get(b.column).expect("presence checked");
                entries.push(wgpu::BindGroupEntry {
                    binding: slot,
                    resource: col.buffer.as_entire_binding(),
                });
                slot += 1;
            }
        }
        for (i, col) in &write_buffers {
            entries.push(wgpu::BindGroupEntry {
                binding: slot,
                resource: col.buffer.as_entire_binding(),
            });
            slot += 1;
            new_cols.push((bindings[*i].column.to_string(), col.clone()));
        }
        // The grid arrays, appended LAST — the exact order `kernel_module` gave
        // `grid_starts` then `grid_sorted` their slots.
        if let Some((_, gb)) = grid {
            entries.push(wgpu::BindGroupEntry {
                binding: slot,
                resource: gb.starts.as_entire_binding(),
            });
            slot += 1;
            entries.push(wgpu::BindGroupEntry {
                binding: slot,
                resource: gb.sorted.as_entire_binding(),
            });
            slot += 1;
        }
        // The reduction results, LAST — the exact order `kernel_module` declared
        // them (spec order, after the grid). One buffer per spec; the sequencer
        // filled them into this same encoder before this pass.
        for buf in reduces.1 {
            entries.push(wgpu::BindGroupEntry {
                binding: slot,
                resource: buf.as_entire_binding(),
            });
            slot += 1;
        }
        let _ = slot;
        let pipeline = &self
            .kernel_pipelines
            .get(&(ty_key, sig))
            .expect("inserted above")
            .pipeline;
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-gpu-cook stage"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &entries,
        });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(manifest.name),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(count.div_ceil(codegen::WORKGROUP_SIZE), 1, 1);
        }

        // The node's output stream: the base (port 0) rides through, consumed
        // columns are dropped, written ones replace. The CPU nodes are written
        // the same way — copy the primary input, rewrite my channels — so this
        // threading IS the shared semantic, not a GPU convention.
        //
        // **Unless the node emits a different KIND of stream.** `value.lfo` and
        // friends take instances and emit a VALUE stream: one `v` column and
        // nothing else. Riding the base there would hand downstream a VALUE stream
        // still carrying `P`/`Index`, which the CPU's does not have — not an ε, a
        // different shape, surfacing far away as a node reading a column that
        // should be gone. The manifest already states it (port 0 in vs out), so no
        // kernel declares anything, and it is a NO-OP for every kernel so far.
        let rides_base = match (manifest.inputs.first(), manifest.outputs.first()) {
            (Some(i), Some(o)) => o.ty == i.ty,
            _ => true,
        };
        let mut out = if rides_base {
            base
        } else {
            GpuStream::default()
        };
        out.count = count;
        for b in bindings.iter().filter(|b| b.access.consumes()) {
            out.cols.remove(b.column);
        }
        for (name, col) in new_cols {
            out.cols.insert(name, col);
        }
        out
    }

    /// Build the neighbourhood grid for a stage (ADR-0140 D2) into the cook's
    /// encoder, over the position column the [`GridSpec`] names on its port. The
    /// grid service is compiled on first use, like the tap pipeline.
    #[allow(clippy::too_many_arguments)] // private seam of `cook`, mirrors the stage encoder
    pub(crate) fn build_grid(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        spec: &GridSpec,
        inputs: &[GpuStream],
        graph: &Graph,
        node: NodeId,
        manifest: &NodeManifest,
    ) -> GridBuffers {
        let n = inputs.get(spec.port).map_or(0, |s| s.count);
        let cell = resolve_param(graph, node, manifest, spec.cell_param);
        let pos = inputs
            .get(spec.port)
            .and_then(|s| s.cols.get(spec.column))
            .map(|c| c.buffer.clone());
        let Self {
            grid, grid_scratch, ..
        } = self;
        let g = grid.get_or_insert_with(|| Grid::new(gpu));
        match &pos {
            Some(buf) => g.build(gpu, encoder, buf, n, cell, grid_scratch),
            None => {
                // No position column (tick 0 / empty `pre` state): `n` is 0, so the
                // count/scatter passes dispatch nothing and this dummy is never
                // read — the grid is empty and the kernel's seed branch ignores it.
                let dummy = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("ph2d-grid dummy pos"),
                    size: 8,
                    usage: wgpu::BufferUsages::STORAGE,
                    mapped_at_creation: false,
                });
                g.build(gpu, encoder, &dummy, n, cell, grid_scratch)
            }
        }
    }
}
