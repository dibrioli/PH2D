//! Unit tests for [`super`] — the WGSL module generator. Split from `codegen.rs`
//! at the HR-18 LOC cap; declared there as a `#[path]` sibling, so `super` is the
//! generator itself.

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
    count_law: None,
    variant_by_param: None,
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
    count_law: None,
    variant_by_param: None,
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
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

#[test]
fn present_columns_bind_buffers_absent_read_becomes_identity() {
    // P present, falloff absent → P reads+writes buffers; falloff reads
    // its identity constant (1.0) with no binding.
    let src = kernel_module(&K, K.bindings, &["in"], None, &[], |b| b.column == "P");
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
    let src = kernel_module(&K, K.bindings, &["in"], None, &[], |_| false);
    assert!(!src.contains("var<storage"));
    assert!(src.contains("fn write_P(i: u32, v: vec2<f32>) { _ = i; _ = v; }"));
}

#[test]
fn presence_signature_distinguishes_column_sets() {
    let all = presence_signature(K.bindings, |_| true);
    let none = presence_signature(K.bindings, |_| false);
    let p_only = presence_signature(K.bindings, |b| b.column == "P");
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
        count_law: None,
        variant_by_param: None,
        applicable: None,
    };
    assert_ne!(
        presence_signature(RW.bindings, |_| true),
        presence_signature(RW.bindings, |_| false),
        "present and absent MUST key different pipelines"
    );
    // …and they really are different modules, which is why.
    assert!(kernel_module(&RW, RW.bindings, &["in"], None, &[], |_| true).contains("in_size"));
    assert!(!kernel_module(&RW, RW.bindings, &["in"], None, &[], |_| false).contains("in_size"));
}

#[test]
fn the_has_const_tells_presence_apart_from_an_identity_read() {
    // The whole point: `read_falloff` answers 1.0 either way, so a body
    // that must BRANCH on absence (integrate's seed) has no other signal.
    let present = kernel_module(&K, K.bindings, &["in"], None, &[], |_| true);
    assert!(present.contains("const HAS_falloff: bool = true;"));
    let absent = kernel_module(&K, K.bindings, &["in"], None, &[], |_| false);
    assert!(absent.contains("const HAS_falloff: bool = false;"));
}

#[test]
fn a_multi_input_kernel_names_every_reader_by_its_port() {
    // `vel` is bound on BOTH ports: unqualified readers would collide, and
    // whichever won would be a plausible wrong answer.
    let src = kernel_module(&SIM, SIM.bindings, SIM_PORTS, None, &[], |_| true);
    assert!(src.contains("fn read_rest_vel(i: u32) -> vec2<f32> { return in_rest_vel[i]; }"));
    assert!(src.contains("fn read_forces_vel(i: u32) -> vec2<f32> { return in_forces_vel[i]; }"));
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
    let src = kernel_module(&SIM, SIM.bindings, SIM_PORTS, None, &[], |_| true);
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
    let src = kernel_module(&GATHER, GATHER.bindings, SIM_PORTS, None, &[], |b| {
        b.column != "id"
    });
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
    let src = kernel_module(&GATHER, GATHER.bindings, SIM_PORTS, None, &[], |_| true);
    assert!(
        src.contains("gather_prev_n: u32,"),
        "the prior-count uniform exists"
    );
    assert!(
        src.contains("fn gather_prev_first() -> u32 { return u32(max(read_forces_id(0u), 0.0)); }"),
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
    let src = kernel_module(&SIM, SIM.bindings, SIM_PORTS, None, &[], |_| true);
    // (Not a bare `contains("id")` — the entry point takes `global_invocation_id`.)
    assert!(!src.contains("in_rest_id"), "no read buffer");
    assert!(!src.contains("fn read_rest_id("), "no accessor");
    assert!(!src.contains("HAS_rest_id"), "no presence const");
    assert!(!src.contains("out_id"), "no write buffer");
    // And it never moves the signature (which keys the pipeline cache).
    assert_eq!(
        presence_signature(SIM.bindings, |b| b.column != "id"),
        presence_signature(SIM.bindings, |_| true)
    );
}
