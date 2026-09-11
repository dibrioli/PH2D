//! Gates da cena `=105` — dois berços de onda (folha 06, célula 35).
//!
//! ⚠️ Medem o que a cena DESENHA ao longo do TEMPO: no instante zero os dois tanques são o
//! mesmo campo plano, e é só depois de as fontes injectarem que eles divergem.

use super::*;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

const TICKS: usize = 180;

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_producers_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// O `size` de cada célula no último tique — é por ele que a altura se vê.
fn run(doc: &MotionDoc, reg: &NodeRegistry, sinks: &[NodeId]) -> Vec<Vec<f32>> {
    let mut cook = Cook::new();
    let mut out = vec![Vec::new(); sinks.len()];
    for k in 0..TICKS {
        let t = k as f64 / 60.0;
        for (s, sink) in sinks.iter().enumerate() {
            let v = cook.cook(&doc.graph, reg, *sink, t).expect("coze");
            if k == TICKS - 1
                && let Some(Column::Vec2(sz)) = v[0].as_stream().get("size")
            {
                out[s] = sz.iter().map(|q| q[0]).collect();
            }
        }
        cook.advance_tick(&doc.graph, reg, t).expect("avanca");
    }
    out
}

/// A cena monta os dois tanques e nenhum diverge.
#[test]
fn the_producers_scene_builds_both_ponds() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), 2, "dois tanques");
    for (k, sz) in run(&doc, &reg, &sinks).into_iter().enumerate() {
        assert_eq!(sz.len(), (SIDE * SIDE) as usize, "tanque {k}");
        assert!(sz.iter().all(|x| x.is_finite()), "tanque {k} divergiu");
    }
}

/// ⭐⭐ **O tanque com fontes tem MAIS relevo onde as fontes estão.** A régua olha as colunas
/// em que as duas caixas ficam, não o campo inteiro — é ali que a diferença nasce.
#[test]
fn only_the_pond_with_sources_has_relief_where_the_sources_sit() {
    let (doc, reg, sinks) = scene();
    let r = run(&doc, &reg, &sinks);
    let side = SIDE as usize;
    // A coluna da fonte esquerda, na linha do meio.
    let cell = side / 2 * side + side / 4;
    let bare = r[0][cell];
    let sourced = r[1][cell];
    // O relevo lê-se como distância ao tamanho de repouso do tanque calado.
    let rest = r[0][0];
    assert!(
        (sourced - rest).abs() > (bare - rest).abs() * 1.5,
        "o tanque com fontes tem de ter mais relevo ali: {sourced:.4} contra {bare:.4} \
         (repouso {rest:.4})"
    );
}

/// ⚠️ **O CONTROLE: o tanque calado NÃO está parado.** A batida do centro continua nos dois —
/// sem esta metade, o gate acima passaria com um tanque da esquerda morto.
#[test]
fn the_bare_pond_is_still_beating_at_its_centre() {
    let (doc, reg, sinks) = scene();
    let r = run(&doc, &reg, &sinks);
    let side = SIDE as usize;
    let centre = side / 2 * side + side / 2;
    let rest = r[0][0];
    assert!(
        (r[0][centre] - rest).abs() > 1e-3,
        "o centro do tanque calado bate: {} contra o repouso {rest}",
        r[0][centre]
    );
}

/// ⚠️ **Os números que o anúncio cita vivem em `const`.**
#[test]
fn the_announcement_cites_the_numbers_the_scene_uses() {
    let src = include_str!("motion_state_demo_announce.rs");
    for k in [
        "gpu_producers_demo::SIDE",
        "gpu_producers_demo::SOURCE_X",
        "gpu_producers_demo::STRENGTH",
    ] {
        assert!(src.contains(k), "o anuncio tem de citar `{k}`");
    }
}

/// **SONDA** — onde os dois tanques diferem, e o que a fonte entrega.
///
/// ⚠️ **Ela achou um defeito da CENA que nenhum gate de unidade podia achar:** as duas
/// caixas estavam ENCADEADAS, e encadear dois `field.box` é uma **intersecção** — como elas
/// não se sobrepõem, o produto era zero em toda parte (`max falloff = 0,0000`) e os dois
/// tanques saíam byte-idênticos com a porta correctamente ligada e o ganho correctamente
/// posto. A cura é `field.combine(Max)`, e é a mesma forma que a folha 13 já registara do
/// outro lado: *encadear colisores é conjunção, não união.*
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn probe_where_the_two_ponds_differ() {
    let (doc, reg, sinks) = scene();
    let r = run(&doc, &reg, &sinks);
    let diff = r[0]
        .iter()
        .zip(&r[1])
        .filter(|(a, b)| (*a - *b).abs() > 1e-6)
        .count();
    eprintln!("\n[sonda] celulas diferentes: {diff} de {}", r[0].len());
    let side = SIDE as usize;
    for c in 0..side {
        let i = side / 2 * side + c;
        eprintln!(
            "  col {c:>2}: calado={:.4}  com fontes={:.4}",
            r[0][i], r[1][i]
        );
    }
}
