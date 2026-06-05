//! T6.2 — codegen coverage: every implemented node lowers to WGSL that **naga
//! accepts**, codegen is deterministic, and W6-stub nodes are rejected.

use ph2d_color::OklchColor;
use ph2d_vector_fill::cache::compile_fill;
use ph2d_vector_fill::wgsl_codegen::codegen;
use ph2d_vector_fill::{
    BlendMode, Connection, CoordMode, FillCodegenError, FillGraph, FillNode, GradientStop, MathOp,
    NodeId, NoiseKind,
};
use smallvec::{SmallVec, smallvec};

const BK: &str = "test-backend";

fn solid(l: f32, c: f32, h: f32) -> FillNode {
    FillNode::Solid {
        color: OklchColor::opaque(l, c, h),
    }
}

fn two_stops() -> SmallVec<[GradientStop; 8]> {
    smallvec![
        GradientStop::new(OklchColor::opaque(0.1, 0.0, 0.0), 0.0),
        GradientStop::new(OklchColor::opaque(0.9, 0.12, 60.0), 1.0),
    ]
}

/// Build a single-output graph from nodes + connections, asserting it validates.
fn graph(nodes: Vec<FillNode>, conns: Vec<Connection>, out: u32) -> FillGraph {
    let g = FillGraph {
        nodes: nodes.into_iter().collect(),
        connections: conns.into_iter().collect(),
        output_node_id: NodeId(out),
    };
    g.validate().expect("fixture graph must validate");
    g
}

fn must_compile(g: &FillGraph, label: &str) {
    match compile_fill(g, BK) {
        Ok(_) => {}
        Err(e) => panic!("{label}: generated WGSL rejected: {e}"),
    }
}

#[test]
fn solid_compiles() {
    must_compile(&graph(vec![solid(0.6, 0.1, 120.0)], vec![], 0), "solid");
}

#[test]
fn every_noise_kind_compiles() {
    for kind in [
        NoiseKind::Simplex,
        NoiseKind::Perlin,
        NoiseKind::Worley,
        NoiseKind::Fbm {
            lacunarity: 2.0,
            persistence: 0.5,
        },
    ] {
        let g = graph(
            vec![
                FillNode::Coord {
                    mode: CoordMode::Local,
                },
                FillNode::Noise {
                    kind,
                    frequency: 4.0,
                    octaves: 4,
                },
                FillNode::Ramp {
                    palette: two_stops(),
                },
            ],
            vec![
                Connection::new(NodeId(0), NodeId(1), 0),
                Connection::new(NodeId(1), NodeId(2), 0),
            ],
            2,
        );
        must_compile(&g, &format!("noise:{kind:?}"));
    }
}

#[test]
fn gradients_compile() {
    let lin = graph(
        vec![FillNode::LinearGradient {
            stops: two_stops(),
            angle: 0.7,
        }],
        vec![],
        0,
    );
    must_compile(&lin, "linear");
    let rad = graph(
        vec![FillNode::RadialGradient {
            stops: two_stops(),
            center: glam::Vec2::new(0.5, 0.5),
            radius: 0.5,
        }],
        vec![],
        0,
    );
    must_compile(&rad, "radial");
}

#[test]
fn voronoi_time_math_random_compile() {
    let voronoi = graph(
        vec![
            FillNode::Voronoi {
                cells: 8,
                jitter: 0.8,
            },
            FillNode::Ramp {
                palette: two_stops(),
            },
        ],
        vec![Connection::new(NodeId(0), NodeId(1), 0)],
        1,
    );
    must_compile(&voronoi, "voronoi");

    let timed = graph(
        vec![
            FillNode::Time,
            FillNode::Math { op: MathOp::Sin },
            FillNode::Ramp {
                palette: two_stops(),
            },
        ],
        vec![
            Connection::new(NodeId(0), NodeId(1), 0),
            Connection::new(NodeId(1), NodeId(2), 0),
        ],
        2,
    );
    must_compile(&timed, "time→math→ramp");

    let rnd = graph(
        vec![
            FillNode::Random { seed: 1234 },
            FillNode::Ramp {
                palette: two_stops(),
            },
        ],
        vec![Connection::new(NodeId(0), NodeId(1), 0)],
        1,
    );
    must_compile(&rnd, "random");
}

