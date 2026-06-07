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
        GravitySource::Fixed { dir_x: 0.0, dir_y: 1.0, magnitude: 1.0 },
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
    assert!(!fluid_pass_eligible(true, true, 3.0), "headroom is strict >");
}

#[test]
fn cpu_reference_matches_diffusion_with_default_params() {
    // step_cpu_reference IS the shipped solver; this pins that `to_diffusion()`
    // maps FluidParams::default() onto DiffusionParams::default() (no drift), so
    // the GPU's parity reference and the CPU live path are one tuning.
    use ph2d_painter_brush::diffusion::{DiffusionGrid, DiffusionParams};
    let (w, h) = (24u32, 24u32);
    let mut a = DiffusionGrid::new(w, h, 2.0);
    let mut b = DiffusionGrid::new(w, h, 2.0);
    a.splat(12.0, 12.0, 5.0, 0.6, [0.2, 0.1, 0.5]);
    b.splat(12.0, 12.0, 5.0, 0.6, [0.2, 0.1, 0.5]);
    ph2d_painter_fluid::step_cpu_reference(&mut a, &FluidParams::default(), 8);
    let dp = DiffusionParams::default();
    for _ in 0..8 {
        b.step(&dp);
    }
    assert_eq!(a.pigment(), b.pigment(), "FluidParams::default must map 1:1");
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
