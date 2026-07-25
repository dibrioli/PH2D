//! **GPU coverage census** — which kernel-less nodes sit in the CPU prefix of
//! the documents that actually exist (item #1 of the continuation handoff, the
//! method of `a2226787`).
//!
//! The next coverage kernel is CHOSEN by measurement, not by a shortlist: a
//! kernel is worth writing when the node it covers is what forces a real
//! document onto the CPU. This runs the planner ([`ph2d_gpu_cook::plan`]) over
//! the boot document (the snow `sim.zone` sim — the one artist-grade document
//! the app opens with) and the six `PH2D_GPU_COOK_DEMO` scenes, and reports, per
//! document: is it fully GPU-resident, and if not, which node type is the
//! **frontier** boundary (the immediate blocker) and which node types make up
//! the whole CPU **prefix** behind it.
//!
//! It is pure plan-time analysis — no device — so it runs on any CI lane. Read
//! the report with `--nocapture`:
//!
//! ```text
//! cargo test -p ph2d-host-desktop gpu_coverage --  --nocapture
//! ```
//!
//! The one assertion is the decision-relevant invariant: at least one real
//! document still has a CPU boundary (there is coverage work to do). The rest is
//! a printout to read, deliberately not a set of exact-boundary pins — those
//! live in `motion_state_gpu_tests.rs`, and pinning them here too would make the
//! census red exactly when a boundary is FIXED, which is the wrong signal for a
//! thing whose job is to point at the next fix.

use super::build_default_document;
use super::gpu_deform_demo::{
    build_gpu_deform_demo_document, build_gpu_four_point_warp_demo_document,
    build_gpu_kaleidoscope_demo_document, build_gpu_spherize_demo_document,
};
use super::gpu_demos::{
    build_gpu_demo_document, build_gpu_emitter_demo_document, build_gpu_field_box_demo_document,
    build_gpu_field_combine_demo_document, build_gpu_field_index_range_demo_document,
    build_gpu_field_radial_sweep_demo_document, build_gpu_hybrid_demo_document,
    build_gpu_sea_demo_document, build_gpu_sim_demo_document,
};
use super::gpu_panel_demo::build_gpu_panel_demo_document;
use super::gpu_zone_demo::build_gpu_zone_demo_document;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Graph, NodeId};
use std::collections::{BTreeMap, BTreeSet};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
    reg
}

/// One document to measure: a display name, its graph, and its render sinks.
struct Doc {
    name: &'static str,
    doc: MotionDoc,
    sinks: Vec<NodeId>,
}

