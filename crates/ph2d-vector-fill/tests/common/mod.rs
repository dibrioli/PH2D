//! Shared fixtures for the codegen-gold snapshots (used by `codegen_gold.rs`).
//! Defines the 5 canonical topologies in one place so the generator
//! (`PH2D_BLESS_GOLD=1`) and the assertion read identical graphs.
#![allow(dead_code)]

use ph2d_color::OklchColor;
use ph2d_vector_fill::{
    BlendMode, Connection, CoordMode, FillGraph, FillNode, GradientStop, NodeId, NoiseKind,
};
use smallvec::smallvec;

fn stops() -> smallvec::SmallVec<[GradientStop; 8]> {
    smallvec![
        GradientStop::new(OklchColor::opaque(0.1, 0.0, 0.0), 0.0),
        GradientStop::new(OklchColor::opaque(0.9, 0.12, 60.0), 1.0),
    ]
}

/// `(name, graph)` for the 5 gold fixtures.
pub fn fixtures() -> Vec<(&'static str, FillGraph)> {
    vec![
        (
            "solid",
            FillGraph {
                nodes: smallvec![FillNode::Solid {
                    color: OklchColor::opaque(0.6, 0.1, 120.0)
                }],
                connections: smallvec![],
                output_node_id: NodeId(0),
            },
        ),
        (
            "linear_gradient",
            FillGraph {
                nodes: smallvec![FillNode::LinearGradient {
                    stops: stops(),
                    angle: 0.6, // param rides the UBO — not in the golden WGSL
                }],
                connections: smallvec![],
                output_node_id: NodeId(0),
            },
        ),
        (
            "noise_ramp",
            FillGraph {
                nodes: smallvec![
                    FillNode::Coord {
                        mode: CoordMode::Local
                    },
                    FillNode::Noise {
                        kind: NoiseKind::Fbm {
                            lacunarity: 2.0,
                            persistence: 0.5
                        },
                        frequency: 3.0,
                        octaves: 4,
                    },
                    FillNode::Ramp { palette: stops() },
                ],
                connections: smallvec![
                    Connection::new(NodeId(0), NodeId(1), 0),
                    Connection::new(NodeId(1), NodeId(2), 0),
                ],
                output_node_id: NodeId(2),
            },
        ),
        (
            "voronoi_ramp",
            FillGraph {
                nodes: smallvec![
                    FillNode::Voronoi {
                        cells: 8,
                        jitter: 0.8
                    },
                    FillNode::Ramp { palette: stops() },
                ],
                connections: smallvec![Connection::new(NodeId(0), NodeId(1), 0)],
                output_node_id: NodeId(1),
            },
        ),
        (
            "mix_two_solids",
            FillGraph {
                nodes: smallvec![
                    FillNode::Solid {
                        color: OklchColor::opaque(0.7, 0.1, 30.0)
                    },
                    FillNode::Solid {
                        color: OklchColor::opaque(0.3, 0.2, 240.0)
                    },
                    FillNode::Mix {
                        mode: BlendMode::Overlay,
                        factor: 0.5,
                    },
                ],
                connections: smallvec![
                    Connection::new(NodeId(0), NodeId(2), 0),
                    Connection::new(NodeId(1), NodeId(2), 1),
                ],
                output_node_id: NodeId(2),
            },
        ),
    ]
}
