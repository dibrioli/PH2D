//! **Arrastar um filho DENTRO de um fluxo é REORDENAR** (ADR-0153, corolário).
//!
//! # Por que o gesto muda de significado
//!
//! A lei da wave é que a posição de um filho colocado é **derivada por frame**. Um arrasto não tem
//! então onde pousar: escrever `Transform.translation` seria escrever num número que o próximo
//! frame recalcula — a forma voltaria para a fila e o artista veria um gesto **quebrado**, com um
//! passo de undo por cima (o undo deste editor regista por DIFF do mundo ECS, e a escrita
//! invisível conta na mesma).
//!
//! Então o mesmo gesto pede outra coisa: **trocar a POSIÇÃO NA FILA**. É o que o Figma faz, e é a
//! única leitura coerente de arrastar algo cuja posição não é sua.
//!
//! # A régua é PUBLICADA, nunca re-derivada
//!
//! Quem decide o slot lê os centros que o passe de layout **acabou de publicar**
//! ([`crate::layout_live::FlowSlots`]) — nunca uma segunda medição das formas. Uma re-derivação
//! divergiria no primeiro `grow`: o artista veria a forma numa posição e o slot ser escolhido por
//! outra, e nada no ecrã diria porquê.
//!
//! # E ela mede CENTROS, não fronteiras
//!
//! O slot é *quantos irmãos têm o centro antes do cursor*. Medir as BORDAS exigiria decidir o que
//! fazer no vão entre duas caixas (onde nenhuma fronteira contém o ponto), e num fluxo com `gap` o
//! artista passa a maior parte do arrasto exactamente ali.

use ph2d_ecs::{ChildOf, Children, Entity, SimWorld, VecLayout};

use crate::layout_live::LayoutLive;

/// **O slot que este ponto pede**, dados os centros dos irmãos JÁ SEM o arrastado, na ordem do
/// fluxo, e a coordenada do cursor no eixo principal.
///
/// Pura e sem estado: é a metade que um gate pode afirmar sem montar uma cena.
#[must_use]
pub(crate) fn slot_at(centres: &[f64], cursor: f64) -> usize {
    centres.iter().filter(|c| **c < cursor).count()
}

/// A moldura que COLOCA esta entidade — `None` quando ela não está dentro de um fluxo.
#[must_use]
pub(crate) fn flow_parent(sim: &SimWorld, e: Entity) -> Option<Entity> {
    let parent = sim.world().get::<ChildOf>(e)?.parent();
    sim.world()
        .get::<VecLayout>(parent)
        .is_some()
        .then_some(parent)
}

/// **O arrasto pousa como uma troca de posição na fila.** Devolve `true` se a ordem mudou.
///
/// `cursor` é o ponto do ponteiro em MUNDO. O eixo lido é o principal do fluxo — no transversal um
/// filho não tem para onde ir, então mexer nele não é informação e ignorá-lo é o que mantém o
/// gesto previsível quando a mão treme.
pub(crate) fn drop_at(
    sim: &mut SimWorld,
    layout: &LayoutLive,
    dragged: Entity,
    cursor: [f32; 2],
) -> bool {
    let Some(parent) = flow_parent(sim, dragged) else {
        return false;
    };
    let Some(slots) = layout.slots_of(parent) else {
        return false;
    };
    let along = f64::from(if slots.main_x { cursor[0] } else { cursor[1] });
    // Os centros dos OUTROS: o arrastado sai da régua, senão o slot que ele já ocupa competiria
    // consigo mesmo e o gesto teria um ponto morto do tamanho da própria forma.
    let centres: Vec<f64> = slots
        .kids
        .iter()
        .filter(|(e, _)| *e != dragged)
        .map(|(_, c)| *c)
        .collect();
    let slot = slot_at(&centres, along);

    // A ordem VIVA da hierarquia é a autoridade sobre quem são os irmãos — a régua do layout pode
    // ser de um frame em que a lista era outra (uma forma acabada de apagar, por exemplo).
    let mut desired: Vec<Entity> = sim
        .world()
        .get::<Children>(parent)
        .map(|c| c.iter().copied().filter(|e| *e != dragged).collect())
        .unwrap_or_default();
    desired.insert(slot.min(desired.len()), dragged);
    ph2d_ecs::reinsert_children_in_order(sim.world_mut(), parent, &desired)
}

#[cfg(test)]
#[path = "layout_reorder_tests.rs"]
mod tests;
