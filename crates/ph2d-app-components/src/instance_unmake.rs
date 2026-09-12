//! ⭐⭐ **TIRAR DA BIBLIOTECA** — o verbo que faltava.
//!
//! Report do Enio (2026-08-30, ao fim do smoke da etapa B): *«não dá para tirar um asset da
//! biblioteca. Ele entra e fica.»* Era verdade — o `Verb::Make` marcava e **nada** desmarcava.
//!
//! # ⭐⭐⭐ A pergunta que este verbo tem de responder, e que o nome esconde
//!
//! *«Tirar da biblioteca»* soa a apagar uma linha de uma lista. Não é: a receita **é** a única
//! cópia daquela sub-árvore que não está a ser desenhada, e as instâncias dela são **elos** que
//! deixam de resolver no instante em que ela deixa de ser receita. ⇒ o verbo tem de dizer, de uma
//! vez, o que acontece aos DOIS lados.
//!
//! | Havia cópias na cena? | O que acontece | Por quê |
//! |---|---|---|
//! | **sim** | cada cópia **destaca-se** (fica um objecto independente, com o aspecto que já tinha) e a receita — que ninguém vê — é **apagada** | os dados sobrevivem nas cópias; deixar a receita seria lixo invisível e inalcançável |
//! | **não** | a receita **VOLTA à cena** como objecto normal, e nada é apagado | ⛔ ela é a **última cópia**: apagá-la seria destruir o trabalho do artista para cumprir um verbo que ele leu como *«tirar da lista»* |
//!
//! ⚠️ **A assimetria é a lei, não um caso especial esquecido.** *Um verbo de arrumação nunca
//! destrói a última cópia de nada* — e é por isso que a resposta se conta (`instances_of`) em vez
//! de se assumir. As duas metades têm gate.
//!
//! # ⚠️ A ordem é load-bearing
//!
//! Os elos recolhem-se **antes** de a receita deixar de o ser: o
//! [`crate::instance_verbs::instance_root_of`] resolve a raiz de uma instância perguntando se o
//! mestre dela é um [`MasterRoot`], e depois do `remove::<MasterRoot>` (ou do `despawn`) essa
//! pergunta responde **não** para toda a gente. Invertida, a função ficaria correcta no
//! compilador e **destacaria zero** instâncias, deixando-as elos mortos — que é exactamente a
//! forma de defeito que a lente 1 da auditoria procura.
//!
//! # A cerca
//!
//! ⛔ Uma **Imagem** não passa por aqui. Ela está na biblioteca porque um objecto a usa
//! ([`crate::asset_index_build`]) — *«tirar»* uma textura significaria tirá-la dos objectos, que é
//! outro gesto com outro sujeito. A recusa é do lado de quem chama, e ela **nomeia os donos**.

use ph2d_ecs::{Entity, InstanceOf, MasterRoot, SimWorld, StableId};

/// O que aconteceu à receita.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Unmade {
    /// Havia cópias: elas ficaram independentes e a receita (invisível) foi apagada.
    Dissolved {
        /// Quantas cópias se destacaram — é o número que a voz diz.
        copies: usize,
    },
    /// ⭐ Não havia cópia nenhuma: a receita **voltou à cena** em vez de ser destruída.
    Returned {
        /// A entidade que voltou — o que a selecção passa a apontar, porque ela **apareceu** e o
        /// artista tem de saber onde.
        root_bits: u64,
    },
}

/// Por que o verbo não correu.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum UnmakeRefusal {
    /// A entidade não é uma receita, nem uma instância de nenhuma.
    NotInTheLibrary,
}

/// **A raiz da receita a que este clique se refere** — e ela aceita os DOIS sujeitos.
///
/// ⚠️ O menu da Hierarquia é uma tabela **plana** (ela não sabe o que a linha é), então este verbo
/// aparece também sobre uma **instância**. Resolver só o caso `MasterRoot` faria o item comer o
/// clique em silêncio em toda linha de instância — que é a doença que o `drain` declara curar.
fn recipe_root_of(sim: &mut SimWorld, clicked: Entity) -> Option<Entity> {
    if let Some(root) = ph2d_ecs::master_root_of(sim.world(), clicked) {
        return Some(root);
    }
    // Uma instância: a raiz da receita é o que o elo dela nomeia.
    let inst_root = crate::instance_verbs::instance_root_of(sim, clicked)?;
    let master_id = sim.world().get::<InstanceOf>(inst_root)?.master;
    crate::instance_verbs::entity_for_stable_id(sim, master_id).map(Entity::from_bits)
}

/// ⭐ **Tirar da biblioteca.** Ver o cabeçalho do módulo para a lei das duas metades.
pub(crate) fn unmake_master(sim: &mut SimWorld, clicked: Entity) -> Result<Unmade, UnmakeRefusal> {
    let root = recipe_root_of(sim, clicked).ok_or(UnmakeRefusal::NotInTheLibrary)?;
    if sim.world().get::<MasterRoot>(root).is_none() {
        return Err(UnmakeRefusal::NotInTheLibrary);
    }
    let master_id = sim
        .world()
        .get::<StableId>(root)
        .ok_or(UnmakeRefusal::NotInTheLibrary)?
        .0;

    // ⚠️ **Antes de tudo** — ver «a ordem é load-bearing» no cabeçalho.
    //
    // Só a RAIZ de uma instância aponta para o `master_id` da receita: as peças de dentro apontam
    // para a peça correspondente do mestre, que não é `MasterRoot`. E os `StableId` são únicos, o
    // que torna esta contagem exacta sem subir árvore nenhuma.
    let mut roots: Vec<(u64, Entity)> = {
        let mut q = sim.world_mut().query::<(Entity, &InstanceOf, &StableId)>();
        q.iter(sim.world())
            .filter(|(_, link, _)| link.master == master_id)
            .map(|(e, _, sid)| (sid.0, e))
            .collect()
    };
    // HR-5: a ordem da consulta é de arquétipo, e um verbo que a herdasse deixaria de ser
    // determinístico entre corridas.
    roots.sort_unstable();

    for (_, r) in &roots {
        // O `detach` é a porta que já existe — ⛔ escrever aqui um segundo laço a tirar
        // `InstanceOf` seria a lei em dois sítios.
        let _ = crate::instance_verbs::detach(sim, *r);
    }

    if roots.is_empty() {
        sim.world_mut().entity_mut(root).remove::<MasterRoot>();
        Ok(Unmade::Returned {
            root_bits: root.to_bits(),
        })
    } else {
        // `despawn` leva os descendentes — a mesma convenção do `HierDelete`.
        sim.world_mut().despawn(root);
        Ok(Unmade::Dissolved {
            copies: roots.len(),
        })
    }
}

#[cfg(test)]
#[path = "instance_unmake_tests.rs"]
mod tests;
