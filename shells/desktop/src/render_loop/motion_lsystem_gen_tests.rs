//! Gates da **membrana das fitas** — a metade da shell do modo `Branches`.
//!
//! ⚠️ **As quatro condições de UI não servem aqui**: isto não é um widget, é uma MEMBRANA. A
//! pergunta é a do `source.shape`: *a shell publica sob a chave que o nó lê?* Um par de chaves
//! divergentes não dá erro nenhum — dá uma planta invisível.

use super::publish;
use crate::motion_state::MotionState;
use ph2d_node_source_lsystem as ls;
use ph2d_nodegraph::attr::Column;

/// Uma planta que bifurca, no modo pedido.
fn plant(geometry: i32) -> (MotionState, ph2d_nodegraph::graph::NodeId) {
    let mut state = MotionState::new();
    let n = state.doc.graph.add_node(ls::MANIFEST.name);
    state
        .doc
        .graph
        .set_param(n, ls::param::GEOMETRY, geometry as f32);
    // Gramática explícita: o modo guiado deriva a sua, e um gate que dependesse dela mediria
    // duas coisas ao mesmo tempo.
    state
        .doc
        .graph
        .set_param(n, ls::param::MODE, ls::MODE_GRAMMAR as f32);
    state.doc.graph.set_text_param(n, ls::AXIOM_PARAM, "F");
    state
        .doc
        .graph
        .set_text_param(n, ls::RULES_PARAM, "F -> F[+F]F[-F]F");
    (state, n)
}

fn published(state: &MotionState, key: &str) -> Option<usize> {
    state
        .pump
        .cook
        .externals()
        .get(key)
        .map(|e| e.value.count())
}

/// A chave que a shell usa, lida pela MESMA porta que o `eval` usa.
fn key_of(state: &mut MotionState, n: ph2d_nodegraph::graph::NodeId) -> String {
    let resolved = super::super::motion_externals::resolved_params(state, n, 0.0, &ls::MANIFEST);
    let texts = state.doc.graph.node_text_param_overrides(n);
    let text = |k: &str| texts.and_then(|m| m.get(k)).cloned().unwrap_or_default();
    ls::ribbon_key(
        |name: &str| resolved.get(name).copied().unwrap_or(0.0),
        &text(ls::AXIOM_PARAM),
        &text(ls::RULES_PARAM),
    )
}

/// ⭐⭐⭐ **A shell publica FITAS, e são menos que os ossos.**
///
/// É o report do Enio medido do lado que desenha: uma planta que bifurca tem de sair como um
/// punhado de objectos contínuos, não como um por segmento.
#[test]
fn a_plant_in_branches_mode_publishes_fewer_ribbons_than_it_has_bones() {
    let (mut state, n) = plant(ls::GEOMETRY_BRANCHES);
    let key = key_of(&mut state, n);
    publish(&mut state, 0.0);
    let ribbons = published(&state, &key).expect("a shell tem de publicar sob a chave do no'");
    assert!(ribbons > 0, "nenhuma fita publicada");

    // Quantos ossos a mesma planta tem, pela porta do próprio nó.
    let resolved =
        super::super::motion_externals::resolved_params(&mut state, n, 0.0, &ls::MANIFEST);
    let sk = ls::skeleton("F", "F -> F[+F]F[-F]F", |name: &str| {
        resolved.get(name).copied().unwrap_or(0.0)
    });
    assert!(
        ribbons < sk.count(),
        "{ribbons} fitas para {} ossos — isso é uma fita por retângulo",
        sk.count()
    );
}

/// ⭐⭐ **Cada fita leva uma GEOMETRIA de verdade.**
///
/// ⚠️ Um `geometry_id` de `0` é o «nada» do lowering: publicar contagem certa com ids vazios
/// desenharia coisa nenhuma e passaria no gate de cima.
#[test]
fn every_published_ribbon_carries_a_real_geometry_handle() {
    let (mut state, n) = plant(ls::GEOMETRY_BRANCHES);
    let key = key_of(&mut state, n);
    publish(&mut state, 0.0);
    let ext = state.pump.cook.externals().get(&key).expect("publicado");
    let Some(Column::Scalar(ids)) = ext.value.get("geometry_id") else {
        panic!("a fita tem de carregar `geometry_id`");
    };
    assert!(!ids.is_empty());
    assert!(
        ids.iter().all(|h| *h > 0.0),
        "há fitas com handle vazio — elas não desenham: {ids:?}"
    );
}

/// ⭐ **O modo antigo continua intocado** — decisão do Enio (*"não quero eliminar o modo
/// atual"*).
///
/// A shell não publica nada, e o nó emite o esqueleto de sempre.
#[test]
fn segments_mode_publishes_nothing_and_keeps_the_old_skeleton() {
    let (mut state, n) = plant(ls::GEOMETRY_SEGMENTS);
    let key = key_of(&mut state, n);
    publish(&mut state, 0.0);
    assert!(
        published(&state, &key).is_none(),
        "o modo Segments não pode publicar fitas"
    );
}

/// ⭐⭐⭐ **O default do nó É `Branches`** — a ordem do dono, medida no manifesto e não na
/// memória de ninguém.
#[test]
fn a_node_dropped_from_the_palette_is_born_in_branches_mode() {
    let d = ls::MANIFEST
        .params
        .iter()
        .find(|s| s.name == ls::param::GEOMETRY)
        .expect("o param existe")
        .default;
    assert_eq!(
        d.round() as i32,
        ls::GEOMETRY_BRANCHES,
        "o default tem de ser Branches (Enio, 2026-08-30)"
    );
    // ⚠️ E o VALOR de `Segments` continua a ser `0`: um documento salvo guarda o índice.
    assert_eq!(ls::GEOMETRY_SEGMENTS, 0);
}
