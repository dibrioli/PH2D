//! Os gates da cena `=71` — a família `force.*`.
//!
//! ⚠️ **Uma cena de FORÇAS não se julga por um cook em `t = 0`**: nada se moveu
//! ainda. O que estes gates defendem é a ESTRUTURA — que a força chega ao
//! integrador pelo laço certo, e que os dois lados de cada par diferem no número
//! que a banda anuncia. O caminho é do olho do Enio, e é para isso que a cena pede
//! PLAY.

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
    let sinks = build_force_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipado");
    (doc, sinks)
}

/// Os nós de um tipo, na ordem em que a cena os criou.
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

/// **SEIS BANDAS, E TODA FORÇA CHEGA AO INTEGRADOR PELO LAÇO.**
///
/// ⚠️ **É o gate que mais vezes disparou nas irmãs, e aqui vale dobrado:** uma força
/// sem o `pre` do integrador **não move nada** e a banda fica parada — o defeito
/// que o diagnoser da casa (ADR-0155) existe para nomear, e que numa cena de smoke
/// leria como *"o knob não faz nada"*.
#[test]
fn every_band_wires_its_force_through_the_integrator_loop() {
    let (doc, sinks) = scene();
    assert_eq!(sinks.len(), 6, "três pares");
    let integs = nodes_of(&doc, "motion.integrate");
    assert_eq!(integs.len(), 6, "um integrador por banda");
    for integ in integs {
        let edges = doc.graph.edges();
        assert!(
            edges
                .iter()
                .any(|e| e.from.0 == integ && e.delayed && e.to.1 == 0),
            "o `pre` do integrador tem de alimentar a CABEÇA da cadeia de forças"
        );
        assert!(
            edges.iter().any(|e| e.to == (integ, 1) && !e.delayed),
            "e a PONTA da cadeia tem de alimentar o `forces`"
        );
    }
}

/// **O PAR 1 DIFERE SÓ NO EIXO Y** — o X fica em `1` dos dois lados.
///
/// ⚠️ O oráculo mede os DOIS knobs. Um par que mudasse os dois eixos provaria que
/// «o arrasto ficou mais forte», não que ele ficou ANISOTRÓPICO — e é a segunda
/// coisa que a célula pede.
#[test]
fn the_drag_pair_changes_only_the_vertical_axis() {
    let (doc, _) = scene();
    let drags = nodes_of(&doc, "force.drag");
    assert_eq!(drags.len(), 2, "um por banda do par 1");
    let (drag_y, _) = authored();
    for d in &drags {
        assert_eq!(
            param(&doc, *d, "scale_x"),
            1.0,
            "o X não muda em nenhum lado"
        );
    }
    assert_eq!(
        param(&doc, drags[0], "scale_y"),
        1.0,
        "a esquerda é isotrópica"
    );
    assert_eq!(
        param(&doc, drags[1], "scale_y"),
        drag_y,
        "e a direita freia Y"
    );
}

/// **O PAR 2 DIFERE SÓ NO PERFIL** — o mesmo aro e a mesma força.
#[test]
fn the_vortex_pair_changes_only_the_profile() {
    let (doc, _) = scene();
    let v = nodes_of(&doc, "force.vortex");
    assert_eq!(v.len(), 2);
    assert_eq!(param(&doc, v[0], "curve"), 0.0, "esquerda: Linear");
    assert_eq!(param(&doc, v[1], "curve"), 3.0, "direita: Smoother");
    for k in ["strength", "radius"] {
        assert_eq!(
            param(&doc, v[0], k),
            param(&doc, v[1], k),
            "`{k}` tem de ser o mesmo — senão o par mede duas coisas"
        );
    }
}

/// **O PAR 3 ESCREVE A COLUNA `density`, e só à direita.**
///
/// ⚠️ **O canal CUSTOM é um número mágico numa cena** (`9`), e este gate é o que o
/// defende: se o enum do `motion.drive` for reordenado, a cena passa a dirigir outro
/// canal **em silêncio** — e a banda da direita ficaria idêntica à da esquerda.
#[test]
fn the_buoyancy_pair_drives_the_density_column_on_the_right_only() {
    let (doc, _) = scene();
    let drives = nodes_of(&doc, "motion.drive");
    assert_eq!(drives.len(), 1, "só a banda da direita dirige");
    let d = drives[0];
    assert_eq!(
        doc.graph
            .node_text_param_overrides(d)
            .and_then(|m| m.get("column"))
            .map(String::as_str),
        Some("density"),
        "a coluna que a força lê"
    );
    // ⚠️ O canal tem de ser o CUSTOM — o único que escreve uma coluna nomeada.
    let ch = param(&doc, d, "channel");
    let reg = registry();
    let labels = reg
        .param_ui(ph2d_nodegraph::node::NodeTypeId::of("motion.drive"))
        .and_then(|hints| {
            hints
                .iter()
                .find(|h| h.param == "channel")
                .map(|h| h.widget)
        });
    if let Some(ph2d_node_registry::ParamWidget::Enum { labels }) = labels {
        #[expect(clippy::cast_possible_truncation, reason = "um índice de enum")]
        #[expect(clippy::cast_sign_loss, reason = "o canal é >= 0")]
        let idx = ch.round() as usize;
        assert!(
            labels.get(idx).is_some_and(|l| l.contains("Custom")),
            "o canal {ch} tem de ser o Custom, e é `{:?}`",
            labels.get(idx)
        );
    } else {
        panic!("o `channel` do motion.drive tem de ser um Enum pintado");
    }
}

/// **AS DUAS BANDAS DE UM PAR PARTEM DA MESMA SEMENTE** — senão o par mede o layout.
#[test]
fn both_halves_of_each_pair_start_from_the_same_seed() {
    let (doc, _) = scene();
    let grids = nodes_of(&doc, "motion.grid");
    assert_eq!(grids.len(), 6, "uma semente por banda, e NENHUMA órfã");
    // Par 1: duas grelhas 4×4. Par 2: duas de 7×7. Par 3: duas fileiras de 8.
    for (a, b) in [(0, 1), (2, 3), (4, 5)] {
        for k in ["rows", "cols", "gap_x"] {
            assert_eq!(
                param(&doc, grids[a], k),
                param(&doc, grids[b], k),
                "as grelhas {a} e {b} discordam em `{k}`"
            );
        }
    }
}
