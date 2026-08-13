//! ADR-0154 gates for the shell half of `source.shape`. The load-bearing one is
//! the CROSS-SIDE single door: the shell publishes a shape's geometry under the
//! content key it computes with [`read_params`], and the node's `eval` reads an
//! external under the key IT computes with `ctx.param` — if those two reads
//! diverged, the node would clone the empty external and emit nothing. The gate
//! drives BOTH sides for real (publish + a live cook) so the two keys are computed
//! independently and shown to agree.

use super::{VecPathStore, build_shape_path, encode};
use crate::motion_state::MotionState;
use ph2d_editor::ProjectSettings;
use ph2d_node_motion_shape::{ShapeKind, ShapeParams, shape_key};
use ph2d_nodegraph::attr::{Column, Stream};

/// **The single door.** The shell interns a shape and publishes it under
/// `read_params`'s key; the node, cooked, reads the external under `ctx.param`'s
/// key and emits the geometry handle — and the store holds the `VecPath` for that
/// handle. FALSIFIED by a `read_params` that computes different param values than
/// the node's `ctx.param` (the keys would diverge → the node reads empty → count 0).
#[test]
fn publish_then_cook_the_node_reads_its_own_shape() {
    let mut state = MotionState::new();
    let n = state.doc.graph.add_node("source.shape");
    state.doc.graph.set_param(n, "kind", 5.0); // Star
    state.doc.graph.set_param(n, "size", 0.8);
    state.doc.graph.set_param(n, "sides", 6.0);

    // The real publish: intern the geometry + set the external on the pump's cook.
    super::publish(&mut state);

    let out = state
        .pump
        .cook
        .cook(&state.doc.graph, &state.registry, n, 0.0)
        .expect("cook");
    let stream = out[0].as_stream();
    assert_eq!(
        stream.count(),
        1,
        "the node read the shell's published shape"
    );
    let Some(Column::Scalar(ids)) = stream.get("geometry_id") else {
        panic!("geometry_id column");
    };
    let handle = ids[0] as u32;
    assert!(handle >= 1, "a live geometry handle");
    assert!(
        state.shape_store.get(handle).is_some(),
        "the store holds the shape's VecPath"
    );
}

/// **The negative control** — proves the round-trip gate is not vacuously green.
/// Publish a shape under the key of a DIFFERENT descriptor than the node authored;
/// the node's `ctx.param` key won't match, so it reads the empty external and
/// emits nothing.
#[test]
fn a_mismatched_key_decouples_the_node_from_the_shape() {
    let state = MotionState::new();
    let mut g = ph2d_nodegraph::graph::Graph::new();
    let n = g.add_node("source.shape");
    g.set_param(n, "kind", 5.0); // the node authors a Star
    // Publish under a Circle's key — a mismatch.
    // Publica sob a chave de um Circle (`kind = 0`), com o resto nos defaults.
    let wrong = shape_key(|name: &str| {
        if name == "kind" {
            0.0
        } else {
            super::manifest_default(name)
        }
    });
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    cook.set_external(
        wrong,
        Stream::new(1).with("geometry_id", Column::Scalar(vec![7.0])),
    );
    let out = cook.cook(&g, &state.registry, n, 0.0).expect("cook");
    assert_eq!(
        out[0].as_stream().count(),
        0,
        "a key mismatch emits nothing — the round-trip gate has teeth"
    );
}

