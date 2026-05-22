#![forbid(unsafe_code)]
//! `debug.const` — the canonical *first* node: a `Pure` generator that emits a
//! one-element scalar stream. Domain-agnostic; it exists to exercise the whole
//! chain end-to-end (drop crate → sync wires it → registry resolves → cook
//! produces output) before the real domain evaluators land in W2. It is also
//! the worked example a fan-out agent copies (the W1.T5 template in miniature).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("debug.const"),
    name: "debug.const",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame),
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
};

struct DebugConst;

impl NodeOp for DebugConst {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(1).with("v", Column::Scalar(vec![1.0])));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(DebugConst))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::OpResolver;

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
