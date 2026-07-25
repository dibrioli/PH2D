//! Gates for the **FIELD family** GPU/M5 documents (`PH2D_GPU_COOK_DEMO=17..22`),
//! split out of `motion_state_gpu_tests.rs` at the HR-18 cap — the sibling of
//! `motion_state_gpu_field_demos.rs`, which holds the documents these pin.
//!
//! Each one pins the PLAN a field document must exercise (fully-GPU), so a scene
//! that quietly stopped being claimed would look identical on screen and the
//! reviewer would sign off on the CPU pump's memo. Includes the A1-gpu Curve
//! contour, now device-resident via the LUT channel.

use super::*;

/// The **FIELD family smoke really runs on the device** (`PH2D_GPU_COOK_DEMO=17`).
///
/// `field.index_range` writes the `falloff` mask on the GPU and a Solid tint reads
/// it there; the whole `grid → field.index_range → tint → output` chain must be
/// claimed whole, or the artist would smoke the CPU pump's memo (the band looks
/// identical on screen, just cooked slower) and sign off on a path that never ran
/// — the exact failure this file exists to prevent.
#[test]
fn the_field_index_range_demo_is_fully_gpu() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_field_index_range_demo_document(&mut doc, &registry)
        .expect("well-typed field.index_range demo");
    let out = *sinks.first().expect("one sink");
    // The scene really contains the node under test — a gate that planned an empty
    // or wrong graph fully-GPU would be vacuously green.
    assert!(
        doc.graph
            .nodes()
            .iter()
            .any(|n| n.type_id() == ph2d_nodegraph::node::NodeTypeId::of("field.index_range")),
        "the demo must contain the field.index_range node it exists to smoke"
    );
    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "field.index_range -> tint must be claimed whole — a CPU boundary here and \
         the smoke would be reading the pump's memo, not the device: {:?}",
        plan.boundaries
    );
}

/// The **spatial** field smoke really runs on the device (`PH2D_GPU_COOK_DEMO=18`).
/// `field.box` reads `P` and writes the `falloff` mask on the GPU; the whole
/// `grid -> motion.scale -> field.box -> tint -> output` chain must be claimed
/// whole, or the artist smokes the CPU pump's memo, not the device.
#[test]
fn the_field_box_demo_is_fully_gpu() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks =
        build_gpu_field_box_demo_document(&mut doc, &registry).expect("well-typed field.box demo");
    let out = *sinks.first().expect("one sink");
    assert!(
        doc.graph
            .nodes()
            .iter()
            .any(|n| n.type_id() == ph2d_nodegraph::node::NodeTypeId::of("field.box")),
        "the demo must contain the field.box node it exists to smoke"
    );
    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "field.box -> tint must be claimed whole — a CPU boundary here and the \
         smoke would be reading the pump's memo, not the device: {:?}",
        plan.boundaries
    );
}

/// The **composition** smoke really runs on the device (`PH2D_GPU_COOK_DEMO=19`).
/// Two field branches off one grid (a fan-out) unioned by `field.combine`; the
/// whole fan-out — grid cooked once, both branches, the 2-input composer, the
/// tint — must be claimed whole, or the artist smokes the CPU pump, not the device.
#[test]
fn the_field_combine_demo_is_fully_gpu() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_field_combine_demo_document(&mut doc, &registry)
        .expect("well-typed field.combine demo");
    let out = *sinks.first().expect("one sink");
    assert!(
        doc.graph
            .nodes()
            .iter()
            .any(|n| n.type_id() == ph2d_nodegraph::node::NodeTypeId::of("field.combine")),
        "the demo must contain the field.combine node it exists to smoke"
    );
    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "the field.combine fan-out must be claimed whole — a CPU boundary here and \
         the smoke reads the pump's memo, not the device: {:?}",
        plan.boundaries
    );
}

