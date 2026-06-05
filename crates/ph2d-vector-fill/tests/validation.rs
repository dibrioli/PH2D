//! T6.1 — edit-time validation (spec §5.1.4): acyclicity, type-checking,
//! single-driver-per-port, reachable-Color output, and the node cap.

use ph2d_color::OklchColor;
use ph2d_vector_fill::{
    Connection, CoordMode, FillGraph, FillGraphError, FillNode, MAX_FILL_NODES, MathOp, NodeId,
};
use smallvec::{SmallVec, smallvec};

fn solid() -> FillNode {
    FillNode::Solid {
        color: OklchColor::opaque(0.5, 0.0, 0.0),
    }
}

#[test]
fn well_formed_graph_validates() {
    let g = FillGraph {
        nodes: smallvec![
            FillNode::Coord {
                mode: CoordMode::Local
            },
            FillNode::Noise {
                kind: ph2d_vector_fill::NoiseKind::Perlin,
                frequency: 4.0,
                octaves: 3,
            },
            FillNode::Ramp {
                palette: smallvec![]
            },
        ],
        connections: smallvec![
            Connection::new(NodeId(0), NodeId(1), 0),
            Connection::new(NodeId(1), NodeId(2), 0),
        ],
        output_node_id: NodeId(2),
    };
    assert_eq!(g.validate(), Ok(()));
}

#[test]
fn output_out_of_range() {
    let g = FillGraph {
        nodes: smallvec![solid()],
        connections: smallvec![],
        output_node_id: NodeId(5),
    };
    assert!(matches!(
        g.validate(),
        Err(FillGraphError::OutputOutOfRange { .. })
    ));
}

#[test]
fn output_must_be_color() {
    // A Math node outputs Scalar — cannot be the fill output.
    let g = FillGraph {
        nodes: smallvec![FillNode::Math { op: MathOp::Abs }],
        connections: smallvec![],
        output_node_id: NodeId(0),
    };
    assert!(matches!(
        g.validate(),
        Err(FillGraphError::OutputNotColor { .. })
    ));
}

#[test]
fn type_mismatch_is_rejected() {
    // Coord (Vec2) wired into Ramp's `t` (Scalar) port.
    let g = FillGraph {
        nodes: smallvec![
            FillNode::Coord {
                mode: CoordMode::Local
            },
            FillNode::Ramp {
                palette: smallvec![]
            },
        ],
        connections: smallvec![Connection::new(NodeId(0), NodeId(1), 0)],
        output_node_id: NodeId(1),
    };
    assert!(matches!(
        g.validate(),
        Err(FillGraphError::TypeMismatch { .. })
    ));
}

#[test]
fn duplicate_driver_is_rejected() {
    // Two Solids both driving Mix port 0.
    let g = FillGraph {
        nodes: smallvec![
            solid(),
            solid(),
            FillNode::Mix {
                mode: ph2d_vector_fill::BlendMode::Normal,
                factor: 0.5,
            },
        ],
        connections: smallvec![
            Connection::new(NodeId(0), NodeId(2), 0),
            Connection::new(NodeId(1), NodeId(2), 0),
        ],
        output_node_id: NodeId(2),
    };
    assert!(matches!(
        g.validate(),
        Err(FillGraphError::DuplicateInput { node: 2, port: 0 })
    ));
}

#[test]
fn port_out_of_range_is_rejected() {
    // Solid has no input ports; connecting into port 0 is invalid.
    let g = FillGraph {
        nodes: smallvec![solid(), solid()],
        connections: smallvec![Connection::new(NodeId(0), NodeId(1), 0)],
        output_node_id: NodeId(1),
    };
    assert!(matches!(
        g.validate(),
        Err(FillGraphError::PortOutOfRange { .. })
    ));
}

#[test]
fn cycle_is_rejected() {
    // Math(0) → Math(1) → Math(0): a 2-cycle on the scalar ports, with a Solid
    // output so the Color check passes first.
    let g = FillGraph {
        nodes: smallvec![
            FillNode::Math { op: MathOp::Abs },
            FillNode::Math { op: MathOp::Abs },
            solid(),
        ],
        connections: smallvec![
            Connection::new(NodeId(0), NodeId(1), 0),
            Connection::new(NodeId(1), NodeId(0), 0),
        ],
        output_node_id: NodeId(2),
    };
    assert_eq!(g.validate(), Err(FillGraphError::Cycle));
}

#[test]
fn too_many_nodes_is_rejected() {
    let nodes: SmallVec<[FillNode; MAX_FILL_NODES]> = std::iter::repeat_with(solid)
        .take(MAX_FILL_NODES + 1)
        .collect();
    let g = FillGraph {
        nodes,
        connections: smallvec![],
        output_node_id: NodeId(0),
    };
    assert!(matches!(
        g.validate(),
        Err(FillGraphError::TooManyNodes { .. })
    ));
}

#[test]
fn dead_nodes_are_allowed() {
    // Node 1 (a dead Coord) never feeds the output — still valid.
    let g = FillGraph {
        nodes: smallvec![
            solid(),
            FillNode::Coord {
                mode: CoordMode::Local
            },
        ],
        connections: smallvec![],
        output_node_id: NodeId(0),
    };
    assert_eq!(g.validate(), Ok(()));
    // Dependency order from the output only includes the reachable Solid.
    assert_eq!(g.dependency_order().unwrap(), vec![NodeId(0)]);
}
