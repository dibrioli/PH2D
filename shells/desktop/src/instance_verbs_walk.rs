//! ⭐ **AS TRAVESSIAS que os verbos partilham** — irmão por ASSUNTO do [`super::instance_verbs`].
//!
//! Elas não são um verbo: são as quatro perguntas que TODO verbo faz antes de agir — *quem tem
//! este `StableId`?*, *qual é a raiz da instância a que isto pertence?*, *que entidades estão
//! debaixo desta?*. Estavam no fim do ficheiro dos verbos, e saem para cá quando ele bateu no
//! tecto de 600 LOC do shell (HR-18).
//!
//! ⚠️ **O corte é o mesmo que o `action_bus_queue` fez, e pela mesma razão:** o que sai é o bloco
//! do FIM, onde ninguém escreve. O `drain` cresce por acrescento de braço **no meio**, e movê-lo
//! poria toda linha paralela que acrescenta um verbo em conflito textual com esta.

use ph2d_ecs::{Children, Entity, InstanceOf, MasterRoot, SimWorld, StableId};

/// `StableId → entidade`, do mundo inteiro.
pub(crate) fn stable_index(sim: &mut SimWorld) -> std::collections::BTreeMap<u64, Entity> {
    let mut q = sim.world_mut().query::<(Entity, &StableId)>();
    q.iter(sim.world()).map(|(e, s)| (s.0, e)).collect()
}

/// **A entidade que tem este `StableId`**, em bits — a porta do navegador de assets (plano
/// `docs/Components/07`, wave A7).
///
/// ⚠️ **Ela vive AQUI, ao lado do [`stable_index`] que já existia**, e não no shell: uma segunda
/// travessia `StableId → Entity` escrita noutro ficheiro seria a segunda resposta à mesma
/// pergunta, e as duas divergiriam no dia em que a identidade mudasse de forma.
///
/// ⛔ Devolve `None` para um id que já não existe — é o caso normal, não um erro: o navegador
/// publica o índice de um quadro, o artista apaga a receita, e o duplo-clique chega a seguir.
pub(crate) fn entity_for_stable_id(sim: &mut SimWorld, stable_id: u64) -> Option<u64> {
    stable_index(sim).get(&stable_id).map(|e| e.to_bits())
}

/// A raiz da instância a que `clicked` pertence — a peça cujo mestre é um [`MasterRoot`].
///
/// ⚠️ Sobe por `ChildOf`, nunca pelo elo: o `InstanceOf` de uma peça aponta para a peça do MESTRE,
/// e subir por ele sairia da instância e ia parar à receita.
///
/// ⚠️⚠️ **A 1.ª linha era `get::<InstanceOf>(clicked)?;` — um bail que apagava a travessia inteira**
/// (auditoria §1.8, 2026-08-27), e o doc do [`make_master`] prometia o contrário. Toda peça nascida
/// da cópia profunda tem elo, mas o que for acrescentado DEPOIS não tem: um *Add Child* sobre uma
/// peça, um reparent para dentro da cópia, um path vetorial cunhado pelo `vec_entities::sync` e
/// arrastado para lá. ⇒ a recusa `InsideAnInstance` não disparava e nascia um `MasterRoot` **dentro
/// de uma instância viva**, cuja sub-árvore virava `MasterPiece`: um pedaço de uma cópia que estava
/// visível desaparecia, com a cópia à volta a continuar a desenhar.
///
/// ⚠️ O oráculo do gate que sancionava o bail usava uma peça que TINHA elo (`piece(&sim, inst,
/// "Arm")`), então a travessia ancestral nunca corria — *o oráculo confirmava o caminho curto e
/// assinava o longo*.
pub(crate) fn instance_root_of(sim: &mut SimWorld, clicked: Entity) -> Option<Entity> {
    let by_id = stable_index(sim);
    let mut e = clicked;
    loop {
        let is_root = sim
            .world()
            .get::<InstanceOf>(e)
            .and_then(|l| by_id.get(&l.master))
            .is_some_and(|&m| sim.world().get::<MasterRoot>(m).is_some());
        if is_root {
            return Some(e);
        }
        e = sim.world().get::<ph2d_ecs::ChildOf>(e)?.0;
    }
}

/// Esta entidade — ou algum ancestral dela — é peça de uma instância?
pub(crate) fn belongs_to_an_instance(sim: &mut SimWorld, entity: Entity) -> bool {
    instance_root_of(sim, entity).is_some()
}

/// A subárvore de `root`, ela incluída.
pub(crate) fn subtree(sim: &SimWorld, root: Entity) -> Vec<Entity> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        out.push(e);
        if let Some(kids) = sim.world().get::<Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    out
}
