//! **O painel edita o conector** — módulo irmão de [`crate::connector_live`] (que cuida do
//! COZIMENTO da rota) e espelho exato de [`crate::vec_shape_params`] (que faz o mesmo pela
//! forma viva).
//!
//! O Enio pediu para calibrar `jetty` e `spread` **com o olho**. Isso impõe três coisas, e
//! nenhuma delas é decoração:
//!
//! 1. **O painel mostra o valor EFETIVO.** Os dois campos são `Option<f32>` no componente:
//!    `None` = automático (o jetty acompanha o tamanho das caixas; o spread, o índice
//!    paralelo). Um campo que nascesse vazio — ou em zero — faria o número SALTAR no
//!    primeiro toque, para longe da linha que está na tela. Então a shell **resolve** o
//!    efetivo aqui ([`effective`]) e publica; o painel só desenha.
//! 2. **Editar FIXA** (`None` → `Some(v)`): o auto→override de qualquer editor.
//! 3. **A edição atinge TODOS os conectores selecionados** ([`edit_selected_connectors`]) —
//!    é o que permite calibrar o diagrama inteiro de uma vez, em vez de linha por linha.
//!
//! # A regra que este módulo existe para honrar: semente = amostra
//!
//! O número que o painel EXIBE e o número que o cozimento USA têm de sair da **mesma
//! função**. Duas cópias da fórmula do automático = o painel mentindo no dia em que alguém
//! ajustar uma delas (é a lição `feedback_derived_coordinate_seed_must_match_sample` — o
//! tempo remapeado quebrou três vezes exatamente assim). Por isso a derivação vive em
//! `ph2d_tool_vector::connector` (`auto_jetty` + `SPREAD_STEP`) e AMBOS os lados a
//! importam: aqui, para exibir; o `connector_live`, para cozinhar.

use ph2d_ecs::{Entity, RouteKind, SimWorld, VecConnector};
use ph2d_editor::NodeId;
use ph2d_tool_vector::connector;
use ph2d_vec_scene::{VecPathId, VecScene, VecXforms};

use crate::vec_entities::VecEntityMap;

/// O conector em FOCO: o primeiro da seleção. Espelho de
/// [`crate::vec_shape_params::panel_shape_target`] — a primeira da seleção manda (é dela
/// que os campos são semeados; a EDIÇÃO, essa, vai para todos).
#[must_use]
pub(crate) fn panel_connector_target(
    sim: &SimWorld,
    map: &VecEntityMap,
    selection: &[VecPathId],
) -> Option<(VecPathId, VecConnector)> {
    selection.iter().find_map(|id| {
        let &bits = map.get(id)?;
        let c = sim.world().get::<VecConnector>(Entity::from_bits(bits))?;
        Some((*id, c.clone()))
    })
}

/// As **meias-extensões** `(hw, hh)` em MUNDO da caixa a que uma ponta se prende. Uma ponta
/// SOLTA (ou cujo alvo sumiu) é um ponto: `(0, 0)` — e o piso do [`connector::auto_jetty`]
/// cuida dela, como no cozimento.
fn half_of(end: &ph2d_ecs::ConnectorEnd, scene: &VecScene, xforms: &VecXforms) -> (f64, f64) {
    let Some(target) = VecConnector::target_of(end) else {
        return (0.0, 0.0);
    };
    scene
        .path_world_curve_bbox(xforms, target)
        .map_or((0.0, 0.0), |(lo, hi)| {
            ((hi[0] - lo[0]) * 0.5, (hi[1] - lo[1]) * 0.5)
        })
}

/// **Os valores EFETIVOS `(jetty, spread)`** deste conector: o que o usuário fixou, ou o
/// automático — pela MESMA função que o cozimento usa (`jetty_or` / `spread_or` do
/// componente, sobre `auto_jetty` / `SPREAD_STEP`).
#[must_use]
pub(crate) fn effective(scene: &VecScene, xforms: &VecXforms, conn: &VecConnector) -> (f64, f64) {
    let auto = connector::auto_jetty(
        half_of(&conn.start, scene, xforms),
        half_of(&conn.end, scene, xforms),
    );
    (conn.jetty_or(auto), conn.spread_or(connector::SPREAD_STEP))
}

/// O que a shell publica para o painel: `(route, jetty efetivo, spread efetivo, corner)` do
/// conector em foco. `None` = **nenhum conector na seleção** ⇒ a seção inteira some.
///
/// O `corner` não tem "efetivo" a resolver — é um `f32` no componente, não um `Option`: zero
/// (quina afiada) já é a resposta certa do fluxograma clássico, e não existe raio automático
/// que faça sentido.
///
/// Devolve uma tupla crua (e não o tipo do painel) porque o painel é opcional na build
/// (`feature = "panel-vector"`): o `vector_bridge` faz a conversão dentro do `cfg`.
#[must_use]
pub(crate) fn selection_snapshot(
    sim: &SimWorld,
    map: &VecEntityMap,
    scene: &VecScene,
    xforms: &VecXforms,
    selection: &[VecPathId],
) -> Option<(u8, f64, f64, f64)> {
    let (_, conn) = panel_connector_target(sim, map, selection)?;
    let (jetty, spread) = effective(scene, xforms, &conn);
    let route = match conn.route {
        RouteKind::Straight => 0,
        RouteKind::Orthogonal => 1,
    };
    Some((route, jetty, spread, f64::from(conn.corner_radius)))
}

