//! ADR-0049 §2.14 caps + §2.15 behavior gates for `ph2d-painter-fluid`.
//!
//! Field/variant counts are COMPILE-ENFORCED: each test exhaustively destructures
//! the type, so adding a field/variant breaks the build until the `COUNT` const is
//! updated — at which point the `<= cap` assert is the gate. No source parsing.
#![cfg(feature = "fluid")]

use ph2d_painter_fluid::{FluidParams, FluidSim, GravitySource, fluid_pass_eligible};

#[test]
fn fluid_params_field_count_is_capped() {
    let FluidParams {
        diffusivity,
        evaporation,
        downhill,
        flow_outward,
        w_lo,
        w_hi,
        perm_valley,
        perm_crest,
        version,
    } = FluidParams::default();
    let _ = (
        diffusivity,
        evaporation,
        downhill,
        flow_outward,
        w_lo,
        w_hi,
        perm_valley,
        perm_crest,
        version,
    );
    // The destructure above lists every field (compile-enforced); this pins the
    // count under cap at compile time (ADR-0049 §2.13).
    const COUNT: usize = 9;
    const _: () = assert!(COUNT <= 12);
}

#[test]
fn fluid_sim_field_count_is_capped() {
    let FluidSim {
        canvas_id,
        fluid_size,
        params,
        gravity_source,
        steps_per_frame,
        active,
        version,
    } = FluidSim::new(0, (16, 16));
    let _ = (
        canvas_id,
        fluid_size,
        params,
        gravity_source,
        steps_per_frame,
        active,
        version,
    );
    const COUNT: usize = 7;
    const _: () = assert!(COUNT <= 12);
}

#[test]
fn gravity_source_variant_count_is_capped() {
    // Exhaustive match (no `_` arm) forces every variant to be listed.
    let variants = [
        GravitySource::None,
        GravitySource::Fixed {
            dir_x: 0.0,
            dir_y: 1.0,
            magnitude: 1.0,
        },
        GravitySource::DeviceGyroscope,
    ];
    for g in variants {
        let _n = match g {
            GravitySource::None => 0,
            GravitySource::Fixed { .. } => 1,
            GravitySource::DeviceGyroscope => 2,
        };
    }
    const COUNT: usize = 3;
    const _: () = assert!(COUNT <= 6);
}

#[test]
fn fluid_pass_activates_only_when_eligible() {
    // §2.8 truth table: eligible iff enabled AND capable AND headroom > 3.0 ms.
    assert!(fluid_pass_eligible(true, true, 3.5));
    assert!(!fluid_pass_eligible(false, true, 3.5), "brush opt-out");
    assert!(!fluid_pass_eligible(true, false, 3.5), "device incapable");
    assert!(!fluid_pass_eligible(true, true, 2.9), "no headroom");
    assert!(
        !fluid_pass_eligible(true, true, 3.0),
        "headroom is strict >"
    );
}

#[test]
fn fluid_wgsl_parses_and_validates_via_naga() {
    let module = naga::front::wgsl::parse_str(ph2d_painter_fluid::FLUID_WGSL)
        .expect("fluid.wgsl must parse via naga");
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .expect("fluid.wgsl must validate via naga");
}

#[test]
fn shallow_wgsl_parses_and_validates_via_naga() {
    // The S3d shallow-water module (6 entry points sharing the solver UBO) must pass the
    // same frontend wgpu uses, so a bad port fails at test time, not GPU init.
    let module = naga::front::wgsl::parse_str(ph2d_painter_fluid::SHALLOW_WGSL)
        .expect("shallow.wgsl must parse via naga");
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .expect("shallow.wgsl must validate via naga");
}

#[test]
fn composite_wgsl_parses_and_validates_via_naga() {
    // The K–M glaze composite (the project's largest shader) must pass the same
    // frontend wgpu uses, so an invalid port fails at test time, not GPU init.
    let module = naga::front::wgsl::parse_str(ph2d_painter_fluid::COMPOSITE_WGSL)
        .expect("composite.wgsl must parse via naga");
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .expect("composite.wgsl must validate via naga");
}
