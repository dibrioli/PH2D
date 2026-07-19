//! The GPU kernels for `motion.spring` — one per target channel.
//!
//! **The state pairing (ADR-0130).** The prior spring arrives on the `pre` port,
//! and `HAS_state_spring_value` is the *global* "is there any prior state?" —
//! false on tick 0, when every element seeds at its target, exactly like the
//! CPU's `pairing() == None`. `gather_row(i)` is the row to read the prior spring
//! from (positional `i`, or the id-gather `current_id − prev_first` on a dense
//! window — the arithmetic form of the CPU's `BTreeMap<id,row>`), and
//! `gather_paired(i)` is the *per-element* seed guard: a freshly-born particle
//! has no prior row and stays at its target, matching the CPU's
//! `None => continue`. Two guards, because they answer different questions
//! ([[feedback_layered_defenses_need_per_layer_gates]]).
//!
//! **Every channel**, by shipping one variant per target column
//! ([`GpuKernel::variant_by_param`]). It used to cover X/Y only — Rotation writes
//! `rot` and Size writes `size`, and a static binding set cannot switch its output
//! column on a param — and that was a real coverage edge rather than a rounding
//! of one: a spring that recedes to `eval` is a boundary, and inside a `pre` loop
//! a boundary makes `plan` refuse the WHOLE simulation.
//!
//! **The solver is one function** ([`SP_LIB`]'s `sp_solve`), shared verbatim by
//! all three variants; only the target it is handed and the column it lands on
//! differ, which is precisely why the variants exist and not a body-level branch.
//! A stiff spring is stiff on both sides, so the sub-step loop is dynamic
//! (`ceil(dt/ideal)`, the reference's stability bound) — iterating on the GPU is
//! not a translation of the CPU's loop into something cheaper, because the
//! sub-step count is part of the ANSWER, not of the schedule.
//!
//! **The channels differ in their IDENTITY, not just their column.** `P` and
//! `rot` read `0` from an absent column, but `size` reads **unit scale** — a
//! spring on Size whose input carries no `size` must settle at `1`, not collapse
//! the sprite through zero. Size is uniform on both sides: it reads the X
//! component and writes BOTH, exactly as the CPU's `channel_get`/`channel_set`
//! pair does.

use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::port::Dim;

const SP_PARAMS: &[&str] = &["channel", "tension", "friction"];

/// The falloff mask and the pin weight, bound identically by every variant.
const SP_FALLOFF: ColumnBinding = ColumnBinding {
    column: "falloff",
    dim: Dim::Scalar,
    access: ColumnAccess::Read,
    identity: [1.0; 4],
    port: 0,
};
const SP_INV_MASS: ColumnBinding = ColumnBinding {
    column: "inv_mass",
    dim: Dim::Scalar,
    access: ColumnAccess::Read,
    identity: [1.0; 4],
    port: 0,
};
/// The state, off the `pre` port. `spring_value`'s presence IS the CPU's
/// `pairing().is_some()` — see the module note.
const SP_VALUE: ColumnBinding = ColumnBinding {
    column: "spring_value",
    dim: Dim::Scalar,
    access: ColumnAccess::ReadWrite,
    identity: [0.0; 4],
    port: 1,
};
const SP_VEL: ColumnBinding = ColumnBinding {
    column: "spring_vel",
    dim: Dim::Scalar,
    access: ColumnAccess::ReadWrite,
    identity: [0.0; 4],
    port: 1,
};
const SP_SIM_T: ColumnBinding = ColumnBinding {
    column: "sim_t",
    dim: Dim::Scalar,
    access: ColumnAccess::ReadWrite,
    identity: [0.0; 4],
    port: 1,
};
/// The prior state's id, off the `pre` port — the module reads `id[0]` for
/// `prev_first`. Absent (tick 0 / an id-less stream) → identity.
const SP_PREV_ID: ColumnBinding = ColumnBinding {
    column: "id",
    dim: Dim::Scalar,
    access: ColumnAccess::Read,
    identity: [0.0; 4],
    port: 1,
};
/// ADR-0130: the id-gather key on the base port. Identity pairing is a gather
/// (`BTreeMap<id, row>`), arithmetic (`current_id − prev_first`) on a dense
/// window (the emitter) and CLAIMED, but a non-dense / unprovable id still
/// recedes rather than animate the wrong particles in silence.
const SP_GATHER: ColumnBinding = ColumnBinding {
    column: "id",
    dim: Dim::Scalar,
    access: ColumnAccess::GatherKey,
    identity: [0.0; 4],
    port: 0,
};