/// The corpus: every graph the repo actually builds. The boot document is the
/// only artist-grade one (the demos are GPU-path smoke scaffolds, several of
/// them hand-shaped to be fully-GPU on purpose) — so a boundary in the BOOT
/// document weighs differently from one in a demo, and the report names it.
fn corpus(reg: &NodeRegistry) -> Vec<Doc> {
    let mut out = Vec::new();
    let mut push = |name: &'static str, build: &dyn Fn(&mut MotionDoc) -> Option<Vec<NodeId>>| {
        let mut doc = MotionDoc::new();
        let sinks = build(&mut doc).unwrap_or_else(|| panic!("{name} builds a well-typed graph"));
        out.push(Doc { name, doc, sinks });
    };
    // The real one: the boot document (snow `sim.zone` sim) — what every user
    // sees on launch, and the only graph here an artist could have authored.
    push("boot (snow sim.zone)", &|d| build_default_document(d, reg));
    push("demo=1 grid->osc->move", &|d| {
        build_gpu_demo_document(d, reg)
    });
    push("demo=2 grid->sort->osc->scale", &|d| {
        build_gpu_hybrid_demo_document(d, reg)
    });
    push("demo=3 sim vortex loop", &|d| {
        build_gpu_sim_demo_document(d, reg)
    });
    push("demo=4 sea wind/buoyancy loop", &|d| {
        build_gpu_sea_demo_document(d, reg)
    });
    push("demo=5 emitter fountain", &|d| {
        build_gpu_emitter_demo_document(d, reg)
    });
    push("demo=6 panel value branch", &|d| {
        build_gpu_panel_demo_document(d, reg)
    });
    // The first field.* focus-field source — the index-keyed `falloff` mask. It
    // is fully-GPU (grid → field.index_range → tint), so a boundary here would be
    // a real regression, not an inherent CPU node.
    push("demo=17 field.index_range band", &|d| {
        build_gpu_field_index_range_demo_document(d, reg)
    });
    // The spatial box field — reads P, writes `falloff`; fully-GPU like its
    // ordinal sibling, so a boundary here is a real regression.
    push("demo=18 field.box band", &|d| {
        build_gpu_field_box_demo_document(d, reg)
    });
    // The 2-input composer over a FAN-OUT (two field branches off one grid) — the
    // first field scene that is not a single chain. If `field.combine` or the
    // fan-out ever refused the device, the census would name the boundary.
    push("demo=19 field.combine cross", &|d| {
        build_gpu_field_combine_demo_document(d, reg)
    });
    // The ANGULAR field — reads P, writes `falloff`; the pseudo-angle sector +
    // radial clip, fully-GPU like its rectangular sibling. Wired in so the census
    // is not BLIND to it (a hole nobody puts in a document is an absence, not a
    // clean frontier — the lesson the deformer scene below pins).
    push("demo=20 field.radial_sweep star", &|d| {
        build_gpu_field_radial_sweep_demo_document(d, reg)
    });
    push("demo=10 sim.zone snow globe", &|d| {
        build_gpu_zone_demo_document(d, reg)
    });
    // ⚠️ The DEFORMER scene, and it earns its place by CHANGING WHAT THE CENSUS
    // CAN SEE. Until this document existed the corpus contained no deformer at
    // all, so the census reported a clean frontier while an entire node family
    // was CPU-only — the census counts FRONTIERS, and a hole nobody wired into a
    // document is not a frontier, it is an absence.
    push("demo=12 deformer cloth (bend->twist)", &|d| {
        build_gpu_deform_demo_document(d, reg)
    });
    // The `Sum` centroid deformer — the two-reduction node.
    push("demo=13 spherize lens (Sum centroid)", &|d| {
        build_gpu_spherize_demo_document(d, reg)
    });
    // The four-reduction bounding-box deformer (the first Min user).
    push("demo=14 four-point-warp (bbox)", &|d| {
        build_gpu_four_point_warp_demo_document(d, reg)
    });
    // The count-changing SourceRows deformer that reads its template.
    push("demo=15 kaleidoscope (SourceRows fan-out)", &|d| {
        build_gpu_kaleidoscope_demo_document(d, reg)
    });
    out
}

fn ty_name(g: &Graph, n: NodeId) -> String {
    g.node(n)
        .map(|i| i.type_name.clone())
        .unwrap_or_else(|| "<gone>".into())
}

/// Every node upstream of (and including) `start`, over EVERY input edge —
/// forward and `pre`/delayed. The CPU cooks a boundary by cooking its whole
/// chain, and for a sim node that chain closes through a `pre` back into the
/// force loop, so a walk that skipped delayed edges would under-count the
/// prefix. This is the set of node types the CPU must run to feed the seam.
fn upstream_closure(g: &Graph, start: NodeId) -> BTreeSet<u32> {
    let mut seen = BTreeSet::new();
    let mut stack = vec![start];
    while let Some(n) = stack.pop() {
        if !seen.insert(n.0) {
            continue;
        }
        for e in g.edges() {
            if e.to.0 == n {
                stack.push(e.from.0);
            }
        }
    }
    seen
}

