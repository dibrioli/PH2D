//! WGSL module generation: wraps a node's [`GpuKernel`] body in the bindings,
//! uniforms and entry point the sequencer dispatches.
//!
//! The module is a pure function of `(kernel, which readable columns the
//! input stream actually carries)` — params and playhead are UNIFORMS, so a
//! slider drag or a playing clock never recompiles; only a graph rewire that
//! changes a column's presence mints a new pipeline (cached by
//! [`crate::GpuCook`] keyed on exactly that signature).
//!
//! Contract seen by a kernel body (documented on [`GpuKernel`]):
//! - `i` — the element index (one invocation per element, `0..params.count`);
//! - `params.count`, `params.playhead`, `params.<name>` per declared param;
//! - `read_<col>(i)` — the bound port's column value, or the binding's declared
//!   identity when that stream lacks the column (generated as a constant
//!   function, the same absent-column fallback the CPU nodes apply);
//! - `HAS_<col>` — a `bool` const: was the column actually there, or is
//!   `read_<col>` handing back the identity? (The CPU nodes that *branch* on
//!   absence rather than substituting need this; see [`GpuKernel`].)
//! - `write_<col>(i, v)` — writes the node's output column; a no-op (and no
//!   buffer) for an absent [`ColumnAccess::ReadWriteExisting`] column.
//!
//! On a **multi-input** node every reader is qualified by port name
//! (`read_rest_P`, `HAS_forces_vel`) — see [`accessor_suffix`].

use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, GridSpec, ReduceSpec};
use ph2d_nodegraph::port::Dim;

pub use crate::field_name::wgsl_field;

/// One invocation per element; must match the generated `@workgroup_size`.
pub const WORKGROUP_SIZE: u32 = 256;

/// The identifier suffix a binding's `read_*` / `HAS_*` symbols carry.
///
/// Single-input (and generator) kernels keep the bare column name — the input
/// is unambiguous, and every Fase 1/2 kernel body is written against it. A node
/// with **several** input ports qualifies by port name, because the same column
/// can be bound on two ports (`motion.integrate` reads `vel` from both `rest`
/// and `forces`) and a bare `read_vel` would silently resolve to whichever came
/// first. Writers are never qualified: a node has one output.
pub fn accessor_suffix(port_names: &[&str], b: &ColumnBinding) -> String {
    match port_names.len() > 1 {
        true => format!(
            "{}_{}",
            port_names.get(b.port).copied().unwrap_or("in"),
            b.column
        ),
        false => b.column.to_string(),
    }
}

/// The WGSL scalar/vector type of a column element.
pub fn wgsl_type(dim: Dim) -> &'static str {
    match dim {
        Dim::Scalar => "f32",
        Dim::Vec2 => "vec2<f32>",
        Dim::Vec3 => "vec3<f32>",
        Dim::Vec4 => "vec4<f32>",
        // Matrices are not part of the instance-stream convention (plan-time
        // eligibility never admits one); a placeholder keeps this total.
        Dim::Mat2 => "mat2x2<f32>",
        Dim::Mat3 => "mat3x3<f32>",
        Dim::Mat4 => "mat4x4<f32>",
    }
}

/// A WGSL literal for `identity`'s first `dim`-many lanes. Shared with the
/// reduce map pass, whose absent-column form folds exactly this constant.
pub fn identity_literal(dim: Dim, identity: [f32; 4]) -> String {
    let lane = |v: f32| format!("{v:?}");
    match dim {
        Dim::Scalar => lane(identity[0]),
        Dim::Vec2 => format!("vec2<f32>({}, {})", lane(identity[0]), lane(identity[1])),
        Dim::Vec3 => format!(
            "vec3<f32>({}, {}, {})",
            lane(identity[0]),
            lane(identity[1]),
            lane(identity[2])
        ),
        _ => format!(
            "vec4<f32>({}, {}, {}, {})",
            lane(identity[0]),
            lane(identity[1]),
            lane(identity[2]),
            lane(identity[3])
        ),
    }
}

