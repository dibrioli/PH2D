//! **AS ÂNCORAS da seleção** — a projeção que o painel lê, e a porta que a edita (plano UI/UX W3).
//!
//! Irmão do [`crate::vec_layout_edit`], e com a mesma divisão de donos: a verdade mora no ECS
//! ([`ph2d_ecs::VecAnchors`], no FILHO) e isto é o que a shell publica por frame. O painel não
//! alcança o mundo — se alcançasse, a resposta que DESENHA o chip aceso divergiria da que HONRA o
//! clique.
//!
//! # Uma tabela por eixo, lida nas duas direções
//!
//! Cada fileira é um rádio de quatro, e a correspondência `par de âncoras ↔ chip` é precisa nos
//! dois sentidos: para publicar o aceso e para resolver o clique. Escrevê-la duas vezes é como um
//! chip novo entra só numa delas, e o sintoma é um botão que acende e não faz nada.
//!
//! ⚠️ **A tradução Y-up mora AQUI e em mais lugar nenhum:** o documento mede para CIMA, então
//! *"Top"* é a âncora `1` e *"Bottom"* é a `0`. Os ids não carregam número, e o motor
//! ([`ph2d_ecs::VecAnchors`]) não conhece as palavras — quem as liga é esta tabela.
//!
//! # A ausência do componente ACENDE o neutro
//!
//! Um filho sem regra fica colado na aresta mínima quando a moldura cresce — que é exactamente o
//! que `Left`/`Bottom` diz. Então a seção mostra esses dois acesos em vez de mostrar nada, e o
//! artista vê o estado em que ele já está. Clicar neles é um no-op honesto; clicar noutro é que
//! arma a regra (e captura a régua).

use ph2d_ecs::{Entity, SimWorld, VecAnchors};
use ph2d_editor::ids;
use ph2d_panel_vector::state::AnchorState;
use ph2d_vec_scene::{VecPathId, VecScene};

use crate::vec_entities::VecEntityMap;

/// Horizontal: `(chip, [min, max])` no eixo X. `0` = esquerda, `1` = direita.
const H: &[(ph2d_editor::NodeId, [f64; 2])] = &[
    (ids::VECTOR_ANCHOR_H_START, [0.0, 0.0]),
    (ids::VECTOR_ANCHOR_H_CENTER, [0.5, 0.5]),
    (ids::VECTOR_ANCHOR_H_END, [1.0, 1.0]),
    (ids::VECTOR_ANCHOR_H_STRETCH, [0.0, 1.0]),
];

/// Vertical: `(chip, [min, max])` no eixo Y. ⚠️ **`Top` é `1`** — o documento é Y-up, e é aqui que
/// a palavra do artista vira o número do motor.
const V: &[(ph2d_editor::NodeId, [f64; 2])] = &[
    (ids::VECTOR_ANCHOR_V_START, [1.0, 1.0]),
    (ids::VECTOR_ANCHOR_V_CENTER, [0.5, 0.5]),
    (ids::VECTOR_ANCHOR_V_END, [0.0, 0.0]),
    (ids::VECTOR_ANCHOR_V_STRETCH, [0.0, 1.0]),
];

/// O que um clique num chip de âncora PEDE: o par `[min, max]` para um dos eixos.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum AnchorEdit {
    H([f64; 2]),
    V([f64; 2]),
}

/// Este id é um chip de âncora? Porta única do roteador — a mesma varredura das duas tabelas.
#[must_use]
pub(crate) fn anchor_edit_for_id(id: ph2d_editor::NodeId) -> Option<AnchorEdit> {
    if let Some(&(_, p)) = H.iter().find(|(i, _)| *i == id) {
        return Some(AnchorEdit::H(p));
    }
    V.iter()
        .find(|(i, _)| *i == id)
        .map(|&(_, p)| AnchorEdit::V(p))
}

/// O chip aceso para um par — a MESMA tabela, lida ao contrário.
///
/// ⚠️ Devolve `None` para um par que nenhum chip nomeia (um `0,25` é exprimível no componente e a
/// UI não o oferece). **Nenhum chip aceso é a verdade** nesse caso, e é o precedente literal dos
/// perfis de largura: *"aí nenhuma acende, que é a verdade"*.
fn chip_of(
    table: &[(ph2d_editor::NodeId, [f64; 2])],
    pair: [f64; 2],
) -> Option<ph2d_editor::NodeId> {
    table.iter().find(|(_, p)| *p == pair).map(|&(i, _)| i)
}

