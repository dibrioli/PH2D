#![forbid(unsafe_code)]
//! **Reroute** — the dot you drop on a wire to bend it *and* to branch from it (Motion
//! Nodes F2, doc 45). Blender's `Reroute`, Nuke's `Dot`.
//!
//! ## It is a NODE, and that is the whole point
//!
//! The first version of this was **decoration**: a waypoint on the document that bent the
//! wire and could not be connected to. It bent wires perfectly well and it was a **lie** —
//! it looked exactly like a junction and was not one. Enio's smoke found it in one
//! question: *"e como se conecta esse ponto no meio do fio? ou não se conecta?"*
//!
//! Blender and Nuke both answer that question the same way, and neither of them ships a
//! decorative bend: **the dot is a node.** Then every question about it has the obvious
//! answer, because it is the answer every other node already has —
//!
//! - *Does it connect?* It has an input and an output. **Drag a wire out of it** and you
//!   have branched mid-route, without going back to a source socket across the canvas.
//! - *What does the knife do?* It cuts **the wire you crossed**. The reroute survives, with
//!   whatever wires you did not cut — exactly like cutting the input of any other node.
//! - *Delete? Undo? Select? Box-select? Ctrl+D?* Free. It is a node.
//!
//! ## Pass-through, and therefore free
//!
//! `eval` emits its input, unchanged. `Effect::Pure`, so it memoizes like anything else and
//! a chain of reroutes costs one clone of an already-computed stream. The cook does see it
//! (it is a node), which is the price of it being real — and the price is a hop, not a
//! recomputation.
//!
//! ## Three types, one gesture
//!
//! `NodeManifest`'s ports are `&'static` (the frozen contract), so a node type carries ONE
//! port type — there is no generic reroute to be had without unfreezing it, and unfreezing
//! the contract to save two `const`s would be a poor trade. The whole 79-node library uses
//! exactly three port types, so there are exactly three reroutes:
//!
//! | node | port type | what flows |
//! |---|---|---|
//! | `util.reroute` | `(Instances, Vec2, Frame)` | an instance stream |
//! | `util.reroute_value` | `(Instances, Scalar, Frame)` | a value |
//! | `util.reroute_pulse` | `(Instances, Scalar, Event)` | a pulse |
//!
//! **The artist never chooses.** The gesture is *double-click a wire*, and the shell knows
//! that wire's type — it inserts the reroute that fits (doc 45). They are in the add-menu
//! too, for anyone who wants to place one first and wire it after.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::Stream;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);

/// The canonical type name the shell's wire-splice gesture inserts for an instance stream.
pub const TYPE_STREAM: &str = "util.reroute";
/// …for a value wire.
pub const TYPE_VALUE: &str = "util.reroute_value";
/// …for a pulse wire.
pub const TYPE_PULSE: &str = "util.reroute_pulse";

/// The three manifests. Identical but for their id, name and port type — a reroute IS its
/// port type, and nothing else.
macro_rules! reroute {
    ($konst:ident, $ty:expr, $name:literal, $clock:expr) => {
        pub const $konst: NodeManifest = NodeManifest {
            id: NodeTypeId::of($name),
            name: $name,
            inputs: &[PortSpec {
                name: "in",
                ty: $ty,
            }],
            outputs: &[PortSpec {
                name: "out",
                ty: $ty,
            }],
            effect: Effect::Pure,
            clock: $clock,
            params: &[],
            lowerings: &[LoweringKind::Cpu],
        };
    };
}

reroute!(MANIFEST_STREAM, INST_VEC2, "util.reroute", Clock::Frame);
reroute!(MANIFEST_VALUE, VALUE, "util.reroute_value", Clock::Frame);
reroute!(MANIFEST_PULSE, PULSE, "util.reroute_pulse", Clock::Event);

/// A reroute of one port type. The op is the same for all three: emit the input, unchanged.
struct Reroute(&'static NodeManifest);

impl NodeOp for Reroute {
    fn manifest(&self) -> &'static NodeManifest {
        self.0
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // Pass through, column for column. An UNCONNECTED reroute emits an empty stream —
        // the same "nothing in, nothing out" as any other node, and the editor's inline
        // readout will show it as `empty` rather than leaving the artist guessing (doc 43).
        let out = {
            let input = ctx.input(0);
            let mut out = Stream::new(input.count());
            for (name, col) in input.columns() {
                out.set(name.clone(), col.clone());
            }
            out
        };
        ctx.emit(out);
    }
}

