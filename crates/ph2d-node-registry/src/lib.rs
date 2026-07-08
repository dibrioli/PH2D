#![forbid(unsafe_code)]
//! Runtime registry of node operations (ADR-0031, ADR-0032).
//!
//! Mirrors `ph2d-tool-registry` for the node world: each `ph2d-node-*` crate
//! registers its [`NodeOp`]; the registry resolves a [`NodeTypeId`] to that op
//! — it **is** the [`OpResolver`] the cook engine consumes — and rejects id
//! collisions at registration (so a duplicate canonical name is caught at boot,
//! not silently at cook time).
//!
//! The aggregation point (`register_all_nodes`, which calls each node crate's
//! `register`) lives in `ph2d-node-registry-init` and is codegen'd from a scan
//! of `crates/ph2d-node-*` (W1.T3 sync tool + staleness gate) — so adding a
//! node is dropping a crate, never hand-editing a central list.

use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::node::{NodeManifest, NodeOp, NodeTypeId};
use std::collections::BTreeMap;

mod ui;
pub use ui::{NodeSilhouette, NodeUiCategory, NodeUiManifest, ParamUiHint, ParamWidget};

/// A registered set of node operations, keyed by their stable type id.
/// Deterministic iteration (`BTreeMap`, ADR-0022 / HR-5).
#[derive(Default)]
pub struct NodeRegistry {
    ops: BTreeMap<NodeTypeId, Box<dyn NodeOp>>,
    /// M1.R1 — per-type UI side-metadata, keyed the same way as `ops` and kept
    /// separate so the frozen `NodeManifest` is untouched. A node registers its
    /// entry in `register` (alongside its op).
    ui: BTreeMap<NodeTypeId, NodeUiManifest>,
    /// M1.P1 — per-type param UI hints (editable range + widget + label for each
    /// declared `ParamSpec`), keyed the same way. A `&'static` slice per type so
    /// registration is a single insert; the params panel reads it to build rows.
    param_ui: BTreeMap<NodeTypeId, &'static [ParamUiHint]>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RegistryError {
    /// Two node types resolve to the same id (same canonical name, or — far
    /// less likely — an FNV-1a collision of distinct names). Reports the id and
    /// the name of the second registrant.
    Collision { id: NodeTypeId, name: &'static str },
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a node op. Errors if its id collides with one already present.
    pub fn register(&mut self, op: Box<dyn NodeOp>) -> Result<(), RegistryError> {
        let manifest = op.manifest();
        let id = manifest.id;
        if self.ops.contains_key(&id) {
            return Err(RegistryError::Collision {
                id,
                name: manifest.name,
            });
        }
        self.ops.insert(id, op);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Deterministic iteration over registered manifests (by id order).
    pub fn manifests(&self) -> impl Iterator<Item = &'static NodeManifest> + '_ {
        self.ops.values().map(|op| op.manifest())
    }

    /// Register a node type's UI metadata (M1.R1). Additive to [`Self::register`]
    /// — a node registers both its op and its UI manifest; last write wins.
    pub fn register_ui(&mut self, id: NodeTypeId, ui: NodeUiManifest) {
        self.ui.insert(id, ui);
    }

    /// The UI metadata for `id`, if registered (M1.R1).
    pub fn ui_manifest(&self, id: NodeTypeId) -> Option<&NodeUiManifest> {
        self.ui.get(&id)
    }

    /// Register a node type's param UI hints (M1.P1). Additive; last write wins.
    pub fn register_param_ui(&mut self, id: NodeTypeId, hints: &'static [ParamUiHint]) {
        self.param_ui.insert(id, hints);
    }

    /// The param UI hints for `id`, if registered (M1.P1). The params panel pairs
    /// each with the manifest's `ParamSpec` (falling back to a plain slider when a
    /// param has no hint).
    pub fn param_ui(&self, id: NodeTypeId) -> Option<&'static [ParamUiHint]> {
        self.param_ui.get(&id).copied()
    }
}

impl OpResolver for NodeRegistry {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        self.ops.get(&ty).map(|boxed| boxed.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::{Cook, EvalCtx};
    use ph2d_nodegraph::effect::Effect;
    use ph2d_nodegraph::graph::{Edge, Graph};
    use ph2d_nodegraph::node::{LoweringKind, PortSpec};
    use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

    const T: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
    const fn port(name: &'static str) -> PortSpec {
        PortSpec { name, ty: T }
    }

    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("reg.src"),
        name: "reg.src",
        inputs: &[],
        outputs: &[port("out")],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    static PASS_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("reg.pass"),
        name: "reg.pass",
        inputs: &[port("in")],
        outputs: &[port("out")],
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
            ctx.emit(Stream::new(2).with("v", Column::Scalar(vec![10.0, 20.0])));
        }
    }

    struct Pass;
    impl NodeOp for Pass {
        fn manifest(&self) -> &'static NodeManifest {
            &PASS_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            let passthrough = ctx.input(0).clone();
            ctx.emit(passthrough);
        }
    }

    /// Shares SRC_MAN's id — a collision.
    struct Dup;
    impl NodeOp for Dup {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, _ctx: &mut EvalCtx<'_>) {}
    }

    #[test]
    fn register_and_resolve() {
        let mut reg = NodeRegistry::new();
        reg.register(Box::new(Src)).unwrap();
        reg.register(Box::new(Pass)).unwrap();
        assert_eq!(reg.len(), 2);
        assert!(reg.resolve(SRC_MAN.id).is_some());
        assert!(reg.resolve(NodeTypeId::of("nope")).is_none());
    }

    #[test]
    fn param_ui_hints_round_trip() {
        static HINTS: &[ParamUiHint] = &[ParamUiHint {
            param: "rows",
            label: "Rows",
            min: 1.0,
            max: 20.0,
            step: 1.0,
            widget: ParamWidget::IntSlider,
        }];
        let mut reg = NodeRegistry::new();
        reg.register(Box::new(Src)).unwrap();
        assert!(reg.param_ui(SRC_MAN.id).is_none()); // none until registered
        reg.register_param_ui(SRC_MAN.id, HINTS);
        let got = reg.param_ui(SRC_MAN.id).expect("registered");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].param, "rows");
        assert!(got[0].widget.is_integer());
        assert!(reg.param_ui(NodeTypeId::of("nope")).is_none());
    }

    #[test]
    fn duplicate_id_is_rejected() {
        let mut reg = NodeRegistry::new();
        reg.register(Box::new(Src)).unwrap();
        assert_eq!(
            reg.register(Box::new(Dup)),
            Err(RegistryError::Collision {
                id: SRC_MAN.id,
                name: "reg.src"
            })
        );
    }

    #[test]
    fn registry_is_a_resolver_the_cook_can_use() {
        // The registry IS the OpResolver — validate + cook a real graph through it.
        let mut reg = NodeRegistry::new();
        reg.register(Box::new(Src)).unwrap();
        reg.register(Box::new(Pass)).unwrap();

        let mut g = Graph::new();
        let s = g.add_node("reg.src");
        let p = g.add_node("reg.pass");
        g.connect(Edge {
            from: (s, 0),
            to: (p, 0),
            delayed: false,
        })
        .unwrap();
        g.validate(&reg).expect("graph is well-typed");

        let mut cook = Cook::new();
        let out = cook.cook(&g, &reg, p, 0.0).unwrap();
        match out[0].as_stream().get("v") {
            Some(Column::Scalar(v)) => assert_eq!(v, &vec![10.0, 20.0]),
            other => panic!("expected scalar column, got {other:?}"),
        }
    }
}
