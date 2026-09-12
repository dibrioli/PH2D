//! ⭐⭐⭐ **O construtor do CARTÃO DE PROPRIEDADES** — irmão do [`super::inspector_instance`], com a
//! mesma divisão de donos: a verdade mora no ECS, isto lê-a, e o painel só mostra.
//!
//! # ⚠️ Quem DECLARA não é quem está selecionado
//!
//! A família é do **componente**, não do exemplar: numa cópia ela lê-se a partir do MESTRE da
//! raiz, e só quando não há mestre nenhum se olha para a própria entidade. *Perguntar ao exemplar
//! faria uma cópia renomeada pelo artista («Bob») perder a família a que ela pertence.*

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
    // ⭐⭐⭐ **A família da RECEITA que esta cópia segue.** Sem cópia não há família — e uma
    // receita solta não tem o que oferecer, que é o que o `axes_for` responde com zero fileiras.
    let subject =
        root_master.or_else(|| sim.world().get::<ph2d_ecs::StableId>(entity).map(|s| s.0));
    let members = subject.map_or_else(Vec::new, |id| {
        super::inspector_instance::family_members(sim, id)
    });
    let (rows, beyond) = ph2d_editor::screens::hero::variant_axes::axes_for(
        &members,
        root_master.or(subject).unwrap_or_default(),
    );
    if rows.is_empty() {
        return None;
    }
    Some(InspectorPropertiesInfo {
        entity_bits: entity.to_bits(),
        // ⚠️ **`0` quando não há cópia**, e o clique honra-o: sem raiz não há a quem pedir a troca.
        // Nesse estado nenhuma fileira tem mais de um valor, então nem chega a haver chip.
        root_bits: root.map_or(0, Entity::to_bits),
        beyond,
        rows,
        // ⭐⭐⭐ **O nome do objecto SELECIONADO, como a Hierarquia o mostra** (report do Enio,
        // 2026-08-31: *«Properties of "Nome do objeto na Hierarquia"»*).
        //
        // ⚠️ **A 1.ª versão punha aqui o nome do COMPONENTE** — a fonte das propriedades — para
        // explicar por que o cartão dizia `Small` sobre uma cópia renomeada para `Big`. Ele pediu
        // o outro: o cartão é sobre o objecto que está seleccionado, e o nome tem de ser o mesmo
        // que ele lê na lista. *Um título que nomeia uma coisa que não está seleccionada faz o
        // artista procurar onde ela está.*
        // ⚠️ **O nome CRU** — desde 2026-09-01 não há gramática a cortar: as propriedades saíram
        // do `Name`, e o que o artista escreveu é o que ele lê.
        source_name: sim
            .world()
            .get::<ph2d_ecs::Name>(entity)
            .map(|n| n.0.clone()),
    })
}

#[cfg(test)]
#[path = "inspector_properties_tests.rs"]
mod tests;
