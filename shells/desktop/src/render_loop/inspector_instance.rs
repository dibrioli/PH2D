//! ⭐⭐⭐ **O construtor da seção COMPONENT** (ADR-0164 / F5) — irmão do
//! [`super::inspector_anchor`], com a mesma divisão de donos: a verdade mora no ECS, isto lê-a, e
//! o painel só mostra.
//!
//! # ⚠️ Ele NÃO tem lei nenhuma própria
//!
//! Quem é a raiz de uma instância pergunta-se ao [`crate::instance_verbs::instance_root_of`] — a
//! mesma travessia que os quatro verbos usam. Escrever aqui uma segunda seria a forma clássica:
//! duas respostas a *«a que cópia esta peça pertence?»*, e a que envelhece é a que o artista lê.
//!
//! E os RÓTULOS vêm do catálogo (`ph2d-component-desc`), o mesmo de que o `+` deriva a paleta.
//! *Um nome escrito aqui divergiria do nome do botão que anexa aquele componente.*

use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor::screens::hero::InspectorInstanceInfo;

/// Lê o estado de instância da entidade selecionada. `None` = ela não é peça de cópia nenhuma, e
/// aí a seção **não existe** (a lei da F3: o Inspector mostra o que o objeto TEM).
pub(super) fn build_instance_info(
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    selected: Option<u64>,
) -> Option<InspectorInstanceInfo> {
    let entity = Entity::from_bits(selected?);
    if sim.world().get_entity(entity).is_err() {
        return None;
    }
    let link = sim.world().get::<ph2d_ecs::InstanceOf>(entity).copied()?;
    let root = crate::instance_verbs::instance_root_of(sim, entity)?;
    // ⚠️ **`unwrap_or_default`, e não `?`** — e o gate apanhou-me a escrever `?`. O
    // `ObjectInstance` só nasce quando a PRIMEIRA excepção é capturada, então uma cópia intacta não
    // o tem — e a seção desapareceria exactamente no estado em que ela mais informa: *«esta cópia
    // segue a receita»*. *Ausência de excepções não é ausência de instância.*
    let inst = sim
        .world()
        .get::<ph2d_ecs::ObjectInstance>(root)
        .cloned()
        .unwrap_or_default();

    // O nome da RECEITA: o mestre da RAIZ é o `MasterRoot`, e é esse que o artista vê na lista.
    let root_master = sim
        .world()
        .get::<ph2d_ecs::InstanceOf>(root)
        .map(|l| l.master);
    let master_name = root_master
        .and_then(|id| master_named(sim, id))
        .unwrap_or_else(|| "component".to_string());

    // ⚠️ **Só as chaves DESTA peça** — ver o doc do modelo: o conjunto mora na raiz e chaveia por
    // `(peça, tipo)`, então mostrar tudo diria ao artista que ele mexeu noutro sítio da cópia.
    let mut overridden: Vec<String> = inst
        .overrides
        .iter()
        .filter(|k| k.piece == link.master)
        .filter_map(|k| registry.get_by_id(k.type_id))
        .map(|e| {
            e.desc
                .map_or(e.canonical_name, |d| d.display_name)
                .to_string()
        })
        .collect();
    // Ordenado por NOME: o `type_id` é um hash, e uma lista que reordena sozinha entre sessões é
    // uma lista que ninguém consegue ler duas vezes.
    overridden.sort();

    Some(InspectorInstanceInfo {
        entity_bits: entity.to_bits(),
        master_name,
        overridden,
        orphans: inst.orphans.len(),
        root_bits: root.to_bits(),
        // ⚠️ Da RAIZ: uma peça dentro de uma variante não é ela própria uma receita, mas pertence
        // a uma — e é isso que o artista precisa de ler antes de a editar.
        is_variant: sim.world().get::<ph2d_ecs::MasterRoot>(root).is_some(),
    })
}

/// ⭐⭐ **A família de `current`** — todo mestre vivo com que a troca tem um mapa determinístico.
///
/// ⚠️ **O critério é o MAPA, e não uma marca**: um mestre entra aqui exactamente quando
/// [`crate::instance_variant::piece_map`] o alcança, que é a mesma pergunta que a troca faz. *Duas
/// respostas a «isto é uma variante disto?» divergem no dia em que uma delas for escrita sozinha* —
/// e o sintoma seria um chip que o artista clica e que recusa.
///
/// ⚠️ **Ela devolve a ESTRUTURA, e não as fileiras**: quem decide se um membro vira chip, texto ou
/// nada é a lei (`variant_axes`), que é pura e se testa sem um mundo.
pub(crate) fn family_members(sim: &mut SimWorld, current: u64) -> Vec<(u64, String)> {
    // Ordenado por `StableId` — a ordem de autoria, e a única que é a mesma em toda máquina.
    let masters: Vec<u64> = {
        let mut q = sim
            .world_mut()
            .query_filtered::<&ph2d_ecs::StableId, bevy_ecs::prelude::With<ph2d_ecs::MasterRoot>>();
        let mut v: Vec<u64> = q.iter(sim.world()).map(|s| s.0).collect();
        v.sort_unstable();
        v
    };
    let mut members: Vec<(u64, String)> = Vec::new();
    for id in masters {
        if id != current && crate::instance_variant::piece_map(sim, current, id).is_none() {
            continue;
        }
        members.push((
            id,
            master_named(sim, id).unwrap_or_else(|| "component".to_string()),
        ));
    }
    // ⭐⭐ **A ESTRUTURA sai daqui e a LEI sai de lá.** O shell responde *«quem é da família»* (elos
    // no mundo) e o `variant_axes` responde *«que perguntas ela faz»* (só nomes). ⚠️ Separá-las é o
    // que torna a lei testável sem um mundo — e é o que a deixa sobreviver ao apagar do sistema
    // vetorial, de onde ela veio.
    members
}

/// O `Name` da entidade cujo `StableId` é `id`.
pub(super) fn master_named(sim: &mut SimWorld, id: u64) -> Option<String> {
    let mut q = sim
        .world_mut()
        .query::<(&ph2d_ecs::StableId, &ph2d_ecs::Name)>();
    q.iter(sim.world())
        .find(|(s, _)| s.0 == id)
        .map(|(_, n)| n.0.clone())
}

/// ⭐⭐ **Limpar as excepções sem alvo** — o gesto que o botão dispara.
///
/// ⚠️ **Ele só toca nos ÓRFÃOS.** Uma excepção com alvo é o que o artista está a ver e a usar; um
/// botão que apagasse as duas seria *Revert to Master* com outro nome, e ninguém o adivinharia.
pub(super) fn clear_orphans(sim: &mut SimWorld, root_bits: u64) -> usize {
    let root = Entity::from_bits(root_bits);
    let Some(mut inst) = sim.world().get::<ph2d_ecs::ObjectInstance>(root).cloned() else {
        return 0;
    };
    let n = inst.orphans.len();
    if n > 0 {
        inst.orphans.clear();
        sim.world_mut().entity_mut(root).insert(inst);
    }
    n
}

#[cfg(test)]
#[path = "inspector_instance_tests.rs"]
mod tests;
