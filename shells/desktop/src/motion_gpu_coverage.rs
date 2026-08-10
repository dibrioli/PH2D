//! **GPU coverage census** — which kernel-less nodes sit in the CPU prefix of
//! the documents that actually exist (item #1 of the continuation handoff, the
//! method of `a2226787`).
//!
//! The next coverage kernel is CHOSEN by measurement, not by a shortlist: a
//! kernel is worth writing when the node it covers is what forces a real
//! document onto the CPU. This runs the planner ([`ph2d_gpu_cook::plan`]) over
//! the snow `sim.zone` sim — the one artist-grade document the repo builds — and
//! the six `PH2D_GPU_COOK_DEMO` scenes, and reports, per document: is it fully
//! GPU-resident, and if not, which node type is the **frontier** boundary (the
//! immediate blocker) and which node types make up the whole CPU **prefix**
//! behind it.
//!
//! ⚠️ A neve **deixou de ser o documento de boot** (Enio, 2026-08-07: *"tire a cena
//! da cachoeira"*) — o editor abre vazio. Ela segue no corpus, e é o corpus que
//! importa aqui: um documento que ninguém mais vê ao abrir o app continua sendo o
//! único grafo desta lista que um artista poderia ter autorado, e é contra ele que
//! a fronteira de CPU significa alguma coisa. Tirá-lo daqui junto com o boot teria
//! deixado o censo medindo só andaimes — vários deles moldados à mão para serem
//! 100% device, ou seja, incapazes de apontar trabalho.
//!
//! It is pure plan-time analysis — no device — so it runs on any CI lane. Read
//! the report with `--nocapture`:
//!
//! ```text
//! cargo test -p ph2d-host-desktop gpu_coverage --  --nocapture
//! ```
//!
//! The one assertion is the decision-relevant invariant: at least one real
//! document still has a CPU boundary (there is coverage work to do). The rest is
//! a printout to read, deliberately not a set of exact-boundary pins — those
//! live in `motion_state_gpu_tests.rs`, and pinning them here too would make the
//! census red exactly when a boundary is FIXED, which is the wrong signal for a
//! thing whose job is to point at the next fix.

use super::build_default_document;
use super::gpu_deform_demo::{
    build_gpu_deform_demo_document, build_gpu_four_point_warp_demo_document,
    build_gpu_kaleidoscope_demo_document, build_gpu_spherize_demo_document,
};
use super::gpu_demos::{
    build_gpu_demo_document, build_gpu_emitter_demo_document, build_gpu_hybrid_demo_document,
    build_gpu_sea_demo_document, build_gpu_sim_demo_document,
};
use super::gpu_field_demos::{
    build_gpu_field_box_demo_document, build_gpu_field_combine_demo_document,
    build_gpu_field_curve_demo_document, build_gpu_field_index_range_demo_document,
    build_gpu_field_radial_sweep_demo_document, build_gpu_field_remap_demo_document,
};
use super::gpu_neighbour_demos::{
    build_gpu_boids_demo_document, build_gpu_collide_demo_document, build_gpu_sweep_demo_document,
};
use super::gpu_panel_demo::build_gpu_panel_demo_document;
use super::gpu_pulse_demo::build_gpu_pulse_gate_demo_document;
use super::gpu_voronoi_demo::build_gpu_voronoi_demo_document;
use super::gpu_zone_demo::build_gpu_zone_demo_document;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Graph, NodeId};
use std::collections::{BTreeMap, BTreeSet};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
    reg
}

/// One document to measure: a display name, its graph, and its render sinks.
struct Doc {
    name: &'static str,
    doc: MotionDoc,
    sinks: Vec<NodeId>,
}