/// `field.radial_sweep` reads `P` and writes the `falloff` mask on the GPU; the
/// whole `grid -> motion.scale -> field.radial_sweep -> tint -> output` chain must
/// be claimed whole, or the `=20` smoke reads the CPU pump's memo, not the device.
/// The angular sector is HR-5 (no `atan2`), so nothing about it forces a boundary.
#[test]
fn the_field_radial_sweep_demo_is_fully_gpu() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_field_radial_sweep_demo_document(&mut doc, &registry)
        .expect("well-typed field.radial_sweep demo");
    let out = *sinks.first().expect("one sink");
    assert!(
        doc.graph
            .nodes()
            .iter()
            .any(|n| n.type_id() == ph2d_nodegraph::node::NodeTypeId::of("field.radial_sweep")),
        "the demo must contain the field.radial_sweep node it exists to smoke"
    );
    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "field.radial_sweep -> tint must be claimed whole — a CPU boundary here and \
         the smoke reads the pump's memo, not the device: {:?}",
        plan.boundaries
    );
}

/// `field.remap` reads and rewrites the `falloff` mask on the GPU; the whole
/// `grid -> motion.scale -> field.box -> field.remap -> tint -> output` chain must be
/// claimed whole, or the `=21` smoke reads the CPU pump's memo, not the device. The
/// remap is a position-blind transfer function, so nothing about it forces a boundary.
#[test]
fn the_field_remap_demo_is_fully_gpu() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_field_remap_demo_document(&mut doc, &registry)
        .expect("well-typed field.remap demo");
    let out = *sinks.first().expect("one sink");
    assert!(
        doc.graph
            .nodes()
            .iter()
            .any(|n| n.type_id() == ph2d_nodegraph::node::NodeTypeId::of("field.remap")),
        "the demo must contain the field.remap node it exists to smoke"
    );
    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    assert!(
        plan.is_fully_gpu(),
        "field.box -> field.remap -> tint must be claimed whole — a CPU boundary here \
         and the smoke reads the pump's memo, not the device: {:?}",
        plan.boundaries
    );
}

/// The **Curve contour demo is now fully-GPU** (`PH2D_GPU_COOK_DEMO=22`, A1-gpu): the
/// same `field.remap`, but with `contour = Curve`. Its shape is a text param the uniform
/// layout cannot carry, so before A1-gpu the kernel declined (`applicable` false at mode
/// 4) and the remap was a CPU boundary. The LUT channel bakes that curve to a device
/// buffer the kernel samples (`rm_curve_sample`), so the whole
/// `grid -> scale -> field.box -> field.remap(Curve) -> tint -> output` chain is claimed
/// whole. This pins the win at the PLAN level: a regression that re-declined mode 4 would
/// split the smoke back onto the CPU pump's memo with nothing on screen to say so.
#[test]
fn the_field_curve_demo_is_fully_gpu_via_the_lut_channel() {
    let mut registry = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut registry).expect("registry builds");
    let mut doc = MotionDoc::new();
    let sinks = build_gpu_field_curve_demo_document(&mut doc, &registry)
        .expect("well-typed field.remap curve demo");
    let out = *sinks.first().expect("one sink");
    // The demo (built by `build_gpu_field_curve_demo_document`, which sets contour = 4)
    // must contain the field.remap node it exists to smoke — the node A1-gpu lowered.
    assert!(
        doc.graph
            .nodes()
            .iter()
            .any(|n| n.type_id() == ph2d_nodegraph::node::NodeTypeId::of("field.remap")),
        "the demo must contain the field.remap running the Curve contour"
    );
    let plan = ph2d_gpu_cook::plan(&doc.graph, &registry, &registry, out);
    // Fully-GPU now — the Curve contour samples its LUT on the device, no boundary.
    assert!(
        plan.is_fully_gpu(),
        "the Curve contour must cook on the device via the LUT (A1-gpu), not fall back: {:?}",
        plan.boundaries
    );
}
