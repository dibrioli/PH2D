//! ⭐ **DEVOLVER à receita** — o verbo *Revert to Master* (ADR-0164 / F4.4).
//!
//! ⚠️ **Irmão de [`super::instance_sync`] por ASSUNTO** (e pelo tecto de 600 LOC da shell): lá mora
//! o passe que PROPAGA; aqui, o verbo que desfaz uma excepção. Os dois falam a mesma
//! [`ph2d_ecs::OverrideKey`], e é isso que os mantém a concordar.

use ph2d_ecs::{Entity, InstanceOf, MasterRoot, ObjectInstance, SimWorld, StableId};
use std::collections::BTreeMap;

use crate::instance_sync::{MasterEcho, TRANSFORM};

/// ⭐ **DEVOLVER a peça ao mestre** — o inverso do override (ADR-0164 / F4.4).
///
/// Tira a chave do conjunto; o passe seguinte volta a propagar o valor da receita. Devolve `true`
/// se havia o que devolver.
///
/// ⚠️ **Ele não escreve o valor do mestre aqui** — quem propaga é o sync, e escrever também neste
/// sítio seria a segunda porta que discorda da primeira no dia em que a regra do `pose_owner`
/// mudar. *Um verbo, um efeito.*
///
/// ⚠️⚠️ **Mas tirar a chave NÃO chega, e foi um gate que o disse:** no passe seguinte a peça ainda
/// difere do mestre e o mestre não mexeu — que é exatamente a assinatura de *«a instância
/// mexeu-se»*. O override renascia no quadro a seguir ao revert, e o verbo era um **no-op
/// visível**.
///
/// ⇒ o revert também **apaga o eco daquela chave**. Sem memória, o passe cai na regra do 1.º
/// encontro — *o mestre ganha* — que já estava escrita e justificada. *A saída não precisou de uma
/// regra nova: precisou de esquecer.*
pub(crate) fn revert_override(
    sim: &mut SimWorld,
    echo: &mut MasterEcho,
    instance_root: Entity,
    key: ph2d_ecs::OverrideKey,
) -> bool {
    let Some(mut ov) = sim.world().get::<ObjectInstance>(instance_root).cloned() else {
        return false;
    };
    if !ov.overrides.remove(&key) {
        return false;
    }
    echo.master.remove(&(key.piece, key.type_id));
    sim.world_mut().entity_mut(instance_root).insert(ov);
    true
}

/// ⭐ **Devolve à receita o que a MÃO do artista aponta** — o verbo que o menu da Hierarquia chama.
///
/// `None` quando a entidade não pertence a instância nenhuma; `Some(n)` com quantas excepções
/// foram apagadas.
///
/// # ⚠️ Ele aceita QUALQUER peça, e não só a raiz — o report do Enio (2026-08-26)
///
/// A 1.ª versão exigia a raiz e respondia *«Not an instance»* na peça. Estava tecnicamente certa e
/// **inutilmente** certa: para pintar o braço de uma cópia o artista tem de selecionar a linha do
/// **braço**, e é lá que a mão dele está quando ele quer desfazer. *Um aviso que diz o que a coisa
/// NÃO é, sem dizer o que fazer, é um botão mudo com legenda.*
///
/// # O ESCOPO é o que se clicou
///
/// - numa **peça** ⇒ só as excepções daquela peça;
/// - na **raiz** ⇒ todas as da instância.
///
/// ⚠️ *Devolver o rig inteiro porque o artista pediu um braço seria apagar trabalho que ele não
/// mandou apagar* — e o Ctrl+Z existe, mas um verbo que faz mais do que diz não se desculpa com ele.
///
/// # ⛔⛔ Ele NÃO devolve a POSE — decisão do Enio (2026-08-26)
///
/// > *«Revert to master modifica a posição global do objeto e isso não é uma boa idéia. Melhor o
/// > objeto ficar onde está.»*
///
/// Medido antes de decidir: arrastar uma peça de uma instância captura um override de `Transform`,
/// e o revert punha-a de volta na pose da receita — a peça **teletransportava-se** ao clicar num
/// item de menu que fala de *conteúdo*. ⚠️ **A pose de uma peça continua a ser um override** (senão
/// o passe seguinte reescrevia por cima do arrasto do artista, que é pior); o que muda é que este
/// verbo **não lhe toca**.
///
/// ⚠️ **É a mesma lei que a raiz já tinha** ([`ROOT_IS_ITS_OWN`]) descida um nível pelo report:
/// *onde uma coisa está é do artista que a largou lá*. A receita continua a poder mandar — quando o
/// MESTRE mexe a peça dele, o empate resolve-se a favor dele e a instância segue.
///
/// ⇒ o resultado tem **dois** números: o que foi devolvido e a pose que ficou. *Um `0` de «não
/// havia nada» e um `0` de «só havia a posição, e ela fica» são respostas diferentes ao artista.*
pub(crate) fn revert_all_overrides(
    sim: &mut SimWorld,
    echo: &mut MasterEcho,
    clicked: Entity,
) -> Option<Reverted> {
    let pose = ph2d_ecs::scene::stable_type_id(TRANSFORM);
    let (root, scope) = instance_root_of(sim, clicked)?;
    let keys: Vec<ph2d_ecs::OverrideKey> = sim
        .world()
        .get::<ObjectInstance>(root)
        .map(|o| {
            o.overrides
                .iter()
                .copied()
                .filter(|k| scope.is_none_or(|piece| k.piece == piece))
                .collect()
        })
        .unwrap_or_default();
    let mut out = Reverted::default();
    for key in keys {
        if key.type_id == pose {
            out.poses_kept += 1;
            continue;
        }
        if revert_override(sim, echo, root, key) {
            out.count += 1;
        }
    }
    Some(out)
}

