//! **O ARNÊS partilhado dos gates do L-System** — uma planta, a chave dela, o objecto
//! publicado, e a travessia até às instâncias.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 600 para `shells/`) e por uma razão
//! de desenho que veio junto: os gates das FITAS e os das FOLHAS vivem em módulos irmãos, e
//! duas cópias do arnês divergiriam — *uma fixtura escrita duas vezes é duas fixturas*.

use crate::motion_state::MotionState;
use ph2d_node_source_lsystem as ls;
use ph2d_nodegraph::attr::Column;

/// Uma planta que bifurca, no modo pedido.
pub(crate) fn plant(geometry: i32) -> (MotionState, ph2d_nodegraph::graph::NodeId) {
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

pub(crate) fn published(state: &MotionState, key: &str) -> Option<usize> {
    state
        .pump
        .cook
        .externals()
        .get(key)
        .map(|e| e.value.count())
}

/// A chave que a shell usa, lida pela MESMA porta que o `eval` usa.
pub(crate) fn key_of(state: &mut MotionState, n: ph2d_nodegraph::graph::NodeId) -> String {
    let resolved = super::motion_externals::resolved_params(state, n, 0.0, &ls::MANIFEST);
    let texts = state.doc.graph.node_text_param_overrides(n);
    let text = |k: &str| texts.and_then(|m| m.get(k)).cloned().unwrap_or_default();
    ls::ribbon_key(
        |name: &str| resolved.get(name).copied().unwrap_or(0.0),
        &text(ls::AXIOM_PARAM),
        &text(ls::RULES_PARAM),
    )
}

/// Uma planta cuja gramática pousa um `J` em cada ponta, com o nome pedido no slot pedido.
pub(crate) fn plant_with_leaves(names: [&str; 3]) -> (MotionState, ph2d_nodegraph::graph::NodeId) {
    let (mut state, n) = plant(ls::GEOMETRY_BRANCHES);
    state.doc.graph.set_param(n, ls::param::GENERATIONS, 3.0);
    state.doc.graph.set_text_param(n, ls::AXIOM_PARAM, "F");
    // Cada ponta ganha as TRÊS letras, para uma fixtura só exercitar os três slots.
    state
        .doc
        .graph
        .set_text_param(n, ls::RULES_PARAM, "F -> F[+F[JKM]]F[-F[JKM]]");
    for (i, name) in names.iter().enumerate() {
        if !name.is_empty() {
            state.doc.graph.set_text_param(n, ls::LEAF_PARAMS[i], *name);
        }
    }
    (state, n)
}

/// Publica um objecto nomeado com a aparência que o `publish_objects` publicaria.
pub(crate) fn publish_object(state: &mut MotionState, name: &str, texture_id: u32) {
    publish_object_alpha(state, name, texture_id, false);
}

/// A mesma, com a bandeira de alfa da fonte — a metade que o report de 2026-08-30 pediu.
pub(crate) fn publish_object_alpha(
    state: &mut MotionState,
    name: &str,
    texture_id: u32,
    premultiplied: bool,
) {
    state.pump.cook.set_external(
        name.to_string(),
        super::motion_bridge::appearance_tile(
            [2.0, 3.0],
            [1.0, 1.0, 1.0, 1.0],
            [0.25, 0.25, 0.75, 0.75],
            texture_id,
            premultiplied,
        ),
    );
}

pub(crate) fn column_v1(state: &MotionState, key: &str, col: &str) -> Vec<f32> {
    match state
        .pump
        .cook
        .externals()
        .get(key)
        .map(|e| e.value.get(col))
    {
        Some(Some(Column::Scalar(v))) => v.clone(),
        _ => Vec::new(),
    }
}

/// **Baixa a corrente publicada até às INSTÂNCIAS que o desenho recebe.**
///
/// ⛔⛔ **É esta travessia que apanha o defeito de 2026-08-30**, e nenhuma leitura de coluna a
/// apanharia: a folha publicava a rotação numa coluna chamada `rotation`, e a convenção do
/// Motion chama-lhe **`rot`**. *Um nome de coluna errado não dá erro — a coluna é ignorada e o
/// default (identidade) desenha.* ⇒ o gate tem de perguntar ao consumidor, não ao produtor.
pub(crate) fn instances_of(state: &MotionState, key: &str) -> Vec<ph2d_render::RenderInstance> {
    let mut out = Vec::new();
    if let Some(e) = state.pump.cook.externals().get(key) {
        ph2d_eval_motion::lower_to_instances_onto(
            &e.value,
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 1.0],
            ph2d_render::SinkStyle::PLAIN,
            &mut out,
        );
    }
    out
}

/// Uma planta de fábrica com a folha `J` nomeada, no molde `Tree`.
pub(crate) fn factory_plant_with_leaf(
    gens: f32,
    premultiplied: bool,
) -> (MotionState, ph2d_nodegraph::graph::NodeId) {
    let (mut state, n) = plant(ls::GEOMETRY_BRANCHES);
    let p = &ls::PRESETS[0];
    state.doc.graph.set_text_param(n, ls::AXIOM_PARAM, p.axiom);
    state.doc.graph.set_text_param(n, ls::RULES_PARAM, p.rules);
    state.doc.graph.set_param(n, ls::param::GENERATIONS, gens);
    state.doc.graph.set_param(n, ls::param::ANGLE, p.angle);
    state.doc.graph.set_param(n, ls::param::STEP, p.step);
    state
        .doc
        .graph
        .set_text_param(n, ls::LEAF_PARAMS[0], "folha");
    publish_object_alpha(&mut state, "folha", 7, premultiplied);
    (state, n)
}

/// Publica um objecto NOMEADO que é uma FORMA desenhada (vector vivo) — a outra media, a que
/// pode ir à frente dos galhos.
pub(crate) fn publish_vector_object(state: &mut MotionState, name: &str, geometry_id: u32) {
    state.pump.cook.set_external(
        name.to_string(),
        super::motion_bridge::appearance_vector([2.0, 3.0], [1.0, 1.0, 1.0, 1.0], geometry_id),
    );
}

/// **As instâncias VECTORIAIS desta corrente**, na ordem em que o passe as desenha — irmã de
/// [`instances_of`], que lê o passe das sprites.
///
/// ⚠️ **Ela mudou-se para o testkit em 2026-08-31**, quando um segundo ficheiro de gates
/// precisou dela: um leitor privado de um ficheiro obriga o seguinte a reimplementá-lo, e aí os
/// dois medem coisas parecidas em vez da mesma. *Duas leituras da mesma corrente têm de sair da
/// mesma porta.*
///
/// ⚠️ **Desde a TERCEIRA MÉDIA os dois leitores são obrigatórios em par:** com
/// `Leaves In Front > 0` a copa inteira sai do passe das sprites e entra neste, e um teste que
/// só leia um dos dois mede uma lista vazia e acusa a própria régua.
pub(crate) fn vector_instances_of(
    state: &MotionState,
    key: &str,
) -> Vec<ph2d_eval_motion::VectorInstance> {
    let mut out = Vec::new();
    if let Some(e) = state.pump.cook.externals().get(key) {
        ph2d_eval_motion::lower_to_vector_instances_onto(
            &e.value,
            ph2d_render::SinkStyle::PLAIN,
            &mut out,
        );
    }
    out
}