/// **Publica o conector em foco para o painel** — os valores já EFETIVOS. Chamado pelo
/// `vector_bridge` a cada frame; `vector_active == false` (ou seleção sem conector) limpa a
/// seção.
///
/// O corpo vive aqui, e não no bridge, por duas razões: o assunto é deste módulo, e o
/// `vector_bridge` estourou o teto de 600 LOC por arquivo da shell quando a publicação
/// nasceu lá (HR-18 — *split, não allowlist*).
pub(crate) fn publish(
    sim: &SimWorld,
    map: &VecEntityMap,
    scene: &VecScene,
    xforms: &VecXforms,
    selection: &[VecPathId],
    vector_active: bool,
) {
    #[cfg(feature = "panel-vector")]
    ph2d_panel_vector::set_current_connector(
        vector_active
            .then(|| selection_snapshot(sim, map, scene, xforms, selection))
            .flatten()
            .map(
                |(route, jetty, spread, corner)| ph2d_panel_vector::ConnectorSnapshot {
                    route,
                    jetty,
                    spread,
                    corner,
                },
            ),
    );
    // Sem o painel na build não há a quem publicar (mas o módulo compila igual).
    #[cfg(not(feature = "panel-vector"))]
    let _ = (sim, map, scene, xforms, selection, vector_active);
}

/// `true` se `id` é um dos campos do conector (o que a shell captura no dreno). Um campo
/// novo entra AQUI e no [`apply_field`] — fora daqui ele pinta e está morto.
#[must_use]
pub(crate) fn is_connector_field_id(id: NodeId) -> bool {
    id == ph2d_editor::ids::VECTOR_CONNECTOR_ROUTE
        || id == ph2d_editor::ids::VECTOR_CONNECTOR_JETTY
        || id == ph2d_editor::ids::VECTOR_CONNECTOR_SPREAD
        || id == ph2d_editor::ids::VECTOR_CONNECTOR_CORNER
}

/// Aplica a edição de um campo no componente. **Editar FIXA** — `None` vira `Some(v)`, e o
/// valor para de acompanhar o automático. `false` = o id não é um campo de conector.
fn apply_field(conn: &mut VecConnector, id: NodeId, v: f64) -> bool {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "o campo do painel e f64; o componente guarda f32 (o save e postcard)"
    )]
    if id == ph2d_editor::ids::VECTOR_CONNECTOR_JETTY {
        conn.jetty = Some(connector::clamp_to(&connector::JETTY, v) as f32);
    } else if id == ph2d_editor::ids::VECTOR_CONNECTOR_SPREAD {
        conn.spread = Some(connector::clamp_to(&connector::SPREAD, v) as f32);
    } else if id == ph2d_editor::ids::VECTOR_CONNECTOR_CORNER {
        // O raio das quinas do PERCURSO. Não é `Option` (não há automático que faça sentido):
        // o campo é a única verdade, e `0` = quina afiada.
        conn.corner_radius = connector::clamp_to(&connector::CORNER, v) as f32;
    } else if id == ph2d_editor::ids::VECTOR_CONNECTOR_ROUTE {
        // O botão cicla e emite o PRÓXIMO discriminante; um valor fora da tabela (não pode
        // acontecer) cai na rota default em vez de entrar em pânico.
        conn.route = if v.round() as i64 == 0 {
            RouteKind::Straight
        } else {
            RouteKind::Orthogonal
        };
    } else {
        return false;
    }
    true
}

/// **A edição atinge TODOS os conectores selecionados.**
///
/// É o ponto da feature: o Enio calibra o diagrama inteiro de uma vez (selecionar tudo →
/// arrastar o Jetty), em vez de visitar linha por linha. Um `for` sobre o primeiro da
/// seleção seria uma UI que promete e não cumpre — e nenhum teste de unidade do componente
/// perceberia.
///
/// A geometria NÃO é reescrita aqui: ela é uma função pura da relação, e o
/// `connector_live::recook` deste mesmo frame (que roda depois) a refaz. O undo global pega
/// a mudança sozinho (o `VecConnector` está no `ComponentRegistry`, e o registro é por
/// DIFF). Devolve quantos conectores mudaram.
pub(crate) fn edit_selected_connectors(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    selection: &[VecPathId],
    id: NodeId,
    v: f64,
) -> usize {
    let targets: Vec<Entity> = selection
        .iter()
        .filter_map(|p| map.get(p).copied())
        .map(Entity::from_bits)
        .filter(|e| sim.world().get::<VecConnector>(*e).is_some())
        .collect();
    let mut hit = 0;
    for e in targets {
        let Some(mut conn) = sim.world().get::<VecConnector>(e).cloned() else {
            continue;
        };
        if !apply_field(&mut conn, id, v) {
            return 0; // o id nao e de conector: nem vale iterar o resto
        }
        if let Ok(mut ent) = sim.world_mut().get_entity_mut(e) {
            ent.insert(conn);
            hit += 1;
        }
    }
    hit
}

#[cfg(test)]
#[path = "vec_connector_panel_tests.rs"]
mod tests;