/// How one binding materializes in a generated module, given the input
/// stream's actual columns. Also the bind-group recipe the sequencer follows —
/// module text and bind group are derived from the SAME decisions, so they
/// cannot drift.
#[derive(Debug, PartialEq, Eq)]
pub enum BindingPlan {
    /// `@binding(n) var<storage, read>` on the input column's buffer.
    ReadBuffer,
    /// No buffer — `read_<col>` returns the declared identity.
    ReadIdentity,
    /// `@binding(n) var<storage, read_write>` on a fresh output buffer.
    WriteBuffer,
    /// No buffer — `write_<col>` is a no-op (`ReadWriteExisting`, absent).
    WriteDropped,
}

/// The per-binding plan for a kernel against a concrete input column set:
/// `(read plan, write plan)` in `kernel.bindings` order. `present` answers
/// "does the binding's port carry this column?".
pub fn plan_bindings(
    bindings: &[ColumnBinding],
    mut present: impl FnMut(&ColumnBinding) -> bool,
) -> Vec<(Option<BindingPlan>, Option<BindingPlan>)> {
    bindings
        .iter()
        .map(|b| {
            let here = b.access.reads() && present(b);
            let read = b.access.reads().then_some(if here {
                BindingPlan::ReadBuffer
            } else {
                BindingPlan::ReadIdentity
            });
            // **Ask [`ColumnAccess::writes`], never re-enumerate it.** This used
            // to list the accesses that write nothing (`Read`, `Consume`, the
            // `RefuseIfPresent` that binds nothing on either side, the
            // `GatherKey` that only reads an id) and default everything else to
            // a write — so the NEXT read-only access silently became a writer.
            // `ReadBroadcast` was that access, and the failure was not subtle
            // for once: two broadcast ports both claimed to write the same
            // column, and naga rejected `redefinition of out_v`. It would have
            // been subtle for a kernel with only ONE such port.
            // [[feedback_a_condition_that_enumerates_its_readers_rots]]
            let write = match b.access {
                // The one shape a boolean cannot carry: absent, so no buffer,
                // but the body still needs a `write_` symbol to call (a no-op).
                ColumnAccess::ReadWriteExisting if !here => Some(BindingPlan::WriteDropped),
                a if a.writes(here) => Some(BindingPlan::WriteBuffer),
                _ => None,
            };
            (read, write)
        })
        .collect()
}

/// How many **storage buffers** the generated module binds for this column set
/// — one per `ReadBuffer` plus one per `WriteBuffer`, which is exactly what
/// [`kernel_module`] emits and therefore what the device's
/// `max_storage_buffers_per_shader_stage` is counted against. Derived by
/// replaying [`plan_bindings`], so it cannot drift from the module it describes.
pub fn storage_bindings(
    bindings: &[ColumnBinding],
    present: impl FnMut(&ColumnBinding) -> bool,
) -> u32 {
    plan_bindings(bindings, present)
        .iter()
        .map(|(read, write)| {
            u32::from(matches!(read, Some(BindingPlan::ReadBuffer)))
                + u32::from(matches!(write, Some(BindingPlan::WriteBuffer)))
        })
        .sum()
}

