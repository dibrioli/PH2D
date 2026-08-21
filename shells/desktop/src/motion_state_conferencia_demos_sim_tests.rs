//! Os gates da cena `=75` — o pin que rasga e o bando que desvia.

use super::*;
use ph2d_nodegraph::graph::NodeId;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

fn scene() -> (MotionDoc, Vec<NodeId>) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_sim_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipado");
    (doc, sinks)
}

fn nodes_of(doc: &MotionDoc, ty: &str) -> Vec<NodeId> {
    doc.graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == ty)
        .map(|n| n.id)
        .collect()
}

fn param(doc: &MotionDoc, id: NodeId, name: &str) -> f32 {
    doc.graph
        .node_param_overrides(id)
        .and_then(|m| m.get(name).copied())
        .unwrap_or(f32::NAN)
}

/// **DUAS LINHAS, QUATRO NUVENS, E AS PEDRAS DESENHADAS.**
#[test]
fn the_scene_is_two_rows_with_its_marks_drawn() {
    let (doc, sinks) = scene();
    assert_eq!(sinks.len(), 6, "quatro nuvens + duas pedras");
    assert_eq!(nodes_of(&doc, "motion.integrate").len(), 2, "só a linha 1");
    assert_eq!(nodes_of(&doc, "motion.boids").len(), 2);
    assert_eq!(nodes_of(&doc, "motion.pin_constraint").len(), 2);
}

/// **O PIN VIVE NO LAÇO DA FORÇA, DEPOIS DO VENTO — e é isso que lhe dá carga.**
///
/// ⚠️ Duas afirmações numa, e as duas são load-bearing. O integrador lê `accel` **e**
/// `inv_mass` da porta `forces`, nunca da `rest`: um pin fora do laço escreveria um
/// `inv_mass` que ninguém lê. E um pin ANTES do vento não veria carga nenhuma, então
/// nunca rasgaria — a linha ficaria verde e muda.
#[test]
fn the_pin_sits_in_the_force_loop_downstream_of_the_wind() {
    let (doc, _) = scene();
    let edges = doc.graph.edges();
    for (pin, wind, integ) in nodes_of(&doc, "motion.pin_constraint")
        .into_iter()
        .zip(nodes_of(&doc, "force.wind"))
        .zip(nodes_of(&doc, "motion.integrate"))
        .map(|((p, w), i)| (p, w, i))
    {
        assert!(
            edges
                .iter()
                .any(|e| e.from.0 == wind && e.to == (pin, 0) && !e.delayed),
            "o vento tem de estar A MONTANTE do pin"
        );
        assert!(
            edges
                .iter()
                .any(|e| e.from.0 == pin && e.to == (integ, 1) && !e.delayed),
            "e o pin tem de alimentar a porta `forces` do integrador"
        );
        assert!(
            edges
                .iter()
                .any(|e| e.from.0 == integ && e.to == (wind, 0) && e.delayed),
            "o `pre` fecha o laço na cabeça da cadeia"
        );
        assert!(
            edges
                .iter()
                .any(|e| e.from.0 == pin && e.to == (pin, 1) && e.delayed),
            "e o pin precisa da própria memória, senão ele CEDE em vez de rasgar"
        );
    }
}

/// **A LINHA 1 DIFERE SÓ NO `break_above`, e o pin selecciona a MESMA fileira.**
#[test]
fn the_tear_row_changes_only_the_threshold() {
    let (doc, _) = scene();
    let pins = nodes_of(&doc, "motion.pin_constraint");
    let (first, count) = pinned_run();
    for p in &pins {
        assert_eq!(param(&doc, *p, "first"), first, "a mesma fileira");
        assert_eq!(param(&doc, *p, "count"), count);
        assert_eq!(param(&doc, *p, "strength"), 1.0);
    }
    assert_eq!(param(&doc, pins[0], "break_above"), 0.0, "esquerda: nunca");
    assert_eq!(param(&doc, pins[1], "break_above"), BREAK_ABOVE);
    // E os dois ventos são o mesmo vento.
    let winds = nodes_of(&doc, "force.wind");
    assert_eq!(
        param(&doc, winds[0], "strength"),
        param(&doc, winds[1], "strength")
    );
}

/// **O LIMIAR ESTÁ ABAIXO DO VENTO — senão o pin NUNCA rasga.**
///
/// ⚠️ Derivado das duas constantes, não repetido: um vento afinado para baixo deixaria
/// a metade da direita idêntica à da esquerda, e a linha ficaria verde e muda. É a
/// mesma classe de erro que o pentágono maior que a grelha na `=73`.
#[test]
fn the_tear_threshold_is_below_the_wind() {
    // ⚠️ `black_box` para que o compilador NÃO dobre a comparação: dois `const` dão um
    // `assert!` de valor constante, que o clippy recusa — e com razão, porque um
    // assert dobrado não corre. Ele tem de correr, porque é o que reprova quando
    // alguém afinar o vento para baixo.
    let (limiar, vento) = (
        std::hint::black_box(BREAK_ABOVE),
        std::hint::black_box(WIND),
    );
    assert!(
        limiar < vento,
        "o limiar ({limiar}) tem de ser menor que o vento ({vento})"
    );
}

