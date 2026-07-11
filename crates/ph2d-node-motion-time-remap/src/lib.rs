//! `motion.time_remap` — rewrite the clock of everything wired ABOVE this node
//! (Motion Nodes M2.N1, plan §1.5).
//!
//! Slow motion on the impact, a looping background rig, a frozen still while the
//! physics below keeps running, an explosion played backwards. The reference
//! catalogue's timeRemap: *"Modifica o `time` passado à sub-árvore UPSTREAM (não
//! a si mesmo)"* — and that is exactly what this node cannot do by itself.
//!
//! **The node is a passthrough. The remap happens in the puller.** A node only
//! ever sees its own resolved inputs (`EvalCtx`, the FBP black box of ADR-0031);
//! nothing inside `eval` can change the playhead its upstream was pulled at. So
//! the params here are *read by the domain layer* (`ph2d_eval_motion::time_scopes`)
//! into a [`ph2d_nodegraph::time::TimeMap`], and the cook applies it while
//! descending into this node's inputs ([`ph2d_nodegraph::cook::Cook::cook_scoped`]).
//! `eval` then just forwards the (already correctly-timed) stream.
//!
//! Consequences the artist can see:
//! - **Freeze is free**: a constant `t'` means the whole subtree hits the memo
//!   every frame — one cook for the life of the still.
//! - **Downstream keeps its own clock**: physics under a frozen rig still runs.
//! - **A sequential node (spring / integrate) may not sit upstream**: its state
//!   is a recurrence over the outer tick, so a rewritten clock has no meaning.
//!   The cook refuses it (`CookError::SequentialInTimeScope`) instead of
//!   producing a plausible wrong trajectory.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use ph2d_nodegraph::time::{TimeMap, TimeMode};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Option order of the `mode` enum param — mirrors [`TimeMode`]'s declaration
/// order (`TimeMode::from_index`) and the panel's segmented selector labels.
/// Changing one without the others silently re-labels every saved document.
pub const MODE_LABELS: &[&str] = &["Scale", "Loop", "Ping Pong", "Freeze", "Reverse"];

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.time_remap"),
    name: "motion.time_remap",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Pure: this node reads no clock of its own — it is a passthrough. Its
    // output changes only when the upstream (cooked at `t'`) changes, which the
    // cook already tracks through the input revision.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "mode",
            default: 0.0, // Scale — the identity, so a dropped node changes nothing
        },
        ParamSpec {
            name: "scale",
            default: 1.0,
        },
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        ParamSpec {
            name: "duration",
            default: 2.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Read this node's params into the [`TimeMap`] the cook applies to its
/// upstream. `param` is the caller's param resolver (override, else the
/// manifest default) — the domain layer owns that lookup, since the substrate
/// hands params out only inside `eval`.
///
/// Kept here, next to the `ParamSpec`s, so the param names live in exactly one
/// place: a rename that misses this function is a compile-time-silent, but the
/// node's own golden test below catches it.
#[must_use]
pub fn time_map_from(param: impl Fn(&str) -> f32) -> TimeMap {
    TimeMap {
        mode: TimeMode::from_index(param("mode")),
        scale: f64::from(param("scale")),
        offset: f64::from(param("offset")),
        duration: f64::from(param("duration")),
    }
}

/// Collect the time scopes a motion graph declares: one [`TimeMap`] per
/// `motion.time_remap` node, ready for
/// [`ph2d_nodegraph::cook::Cook::cook_scoped`].
///
/// The substrate deliberately knows no node types (scopes are keyed by
/// `NodeId`), so *someone* must translate "this node type means remap" into a
/// map. That someone is this crate — the one that owns both the type name and
/// the param names. Identity maps are skipped, so a freshly-dropped node adds
/// no scope lane at all.
///
/// Cheap enough to rebuild every frame: one pass over the node list, and the
/// map is empty for the overwhelmingly common graph with no remapper.
#[must_use]
pub fn time_scopes(
    graph: &ph2d_nodegraph::graph::Graph,
    ops: &dyn ph2d_nodegraph::cook::OpResolver,
) -> ph2d_nodegraph::cook::TimeScopes {
    let mut scopes = ph2d_nodegraph::cook::TimeScopes::new();
    for inst in graph.nodes() {
        if inst.type_name != MANIFEST.name {
            continue;
        }
        let Some(manifest) = ops.resolve(inst.type_id()).map(NodeOp::manifest) else {
            continue;
        };
        let overrides = graph.node_param_overrides(inst.id);
        let map = time_map_from(|name| {
            overrides
                .and_then(|o| o.get(name).copied())
                .or_else(|| manifest.param_default(name))
                .unwrap_or(0.0)
        });
        if !map.is_identity() {
            scopes.insert(inst.id, map);
        }
    }
    scopes
}

/// Does `remapper`'s upstream subtree contain a **sequential** node (one fed by
/// a `pre` edge — a spring, an integrate)? Such a node integrates a recurrence
/// over the outer tick and cannot run on a rewritten clock, so the cook refuses
/// it ([`ph2d_nodegraph::cook::CookError::SequentialInTimeScope`]).
///
/// The editor calls this **before committing a wire**, to refuse the edit with
/// an explanation instead of letting the scene silently stop cooking.
#[must_use]
pub fn scopes_a_sequential_node(
    graph: &ph2d_nodegraph::graph::Graph,
    remapper: ph2d_nodegraph::graph::NodeId,
) -> bool {
    let is_sequential = |n| graph.edges().iter().any(|e| e.delayed && e.to.0 == n);
    let mut stack = vec![remapper];
    let mut seen = std::collections::BTreeSet::new();
    while let Some(node) = stack.pop() {
        if !seen.insert(node) {
            continue;
        }
        if node != remapper && is_sequential(node) {
            return true;
        }
        // Walk the FORWARD upstream only: a `pre` edge is not a cook-time
        // dependency (it reads last tick's snapshot), so it never carries the
        // remapped clock into its source.
        for e in graph.edges() {
            if !e.delayed && e.to.0 == node {
                stack.push(e.from.0);
            }
        }
    }
    false
}

struct MotionTimeRemap;

impl NodeOp for MotionTimeRemap {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // The stream already arrived sampled at `t'` (the cook remapped the
        // clock on the way up). Nothing left to do but pass it along.
        let passthrough = ctx.input(0).clone();
        ctx.emit(passthrough);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionTimeRemap))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Time Remap",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param rows: a named mode selector (never a number the artist must decode),