#[test]
fn mix_of_two_colours_compiles() {
    for mode in [
        BlendMode::Normal,
        BlendMode::Multiply,
        BlendMode::Screen,
        BlendMode::Overlay,
        BlendMode::Add,
        BlendMode::Lerp,
    ] {
        let g = graph(
            vec![
                solid(0.7, 0.1, 30.0),
                solid(0.3, 0.2, 240.0),
                FillNode::Mix { mode, factor: 0.5 },
            ],
            vec![
                Connection::new(NodeId(0), NodeId(2), 0),
                Connection::new(NodeId(1), NodeId(2), 1),
            ],
            2,
        );
        must_compile(&g, &format!("mix:{mode:?}"));
    }
}

#[test]
fn large_combined_graph_compiles() {
    // Coord→Noise→Ramp (A=node2) and Voronoi→Ramp (B=node4), Mix(A,B)=node5.
    let g = graph(
        vec![
            FillNode::Coord {
                mode: CoordMode::Polar,
            },
            FillNode::Noise {
                kind: NoiseKind::Fbm {
                    lacunarity: 2.0,
                    persistence: 0.5,
                },
                frequency: 3.0,
                octaves: 5,
            },
            FillNode::Ramp {
                palette: two_stops(),
            },
            FillNode::Voronoi {
                cells: 6,
                jitter: 1.0,
            },
            FillNode::Ramp {
                palette: two_stops(),
            },
            FillNode::Mix {
                mode: BlendMode::Overlay,
                factor: 0.4,
            },
        ],
        vec![
            Connection::new(NodeId(0), NodeId(1), 0),
            Connection::new(NodeId(1), NodeId(2), 0),
            Connection::new(NodeId(3), NodeId(4), 0),
            Connection::new(NodeId(2), NodeId(5), 0),
            Connection::new(NodeId(4), NodeId(5), 1),
        ],
        5,
    );
    must_compile(&g, "combined");
}

#[test]
fn codegen_is_deterministic() {
    let g = graph(
        vec![
            FillNode::Coord {
                mode: CoordMode::Local,
            },
            FillNode::Noise {
                kind: NoiseKind::Perlin,
                frequency: 4.0,
                octaves: 3,
            },
            FillNode::Ramp {
                palette: two_stops(),
            },
        ],
        vec![
            Connection::new(NodeId(0), NodeId(1), 0),
            Connection::new(NodeId(1), NodeId(2), 0),
        ],
        2,
    );
    let a = codegen(&g).unwrap();
    let b = codegen(&g).unwrap();
    assert_eq!(a, b, "codegen must be byte-stable");
}

#[test]
fn lean_shader_omits_unused_helpers() {
    // A pure Solid pulls in NO noise/gradient/mix helpers.
    let wgsl = codegen(&graph(vec![solid(0.5, 0.0, 0.0)], vec![], 0)).unwrap();
    assert!(wgsl.contains("fn fill_main"));
    assert!(!wgsl.contains("ph2d_noise1"), "no noise helper for a Solid");
    assert!(
        !wgsl.contains("ph2d_eval_stops"),
        "no ramp helper for a Solid"
    );
    assert!(
        !wgsl.contains("ph2d_mix_blend"),
        "no mix helper for a Solid"
    );
}

#[test]
fn reachable_stub_is_rejected() {
    let g = FillGraph {
        nodes: smallvec![FillNode::MeshGradient { gradient_id: 7 }],
        connections: smallvec![],
        output_node_id: NodeId(0),
    };
    // validate() passes (it's structurally a Color output)...
    g.validate().unwrap();
    // ...but codegen refuses the W6 stub.
    match codegen(&g) {
        Err(FillCodegenError::NotYetImplemented { node, kind }) => {
            assert_eq!(node, 0);
            assert_eq!(kind, "MeshGradient");
        }
        other => panic!("expected NotYetImplemented, got {other:?}"),
    }
}

#[test]
fn dead_stub_is_ignored() {
    // An Image stub that does NOT feed the output is never codegen'd.
    let g = graph(
        vec![solid(0.5, 0.0, 0.0), FillNode::Image { image_ref: 1 }],
        vec![],
        0,
    );
    must_compile(&g, "dead-stub");
}