/// Register all three reroutes. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
///
/// One crate, three node types: they differ only by a `const`, and three crates of forty
/// identical lines would be three places for the same bug to hide.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    for (manifest, display) in [
        (&MANIFEST_STREAM, "Reroute"),
        (&MANIFEST_VALUE, "Reroute (Value)"),
        (&MANIFEST_PULSE, "Reroute (Pulse)"),
    ] {
        reg.register(Box::new(Reroute(manifest)))?;
        reg.register_ui(
            manifest.id,
            ph2d_node_registry::NodeUiManifest {
                display_name: display,
                category: ph2d_node_registry::NodeUiCategory::Utility,
                // A circle: it is a DOT on a wire, which is what every node editor draws and
                // what the gesture that makes it leads the artist to expect.
                silhouette: ph2d_node_registry::NodeSilhouette::Circle,
            },
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::graph::{Edge, Graph};

    fn registry() -> NodeRegistry {
        let mut reg = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
        reg
    }

    /// **A reroute passes its stream through, column for column** — so splicing one into a
    /// wire changes nothing about what the graph computes. That is what makes the gesture
    /// safe: the artist tidies the canvas and the render does not move a pixel.
    #[test]
    fn a_reroute_is_a_pass_through() {
        let reg = registry();
        let mut g = Graph::new();
        let grid = g.add_node("motion.grid");
        let dot = g.add_node(TYPE_STREAM.to_string());
        let out = g.add_node("motion.output");
        g.set_param(grid, "rows", 3.0);
        g.set_param(grid, "cols", 4.0);
        for (from, to) in [(grid, dot), (dot, out)] {
            g.connect(Edge {
                from: (from, 0),
                to: (to, 0),
                delayed: false,
            })
            .expect("wire");
        }
        assert!(
            g.validate(&reg).is_ok(),
            "a reroute is well-typed in a chain"
        );

        let mut cook = Cook::new();
        let cooked = cook.cook(&g, &reg, out, 0.0).expect("cooks");
        let stream = cooked[0].as_stream();
        assert_eq!(stream.count(), 12, "3x4 instances arrived through the dot");
        match stream.get("P") {
            Some(Column::Vec2(v)) => assert_eq!(v.len(), 12),
            _ => panic!("the positions came through"),
        }
    }

    /// **A reroute BRANCHES**: one output feeds several inputs, which is the whole reason to
    /// want a dot mid-wire rather than a bend. FALSIFIED by the decoration this replaced —
    /// a waypoint could not be connected to at all.
    #[test]
    fn one_reroute_feeds_several_consumers() {
        let reg = registry();
        let mut g = Graph::new();
        let grid = g.add_node("motion.grid");
        let dot = g.add_node(TYPE_STREAM.to_string());
        let a = g.add_node("motion.output");
        let b = g.add_node("motion.move");
        for (from, to) in [(grid, dot), (dot, a), (dot, b)] {
            g.connect(Edge {
                from: (from, 0),
                to: (to, 0),
                delayed: false,
            })
            .expect("the dot fans out");
        }
        assert!(g.validate(&reg).is_ok());
        assert_eq!(
            g.edges().iter().filter(|e| e.from.0 == dot).count(),
            2,
            "two wires leave the same dot"
        );
    }

    /// A reroute is typed: an instance-stream dot refuses a value wire (and the value dot
    /// takes it). Three types, and the type system keeps them apart — which is exactly why
    /// the gesture, not the artist, picks the one that fits.
    #[test]
    fn each_reroute_only_takes_its_own_kind_of_wire() {
        let reg = registry();
        let mut g = Graph::new();
        let lfo = g.add_node("value.lfo"); // a VALUE source
        let stream_dot = g.add_node(TYPE_STREAM.to_string());
        let value_dot = g.add_node(TYPE_VALUE.to_string());

        // The graph accepts the edge structurally; `validate` is where typing is enforced.
        g.connect(Edge {
            from: (lfo, 0),
            to: (stream_dot, 0),
            delayed: false,
        })
        .expect("structurally fine");
        assert!(
            g.validate(&reg).is_err(),
            "a value wire does not fit an instance-stream reroute"
        );

        g.disconnect(stream_dot, 0).expect("undo that");
        g.connect(Edge {
            from: (lfo, 0),
            to: (value_dot, 0),
            delayed: false,
        })
        .expect("wire");
        assert!(g.validate(&reg).is_ok(), "…but it fits the value reroute");
    }
}