/// The presence signature a pipeline is cached under: one bit per binding, in
/// declaration order — **did it find its column?**, the single `here` that
/// [`plan_bindings`] branches on. Two frames whose streams carry the same
/// columns (the steady state) hit the same compiled pipeline; two that differ
/// anywhere cannot, because for a fixed kernel `here` determines the module's
/// text exactly.
///
/// It must be derived from `here` and not from the resulting [`BindingPlan`]s:
/// a [`ColumnAccess::ReadWrite`] binding emits a `WriteBuffer` whether or not
/// its column was there, so "binds any buffer" is TRUE in both cases and the
/// two modules — which differ by a whole read binding — would collide on one
/// cache entry. wgpu then rejects the bind group against the wrong layout
/// (a crash), and it stays invisible until something makes a column's presence
/// CHANGE: two nodes of one type at different points of a chain, or a
/// simulation, whose state is absent on tick 0 and present on tick 1.
pub fn presence_signature(
    bindings: &[ColumnBinding],
    mut present: impl FnMut(&ColumnBinding) -> bool,
) -> u64 {
    // **The binding SET is part of the key, not just the presence bits.** A
    // param-dependent kernel (`GpuKernel::variant_by_param`) runs different
    // variants under one node type, and two of them can easily land on the same
    // bit pattern — `motion.drive`'s `P` variant with `{P here, falloff absent,
    // v here}` and its `rot` variant with `{rot here, falloff absent, v here}`
    // are both `0b101`. Keyed on bits alone they would share one compiled
    // pipeline, and wgpu would then validate a bind group against the WRONG
    // layout: a crash, and only for documents that switch a node's channel.
    let mut sig = bindings.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
        let mut h = h;
        for byte in b.column.as_bytes() {
            h = (h ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
        }
        h = (h ^ (b.dim as u64)).wrapping_mul(0x100_0000_01b3);
        h = (h ^ (b.access as u64)).wrapping_mul(0x100_0000_01b3);
        (h ^ (b.port as u64)).wrapping_mul(0x100_0000_01b3)
    });
    for (i, b) in bindings.iter().enumerate() {
        let here = b.access.reads() && present(b);
        sig ^= (here as u64) << (i % 64);
    }
    sig
}

/// Does any binding broadcast ([`ColumnAccess::ReadBroadcast`])? Decides whether
/// the module carries the `bcast_one` uniform — asked by the codegen that
/// DECLARES it and the sequencer that PACKS it, for the reason on
/// [`declares_window`].
///
/// **One bitmask, not one field per port.** The engine would otherwise own a
/// dynamic slice of the struct's namespace (`n_<port name>`, from names authored
/// for the artist), and every new broadcast port would widen the uniform. A bit
/// per port answers the only question the reader has — *"is this port length
/// 1?"* — for any arity, in four bytes, under a name `wgsl_field` can guard like
/// the other engine fields.
pub fn broadcasts_anything(bindings: &[ColumnBinding]) -> bool {
    bindings.iter().any(|b| b.access.broadcasts())
}

/// The `bcast_one` value: bit `p` set when input port `p` carries exactly one
/// element, so [`kernel_module`]'s broadcast readers pin row 0 on it.
pub fn broadcast_mask(counts: &[u32]) -> u32 {
    counts
        .iter()
        .enumerate()
        .filter(|(p, n)| **n == 1 && *p < 32)
        .fold(0u32, |m, (p, _)| m | (1u32 << p))
}

/// Does this node's generated module carry the `window_first` / `window_age`
/// uniforms? **Asked here by the codegen that declares them and by the
/// sequencer that packs them** — two answers to one layout question is a
/// silently misaligned uniform, which reads as plausible garbage.
///
/// A **generator** (no input ports) is the one kind of node that has a window:
/// its elements are minted, so their spawn identity and age are facts only the
/// host knows. A transformer INHERITS its elements — it has no window of its own
/// to be told about.
///
/// **Or a kernel with its own count law** (ADR-0136): `sim.spawn` HAS an input
/// (the template it gathers from) but its elements are minted all the same — the
/// count law that sizes the dispatch is the same expression that knows where the
/// identity window begins, so a law implies a window. Kernels with a law that
/// never read the window (`value.lfo`) carry two unused uniform floats, which is
/// the price of the layout question having exactly one answer.
///
/// A playhead-dependent generator is TOLD its window instead of re-deriving it:
/// `floor(t·rate)` recomputed in WGSL is `f32`, and past 2²⁴ it skips integers,
/// so the identity the whole gather rests on would quietly stop being a
/// bijection. The CPU computes these in `f64` (`SourceWindow`).
pub fn declares_window(has_count_law: bool, port_names: &[&str]) -> bool {
    port_names.is_empty() || has_count_law
}