/// The corpus: every graph the repo actually builds. The snow is the only
/// artist-grade one (the demos are GPU-path smoke scaffolds, several of them
/// hand-shaped to be fully-GPU on purpose) — so a boundary in IT weighs
/// differently from one in a demo, and the report names it.
fn corpus(reg: &NodeRegistry) -> Vec<Doc> {
    let mut out = Vec::new();
    let mut push = |name: &'static str, build: &dyn Fn(&mut MotionDoc) -> Option<Vec<NodeId>>| {
        let mut doc = MotionDoc::new();
        let sinks = build(&mut doc).unwrap_or_else(|| panic!("{name} builds a well-typed graph"));
        out.push(Doc { name, doc, sinks });
    };
    // The real one: the snow `sim.zone` sim — the only graph here an artist could
    // have authored. (Ele já foi o documento de BOOT; hoje o editor abre vazio.)
    push("snow sim.zone (artist-grade)", &|d| {
        build_default_document(d, reg)
    });
    push("demo=1 grid->osc->move", &|d| {
        build_gpu_demo_document(d, reg)
    });
    push("demo=2 grid->sort->osc->scale", &|d| {
        build_gpu_hybrid_demo_document(d, reg)
    });
    push("demo=3 sim vortex loop", &|d| {
        build_gpu_sim_demo_document(d, reg)
    });
    push("demo=4 sea wind/buoyancy loop", &|d| {
        build_gpu_sea_demo_document(d, reg)
    });
    push("demo=5 emitter fountain", &|d| {
        build_gpu_emitter_demo_document(d, reg)
    });
    push("demo=6 panel value branch", &|d| {
        build_gpu_panel_demo_document(d, reg)
    });
    // The first field.* focus-field source — the index-keyed `falloff` mask. It
    // is fully-GPU (grid → field.index_range → tint), so a boundary here would be
    // a real regression, not an inherent CPU node.
    push("demo=17 field.index_range band", &|d| {
        build_gpu_field_index_range_demo_document(d, reg)
    });
    // The spatial box field — reads P, writes `falloff`; fully-GPU like its
    // ordinal sibling, so a boundary here is a real regression.
    push("demo=18 field.box band", &|d| {
        build_gpu_field_box_demo_document(d, reg)
    });
    // The 2-input composer over a FAN-OUT (two field branches off one grid) — the
    // first field scene that is not a single chain. If `field.combine` or the
    // fan-out ever refused the device, the census would name the boundary.
    push("demo=19 field.combine cross", &|d| {
        build_gpu_field_combine_demo_document(d, reg)
    });
    // The ANGULAR field — reads P, writes `falloff`; the pseudo-angle sector +
    // radial clip, fully-GPU like its rectangular sibling. Wired in so the census
    // is not BLIND to it (a hole nobody puts in a document is an absence, not a
    // clean frontier — the lesson the deformer scene below pins).
    push("demo=20 field.radial_sweep star", &|d| {
        build_gpu_field_radial_sweep_demo_document(d, reg)
    });
    // The REMAPPER over a fed field — box -> remap -> tint. A CPU boundary at the
    // remap (or the box feeding it) would name itself here; wired in so the census
    // sees the whole downstream-remap chain.
    push("demo=21 field.remap bands", &|d| {
        build_gpu_field_remap_demo_document(d, reg)
    });
    // The CURVE contour (A1-gpu) — box -> remap[Curve] -> tint. The remap's mode-4 shape
    // is a text param the uniform cannot carry, so it USED to name itself a CPU boundary;
    // the LUT channel now bakes the curve to a device buffer, so this chain is FULLY GPU
    // and the census prints it as such (the boundary demo is the sort in `=2`).
    push("demo=22 field.remap curve", &|d| {
        build_gpu_field_curve_demo_document(d, reg)
    });
    // ⚠️ **A família do PULSO estava FORA, e a ausência tem a mesma forma que as duas que
    // este arquivo já pagou** (os deformers e a vizinhança): o corpus se declara *"todo grafo
    // que o repo constrói"*, e nenhum documento dele continha um `pulse.*` — então o censo
    // reportava a fronteira sem NUNCA ter planejado a cadeia de evento.
    //
    // Ela entra sabendo o que vai dizer: os seis `pulse.*` e os `value.*` desta cadeia **não
    // têm kernel**, e isso não é omissão a fechar — um pulso é um evento POR LINHA com
    // memória de borda no `pre`, não um mapa por texel. O censo passa a NOMEAR essa fronteira
    // em vez de não a ver, que é a diferença entre um limite conhecido e um buraco.
    push("demo=23 pulse gate (field decides who hears)", &|d| {
        build_gpu_pulse_gate_demo_document(d, reg)
    });
    push("demo=10 sim.zone snow globe", &|d| {
        build_gpu_zone_demo_document(d, reg)
    });
    // ⚠️ The DEFORMER scene, and it earns its place by CHANGING WHAT THE CENSUS
    // CAN SEE. Until this document existed the corpus contained no deformer at
    // all, so the census reported a clean frontier while an entire node family
    // was CPU-only — the census counts FRONTIERS, and a hole nobody wired into a
    // document is not a frontier, it is an absence.
    push("demo=12 deformer cloth (bend->twist)", &|d| {
        build_gpu_deform_demo_document(d, reg)
    });
    // The `Sum` centroid deformer — the two-reduction node.
    push("demo=13 spherize lens (Sum centroid)", &|d| {
        build_gpu_spherize_demo_document(d, reg)
    });
    // The four-reduction bounding-box deformer (the first Min user).
    push("demo=14 four-point-warp (bbox)", &|d| {
        build_gpu_four_point_warp_demo_document(d, reg)
    });
    // The count-changing SourceRows deformer that reads its template.
    push("demo=15 kaleidoscope (SourceRows fan-out)", &|d| {
        build_gpu_kaleidoscope_demo_document(d, reg)
    });
    // ⚠️ **A família da VIZINHANÇA estava FORA, e a ausência tinha consequência.**
    // Este corpus se declara *"todo grafo que o repo constrói"* e não incluía as
    // três cenas da grade espacial (ADR-0140) nem o voronoi — então a sonda de
    // tetos digitáveis nascia **CEGA ao demo que a motivou** (o boids de 2²⁰), e o
    // censo de cobertura nunca planejou o caminho da grade. É a mesma doença que
    // esta arquivo já pagou com os deformers, que também entraram tarde: *um
    // buraco que ninguém põe num documento é ausência, não fronteira limpa.*
    push("demo=7 boids murmuration (grid)", &|d| {
        build_gpu_boids_demo_document(d, reg)
    });
    push("demo=8 collide (grid)", &|d| {
        build_gpu_collide_demo_document(d, reg)
    });
    push("demo=9 sweep (grid reuse)", &|d| {
        build_gpu_sweep_demo_document(d, reg)
    });
    push("demo=11 voronoi (JFA)", &|d| {
        build_gpu_voronoi_demo_document(d, reg)
    });
    out
}

