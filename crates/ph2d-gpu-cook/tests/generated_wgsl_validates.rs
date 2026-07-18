//! Every WGSL module this crate can generate must parse + validate under naga
//! at `cargo test` time — no device needed, so a kernel typo is caught on any
//! CI lane, not at first dispatch on somebody's GPU (mirrors the render
//! crate's `sprite_wgsl_valid` gate).
//!
//! Coverage is **exhaustive over the presence space**: each registered F1.1
//! kernel × every subset of its readable columns, and the lowering × all 32
//! column subsets — because absence changes the generated text (identity
//! functions, dropped writes), and the absent variants are exactly the ones a
//! smoke test with a full stream never compiles.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::gpu::KernelResolver;

fn validate(label: &str, src: &str) {
    let module = naga::front::wgsl::parse_str(src).unwrap_or_else(|e| {
        panic!(
            "{label}: generated WGSL failed naga parse:\n{}\n--- module ---\n{src}",
            e.emit_to_string(src)
        )
    });
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator.validate(&module).unwrap_or_else(|e| {
        panic!("{label}: naga validation failed: {e:?}\n--- module ---\n{src}")
    });
}

#[test]
fn every_registered_kernel_validates_across_the_whole_presence_space() {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_oscillator::register(&mut reg).unwrap();
    ph2d_node_motion_move::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    // GPU/M5 Fase 2 nodes.
    ph2d_node_motion_transform::register(&mut reg).unwrap();
    ph2d_node_motion_rotate::register(&mut reg).unwrap();
    ph2d_node_motion_scale::register(&mut reg).unwrap();
    ph2d_node_motion_falloff::register(&mut reg).unwrap();
    ph2d_node_motion_tint::register(&mut reg).unwrap();
    ph2d_node_motion_wiggle::register(&mut reg).unwrap();
    ph2d_node_motion_noise::register(&mut reg).unwrap();
    ph2d_node_value_lfo::register(&mut reg).unwrap();
    ph2d_node_motion_look_at::register(&mut reg).unwrap();
    ph2d_node_motion_luminance::register(&mut reg).unwrap();
    ph2d_node_value_map_range::register(&mut reg).unwrap();
    ph2d_node_motion_orbit::register(&mut reg).unwrap();
    ph2d_node_motion_pin_constraint::register(&mut reg).unwrap();
    ph2d_node_motion_stagger::register(&mut reg).unwrap();
    // GPU/M5 Fase 3 — the simulation loop.
    ph2d_node_motion_integrate::register(&mut reg).unwrap();
    ph2d_node_force_wind::register(&mut reg).unwrap();
    ph2d_node_force_drag::register(&mut reg).unwrap();
    ph2d_node_force_attractor::register(&mut reg).unwrap();
    ph2d_node_force_vortex::register(&mut reg).unwrap();
    ph2d_node_force_curl::register(&mut reg).unwrap();
    ph2d_node_force_buoyancy::register(&mut reg).unwrap();
    ph2d_node_motion_spring::register(&mut reg).unwrap();
    ph2d_node_motion_emitter::register(&mut reg).unwrap();
    ph2d_node_motion_color_ramp::register(&mut reg).unwrap();

    let mut validated = 0usize;
    for manifest in reg.manifests() {
        let Some(kernel) = reg.gpu_kernel(manifest.id) else {
            continue;
        };
        if kernel.is_passthrough() {
            continue;
        }
        // The readers of a multi-input kernel are named by port, exactly as the
        // sequencer names them (`encode_kernel_stage`).
        let port_names: Vec<&str> = manifest.inputs.iter().map(|p| p.name).collect();
        let n = kernel.bindings.len().min(16);
        for mask in 0u32..(1 << n) {
            let src = ph2d_gpu_cook::codegen::kernel_module(kernel, &port_names, |b| {
                let idx = kernel
                    .bindings
                    .iter()
                    .position(|x| std::ptr::eq(x, b))
                    .expect("binding belongs to kernel");
                mask & (1 << idx) != 0
            });
            validate(&format!("{} mask {mask:b}", manifest.name), &src);
            validated += 1;
        }
    }
    // Grid (3 bindings → 8) + oscillator (2 → 4) + move (2 → 4) + the Fase 2
    // deformers transform/rotate/scale (2 → 4 each). If a kernel is added or
    // gains a binding this grows — the assert is a floor, not a pin.
    assert!(validated >= 16, "validated only {validated} variants");
}

#[test]
fn the_lowering_validates_for_all_32_column_subsets() {
    for mask in 0u8..32 {
        let present = std::array::from_fn(|i| mask & (1 << i) != 0);
        let src = ph2d_gpu_cook::lower::lower_module(present);
        validate(&format!("lowering mask {mask:05b}"), &src);
    }
}