/// **O que o *Revert* fez** — e o que ele deliberadamente não fez.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Reverted {
    /// Excepções devolvidas à receita.
    pub(crate) count: usize,
    /// Poses que ficaram como estavam — ver o doc de [`revert_all_overrides`].
    pub(crate) poses_kept: usize,
}

/// **A raiz da instância a que esta entidade pertence, e o escopo do gesto.**
///
/// Devolve `(raiz, Some(peça do mestre))` quando se clicou numa PEÇA, e `(raiz, None)` quando se
/// clicou na própria raiz — `None` aí significa *«sem filtro»*, ou seja a instância inteira.
///
/// ⚠️ **Sobe por `ChildOf`, e não pelo elo:** o `InstanceOf` de uma peça aponta para a peça do
/// MESTRE, não para a raiz da instância. Subir por ele saía da instância e ia parar à receita.
fn instance_root_of(sim: &mut SimWorld, clicked: Entity) -> Option<(Entity, Option<u64>)> {
    let by_id: BTreeMap<u64, Entity> = {
        let mut q = sim.world_mut().query::<(Entity, &StableId)>();
        q.iter(sim.world()).map(|(e, s)| (s.0, e)).collect()
    };
    let is_root = |e: Entity, w: &ph2d_ecs::World| {
        w.get::<InstanceOf>(e)
            .and_then(|l| by_id.get(&l.master))
            .is_some_and(|&m| w.get::<MasterRoot>(m).is_some())
    };
    let clicked_piece = sim.world().get::<InstanceOf>(clicked).map(|l| l.master)?;
    let mut e = clicked;
    loop {
        if is_root(e, sim.world()) {
            let scope = (e != clicked).then_some(clicked_piece);
            return Some((e, scope));
        }
        e = sim.world().get::<ph2d_ecs::ChildOf>(e)?.0;
    }
}

/// ⭐ **O dreno do gesto *Revert to Master*** — resolve a entidade e responde ao artista.
///
/// Devolve `true` quando alguma coisa mudou.
///
/// ⚠️ **Mora aqui e não no dreno da Hierarquia** porque é sobre INSTÂNCIAS, e não sobre a mecânica
/// das linhas — e porque aquele ficheiro estava no teto de 600 LOC. *O corte é por assunto.*
///
/// ⚠️ **Ele responde mesmo quando não se aplica.** A tabela daquele menu é PLANA (ela não sabe o
/// que a linha é), então o item aparece em toda linha; um item que come o clique em silêncio é
/// pior que um ausente.
pub(crate) fn drain_revert_to_master(
    sim: &mut SimWorld,
    echo: &mut MasterEcho,
    entity_bits: u64,
    toasts: &mut ph2d_editor::ToastQueue,
) -> bool {
    use ph2d_editor::Toast;
    match revert_all_overrides(sim, echo, Entity::from_bits(entity_bits)) {
        // ⚠️ *«Não pertence a instância nenhuma»* e *«pertence, e não havia excepção»* são coisas
        // diferentes, e a segunda não é um erro: o artista clicou no sítio certo.
        None => {
            toasts.push(Toast::warning(
                "Not part of an instance — nothing to revert",
            ));
            false
        }
        // ⚠️ **Quatro respostas, e a pose é a razão de serem quatro**: dizer *«nada estava
        // sobrescrito»* a quem acabou de mover a peça seria mentir sobre o que o app sabe.
        Some(r) if r.count == 0 && r.poses_kept == 0 => {
            toasts.push(Toast::info("Nothing overridden here"));
            false
        }
        Some(r) if r.count == 0 => {
            toasts.push(Toast::info(
                "Only the position differs — it stays where you put it",
            ));
            false
        }
        Some(r) if r.poses_kept > 0 => {
            toasts.push(Toast::success(format!(
                "Reverted {} change(s) — position kept",
                r.count
            )));
            true
        }
        Some(r) => {
            toasts.push(Toast::success(format!(
                "Reverted {} override(s) to master",
                r.count
            )));
            true
        }
    }
}