/// The solver, shared verbatim by all three variants.
///
/// ⚠️ The parameter is `goal`, not `target`: **`target` is a WGSL reserved
/// keyword** and naga rejects the module outright. `out` is reserved too,
/// hence `blend`. The `generated_wgsl_validates` gate parses every kernel at
/// `cargo test` time with no device, which is the only reason this cost a
/// minute instead of a confusing runtime failure on one machine.
///
/// `sp_solve(i, goal)` returns `(blend, x, v)`: the blended OUTPUT the channel
/// receives, and the raw state to store. The blend is `!(fs >= 1.0)` and not
/// `fs < 1.0` — they part on a NaN weight, and the CPU takes the `>=` branch.
/// The raw state keeps evolving regardless of the blend, exactly like the
/// reference.
const SP_LIB: &str = "\
    const SPRING_MAX_DT: f32 = 0.1;\n\
    const SPRING_STABLE: f32 = 0.05;\n\
    fn spring_finite(x: f32) -> bool {\n\
        return abs(x) <= 3.4028235e38;\n\
    }\n\
    fn sp_solve(i: u32, goal: f32) -> vec3<f32> {\n\
        let tension = max(params.tension, 0.1);\n\
        let friction = max(params.friction, 0.05);\n\
        // Seeded AT the target (no snap); only what the state knows then steps.\n\
        var sp_x = goal;\n\
        var sp_v = 0.0;\n\
        let row = gather_row(i);\n\
        if (HAS_state_spring_value && gather_paired(i)) {\n\
            var x = read_state_spring_value(row);\n\
            var v = read_state_spring_vel(row);\n\
            let t_prev =\n\
        \x20       select(params.playhead, read_state_sim_t(0u), HAS_state_sim_t);\n\
            let dt = clamp(params.playhead - t_prev, 0.0, SPRING_MAX_DT);\n\
            // Adaptive sub-step from the stability limit (reference parity).\n\
            let ideal = sqrt(SPRING_STABLE / tension);\n\
            var steps = 1u;\n\
            if (dt > 0.0) {\n\
                steps = u32(clamp(ceil(dt / ideal), 1.0, 64.0));\n\
            }\n\
            let sub_dt = dt / f32(steps);\n\
            // A diverged instance recovers AT ITS TARGET (reference parity).\n\
            if (!(spring_finite(x) && spring_finite(v))) {\n\
                x = goal;\n\
                v = 0.0;\n\
            }\n\
            for (var s = 0u; s < steps; s = s + 1u) {\n\
                let a = -friction * v - tension * (x - goal);\n\
                v = v + a * sub_dt;\n\
                x = x + v * sub_dt;\n\
            }\n\
            sp_x = x;\n\
            sp_v = v;\n\
        }\n\
        let fs = read_in_falloff(i) * read_in_inv_mass(i);\n\
        var blend = sp_x;\n\
        if (!(fs >= 1.0)) {\n\
            blend = goal + (sp_x - goal) * max(fs, 0.0);\n\
        }\n\
        return vec3<f32>(blend, sp_x, sp_v);\n\
    }\n\
    fn sp_store(i: u32, r: vec3<f32>) {\n\
        write_spring_value(i, r.y);\n\
        write_spring_vel(i, r.z);\n\
        write_sim_t(i, params.playhead);\n\
    }\n";

/// **X / Y** — springs one component of `P`. The channel test is `< 0.5`, which
/// agrees with the CPU's `round()` for both values this variant is selected for.
const SP_P: GpuKernel = GpuKernel {
    wgsl: "\
        var sp_p = read_in_P(i);\n\
        let sp_r = sp_solve(i, select(sp_p.y, sp_p.x, params.channel < 0.5));\n\
        if (params.channel < 0.5) {\n\
            sp_p.x = sp_r.x;\n\
        } else {\n\
            sp_p.y = sp_r.x;\n\
        }\n\
        write_P(i, sp_p);\n\
        sp_store(i, sp_r);\n",
    wgsl_lib: SP_LIB,
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        SP_FALLOFF,
        SP_INV_MASS,
        SP_VALUE,
        SP_VEL,
        SP_SIM_T,
        SP_PREV_ID,
        SP_GATHER,
    ],
    params: SP_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Rotation** — springs `rot` (identity `0`).
const SP_ROT: GpuKernel = GpuKernel {
    wgsl: "\
        let sp_r = sp_solve(i, read_in_rot(i));\n\
        write_rot(i, sp_r.x);\n\
        sp_store(i, sp_r);\n",
    wgsl_lib: SP_LIB,
    bindings: &[
        ColumnBinding {
            column: "rot",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        SP_FALLOFF,
        SP_INV_MASS,
        SP_VALUE,
        SP_VEL,
        SP_SIM_T,
        SP_PREV_ID,
        SP_GATHER,
    ],
    params: SP_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Size** — springs the X component and writes BOTH (the spring is uniform,
/// like the reference), from the UNIT identity: a spring on a stream carrying no
/// `size` must settle at unit scale, never collapse the sprite through zero.
const SP_SIZE: GpuKernel = GpuKernel {
    wgsl: "\
        let sp_r = sp_solve(i, read_in_size(i).x);\n\
        write_size(i, vec2<f32>(sp_r.x, sp_r.x));\n\
        sp_store(i, sp_r);\n",
    wgsl_lib: SP_LIB,
    bindings: &[
        ColumnBinding {
            column: "size",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [1.0, 1.0, 0.0, 0.0],
            port: 0,
        },
        SP_FALLOFF,
        SP_INV_MASS,
        SP_VALUE,
        SP_VEL,
        SP_SIM_T,
        SP_PREV_ID,
        SP_GATHER,
    ],
    params: SP_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// The registered kernel: the X/Y variant's shape, with the channel switch.
pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    // The top-level shape IS the X/Y variant, so a caller that never resolves
    // still sees a real kernel rather than the empty (pass-through) one.
    wgsl: SP_P.wgsl,
    wgsl_lib: SP_LIB,
    bindings: SP_P.bindings,
    params: SP_PARAMS,
    count_law: None,
    variant_by_param: Some(|param| match param("channel").round() as i32 {
        2 => &SP_ROT,
        0 | 1 => &SP_P,
        _ => &SP_SIZE,
    }),
    applicable: None,
};