/// Each shape family builds a DISTINCT `VecPath` from the same descriptor — a
/// circle is not a square, a star is not a gear. FALSIFIED by a `build_shape_path`
/// arm that collapses two kinds to the same geometry.
#[test]
fn every_kind_builds_distinct_geometry() {
    let base = ShapeParams::read(super::manifest_default); // os defaults do manifesto
    let kinds = [
        ShapeKind::Circle,
        ShapeKind::Square,
        ShapeKind::Ellipse,
        ShapeKind::Rectangle,
        ShapeKind::Polygon,
        ShapeKind::Star,
        ShapeKind::Heart,
        ShapeKind::Gear,
    ];
    // aspect ≠ 1 so Ellipse ≠ Circle and Rectangle ≠ Square even at the same size.
    let geoms: Vec<_> = kinds
        .iter()
        .map(|&kind| {
            build_shape_path(&ShapeParams {
                stroke: None,
                kind,
                aspect: 1.5,
                ..base
            })
            .verts
        })
        .collect();
    for (i, a) in geoms.iter().enumerate() {
        for (j, b) in geoms.iter().enumerate().skip(i + 1) {
            assert_ne!(a, b, "kind {i} and kind {j} build distinct geometry");
        }
    }
}

/// A handle with no stored geometry (an empty store, or a forward cook) resolves
/// to `None`, and encoding an instance that carries it draws nothing rather than
/// panicking. FALSIFIED by a `get` that indexes out of bounds or an `encode` that
/// unwraps.
#[test]
fn an_unpublished_handle_is_none_and_encodes_without_panic() {
    let store = VecPathStore::default();
    assert!(store.get(0).is_none(), "handle 0 is 'no geometry'");
    assert!(
        store.get(99).is_none(),
        "an unpublished handle resolves to None"
    );

    let inst = ph2d_eval_motion::VectorInstance {
        geometry_id: 99,
        world_pos: [0.0, 0.0],
        size: [1.0, 1.0],
        basis: [1.0, 0.0, 0.0, 1.0],
        tint: [1.0, 1.0, 1.0, 1.0],
    };
    let mut scene = ph2d_vector::VectorScene::new();
    encode(&[inst], &store, ph2d_vector::Affine::IDENTITY, &mut scene);
    // Reaching here without a panic IS the assertion.
}

/// **The per-shape param gate, end to end.** The params bridge shows ONLY the
/// controls the current `kind` uses (`ParamGate`): a circle is Shape + Size; a gear
/// adds Sides + Tooth Depth + Hole and nothing else. FALSIFIED by dropping the
/// `gated_off` filter in `build_params_snapshot` (a circle would show all nine).
#[cfg(all(feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[test]
fn the_params_panel_shows_only_the_shapes_kind_params() {
    use ph2d_panel_motion_params::ParamRow;
    let names = |motion: &MotionState| -> Vec<&'static str> {
        crate::render_loop::motion_bridge::params::build_params_snapshot(
            motion,
            ProjectSettings::default(),
        )
        .expect("shape node resolvable")
        .rows
        .iter()
        .filter_map(|r| match r {
            ParamRow::Enum(e) => Some(e.name),
            ParamRow::Scalar(s) => Some(s.name),
            _ => None,
        })
        .collect()
    };
    let mut motion = MotionState::new();
    let n = motion.doc.graph.add_node("source.shape"); // default kind = Circle
    ph2d_panel_motion_graph::set_graph_selection(vec![n.0]);

    let circle = names(&motion);
    assert!(
        circle.contains(&"kind") && circle.contains(&"size"),
        "a circle shows Shape + Size"
    );
    for p in [
        "aspect",
        "sides",
        "corner",
        "star_depth",
        "cleft",
        "tooth_depth",
        "hole",
    ] {
        assert!(!circle.contains(&p), "a circle hides {p}");
    }

    motion
        .doc
        .graph
        .set_param(n, "kind", ShapeKind::Gear as u32 as f32);
    let gear = names(&motion);
    for p in ["kind", "size", "sides", "tooth_depth", "hole"] {
        assert!(gear.contains(&p), "a gear shows {p}");
    }
    for p in ["aspect", "corner", "star_depth", "cleft"] {
        assert!(!gear.contains(&p), "a gear hides {p}");
    }
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// **The whole path, end to end** — the smoke's number, gated. A `source.shape`
/// (Star) is crossed with a `motion.grid` (4×4) through a `motion.duplicator`; the
/// cooked sink lowers to 16 `VectorInstance`s, every copy carrying the ONE handle
/// the shape interned (a duplicator that dropped `geometry_id` would give sprites,
/// not shapes). FALSIFIED by the duplicator not preserving the convention column.
#[test]
fn a_shape_stamped_on_a_grid_lowers_to_sixteen_vectors() {
    use ph2d_eval_motion::lower_to_vector_instances_onto;
    use ph2d_nodegraph::graph::{Edge, NodeId};

    let mut state = MotionState::new();
    let (src, out) = {
        let g = &mut state.doc.graph;
        let src = g.add_node("source.shape");
        let grid = g.add_node("motion.grid");
        let dup = g.add_node("motion.duplicator");
        let out = g.add_node("motion.output");
        g.set_param(src, "kind", 5.0); // Star
        g.set_param(src, "size", 0.4);
        g.set_param(grid, "rows", 4.0);
        g.set_param(grid, "cols", 4.0);
        let mut wire = |a: NodeId, ap: u16, b: NodeId, bp: u16| {
            g.connect(Edge {
                from: (a, ap),
                to: (b, bp),
                delayed: false,
            })
            .expect("connect");
        };
        wire(src, 0, dup, 0); // shape → duplicator.shape
        wire(grid, 0, dup, 1); // grid → duplicator.points
        wire(dup, 0, out, 0); // duplicator → output
        (src, out)
    };
    let _ = src;

    super::publish(&mut state);
    let cooked = state
        .pump
        .cook
        .cook(&state.doc.graph, &state.registry, out, 0.0)
        .expect("cook");
    let mut vecs = Vec::new();
    lower_to_vector_instances_onto(cooked[0].as_stream(), &mut vecs);
    assert_eq!(
        vecs.len(),
        16,
        "a Star on a 4x4 grid = 16 crisp vector copies"
    );

    let handle = vecs[0].geometry_id;
    assert!(handle >= 1, "a live geometry handle");
    assert!(
        vecs.iter().all(|v| v.geometry_id == handle),
        "one shape, one interned handle across all copies"
    );
    assert!(
        state.shape_store.get(handle).is_some(),
        "the store holds the stamped shape's VecPath"
    );
}