/// **O filho ancorável da seleção, e a moldura que o ancora.**
///
/// Espelho exacto do `item_of_selection` do auto layout, e pela mesma razão: uma moldura aninhada
/// é ela própria um filho, então ela responde primeiro; fora disso, só uma seleção de UMA forma
/// tem um "filho" de que falar. A recusa do fluxo mora na porta do passe
/// ([`crate::layout_live::anchors::anchoring_frame`]) — o painel não pode oferecer a regra onde o
/// passe a ignora.
fn anchored_subject(
    sim: &SimWorld,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> Option<(Entity, Entity)> {
    let subject = crate::vec_frame_edit::frame_of_selection(sim, map, selected).or_else(|| {
        let [only] = selected else { return None };
        let &bits = map.get(only)?;
        let e = Entity::from_bits(bits);
        sim.world().get_entity(e).ok().map(|_| e)
    })?;
    let frame = crate::layout_live::anchors::anchoring_frame(sim, subject)?;
    Some((subject, frame))
}

/// A regra do filho selecionado — `None` = a seleção não é um filho ancorável.
#[must_use]
pub(crate) fn selected_anchors(
    sim: &SimWorld,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> Option<AnchorState> {
    let (kid, _) = anchored_subject(sim, map, selected)?;
    // Sem componente, o neutro: é o que a ausência de facto produz.
    let a = sim
        .world()
        .get::<VecAnchors>(kid)
        .copied()
        .unwrap_or_else(|| VecAnchors::armed([0.0; 4]));
    Some(AnchorState {
        h: chip_of(H, [a.min[0], a.max[0]]),
        v: chip_of(V, [a.min[1], a.max[1]]),
    })
}

/// A caixa local da moldura que ancora esta seleção — a régua a capturar ao armar.
fn frame_ruler(sim: &SimWorld, scene: &VecScene, frame: Entity) -> Option<[f64; 4]> {
    let id = sim.world().get::<ph2d_ecs::VecPathRef>(frame)?.0;
    crate::layout_live::anchors::frame_local_box(scene, id)
}

/// Aplica um clique de chip. Devolve `true` se o mundo mudou — o `post_frame_undo` regista por
/// diff, então um no-op não custa passo de undo.
///
/// ⚠️ **A régua é capturada UMA vez, quando a regra NASCE**, e sobrevive a toda troca de chip
/// depois disso. É o que faz trocar `Right` por `Center` mostrar *o que a regra nova diz sobre o
/// redimensionamento que já existe*, em vez de reiniciar do zero e devolver o filho à posição
/// autorada — um salto que o artista não pediu.
pub(crate) fn apply_anchor_edit(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    selected: &[VecPathId],
    edit: AnchorEdit,
) -> bool {
    let Some((kid, frame)) = anchored_subject(sim, map, selected) else {
        return false;
    };
    let cur = sim.world().get::<VecAnchors>(kid).copied();
    let mut next = match cur {
        Some(a) => a,
        None => {
            let Some(now) = frame_ruler(sim, scene, frame) else {
                return false; // moldura sem geometria: não há régua a capturar
            };
            VecAnchors::armed(now)
        }
    };
    match edit {
        AnchorEdit::H([mn, mx]) => {
            next.min[0] = mn;
            next.max[0] = mx;
        }
        AnchorEdit::V([mn, mx]) => {
            next.min[1] = mn;
            next.max[1] = mx;
        }
    }
    if cur == Some(next) {
        return false;
    }
    // O neutro DESTACA: uma regra que não move nada não viaja no arquivo (o precedente do
    // `VecLayoutItem`). Ver [`VecAnchors::is_neutral`] para a condição que revoga isto.
    if next.is_neutral() {
        if cur.is_none() {
            return false;
        }
        if let Ok(mut em) = sim.world_mut().get_entity_mut(kid) {
            em.remove::<VecAnchors>();
            return true;
        }
        return false;
    }
    if let Ok(mut em) = sim.world_mut().get_entity_mut(kid) {
        em.insert(next);
        return true;
    }
    false
}

#[cfg(test)]
#[path = "vec_anchor_edit_tests.rs"]
mod tests;
