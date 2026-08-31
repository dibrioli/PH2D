//! ⭐⭐⭐ **O construtor do CARTÃO DE PROPRIEDADES** — irmão do [`super::inspector_instance`], com a
//! mesma divisão de donos: a verdade mora no ECS, isto lê-a, e o painel só mostra.
//!
//! # ⛔⛔ O buraco que ele fecha (report do Enio, 2026-08-31)
//!
//! *«quando mudo o conteúdo entre `{}` o inspector não muda»*. As chaves de um nome tinham **dois**
//! leitores em todo o app — o selo `*²` da Hierarquia e a fileira de troca do cartão de instância —
//! e o segundo exige uma família de **duas ou mais receitas**. ⇒ num objecto solto, ou numa cópia de
//! um mestre único, reescrever as chaves não mudava um pixel do Inspector.
//!
//! # ⚠️ Quem DECLARA não é sempre quem está selecionado
//!
//! Uma propriedade é do **componente**, não do exemplar: numa cópia a declaração lê-se do nome do
//! MESTRE da raiz, e só quando não há mestre nenhum se lê o nome próprio. *Ler sempre o nome próprio
//! faria uma cópia renomeada pelo artista («Bob») perder as propriedades que ela de facto tem.*

use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor::screens::hero::InspectorPropertiesInfo;

/// Lê as propriedades da entidade selecionada. `None` = ela não declara nenhuma **e** não pertence
/// a família nenhuma — e aí o cartão não existe (a lei da F3: o Inspector mostra o que o objeto TEM).
pub(super) fn build_properties_info(
    sim: &mut SimWorld,
    selected: Option<u64>,
) -> Option<InspectorPropertiesInfo> {
    let entity = Entity::from_bits(selected?);
    if sim.world().get_entity(entity).is_err() {
        return None;
    }
    // A raiz da cópia, quando isto é uma cópia. ⚠️ **`instance_root_of` é a MESMA travessia dos
    // quatro verbos** — escrever aqui uma segunda daria duas respostas a *«a que cópia esta peça
    // pertence?»*.
    let root = crate::instance_verbs::instance_root_of(sim, entity);
    let root_master =
        root.and_then(|r| sim.world().get::<ph2d_ecs::InstanceOf>(r).map(|l| l.master));
    let declared_by = root_master
        .and_then(|id| super::inspector_instance::master_named(sim, id))
        .or_else(|| {
            sim.world()
                .get::<ph2d_ecs::Name>(entity)
                .map(|n| n.0.clone())
        })?;

    let members = root_master.map_or_else(Vec::new, |id| {
        super::inspector_instance::family_members(sim, id)
    });
    let (rows, beyond) = ph2d_editor::screens::hero::variant_axes::rows_for(
        &members,
        root_master.unwrap_or_default(),
        &declared_by,
    );
    if rows.is_empty() {
        return None;
    }
    Some(InspectorPropertiesInfo {
        entity_bits: entity.to_bits(),
        // ⚠️ **`0` quando não há cópia**, e o clique honra-o: sem raiz não há a quem pedir a troca.
        // Nesse estado nenhuma fileira tem mais de um valor, então nem chega a haver chip.
        root_bits: root.map_or(0, Entity::to_bits),
        rows,
        beyond,
        // ⚠️ **Só quando NÃO é do próprio objecto** — ver o doc do campo. Num objecto solto a
        // declaração é dele, e nomear a fonte seria dizer-lhe o nome dele próprio.
        source_name: root_master
            .map(|_| ph2d_editor::screens::hero::variant_axes::display_name(&declared_by)),
    })
}

#[cfg(test)]
#[path = "inspector_properties_tests.rs"]
mod tests;