/// **A live DOCUMENT vector renders its AUTHORED colours** (Part 1, Vetor Vivo). A
/// `source.object` vector is stored and drawn by the SAME `encode`/`draw_shape_instance`
/// as a `source.shape`, but it carries its own fill (an orange star) — so the live
/// render must honour that fill, NOT the (WHITE) instance tint. Stores an orange star,
/// encodes one instance, renders it, and reads back the centre texel: it is ORANGE
/// (`r > g > b`, low blue), not white. RED-FIRST: the pre-fix `draw_shape_instance`
/// filled with `Color::new(tint)` (white) ⇒ the centre reads near-white ⇒ this fails.
/// Needs a GPU adapter (RTX); `#[ignore]`, run with `-- --ignored`.
#[test]
#[ignore = "needs a GPU adapter (RTX); run with --ignored"]
fn a_live_document_vector_renders_its_authored_fill_not_the_tint() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter; skipping a_live_document_vector_renders_its_authored_fill");
        return;
    };
    // An orange-filled star — a DOCUMENT vector's own paint (source.object), not a
    // paint-less source.shape primitive.
    let mut star = ph2d_vec_scene::star([0.0, 0.0], 0.5, 0.5, 5, 0.45);
    star.fill = Some(ph2d_vec_scene::Paint::solid(ph2d_vec_scene::Rgba8::new(
        255, 170, 40, 255,
    )));
    let mut store = VecPathStore::default();
    let handle = store.push(star);
    let inst = ph2d_eval_motion::VectorInstance {
        geometry_id: handle,
        world_pos: [0.0, 0.0],
        size: [1.0, 1.0],
        basis: [1.0, 0.0, 0.0, 1.0], // identity rotation
        tint: [1.0, 1.0, 1.0, 1.0],  // WHITE — the object tint must NOT paint the star
    };
    // Fit the unit-radius star into a 64² tile: world [-0.5, 0.5] → device [7, 57].
    let (w, h) = (64u32, 64u32);
    let cam = ph2d_vector::Affine::translate((32.0, 32.0)) * ph2d_vector::Affine::scale(50.0);
    let mut scene = ph2d_vector::VectorScene::new();
    encode(&[inst], &store, cam, &mut scene);

    let mut pass =
        ph2d_render::VelloPass::new(&gpu, wgpu::TextureFormat::Rgba8UnormSrgb, (w, h)).unwrap();
    let rgba = pass
        .render_and_readback(&gpu, scene.inner(), (w, h))
        .expect("render");
    // The centre of a filled star is inside it ⇒ the authored orange.
    let c = ((h / 2 * w + w / 2) * 4) as usize;
    let (r, g, b) = (rgba[c] as i32, rgba[c + 1] as i32, rgba[c + 2] as i32);
    assert!(
        r > g + 30 && g > b && b < 150,
        "the star's centre is the AUTHORED orange (r>g>b, low blue); got ({r},{g},{b}) — \
         near-white means draw_shape_instance filled with the tint, ignoring the fill"
    );
}

