//! ADR-0154 gates for the shell half of `source.shape`. The load-bearing one is
//! the CROSS-SIDE single door: the shell publishes a shape's geometry under the
//! content key it computes with [`read_params`], and the node's `eval` reads an
//! external under the key IT computes with `ctx.param` — if those two reads
//! diverged, the node would clone the empty external and emit nothing. The gate
//! drives BOTH sides for real (publish + a live cook) so the two keys are computed
//! independently and shown to agree.

use super::{
    VecPathStore, build_shape_path, build_shape_path_as_it_shipped, encode, read_params, vec_recipe,
};
use crate::motion_state::MotionState;
use ph2d_editor::ProjectSettings;
use ph2d_node_motion_shape::{ALL_KINDS, KIND_LABELS, ShapeKind, ShapeParams, shape_key};
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
    let wrong = shape_key(&ShapeParams {
        kind: ShapeKind::Circle,
        ..read_params(g.node_param_overrides(n))
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
    let base = read_params(None); // the manifest defaults
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
// O catálogo (doc 89 §14) — 35 formas que o editor já desenhava e o grafo não
// alcançava, e a prova de que as oito que shipavam não se moveram.
// ---------------------------------------------------------------------------

/// **A que decide se a wave pode existir.** Rotear tudo pelo `cook()` só é seguro se
/// as oito formas que shipavam saírem iguais — um grafo salvo guarda o índice do
/// `kind`, e uma forma que mudasse calada reescreveria a arte de quem já a autorou.
///
/// O oráculo é o construtor CONGELADO (`build_shape_path_as_it_shipped`), não uma
/// re-derivação: comparar o `cook()` com uma segunda escrita da mesma receita
/// provaria que eu sei somar duas vezes, e não que o produto não mudou.
///
/// ⚠️ **E a igualdade NÃO é bit a bit, porque nunca foi — este gate achou uma
/// divergência que já existia.** O círculo tinha DUAS derivações do mesmo número:
/// `ellipse()` usa a constante literal `KAPPA` e `ellipse_sweep` calcula
/// `(4/3)·tan(α/4)`, que o doc do `round.rs` declara ser *"a generalização do
/// `KAPPA` (que é esse valor para 90°)"*. São o mesmo valor por dois caminhos, e
/// eles tinham deslizado no último bit: **1,7e-12 de handle**, ou 3e-10 relativo.
/// Esta wave colapsa as duas portas numa; o número mede o quanto elas estavam
/// separadas, não uma regressão introduzida.
///
/// Então a ESTRUTURA é afirmada exata (contagem, fechamento, regra de
/// preenchimento, espécie de vértice — onde um erro de tradução apareceria) e a
/// GEOMETRIA a uma barra MEDIDA. Um `assert_eq` de bits aqui teria reprovado uma
/// wave correta; uma tolerância folgada teria deixado passar um round-rect com o
/// canto errado.
///
/// A varredura é adversarial de propósito — `corner` no máximo (onde o round-rect
/// e o polígono divergiriam se os campos apendados não fossem neutros), `aspect`
/// nos dois lados de 1, `sides` nos extremos.
#[test]
fn the_eight_that_shipped_cook_exactly_what_they_cooked() {
    let (mut checked, mut worst) = (0usize, 0.0f64);
    for kind in [
        ShapeKind::Circle,
        ShapeKind::Square,
        ShapeKind::Ellipse,
        ShapeKind::Rectangle,
        ShapeKind::Polygon,
        ShapeKind::Star,
        ShapeKind::Heart,
        ShapeKind::Gear,
    ] {
        for size in [0.01f32, 1.0, 7.5] {
            for aspect in [0.05f32, 1.0, 3.25] {
                for sides in [3u32, 6, 32] {
                    for corner in [0.0f32, 0.37, 1.0] {
                        let p = ShapeParams {
                            kind,
                            size,
                            aspect,
                            sides,
                            corner,
                            star_depth: 0.45,
                            cleft: 0.2,
                            tooth_depth: 0.35,
                            hole: 0.45,
                        };
                        let now = build_shape_path(&p);
                        let then = build_shape_path_as_it_shipped(&p);
                        assert_eq!(
                            now.verts.len(),
                            then.verts.len(),
                            "{kind:?} size={size} aspect={aspect} sides={sides} corner={corner}"
                        );
                        assert_eq!(now.closed, then.closed, "{kind:?}");
                        assert_eq!(now.fill_rule, then.fill_rule, "{kind:?}");
                        // O FURO da engrenagem viaja num subcontorno: comparar so o
                        // contorno principal deixaria uma engrenagem macica passar
                        // por uma furada.
                        assert_eq!(
                            now.subpaths.len(),
                            then.subpaths.len(),
                            "{kind:?}: numero de subcontornos"
                        );
                        let (gn, gt) = (all_geometry(&now), all_geometry(&then));
                        assert_eq!(gn.len(), gt.len(), "{kind:?}: total de coordenadas");
                        for (i, (x, y)) in gn.iter().zip(&gt).enumerate() {
                            let d = (x - y).abs() / f64::from(size).max(1e-3);
                            worst = worst.max(d);
                            assert!(
                                d < 1e-8,
                                "{kind:?} coord {i} size={size} aspect={aspect} corner={corner}: \
                                 desvio relativo {d:e} — isto e traducao errada, nao arredondamento"
                            );
                        }
                        for (i, (a, b)) in now.verts.iter().zip(&then.verts).enumerate() {
                            assert_eq!(
                                a.kind, b.kind,
                                "{kind:?} vert {i}: a ESPECIE do vertice tem de bater"
                            );
                        }
                        checked += 1;
                    }
                }
            }
        }
    }
    assert!(
        checked >= 600,
        "a varredura tem de ser larga: {checked} celulas"
    );
    // O numero fica PINADO: se ele subir, alguem trocou uma receita e nao so um
    // arredondamento, e a mensagem diz de quanto.
    assert!(
        worst < 1e-8,
        "pior desvio relativo do catalogo antigo: {worst:e}"
    );
}

/// **Toda etiqueta do dropdown desenha alguma coisa.** As duas listas do nó (os
/// rótulos e as espécies) e a tradução do shell são três coisas que têm de
/// concordar, e nada no compilador as alinha — um rótulo a mais é uma linha do menu
/// que cozinha a forma do vizinho, e um a menos é uma forma inalcançável.
///
/// Também prova que a wave ENTREGA: uma contagem de espécies DISTINTAS do
/// `ph2d-vec-scene`, porque oito rótulos apontando para a mesma elipse seriam um
/// catálogo que só parece grande.
#[test]
fn every_label_names_a_kind_and_every_kind_draws() {
    use std::collections::BTreeSet;
    assert_eq!(
        KIND_LABELS.len(),
        ALL_KINDS.len(),
        "rotulo sem especie por tras (ou o contrario)"
    );
    let mut vec_kinds = BTreeSet::new();
    for (i, k) in ALL_KINDS.iter().enumerate() {
        assert_eq!(
            ShapeKind::from_index(i as f32),
            *k,
            "o indice {i} nao devolve a especie {k:?}"
        );
        assert_eq!(k.index(), i, "e a volta tem de fechar");
        let p = ShapeParams {
            kind: *k,
            size: 1.0,
            aspect: 1.3,
            sides: 6,
            corner: 0.2,
            star_depth: 0.45,
            cleft: 0.2,
            tooth_depth: 0.35,
            hole: 0.45,
        };
        let path = build_shape_path(&p);
        assert!(
            path.verts.len() >= 2,
            "{k:?} ({}) nao desenhou nada",
            KIND_LABELS[i]
        );
        assert!(
            path.closed,
            "{k:?} e ABERTA — so as preenchiveis entram nesta wave (as cinco de traco esperam)"
        );
        assert!(
            path.verts
                .iter()
                .all(|v| v.anchor[0].is_finite() && v.anchor[1].is_finite()),
            "{k:?} produziu coordenada nao-finita"
        );
        vec_kinds.insert(format!("{:?}", vec_recipe(&p).0));
    }
    assert!(
        vec_kinds.len() >= 41,
        "o catalogo tem de ser de especies DISTINTAS, e sao {}",
        vec_kinds.len()
    );
}

/// **Nenhuma espécie esconde um controlo vivo nem mostra um morto.** Os `ParamGate`
/// decidem que sliders o painel pinta por espécie, e a lista é escrita à mão — a
/// forma exata que apodrece quando o catálogo cresce.
///
/// O oráculo não conhece a tabela: para cada espécie e cada param gateado, ele MEXE
/// no número e olha se a geometria mudou. Se mudou, o param tem de estar visível
/// (senão é um controlo vivo escondido, e o artista conclui que a forma não se
/// ajusta); se não mudou, não pode estar (o botão morto que este codebase recusa).
#[test]
fn no_kind_hides_a_live_knob_or_shows_a_dead_one() {
    let base = |kind: ShapeKind| ShapeParams {
        kind,
        size: 1.0,
        aspect: 1.0,
        sides: 6,
        corner: 0.0,
        star_depth: 0.45,
        cleft: 0.2,
        tooth_depth: 0.35,
        hole: 0.45,
    };
    // Um valor claramente diferente por param — o suficiente para a geometria
    // responder se ela responde de todo.
    let nudge: &[(&str, fn(&mut ShapeParams))] = &[
        ("aspect", |p| p.aspect = 2.5),
        ("sides", |p| p.sides = 11),
        ("corner", |p| p.corner = 0.6),
        ("star_depth", |p| p.star_depth = 0.85),
        ("cleft", |p| p.cleft = 0.42),
        ("tooth_depth", |p| p.tooth_depth = 0.55),
        ("hole", |p| p.hole = 0.9),
    ];
    let same = |a: &ph2d_vec_scene::VecPath, b: &ph2d_vec_scene::VecPath| {
        all_geometry(a) == all_geometry(b)
    };
    for (i, kind) in ALL_KINDS.iter().enumerate() {
        let untouched = build_shape_path(&base(*kind));
        for (name, apply) in nudge {
            let mut p = base(*kind);
            apply(&mut p);
            let live = !same(&build_shape_path(&p), &untouched);
            let shown = param_gate_shows(name, i);
            assert_eq!(
                live,
                shown,
                "{} ({:?}): o slider `{name}` {} e o painel {}",
                KIND_LABELS[i],
                kind,
                if live { "MUDA a forma" } else { "nao faz nada" },
                if shown { "mostra-o" } else { "esconde-o" },
            );
        }
    }
}

/// Toda a geometria de um path como uma lista chata de números — os vértices do
/// contorno principal E os dos SUBCONTORNOS.
///
/// ⚠️ **Ler só `verts` foi um oráculo cego, e ele já me mentiu uma vez.** O furo de
/// uma engrenagem e o miolo de uma rosquinha viajam em `subpaths` (compound path),
/// não no contorno principal, então um comparador que os ignora reporta *"mexer no
/// `hole` não muda nada"* sobre um knob perfeitamente VIVO — e, pior, deixaria
/// passar o dia em que o furo desaparecesse de verdade.
fn all_geometry(p: &ph2d_vec_scene::VecPath) -> Vec<f64> {
    let mut out = Vec::new();
    let mut push = |vs: &[ph2d_vec_scene::VecVertex]| {
        for v in vs {
            out.extend([
                v.anchor[0],
                v.anchor[1],
                v.in_handle[0],
                v.in_handle[1],
                v.out_handle[0],
                v.out_handle[1],
            ]);
        }
    };
    push(&p.verts);
    for c in &p.subpaths {
        push(&c.verts);
    }
    out
}

/// O painel mostra `name` para a espécie de índice `idx`? Lê a MESMA tabela que o
/// registry entrega ao painel — uma cópia aqui seria a segunda lista a driftar.
fn param_gate_shows(name: &str, idx: usize) -> bool {
    let mut reg = ph2d_node_registry::NodeRegistry::new();
    ph2d_node_motion_shape::register(&mut reg).unwrap();
    let gates = reg
        .param_gates(ph2d_node_motion_shape::MANIFEST.id)
        .unwrap_or(&[]);
    match gates.iter().find(|g| g.param == name) {
        // Sem porta = sempre visível.
        None => true,
        Some(g) => g.values.contains(&(idx as i32)),
    }
}
