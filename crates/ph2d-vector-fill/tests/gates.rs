//! Closing gates (ADR-0060 §2.7, spec §5.4.6):
//! - `procedural_fill_no_recompile_on_animate` — animating params **and** the
//!   noise-kind enum across 60 frames triggers exactly **one** compile.
//! - cache hit-rate > 95% over that animation.
//!
//! The mechanism: [`topology_hash`](ph2d_vector_fill::wgsl_codegen) ignores every
//! authored param/enum (they ride the UBO), so a frame that changes only values
//! maps to the same cache key.

use ph2d_color::OklchColor;
use ph2d_vector_fill::{
    CompileCache, Connection, CoordMode, FillGraph, FillNode, GradientStop, NodeId, NoiseKind,
    ubo::FillParamsUbo,
};
use smallvec::smallvec;

const FRAMES: u64 = 60;

fn animated_graph(freq: f32, kind: NoiseKind) -> FillGraph {
    FillGraph {
        nodes: smallvec![
            FillNode::Coord {
                mode: CoordMode::Local
            },
            FillNode::Noise {
                kind,
                frequency: freq,
                octaves: 4,
            },
            FillNode::Ramp {
                palette: smallvec![
                    GradientStop::new(OklchColor::opaque(0.1, 0.0, 0.0), 0.0),
                    GradientStop::new(OklchColor::opaque(0.9, 0.1, 60.0), 1.0),
                ],
            },
        ],
        connections: smallvec![
            Connection::new(NodeId(0), NodeId(1), 0),
            Connection::new(NodeId(1), NodeId(2), 0),
        ],
        output_node_id: NodeId(2),
    }
}

fn kind_for(frame: u64) -> NoiseKind {
    match frame % 4 {
        0 => NoiseKind::Simplex,
        1 => NoiseKind::Perlin,
        2 => NoiseKind::Worley,
        _ => NoiseKind::Fbm {
            lacunarity: 2.0,
            persistence: 0.5,
        },
    }
}

#[test]
fn procedural_fill_no_recompile_on_animate() {
    let mut cache = CompileCache::new("metal-test");
    for f in 0..FRAMES {
        let t = f as f32 / FRAMES as f32;
        // Animate BOTH a scalar param (frequency) and the enum (noise kind).
        let g = animated_graph(1.0 + 4.0 * t, kind_for(f));
        cache.get_or_compile(&g).expect("compile");
    }
    assert_eq!(
        cache.compiles(),
        1,
        "param+enum animation must compile exactly once"
    );
    assert_eq!(cache.hits(), FRAMES - 1);
}

#[test]
fn cache_hit_rate_exceeds_95_percent() {
    let mut cache = CompileCache::new("metal-test");
    for f in 0..FRAMES {
        let g = animated_graph(1.0 + f as f32 * 0.01, kind_for(f));
        cache.get_or_compile(&g).expect("compile");
    }
    let rate = cache.hit_rate();
    assert!(rate > 0.95, "hit-rate {rate} must exceed 0.95");
    // 1 compile + 59 hits over 60 frames = 0.9833…
    assert!((rate - 59.0 / 60.0).abs() < 1e-9);
}

#[test]
fn ubo_carries_the_enum_so_the_hot_path_is_a_pure_write() {
    // The per-frame hot path: rebuild the UBO from the animated graph and confirm
    // the enum change shows up in `ucontrol` (i.e. it is a buffer write, not a
    // shader edit).
    let g_simplex = animated_graph(2.0, NoiseKind::Simplex);
    let g_worley = animated_graph(2.0, NoiseKind::Worley);
    let u0 = FillParamsUbo::from_graph(&g_simplex);
    let u1 = FillParamsUbo::from_graph(&g_worley);
    assert_eq!(u0.ucontrol[1][0], NoiseKind::Simplex.index());
    assert_eq!(u1.ucontrol[1][0], NoiseKind::Worley.index());
    // Both graphs share one topology key → one compiled shader serves both.
    let h0 = ph2d_vector_fill::wgsl_codegen::topology_hash(&g_simplex, "metal-test");
    let h1 = ph2d_vector_fill::wgsl_codegen::topology_hash(&g_worley, "metal-test");
    assert_eq!(h0, h1, "noise kind must not change the topology hash");
}

#[test]
fn genuine_topology_change_does_recompile() {
    let mut cache = CompileCache::new("metal-test");
    cache
        .get_or_compile(&animated_graph(2.0, NoiseKind::Perlin))
        .unwrap();
    // Drop the Ramp's only connection → output node changes to a bare Solid.
    let g2 = FillGraph {
        nodes: smallvec![FillNode::Solid {
            color: OklchColor::opaque(0.5, 0.0, 0.0)
        }],
        connections: smallvec![],
        output_node_id: NodeId(0),
    };
    cache.get_or_compile(&g2).unwrap();
    assert_eq!(cache.compiles(), 2, "a real topology change must recompile");
}

#[test]
fn ubo_update_is_zero_alloc_bytes_view() {
    // The per-frame write is `bytemuck::bytes_of(&ubo)` — a borrow, no alloc.
    let mut ubo = FillParamsUbo::from_graph(&animated_graph(2.0, NoiseKind::Perlin));
    let len_before = ubo.as_bytes().len();
    ubo.set_scalar(NodeId(1), 0, 9.0); // animate frequency in place
    ubo.set_noise_kind(NodeId(1), NoiseKind::Worley); // animate enum in place
    ubo.set_time(1.5);
    assert_eq!(ubo.as_bytes().len(), len_before);
    assert_eq!(ubo.scalars[1][0], 9.0);
    assert_eq!(ubo.ucontrol[1][0], NoiseKind::Worley.index());
}