// ---------------------------------------------------------------------------

/// **O TRAÇO chega ao `VecPath`, e o default é a forma que sempre shipou**
/// (doc 89 folha 14, P0).
///
/// O renderer já desenhava traço — `tessellate_shape_instance` ramifica em
/// `fill.is_some() || stroke.is_some()` desde sempre —, e o que faltava era o
/// shell PÔ-LO no caminho: o `cook` devolve os dois em `None`. Este gate mede o
/// `VecPath` construído, que é onde a omissão vivia.
#[test]
fn the_shape_carries_the_stroke_the_artist_authored() {
    use super::{build_shape_path, manifest_default};
    let get = |w: f32| {
        move |name: &str| {
            if name == "stroke_width" {
                w
            } else if name == "stroke_r" {
                1.0
            } else {
                manifest_default(name)
            }
        }
    };
    // Sem largura: NENHUM traço — e é o mundo de sempre.
    let bare = build_shape_path(&ShapeParams::read(get(0.0)));
    assert!(
        bare.stroke.is_none(),
        "`stroke_width = 0` nao pode pôr um StrokeSpec de largura zero no caminho"
    );
    // Com largura: o traço está lá, com a largura e a cor autoradas.
    let inked = build_shape_path(&ShapeParams::read(get(0.25)));
    let st = inked.stroke.expect("o traco autorado chega ao caminho");
    assert!(
        (st.width - 0.25).abs() < 1e-6,
        "a largura, deu {}",
        st.width
    );
    assert_eq!(st.color.r, 255, "e a cor: R cheio");
    assert_eq!(st.color.g, 0);
    // A GEOMETRIA não muda — um traço é tinta, não forma.
    assert_eq!(
        inked.verts.len(),
        bare.verts.len(),
        "o traco nao mexe na geometria"
    );
}

/// **A CHAVE distingue duas larguras de traço.**
///
/// ⚠️ É a metade que o cache pode comer: a `shape_key` é o que o `intern` usa, e
/// uma chave que ignorasse o traço devolveria a forma ANTIGA do store — o
/// controle ficaria inerte **depois da primeira vez**, com a primeira largura
/// escolhida a valer para sempre. Por isso a chave é derivada de `param::ALL` em
/// vez de enumerar campos.
#[test]
fn the_content_key_separates_two_stroke_widths() {
    use super::manifest_default;
    let key = |w: f32| {
        ph2d_node_motion_shape::shape_key(move |name: &str| {
            if name == "stroke_width" {
                w
            } else {
                manifest_default(name)
            }
        })
    };
    assert_ne!(key(0.0), key(0.25), "duas larguras, duas geometrias");
    assert_eq!(key(0.25), key(0.25), "e a chave e determinista");
}