fn ty_name(g: &Graph, n: NodeId) -> String {
    g.node(n)
        .map(|i| i.type_name.clone())
        .unwrap_or_else(|| "<gone>".into())
}

/// Every node upstream of (and including) `start`, over EVERY input edge —
/// forward and `pre`/delayed. The CPU cooks a boundary by cooking its whole
/// chain, and for a sim node that chain closes through a `pre` back into the
/// force loop, so a walk that skipped delayed edges would under-count the
/// prefix. This is the set of node types the CPU must run to feed the seam.
fn upstream_closure(g: &Graph, start: NodeId) -> BTreeSet<u32> {
    let mut seen = BTreeSet::new();
    let mut stack = vec![start];
    while let Some(n) = stack.pop() {
        if !seen.insert(n.0) {
            continue;
        }
        for e in g.edges() {
            if e.to.0 == n {
                stack.push(e.from.0);
            }
        }
    }
    seen
}

#[test]
fn gpu_coverage_census() {
    let reg = registry();
    let docs = corpus(&reg);

    // How often each type is the IMMEDIATE boundary (the frontier — the node a
    // kernel would have to cover to advance the claim) across every sink.
    let mut frontier: BTreeMap<String, usize> = BTreeMap::new();
    // How often each type appears ANYWHERE in a CPU prefix (frontier + its
    // upstream), counted once per sink.
    let mut prefix: BTreeMap<String, usize> = BTreeMap::new();

    let mut any_boundary = false;

    println!("\n=== GPU coverage census — plan() over the documents that exist ===\n");
    for Doc { name, doc, sinks } in &docs {
        let g = &doc.graph;
        for &sink in sinks {
            let plan = ph2d_gpu_cook::plan(g, &reg, &reg, sink);
            let full = plan.is_fully_gpu();
            let dispatching = plan.dispatching_stages(&reg);
            print!(
                "[{}] sink `{}`: {} — {} GPU stage(s) dispatch",
                name,
                ty_name(g, sink),
                if full { "FULLY GPU" } else { "HYBRID/CPU" },
                dispatching,
            );
            if full {
                println!();
                continue;
            }
            any_boundary = true;

            // Frontier: the immediate boundaries, with whether the node has a
            // kernel at all (refused-despite-kernel is a different problem than
            // no-kernel — a driven param, a column-shape refusal, or the whole
            // sim-state recede).
            let mut frontier_names = Vec::new();
            let mut prefix_ids: BTreeSet<u32> = BTreeSet::new();
            for &(bnode, _port) in &plan.boundaries {
                let tn = ty_name(g, bnode);
                let has_kernel = g
                    .node(bnode)
                    .and_then(|i| reg_has_kernel(&reg, i.type_id()))
                    .unwrap_or(false);
                frontier_names.push(if has_kernel {
                    format!("{tn} [refused-despite-kernel]")
                } else {
                    format!("{tn} [no-kernel]")
                });
                *frontier.entry(tn).or_default() += 1;
                prefix_ids.extend(upstream_closure(g, bnode));
            }
            println!("  boundaries: {}", frontier_names.join(", "));

            // Prefix: every node type the CPU must cook behind the seam.
            let mut prefix_types: BTreeSet<String> = BTreeSet::new();
            for id in &prefix_ids {
                prefix_types.insert(ty_name(g, NodeId(*id)));
            }
            for t in &prefix_types {
                *prefix.entry(t.clone()).or_default() += 1;
            }
            println!(
                "  CPU prefix ({} nodes, {} types): {}",
                prefix_ids.len(),
                prefix_types.len(),
                prefix_types.into_iter().collect::<Vec<_>>().join(", "),
            );
        }
    }

    println!("\n--- frontier tally (immediate boundary; the node a kernel must cover) ---");
    for (t, n) in sorted_desc(&frontier) {
        println!("  {n:>3}×  {t}");
    }
    println!("\n--- CPU-prefix tally (appears behind a seam, per sink) ---");
    for (t, n) in sorted_desc(&prefix) {
        println!("  {n:>3}×  {t}");
    }
    println!();

    assert!(
        !docs.is_empty() && any_boundary,
        "the census measures nothing if the corpus is empty or every document is already fully GPU"
    );
}