/// **A FILEIRA PINADA É A DE CIMA, e o índice é DERIVADO da grelha.**
///
/// ⚠️ A `motion.grid` é row-major da MENOR altura para cima, então as últimas `cols`
/// peças são o topo. Escrever `30` à mão aqui seria um número que deixa de significar o
/// topo assim que alguém mexer na grelha.
#[test]
fn the_pinned_run_is_the_top_row_derived_from_the_grid() {
    let (cols, rows, ..) = CURTAIN;
    let (first, count) = pinned_run();
    assert_eq!(count, cols, "uma fileira inteira");
    assert_eq!(first, (rows - 1.0) * cols, "a ÚLTIMA fileira = o topo");
    assert!(first + count <= rows * cols, "e ela cabe na grelha");
}

/// **A LINHA 2 DIFERE SÓ NO `avoid`, E SÓ A DIREITA TEM PEDRAS LIGADAS — na porta 3.**
#[test]
fn the_avoid_row_changes_only_the_steer() {
    let (doc, _) = scene();
    let flocks = nodes_of(&doc, "motion.boids");
    assert_eq!(param(&doc, flocks[0], "avoid"), 0.0, "esquerda: atravessa");
    assert_eq!(param(&doc, flocks[1], "avoid"), AVOID);
    for k in ["count", "seed", "radius", "separation", "seek", "lookahead"] {
        assert_eq!(
            param(&doc, flocks[0], k),
            param(&doc, flocks[1], k),
            "`{k}` tem de ser o mesmo — senão a linha mede duas coisas"
        );
    }
    let edges = doc.graph.edges();
    let rings = nodes_of(&doc, "motion.distribute_radial");
    assert_eq!(rings.len(), 2, "as duas metades DESENHAM as pedras");
    assert!(
        edges
            .iter()
            .any(|e| e.from.0 == rings[1] && e.to == (flocks[1], 3) && !e.delayed),
        "só a direita LIGA as pedras à porta `obstacle`"
    );
    assert!(
        !edges.iter().any(|e| e.to == (flocks[0], 3)),
        "e a esquerda não pode tê-las ligadas — é o controlo"
    );
    for f in &flocks {
        assert!(
            edges
                .iter()
                .any(|e| e.from.0 == *f && e.to == (*f, 2) && e.delayed),
            "todo bando precisa do próprio estado"
        );
    }
}

/// **AS PEDRAS SÃO DESENHADAS NOS DOIS LADOS** — senão a metade da esquerda pareceria
/// não ter obstáculo nenhum, e o par mediria *"há pedra?"* em vez de *"ele desvia?"*.
#[test]
fn both_halves_draw_the_rocks() {
    let (doc, _) = scene();
    let rings = nodes_of(&doc, "motion.distribute_radial");
    for r in &rings {
        assert_eq!(param(&doc, *r, "count"), ROCKS);
        assert_eq!(param(&doc, *r, "radius"), ROCK_RING);
    }
}

/// **O RAIO DO DESVIO ALCANÇA O ESPAÇAMENTO DAS PEDRAS** — senão o bando passa PELOS
/// VÃOS e o desvio nunca se vê.
///
/// ⚠️ A corda entre duas pedras vizinhas de um anel de `n` é `2·R·sin(π/n)`; para `n`
/// grande ela tende a `2πR/n`, e a aproximação **subestima** a corda, o que torna a
/// asserção conservadora. Derivada do anel, não escrita à mão.
#[test]
fn the_avoid_radius_covers_the_gap_between_rocks() {
    let vao = 2.0 * core::f32::consts::PI * ROCK_RING / ROCKS;
    assert!(
        AVOID_RADIUS >= vao * 0.5,
        "o raio ({AVOID_RADIUS}) tem de cobrir meio vão ({:.2})",
        vao * 0.5
    );
}

/// **O DIAGNOSER DA CASA NÃO ACHA BURACO NESTA CENA.**
#[test]
fn the_house_diagnoser_finds_no_hole_in_this_scene() {
    let (doc, _) = scene();
    let reg = registry();
    let d = ph2d_motion_diagnose::diagnose(&doc.graph, &reg);
    assert!(d.is_empty(), "a cena não encena defeito nenhum: {d:?}");
}