/// then the three scalars the modes read. Seconds, matching the playhead.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: MODE_LABELS,
        },
    },
    ParamUiHint {
        param: "scale",
        label: "Speed",
        min: -4.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset",
        label: "Offset",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "duration",
        label: "Duration",
        min: 0.1,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::{Column, Stream};

    /// The mode selector's labels and `TimeMode`'s indices are one vocabulary
    /// split across two crates: a reorder on either side silently re-labels
    /// every saved document (a "Loop" that becomes a "Ping Pong" on reload).
    #[test]
    fn the_mode_labels_match_the_time_mode_indices() {
        let modes = [
            TimeMode::Scale,
            TimeMode::Loop,
            TimeMode::PingPong,
            TimeMode::Freeze,
            TimeMode::Reverse,
        ];
        assert_eq!(MODE_LABELS.len(), modes.len());
        for (i, mode) in modes.into_iter().enumerate() {
            assert_eq!(
                TimeMode::from_index(i as f32),
                mode,
                "label {:?} decodes to the wrong mode",
                MODE_LABELS[i]
            );
        }
    }

    /// The manifest's defaults must build the IDENTITY map: dropping the node
    /// onto a live chain has to change nothing until the artist picks a mode.
    /// (A default `duration` of 0 would also be a latent divide-by-zero.)
    #[test]
    fn the_default_params_build_the_identity_map() {
        let map = time_map_from(|name| MANIFEST.param_default(name).expect("declared param"));
        assert!(map.is_identity(), "a fresh Time Remap warps time: {map:?}");
        assert_eq!(map.apply(1.25), 1.25);
        assert!(map.duration > 0.0);
    }

    #[test]
    fn params_decode_into_the_map_the_cook_applies() {
        let map = time_map_from(|name| match name {
            "mode" => 1.0, // Loop
            "scale" => 2.0,
            "offset" => 0.5,
            "duration" => 3.0,
            _ => panic!("undeclared param {name}"),
        });
        assert_eq!(map.mode, TimeMode::Loop);
        // t=2 → scaled 4 → 4 mod 3 = 1 → +0.5.
        assert_eq!(map.apply(2.0), 1.5);
    }

    /// The node is a passthrough: it forwards its input verbatim (the cook did
    /// the remapping upstream). Anything else would double-count the warp.
    #[test]
    fn eval_forwards_the_stream_verbatim() {
        struct Reg;
        impl ph2d_nodegraph::cook::OpResolver for Reg {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == MANIFEST.id).then_some(&MotionTimeRemap)
            }
        }
        use ph2d_nodegraph::cook::Cook;
        use ph2d_nodegraph::graph::{Edge, Graph};

        // A tiny source so the remap has something to forward.
        static SRC_MAN: NodeManifest = NodeManifest {
            id: NodeTypeId::of("test.src"),
            name: "test.src",
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: INST_VEC2,
            }],
            effect: Effect::Pure,
            clock: Clock::Frame,
            params: &[],
            lowerings: &[LoweringKind::Cpu],
        };
        struct Src;
        impl NodeOp for Src {
            fn manifest(&self) -> &'static NodeManifest {
                &SRC_MAN
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[3.0, 4.0]])));
            }
        }
        struct Ops;
        impl ph2d_nodegraph::cook::OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == MANIFEST.id => Some(&MotionTimeRemap),
                    t if t == SRC_MAN.id => Some(&Src),
                    _ => None,
                }
            }
        }
        let _ = Reg;

        let mut g = Graph::new();
        let src = g.add_node("test.src");
        let remap = g.add_node("motion.time_remap");
        g.connect(Edge {
            from: (src, 0),
            to: (remap, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, remap, 0.0).unwrap();
        match out[0].as_stream().get("P") {
            Some(Column::Vec2(p)) => assert_eq!(p, &vec![[3.0, 4.0]]),
            other => panic!("expected the input stream verbatim, got {other:?}"),
        }
    }

    /// The SAFETY refusal the editor leans on (audit 2026-07-10 flagged it as
    /// untested): a sequential node (one fed by a `pre` edge — spring,
    /// integrate) in the remapper's FORWARD upstream must be detected, so the
    /// wire is refused instead of the scene going dark at cook time
    /// (`SequentialInTimeScope`).
    #[test]
    fn detects_a_sequential_node_in_the_forward_upstream_only() {
        use ph2d_nodegraph::graph::{Edge, Graph};
        let fwd = |from, to| Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        };

        // src → spring(⟲ pre self-loop on port 1) → remap: upstream IS
        // sequential → refused.
        let mut g = Graph::new();
        let src = g.add_node("test.src");
        let spring = g.add_node("test.spring");
        let remap = g.add_node("motion.time_remap");
        g.connect(fwd(src, spring)).unwrap();
        g.connect(Edge {
            from: (spring, 0),
            to: (spring, 1),
            delayed: true,
        })
        .unwrap();
        g.connect(fwd(spring, remap)).unwrap();
        assert!(
            scopes_a_sequential_node(&g, remap),
            "a pre-fed node upstream must be refused"
        );

        // src → remap → spring(⟲): the sequential node is DOWNSTREAM — the
        // remapper never rewrites its clock → allowed.
        let mut g = Graph::new();
        let src = g.add_node("test.src");
        let remap = g.add_node("motion.time_remap");
        let spring = g.add_node("test.spring");
        g.connect(fwd(src, remap)).unwrap();
        g.connect(fwd(remap, spring)).unwrap();
        g.connect(Edge {
            from: (spring, 0),
            to: (spring, 1),
            delayed: true,
        })
        .unwrap();
        assert!(
            !scopes_a_sequential_node(&g, remap),
            "downstream sequential nodes are not scoped"
        );

        // spring(⟲) --pre--> remap: reachable only across a `pre` edge — not a
        // cook-time dependency, the remapped clock never crosses it → allowed.
        let mut g = Graph::new();
        let spring = g.add_node("test.spring");
        let remap = g.add_node("motion.time_remap");
        g.connect(Edge {
            from: (spring, 0),
            to: (spring, 1),
            delayed: true,
        })
        .unwrap();
        g.connect(Edge {
            from: (spring, 0),
            to: (remap, 0),
            delayed: true,
        })
        .unwrap();
        assert!(
            !scopes_a_sequential_node(&g, remap),
            "a pre edge does not carry the remapped clock upstream"
        );

        // A purely combinational upstream chain → allowed.
        let mut g = Graph::new();
        let src = g.add_node("test.src");
        let mid = g.add_node("test.mid");
        let remap = g.add_node("motion.time_remap");
        g.connect(fwd(src, mid)).unwrap();
        g.connect(fwd(mid, remap)).unwrap();
        assert!(!scopes_a_sequential_node(&g, remap));
    }
}