/// Sorted by count desc, then name — a stable order for the report.
fn sorted_desc(m: &BTreeMap<String, usize>) -> Vec<(String, usize)> {
    let mut v: Vec<(String, usize)> = m.iter().map(|(k, &n)| (k.clone(), n)).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v
}

/// Does the registry carry a kernel for this type? (`KernelResolver` is the
/// same `reg`; this is just the `Option::is_some` in a spot the borrow checker
/// likes.)
fn reg_has_kernel(reg: &NodeRegistry, ty: ph2d_nodegraph::node::NodeTypeId) -> Option<bool> {
    use ph2d_nodegraph::gpu::KernelResolver;
    Some(reg.gpu_kernel(ty).is_some())
}

/// **Que valores o repo AUTORA que o artista não consegue DIGITAR?** — a sonda que nasceu a
/// caminho do teto do boids (doc 89 W1 · CLAUDE.md §0.0) e achou uma família inteira.
///
/// A cena `=7` shipa `motion.boids` com `count = 1.048.576` — três rodadas de smoke, 60 fps
/// medidos — enquanto o teto digitável do param dizia **2.000**, herdado do caminho de CPU
/// (`O(N²)`) que este nó só usa como referência. ⚠️ **Isso não quebra o documento**, e é por isso
/// que nenhum gate via: o painel ALARGA a faixa para conter o que o arquivo traz. O que quebra é o
/// artista **reproduzir à mão o número que o próprio app roda**.
///
/// ⚠️ **E a varredura mostrou que o boids era o caso MENOS comum.** As outras catorze violações
/// não são tetos herdados de caminho lento: são params **sem `ParamHardMax` nenhum**, onde o teto
/// digitável **colapsa na faixa de arrasto** — `motion.move.dx` para em 10 num demo que translada
/// 260, `motion.spherize.radius` em 20 sobre um raio de 320. É o doc 88 B2 dizendo que faltam as
/// duas metades: o soft (arrasto confortável) existe, o hard (onde o disfuncional começa) não.
/// ⇒ **É a varredura por família da §9 do doc 88**, não desta wave, e o teto de cada um se MEDE.
///
/// Por isso isto é SONDA e não gate: transformá-la em vermelho hoje só ofereceria duas saídas
/// ruins — fazer a wave alheia por dentro desta, ou shipar uma allowlist de catorze nomes, que é a
/// enumeração que apodrece. O gate que ESTA wave sustenta é o do boids, e mora ao lado do demo que
/// ele descreve (`motion_state_gpu_neighbour_tests.rs`).
///
/// A varredura mora aqui porque [`corpus`] já é a lista única de todo grafo que o repo constrói —
/// uma segunda lista seria a porta que apodrece na primeira cena nova.
///
/// ⚠️ **A régua é a do PAINEL, verbatim** (`motion_bridge_params`): o alcance digitável é
/// `param_hard_max.unwrap_or(hint.max).max(hint.max)`, deliberadamente **sem** o `contain` que
/// alarga a faixa para conter o valor do documento — é justamente esse alargamento que esconde o
/// defeito, então a pergunta tem de ser o que uma sessão de autoria NOVA alcança.
///
/// ⚠️ **Param sem hint nenhum é PULADO, e não por preguiça:** ali o painel deriva uma faixa
/// neutra do default do manifesto e a contém no valor, ou seja **não existe teto a violar**.
/// Afirmar um sobre ele seria inventar a régua que o produto não tem.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins what_the_corpus_authors_and_no_one_can_type -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda: imprime uma tabela para a varredura por família do doc 88 §9"]
fn what_the_corpus_authors_and_no_one_can_type() {
    let reg = registry();
    let mut offenders: Vec<String> = Vec::new();
    let mut checked = 0usize;

    for Doc { name, doc, .. } in &corpus(&reg) {
        let g = &doc.graph;
        for (i, inst) in g.nodes().iter().enumerate() {
            let node = NodeId(i as u32);
            let Some(overrides) = g.node_param_overrides(node) else {
                continue;
            };
            let hints = reg.param_ui(inst.type_id()).unwrap_or(&[]);
            for (param, &value) in overrides {
                let Some(h) = hints.iter().find(|h| h.param == param) else {
                    continue;
                };
                checked += 1;
                let ceiling = reg
                    .param_hard_max(inst.type_id(), param)
                    .unwrap_or(h.max)
                    .max(h.max);
                let floor = reg
                    .param_hard_min(inst.type_id(), param)
                    .unwrap_or(h.min)
                    .min(h.min);
                if value > ceiling || value < floor {
                    offenders.push(format!(
                        "[{name}] {}.{param} = {value} está fora do digitável [{floor}, {ceiling}]",
                        inst.type_name
                    ));
                }
            }
        }
    }

    assert!(
        checked > 0,
        "varredura vazia: o corpus não autorou um único param com hint — a sonda mediria nada"
    );
    println!(
        "\n=== valores autorados que o artista NÃO consegue digitar ({} de {checked}) ===\n",
        offenders.len()
    );
    for o in &offenders {
        println!("  {o}");
    }
    println!();
}