/// Does the module also carry `window_src_n` — input port 0's element count?
/// Only a minted-window kernel WITH inputs wants it (ADR-0136: `sim.spawn`'s
/// slot arithmetic is `id % template_n`, and the template's length is a fact
/// only the host knows — `arrayLength` lies, the pool rounds buffers up).
/// Same one-answer discipline as [`declares_window`]: asked by the codegen that
/// declares the field and the sequencer that packs it.
pub fn declares_src_n(has_count_law: bool, port_names: &[&str]) -> bool {
    has_count_law && !port_names.is_empty()
}

/// Generate the full compute module for `kernel` against a concrete input
/// column set. `port_names` are the node manifest's input port names (they name
/// the readers of a multi-input kernel — [`accessor_suffix`]); binding indices
/// are: `0` = uniforms, then one slot per `ReadBuffer`, then one per
/// `WriteBuffer`, in `kernel.bindings` order — the sequencer builds the bind
/// group by replaying [`plan_bindings`].
pub fn kernel_module(
    kernel: &GpuKernel,
    bindings: &[ColumnBinding],
    port_names: &[&str],
    grid: Option<&GridSpec>,
    reduces: &[ReduceSpec],
    mut present: impl FnMut(&ColumnBinding) -> bool,
) -> String {
    // ADR-0130: is this an `id`-gather kernel, and is the gather active for this
    // input set? A kernel declares a gather with a [`ColumnAccess::GatherKey`]
    // binding on its base port; the gather is ACTIVE when that port's stream
    // carries the id (a dense window — the plan already refused a non-dense one).
    // When inactive (a grid, no id) the helpers below reduce to positional
    // (`gather_row(i) = i`), so integrate/spring compile ONE body for both cases.
    let gather_key = bindings.iter().find(|b| b.access.is_gather_key());
    let gather_on = gather_key.is_some_and(&mut present);
    let plans = plan_bindings(bindings, present);
    let mut src = String::with_capacity(1024);

    // Uniforms: count + playhead + one f32 per declared param. A param named
    // like a builtin would collide in the struct; the sequencer's eligibility
    // check refuses those kernels up front (`crate::plan`). A gather also carries
    // `gather_prev_n` (the prior state's element count) — CPU-known, so a uniform
    // rather than `arrayLength` (the pool rounds buffers to a pow2 class).
    src.push_str("struct KernelParams {\n    count: u32,\n    playhead: f32,\n");
    for p in kernel.params {
        src.push_str(&format!("    {}: f32,\n", wgsl_field(p)));
    }
    if gather_on {
        src.push_str("    gather_prev_n: u32,\n");
    }
    if declares_window(kernel.count_law.is_some(), port_names) {
        src.push_str("    window_first: u32,\n    window_age: f32,\n");
    }
    if declares_src_n(kernel.count_law.is_some(), port_names) {
        src.push_str("    window_src_n: u32,\n");
    }
    if broadcasts_anything(bindings) {
        src.push_str("    bcast_one: u32,\n");
    }
    // The grid's cell size and bucket count (ADR-0140 D2): CPU-known, so uniforms,
    // and the SAME values packed into the grid-build's own uniform — the body's
    // `grid_bucket_of` must agree bit-for-bit with the build's binning.
    if grid.is_some() {
        src.push_str("    grid_num_buckets: u32,\n    grid_cell: f32,\n");
    }
    src.push_str("}\n@group(0) @binding(0) var<uniform> params: KernelParams;\n\n");

    // Storage bindings: reads first, then writes (stable, replayable order).
    let mut slot = 1u32;
    for (b, (read, _)) in bindings.iter().zip(&plans) {
        if matches!(read, Some(BindingPlan::ReadBuffer)) {
            src.push_str(&format!(
                "@group(0) @binding({slot}) var<storage, read> in_{}: array<{}>;\n",
                accessor_suffix(port_names, b),
                wgsl_type(b.dim)
            ));
            slot += 1;
        }
    }
    for (b, (_, write)) in bindings.iter().zip(&plans) {
        if matches!(write, Some(BindingPlan::WriteBuffer)) {
            src.push_str(&format!(
                "@group(0) @binding({slot}) var<storage, read_write> out_{}: array<{}>;\n",
                b.column,
                wgsl_type(b.dim)
            ));
            slot += 1;
        }
    }
    // The grid's two arrays, LAST (ADR-0140 D2) — after every column binding, so
    // adding a grid never shifts a stream binding's slot. The sequencer appends
    // them to the bind group in the same order.
    if grid.is_some() {
        src.push_str(&format!(
            "@group(0) @binding({slot}) var<storage, read> grid_starts: array<u32>;\n"
        ));
        slot += 1;
        src.push_str(&format!(
            "@group(0) @binding({slot}) var<storage, read> grid_sorted: array<u32>;\n"
        ));
        slot += 1;
    }
    // The whole-stream reduction results, LAST of all — after the columns AND
    // after the grid, so a node that declares one never shifts anybody's slot.
    // Each is a 1-element buffer the sequencer filled before this pass ran.
    for r in reduces {
        src.push_str(&format!(
            "@group(0) @binding({slot}) var<storage, read> reduce_buf_{}: array<f32>;\n",
            r.name
        ));
        slot += 1;
    }
    let _ = slot;
    src.push('\n');

    // read_/write_ helpers + the HAS_ presence consts — the body's whole view
    // of the stream.
    for (b, (read, write)) in bindings.iter().zip(&plans) {
        let ty = wgsl_type(b.dim);
        let c = accessor_suffix(port_names, b);
        match read {
            // A broadcast port reads row 0 when it carries exactly ONE value —
            // the `1 => vals[0]` arm of the CPU's `target_at`, decided per
            // dispatch from a uniform bit rather than per pipeline, so one
            // compiled module serves both a global target and a per-element one.
            Some(BindingPlan::ReadBuffer) if b.access.broadcasts() => src.push_str(&format!(
                "const HAS_{c}: bool = true;\n\
                 fn read_{c}(i: u32) -> {ty} {{\n\
                 \x20   let one = (params.bcast_one & {bit}u) != 0u;\n\
                 \x20   return in_{c}[select(i, 0u, one)];\n\
                 }}\n",
                bit = 1u32 << b.port
            )),
            Some(BindingPlan::ReadBuffer) => src.push_str(&format!(
                "const HAS_{c}: bool = true;\n\
                 fn read_{c}(i: u32) -> {ty} {{ return in_{c}[i]; }}\n"
            )),
            Some(BindingPlan::ReadIdentity) => src.push_str(&format!(
                "const HAS_{c}: bool = false;\n\
                 fn read_{c}(i: u32) -> {ty} {{ _ = i; return {id}; }}\n",
                id = identity_literal(b.dim, b.identity)
            )),
            _ => {}
        }
        match write {
            Some(BindingPlan::WriteBuffer) => src.push_str(&format!(
                "fn write_{c}(i: u32, v: {ty}) {{ out_{c}[i] = v; }}\n",
                c = b.column
            )),
            Some(BindingPlan::WriteDropped) => src.push_str(&format!(
                "fn write_{c}(i: u32, v: {ty}) {{ _ = i; _ = v; }}\n",
                c = b.column
            )),
            _ => {}
        }
    }
    src.push('\n');

    // ADR-0130: the id-gather helpers (only for a kernel with a GatherKey). They
    // sit AFTER the read accessors (they call them) and BEFORE the body:
    //   • `gather_row(i)` — the prior-state row element `i` pairs to. Positional
    //     `i` when the base is not a dense window; `current_id − prev_first` (an
    //     UNSIGNED row offset) when it is — the arithmetic that equals the CPU's
    //     `BTreeMap<id,row>` on a dense window (state ids are `[prev_first,
    //     prev_first+prev_n)`, so id-X's row IS `X − prev_first`).
    //   • `gather_paired(i)` — whether that row exists (`< prev_n`). This is the
    //     PER-ELEMENT seed guard (a fresh id has no prior row), DISTINCT from the
    //     global `HAS_*` "is there any prior state at all?"
    //     ([[feedback_layered_defenses_need_per_layer_gates]]).
    // The casts are VALUE casts (`u32(max(id, 0.0))`), matching the CPU's
    // `f.max(0.0) as u32` — ids are stored as their numeric value (`f32(em_id)`),
    // not bit-packed, so a `bitcast` would read the IEEE bits, not the id. An id
    // below `prev_first` (only off the monotonic path) wraps u32 → `≥ prev_n` →
    // unpaired → seed, matching the BTreeMap miss (ADR-0130 D5).
    if let Some(key) = gather_key {
        if gather_on {
            let cur_id = accessor_suffix(port_names, key);
            // `prev_first` is the prior state's min id — the SAME column, bound
            // on the state port (any port but the key's). Absent (tick 0, the
            // `pre` is Empty) → 0, and prev_n is 0 too, so nothing pairs: the
            // seed-everything tick, matching the CPU's `pre = Empty` seed.
            let prev_first = kernel
                .bindings
                .iter()
                .find(|b| b.column == key.column && b.port != key.port && b.access.reads())
                .map(|b| format!("u32(max(read_{}(0u), 0.0))", accessor_suffix(port_names, b)))
                .unwrap_or_else(|| "0u".to_string());
            src.push_str(&format!(
                "fn gather_prev_first() -> u32 {{ return {prev_first}; }}\n\
                 fn gather_row(i: u32) -> u32 {{ return (u32(max(read_{cur_id}(i), 0.0)) - gather_prev_first()) % 16777216u; }}\n\
                 fn gather_paired(i: u32) -> bool {{ return gather_row(i) < params.gather_prev_n; }}\n\n"
            ));
        } else {
            src.push_str(
                "fn gather_row(i: u32) -> u32 { return i; }\n\
                 fn gather_paired(i: u32) -> bool { _ = i; return true; }\n\n",
            );
        }
    }

    // The grid helpers (ADR-0140 D2) — the SAME binning as the build's
    // `bucket_of` (`grid.rs`), split so the body can name a neighbour CELL (for
    // the 3×3 sweep and the exact-cell dedup) and its bucket. The body iterates
    // `grid_sorted[grid_starts[b] .. grid_starts[b+1]]` for each of the 9 cells.
    if grid.is_some() {
        src.push_str(
            "fn grid_cell_of(p: vec2<f32>) -> vec2<i32> {\n\
             \x20   return vec2<i32>(i32(floor(p.x / params.grid_cell)), i32(floor(p.y / params.grid_cell)));\n\
             }\n\
             fn grid_bucket_of(c: vec2<i32>) -> u32 {\n\
             \x20   let h = (c.x * 73856093) ^ (c.y * 19349663);\n\
             \x20   return bitcast<u32>(h) & (params.grid_num_buckets - 1u);\n\
             }\n\n",
        );
    }

    // The reduction accessors — one number about the WHOLE stream, the same for
    // every invocation. `reduce_<name>()` and not a bare `let`, so the body reads
    // it exactly like it reads a column (`read_P(i)`), and so a kernel that never
    // calls it costs nothing.
    for r in reduces {
        src.push_str(&format!(
            "fn reduce_{n}() -> f32 {{ return reduce_buf_{n}[0]; }}\n",
            n = r.name
        ));
    }
    if !reduces.is_empty() {
        src.push('\n');
    }

    src.push_str(kernel.wgsl_lib);
    src.push('\n');

    src.push_str(&format!(
        "@compute @workgroup_size({WORKGROUP_SIZE})\n\
         fn main(@builtin(global_invocation_id) gid: vec3<u32>) {{\n\
         \x20   let i = gid.x;\n\
         \x20   if (i >= params.count) {{ return; }}\n"
    ));
    src.push_str(kernel.wgsl);
    src.push_str("}\n");
    src
}

#[cfg(test)]
#[path = "codegen_tests.rs"]
mod tests;
