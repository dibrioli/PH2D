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

use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
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

/// A WGSL literal for `identity`'s first `dim`-many lanes.
fn identity_literal(dim: Dim, identity: [f32; 4]) -> String {
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
    kernel: &GpuKernel,
    mut present: impl FnMut(&ColumnBinding) -> bool,
) -> Vec<(Option<BindingPlan>, Option<BindingPlan>)> {
    kernel
        .bindings
        .iter()
        .map(|b| {
            let here = b.access.reads() && present(b);
            let read = b.access.reads().then_some(if here {
                BindingPlan::ReadBuffer
            } else {
                BindingPlan::ReadIdentity
            });
            let write = match (b.access, here) {
                (ColumnAccess::Read | ColumnAccess::Consume, _) => None,
                // A refusal is not an access: it binds nothing on either side —
                // by the time a plan reaches codegen the node was accepted, so
                // the refused column is provably absent anyway.
                (ColumnAccess::RefuseIfPresent, _) => None,
                // The gather key READS the current element's id (so `read`
                // above binds it) but writes nothing — the id rides through the
                // base like any unwritten column (ADR-0130).
                (ColumnAccess::GatherKey, _) => None,
                (ColumnAccess::ReadWriteExisting, false) => Some(BindingPlan::WriteDropped),
                _ => Some(BindingPlan::WriteBuffer),
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
pub fn storage_bindings(kernel: &GpuKernel, present: impl FnMut(&ColumnBinding) -> bool) -> u32 {
    plan_bindings(kernel, present)
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
    kernel: &GpuKernel,
    mut present: impl FnMut(&ColumnBinding) -> bool,
) -> u64 {
    kernel
        .bindings
        .iter()
        .enumerate()
        .fold(0u64, |sig, (i, b)| {
            let here = b.access.reads() && present(b);
            sig | ((here as u64) << i)
        })
}

/// Generate the full compute module for `kernel` against a concrete input
/// column set. `port_names` are the node manifest's input port names (they name
/// the readers of a multi-input kernel — [`accessor_suffix`]); binding indices
/// are: `0` = uniforms, then one slot per `ReadBuffer`, then one per
/// `WriteBuffer`, in `kernel.bindings` order — the sequencer builds the bind
/// group by replaying [`plan_bindings`].
pub fn kernel_module(
    kernel: &GpuKernel,
    port_names: &[&str],
    mut present: impl FnMut(&ColumnBinding) -> bool,
) -> String {
    // ADR-0130: is this an `id`-gather kernel, and is the gather active for this
    // input set? A kernel declares a gather with a [`ColumnAccess::GatherKey`]
    // binding on its base port; the gather is ACTIVE when that port's stream
    // carries the id (a dense window — the plan already refused a non-dense one).
    // When inactive (a grid, no id) the helpers below reduce to positional
    // (`gather_row(i) = i`), so integrate/spring compile ONE body for both cases.
    let gather_key = kernel.bindings.iter().find(|b| b.access.is_gather_key());
    let gather_on = gather_key.is_some_and(&mut present);
    let plans = plan_bindings(kernel, present);
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
    // A playhead-dependent generator is TOLD its window instead of re-deriving
    // it: `floor(t·rate)` recomputed here is `f32`, and past 2²⁴ it skips
    // integers, so the identity the whole gather rests on would quietly stop
    // being a bijection. The CPU computes these in `f64` (`SourceWindow`).
    if kernel.source_window.is_some() {
        src.push_str("    window_first: u32,\n    window_age: f32,\n");
    }
    src.push_str("}\n@group(0) @binding(0) var<uniform> params: KernelParams;\n\n");

    // Storage bindings: reads first, then writes (stable, replayable order).
    let mut slot = 1u32;
    for (b, (read, _)) in kernel.bindings.iter().zip(&plans) {
        if matches!(read, Some(BindingPlan::ReadBuffer)) {
            src.push_str(&format!(
                "@group(0) @binding({slot}) var<storage, read> in_{}: array<{}>;\n",
                accessor_suffix(port_names, b),
                wgsl_type(b.dim)
            ));
            slot += 1;
        }
    }
    for (b, (_, write)) in kernel.bindings.iter().zip(&plans) {
        if matches!(write, Some(BindingPlan::WriteBuffer)) {
            src.push_str(&format!(
                "@group(0) @binding({slot}) var<storage, read_write> out_{}: array<{}>;\n",
                b.column,
                wgsl_type(b.dim)
            ));
            slot += 1;
        }
    }
    src.push('\n');

    // read_/write_ helpers + the HAS_ presence consts — the body's whole view
    // of the stream.
    for (b, (read, write)) in kernel.bindings.iter().zip(&plans) {
        let ty = wgsl_type(b.dim);
        let c = accessor_suffix(port_names, b);
        match read {
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
mod tests {
    use super::*;

    const K: GpuKernel = GpuKernel {
        wgsl: "write_P(i, read_P(i) + vec2<f32>(params.dx, 0.0) * read_falloff(i));\n",
        wgsl_lib: "",
        bindings: &[
            ColumnBinding {
                column: "P",
                dim: Dim::Vec2,
                access: ColumnAccess::ReadWriteExisting,
                identity: [0.0; 4],
                port: 0,
            },
            ColumnBinding {
                column: "falloff",
                dim: Dim::Scalar,
                access: ColumnAccess::Read,
                identity: [1.0; 4],
                port: 0,
            },
        ],
        params: &["dx"],
        source_window: None,
        applicable: None,
    };

    /// A two-input kernel shaped like `motion.integrate`: the same column
    /// (`vel`) bound on both ports, a consumed transient, and a refusal.
    const SIM: GpuKernel = GpuKernel {
        wgsl: "write_vel(i, read_rest_vel(i) + read_forces_vel(i) + read_forces_accel(i));\n",
        wgsl_lib: "",
        bindings: &[
            ColumnBinding {
                column: "vel",
                dim: Dim::Vec2,
                access: ColumnAccess::Read,
                identity: [0.0; 4],
                port: 0,
            },
            ColumnBinding {
                column: "vel",
                dim: Dim::Vec2,
                access: ColumnAccess::ReadWrite,
                identity: [0.0; 4],
                port: 1,
            },
            ColumnBinding {
                column: "accel",
                dim: Dim::Vec2,
                access: ColumnAccess::Consume,
                identity: [0.0; 4],
                port: 1,
            },
            ColumnBinding {
                column: "id",
                dim: Dim::Scalar,
                access: ColumnAccess::RefuseIfPresent,
                identity: [0.0; 4],
                port: 0,
            },
        ],
        params: &[],
        source_window: None,
        applicable: None,
    };
    const SIM_PORTS: &[&str] = &["rest", "forces"];

    /// A two-input kernel shaped like the ADR-0130 gather: an `id` GatherKey on
    /// the base port, the SAME `id` bound `Read` on the state port (for
    /// `prev_first`), and a state column paired per element.
    const GATHER: GpuKernel = GpuKernel {
        wgsl: "\
            let row = gather_row(i);\n\
            var v = read_rest_vel(i);\n\
            if (HAS_forces_vel && gather_paired(i)) { v = read_forces_vel(row); }\n\
            write_vel(i, v);\n",
        wgsl_lib: "",
        bindings: &[
            ColumnBinding {
                column: "vel",
                dim: Dim::Vec2,
                access: ColumnAccess::Read,
                identity: [0.0; 4],
                port: 0,
            },
            ColumnBinding {
                column: "vel",
                dim: Dim::Vec2,
                access: ColumnAccess::ReadWrite,
                identity: [0.0; 4],
                port: 1,
            },
            ColumnBinding {
                column: "id",
                dim: Dim::Scalar,
                access: ColumnAccess::GatherKey,
                identity: [0.0; 4],
                port: 0,
            },
            ColumnBinding {
                column: "id",
                dim: Dim::Scalar,
                access: ColumnAccess::Read,
                identity: [0.0; 4],
                port: 1,
            },
        ],
        params: &[],
        source_window: None,
        applicable: None,
    };

    #[test]
    fn present_columns_bind_buffers_absent_read_becomes_identity() {
        // P present, falloff absent → P reads+writes buffers; falloff reads
        // its identity constant (1.0) with no binding.
        let src = kernel_module(&K, &["in"], |b| b.column == "P");
        assert!(src.contains("var<storage, read> in_P"));
        assert!(src.contains("var<storage, read_write> out_P"));
        assert!(!src.contains("in_falloff"));
        assert!(src.contains("fn read_falloff(i: u32) -> f32 { _ = i; return 1.0; }"));
        assert!(src.contains("dx: f32,"));
    }

    #[test]
    fn absent_read_write_existing_drops_the_write() {
        // Nothing present → P's write is a no-op (the CPU pass-through), and
        // NO storage binding exists at all.
        let src = kernel_module(&K, &["in"], |_| false);
        assert!(!src.contains("var<storage"));
        assert!(src.contains("fn write_P(i: u32, v: vec2<f32>) { _ = i; _ = v; }"));
    }

    #[test]
    fn presence_signature_distinguishes_column_sets() {
        let all = presence_signature(&K, |_| true);
        let none = presence_signature(&K, |_| false);
        let p_only = presence_signature(&K, |b| b.column == "P");
        assert_ne!(all, none);
        assert_ne!(all, p_only);
        assert_ne!(p_only, none);
    }

    #[test]
    fn a_read_write_columns_presence_moves_the_signature() {
        // The collision that shipped in Fase 1 and hid until Fase 3: a
        // `ReadWrite` binding emits its write buffer either way, so a signature
        // derived from "binds any buffer" was IDENTICAL present and absent —
        // while the modules differ by an entire read binding. Two chains, one
        // cache entry, and wgpu rejecting the bind group against the layout it
        // compiled for the other one.
        const RW: GpuKernel = GpuKernel {
            wgsl: "write_size(i, read_size(i) * params.amount);\n",
            wgsl_lib: "",
            bindings: &[ColumnBinding {
                column: "size",
                dim: Dim::Vec2,
                access: ColumnAccess::ReadWrite,
                identity: [1.0; 4],
                port: 0,
            }],
            params: &["amount"],
            source_window: None,
            applicable: None,
        };
        assert_ne!(
            presence_signature(&RW, |_| true),
            presence_signature(&RW, |_| false),
            "present and absent MUST key different pipelines"
        );
        // …and they really are different modules, which is why.
        assert!(kernel_module(&RW, &["in"], |_| true).contains("in_size"));
        assert!(!kernel_module(&RW, &["in"], |_| false).contains("in_size"));
    }

    #[test]
    fn the_has_const_tells_presence_apart_from_an_identity_read() {
        // The whole point: `read_falloff` answers 1.0 either way, so a body
        // that must BRANCH on absence (integrate's seed) has no other signal.
        let present = kernel_module(&K, &["in"], |_| true);
        assert!(present.contains("const HAS_falloff: bool = true;"));
        let absent = kernel_module(&K, &["in"], |_| false);
        assert!(absent.contains("const HAS_falloff: bool = false;"));
    }

    #[test]
    fn a_multi_input_kernel_names_every_reader_by_its_port() {
        // `vel` is bound on BOTH ports: unqualified readers would collide, and
        // whichever won would be a plausible wrong answer.
        let src = kernel_module(&SIM, SIM_PORTS, |_| true);
        assert!(src.contains("fn read_rest_vel(i: u32) -> vec2<f32> { return in_rest_vel[i]; }"));
        assert!(
            src.contains("fn read_forces_vel(i: u32) -> vec2<f32> { return in_forces_vel[i]; }")
        );
        assert!(src.contains("const HAS_rest_vel: bool = true;"));
        assert!(src.contains("const HAS_forces_vel: bool = true;"));
        // Two distinct read buffers, and no bare `read_vel` to grab by mistake.
        assert!(src.contains("var<storage, read> in_rest_vel"));
        assert!(src.contains("var<storage, read> in_forces_vel"));
        assert!(!src.contains("fn read_vel("));
        // One output → the writer stays unqualified.
        assert!(src.contains("fn write_vel(i: u32, v: vec2<f32>) { out_vel[i] = v; }"));
        assert!(src.contains("var<storage, read_write> out_vel"));
    }

    #[test]
    fn a_consumed_column_is_read_but_never_written() {
        let src = kernel_module(&SIM, SIM_PORTS, |_| true);
        assert!(src.contains("fn read_forces_accel(i: u32) -> vec2<f32>"));
        // No output buffer, no writer: the sequencer drops it from the output
        // stream instead (`GpuStream` threading), which is what "transient" means.
        assert!(!src.contains("out_accel"));
        assert!(!src.contains("fn write_accel("));
    }

    #[test]
    fn a_gather_key_reads_the_id_and_the_positional_default_is_the_identity() {
        // ADR-0130: with the base id ABSENT the gather is inactive — the helpers
        // reduce to positional (`gather_row(i) = i`, everyone paired) and there
        // is no `gather_prev_n` uniform. This is the grid path, byte-for-byte the
        // pre-0130 behaviour.
        let src = kernel_module(&GATHER, SIM_PORTS, |b| b.column != "id");
        assert!(src.contains("fn gather_row(i: u32) -> u32 { return i; }"));
        assert!(src.contains("fn gather_paired(i: u32) -> bool { _ = i; return true; }"));
        assert!(
            !src.contains("gather_prev_n"),
            "no gather uniform when positional"
        );
        assert!(
            !src.contains("gather_prev_first"),
            "no id arithmetic when positional"
        );
    }

    #[test]
    fn a_present_dense_id_activates_the_arithmetic_gather() {
        // The base id present → gather ON: `gather_row` is `current_id −
        // prev_first`, `gather_paired` bounds-checks against `prev_n`, and the
        // prior state's min id is read RAW off element 0 of the STATE port's id
        // (not the base's). VALUE casts (`u32(max(...))`), never `bitcast`.
        let src = kernel_module(&GATHER, SIM_PORTS, |_| true);
        assert!(
            src.contains("gather_prev_n: u32,"),
            "the prior-count uniform exists"
        );
        assert!(
            src.contains(
                "fn gather_prev_first() -> u32 { return u32(max(read_forces_id(0u), 0.0)); }"
            ),
            "prev_first is the STATE port's id[0], value-cast"
        );
        assert!(src.contains(
            "fn gather_row(i: u32) -> u32 { return (u32(max(read_rest_id(i), 0.0)) - gather_prev_first()) % 16777216u; }"
        ));
        assert!(src.contains(
            "fn gather_paired(i: u32) -> bool { return gather_row(i) < params.gather_prev_n; }"
        ));
        // The base id is a REAL read now (the current element's id), and the
        // state id is bound too (for prev_first) — two distinct read buffers.
        assert!(src.contains("var<storage, read> in_rest_id"));
        assert!(src.contains("var<storage, read> in_forces_id"));
        assert!(
            !src.contains("bitcast"),
            "ids are value-stored, not bit-packed"
        );
    }

    #[test]
    fn a_refusal_generates_nothing_at_all() {
        // It answers eligibility at plan time; by codegen the column is
        // provably absent, so a binding for it would be dead weight in every
        // pipeline.
        let src = kernel_module(&SIM, SIM_PORTS, |_| true);
        // (Not a bare `contains("id")` — the entry point takes `global_invocation_id`.)
        assert!(!src.contains("in_rest_id"), "no read buffer");
        assert!(!src.contains("fn read_rest_id("), "no accessor");
        assert!(!src.contains("HAS_rest_id"), "no presence const");
        assert!(!src.contains("out_id"), "no write buffer");
        // And it never moves the signature (which keys the pipeline cache).
        assert_eq!(
            presence_signature(&SIM, |b| b.column != "id"),
            presence_signature(&SIM, |_| true)
        );
    }
}