#[test]
fn gpu_coverage_census() {
    let reg = registry();
    let docs = corpus(&reg);

    // How often each type is the IMMEDIATE boundary (the frontier — the node a
    // kernel would have to cover to advance the claim) across every sink.
    let mut frontier: BTreeMap<String, usize> = BTreeMap::new();
    // How often each type appears ANYWHERE in a CPU prefix (frontier + its
    // upstream), counted once per sink.
    let mut prefix: BTreeMap<String, usize> = BTreeMap::new();

    let mut any_boundary = false;

    println!("\n=== GPU coverage census — plan() over the documents that exist ===\n");
    for Doc { name, doc, sinks } in &docs {
        let g = &doc.graph;
        for &sink in sinks {
            let plan = ph2d_gpu_cook::plan(g, &reg, &reg, sink);
            let full = plan.is_fully_gpu();
            let dispatching = plan.dispatching_stages(&reg);
            print!(
                "[{}] sink `{}`: {} — {} GPU stage(s) dispatch",
                name,
                ty_name(g, sink),
                if full { "FULLY GPU" } else { "HYBRID/CPU" },
                dispatching,
            );
            if full {
                println!();
                continue;
            }
            any_boundary = true;

            // Frontier: the immediate boundaries, with whether the node has a
            // kernel at all (refused-despite-kernel is a different problem than
            // no-kernel — a driven param, a column-shape refusal, or the whole
            // sim-state recede).
            let mut frontier_names = Vec::new();
            let mut prefix_ids: BTreeSet<u32> = BTreeSet::new();
            for &(bnode, _port) in &plan.boundaries {
                let tn = ty_name(g, bnode);
                let has_kernel = g
                    .node(bnode)
                    .and_then(|i| reg_has_kernel(&reg, i.type_id()))
                    .unwrap_or(false);
                frontier_names.push(if has_kernel {
                    format!("{tn} [refused-despite-kernel]")
                } else {
                    format!("{tn} [no-kernel]")
                });
                *frontier.entry(tn).or_default() += 1;
                prefix_ids.extend(upstream_closure(g, bnode));
            }
            println!("  boundaries: {}", frontier_names.join(", "));

            // Prefix: every node type the CPU must cook behind the seam.
            let mut prefix_types: BTreeSet<String> = BTreeSet::new();
            for id in &prefix_ids {
                prefix_types.insert(ty_name(g, NodeId(*id)));
            }
            for t in &prefix_types {
                *prefix.entry(t.clone()).or_default() += 1;
            }
            println!(
                "  CPU prefix ({} nodes, {} types): {}",
                prefix_ids.len(),
                prefix_types.len(),
                prefix_types.into_iter().collect::<Vec<_>>().join(", "),
            );
        }
    }

    println!("\n--- frontier tally (immediate boundary; the node a kernel must cover) ---");
    for (t, n) in sorted_desc(&frontier) {
        println!("  {n:>3}×  {t}");
    }
    println!("\n--- CPU-prefix tally (appears behind a seam, per sink) ---");
    for (t, n) in sorted_desc(&prefix) {
        println!("  {n:>3}×  {t}");
    }
    println!();

    assert!(
        !docs.is_empty() && any_boundary,
        "the census measures nothing if the corpus is empty or every document is already fully GPU"
    );
}

/// Sorted by count desc, then name — a stable order for the report.
fn sorted_desc(m: &BTreeMap<String, usize>) -> Vec<(String, usize)> {
    let mut v: Vec<(String, usize)> = m.iter().map(|(k, &n)| (k.clone(), n)).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v
}

/// Does the registry carry a kernel for this type? (`KernelResolver` is the
/// same `reg`; this is just the `Option::is_some` in a spot the borrow checker
/// likes.)
fn reg_has_kernel(reg: &NodeRegistry, ty: ph2d_nodegraph::node::NodeTypeId) -> Option<bool> {
    use ph2d_nodegraph::gpu::KernelResolver;
    Some(reg.gpu_kernel(ty).is_some())
}
