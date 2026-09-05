//! ⭐ **A CÓPIA PROFUNDA** — a peça mecânica de que *Duplicar* e *Instanciar* nascem, e o elo
//! [`InstanceOf`] que liga uma instância ao mestre (ADR-0164 / plano F4.2).
//!
//! # Porque ela não é o `spawn` de uma lista de componentes
//!
//! Copiar uma subárvore é copiar **bytes de componente que ninguém desta crate conhece** — a
//! física, o vetor, o 3D e os scripts registam os deles. A porta que existe para isso é a vtable
//! do [`ComponentRegistry`]: [`extract_component_snapshot`] serializa o que a entidade tem, e
//! `insert_from_bytes` põe o mesmo no destino. A auditoria de 2026-08-21 mediu que os dois
//! existiam e **não tinham consumidor nenhum**; este módulo é o primeiro.
//!
//! # ⚠️ O que a cópia NÃO leva, e porquê
//!
//! - **A identidade.** O [`crate::StableId`] não é registado ([`crate::scene::registry`]) — a
//!   ausência é a decisão, e é ela que faz a cópia nascer **sem** id em vez de nascer com o id
//!   do original. Quem lho dá é o [`crate::assign_missing_stable_ids`], no fim.
//! - **A ORDEM.** `RootOrder`/`SiblingOrder` são registados e viriam verbatim — dois irmãos com
//!   a mesma ordem é o empate que a casa não tem (*"não se escolhe um desempate melhor, não se
//!   tem empate"*). A raiz da cópia perde-os e ganha os seus.
//!
//! # ⛔ Esta função sozinha NÃO instancia
//!
//! Ela copia bytes. Uma referência guardada por identidade (`PhysicsJoint.body_a`, a corda de
//! uma `PulleyWheel`) continua a apontar para o **ORIGINAL** — e é assim que uma junta copiada
//! prende os corpos do mestre. O remap é o segundo passo, e a porta do produto que compõe os
//! dois é `instantiate::instantiate_master` na shell, com um censo a ligar cada campo declarado
//! `RefKind::Object` ao remapeador dele. *Duas portas em que uma tem de seguir a outra são uma
//! porta e uma armadilha* — por isso há gate a exigir que esta tenha um chamador só.
//!
//! [`extract_component_snapshot`]: crate::scene::extract_component_snapshot
//! [`ComponentRegistry`]: crate::scene::ComponentRegistry

use crate::scene::{
    ComponentRegistry, ComponentSnapshot, RegistryError, extract_component_snapshot,
};
use crate::{ChildOf, Children, Entity, RootOrder, SiblingOrder, StableId};
use bevy_ecs::prelude::Component;
use bevy_ecs::world::World;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// **De que mestre esta raiz é uma instância** — o [`StableId`] dele.
///
/// ⚠️ Registado e persistido: o elo é autoria, e sem ele um projeto reaberto tem instâncias que
/// já não sabem de onde vieram (o sync da F4.3 deixaria de as alcançar, calado).
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceOf {
    /// `0` = nenhum, a convenção do [`StableId::NONE`].
    pub master: u64,
}

/// **A chave de um OVERRIDE** — *"nesta peça, este componente é da instância"* (ADR-0164 / F4.4).
///
/// # ⚠️ Não há `field_id`, e a ausência é MEDIDA
///
/// O plano pedia a chave `(peça, type_id, field_id)`, e a
/// [refutação 3](https://github.com/dibrioli/PH2D/blob/main/docs/Components/pesquisa/instancias_2026-08-21/refutacao_3_override_aninhado.md)
/// já tinha achado o furo: *«`(type_id, field)` não significa nada hoje para os 91 tipos — a vtable
/// do registo é **blob inteiro** postcard. Sem `patch_field` por tipo, "campo tocado bloqueia
/// propagação" vira "**componente** tocado bloqueia propagação"»*.
///
/// ⇒ a granularidade é o COMPONENTE, e a consequência tem de ser dita: mexer na posição de uma
/// peça da instância congela também a **escala** e a **rotação** dela contra o mestre, porque as
/// três vivem no mesmo `Transform`. Comprar o campo custa um acessor tipado por componente — os
/// mesmos 107 sítios que a F0 mediu e recusou.
///
/// # A `peça` é a do MESTRE
///
/// É o `StableId` da peça de que a da instância nasceu. Renomear, reordenar e reparentear **dentro**
/// do mestre não tocam a chave, porque o id é opaco — a propriedade que o `VecInstance.sub` já
/// paga no vetor.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OverrideKey {
    /// `StableId` da peça **do mestre**.
    pub piece: u64,
    /// O `ComponentTypeId` do registo.
    pub type_id: u64,
}

/// **Os overrides de uma instância**, guardados na RAIZ dela (ADR-0164 / F4.4).
///
/// # ⚠️ É um CONJUNTO, e não um mapa de bytes — e isto refuta o plano
///
/// O plano pedia `BTreeMap<OverrideKey, Bytes>`, copiando o modelo do Unity. Lá os bytes são
/// obrigatórios porque **a instância não é uma entidade real** até ser instanciada: a lista de
/// modificações é a única cópia do valor.
///
/// ⭐ **Aqui a instância É uma entidade real** — é a tese do próprio ADR-0164. O valor já vive no
/// componente da peça, viaja no ficheiro e no undo pela porta de sempre, e guardá-lo outra vez
/// criaria **duas fontes para o mesmo número**, que discordam no dia em que uma delas for escrita
/// sozinha. *A representação apaga o caso especial.*
///
/// ⇒ o que falta guardar é só a PERGUNTA *«este componente é da instância?»*, e isso é um conjunto.
///
/// ⚠️ **Incluindo a AUSÊNCIA:** um componente que o artista tirou da instância é um override —
/// a chave diz *«este componente é assunto da instância»*, e não *«a instância tem outro valor»*.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectInstance {
    /// Ordenado por construção (HR-5): a serialização tem de ser determinística.
    pub overrides: std::collections::BTreeSet<OverrideKey>,
    /// ⭐⭐⭐ **Os overrides SEM ALVO** — a excepção de uma peça que o mestre já não tem
    /// (ADR-0164 / F5.3).
    ///
    /// # ⚠️ Aqui os bytes são obrigatórios, e isso NÃO contradiz a refutação acima
    ///
    /// A refutação diz que guardar bytes cria **duas fontes para o mesmo número** — e ela vale
    /// enquanto a peça existe, porque aí o valor vive no componente dela. Uma peça órfã **não
    /// existe**: o mestre apagou-a, e a F5.1 tira-a da instância a seguir. ⇒ não há segunda fonte,
    /// há a ÚNICA. *A premissa da refutação era «a peça é uma entidade real», e a F5.1 tornou-a
    /// destruível — quem move o número que tornava algo inalcançável tem de reconferir a nota.*
    ///
    /// # O que isto compra, medido
    ///
    /// Sem esta metade, apagar uma peça no mestre e **desfazer** devolvia a peça com o valor do
    /// MESTRE e a chave de override intacta ⇒ a cópia ficava com a excepção perdida **e surda à
    /// receita para sempre** (o passe salta o que a instância possui). Medido por sonda em
    /// 2026-08-27: `tint da copia = [1,1,1,1]`, quando a excepção era `[0.9,…]`.
    ///
    /// ⛔ **Nunca se apagam sozinhos** — é a lei do *«unused overrides»* do Unity, e a razão é que
    /// apagar a excepção do artista por causa de um `Delete` no mestre é perder trabalho em
    /// silêncio. Sai por gesto, ou quando a peça volta e ela é reposta.
    pub orphans: std::collections::BTreeMap<OverrideKey, OrphanOverride>,
}

/// ⭐⭐⭐ **UMA EXCEPÇÃO SEM ALVO** — o que ficou de uma peça que o mestre apagou (ADR-0164 / F5.3).
///
/// # ⚠️ O NOME viaja junto com os bytes, e pela MESMA razão
///
/// A [refutação da F4.4] diz que guardar um valor cria **duas fontes** para o mesmo facto — e ela
/// vale **enquanto a peça existe**. Uma peça órfã **não existe**: o mestre apagou-a e a F5.1
/// tira-a da cópia a seguir. ⇒ não há segunda fonte, há a **única**. Foi esse argumento que já
/// justificou guardar os `bytes`; o nome cai exactamente na mesma categoria.
///
/// ⚠️ **A janela em que ele se lê é ESTREITA:** quem o grava é o `entomb`, no instante em que a
/// peça da instância ainda está viva (o `despawn` vem a seguir) — e o `Name` dela é o **mesmo do
/// mestre**, porque o passe propaga-o e só a RAIZ é dona do dela. Um passe depois não há onde o ir
/// buscar, e o painel fica a poder dizer *«há três»* sem nunca poder dizer *«quais três»*.
///
/// ⛔ **Ele é para MOSTRAR, nunca para procurar.** A chave de re-encontro continua a ser o
/// `StableId` da peça (é ele que sobrevive ao respawn do undo); um nome usado como endereço
/// reabria a doença que o `Name` já custou seis reports noutro subsistema.
///
/// [refutação da F4.4]: https://github.com/dibrioli/PH2D/blob/main/docs/Components/pesquisa/instancias_2026-08-21/refutacao_3_override_aninhado.md
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrphanOverride {
    /// Os bytes que a peça tinha. ⚠️ **Vazio significa AUSÊNCIA do componente** — o artista tinha
    /// tirado o componente da cópia, e isso também é uma excepção.
    pub bytes: Vec<u8>,
    /// O `Name` que a peça tinha quando morreu — **display only**, ver acima.
    pub piece_name: String,
}

/// ⭐⭐⭐ **Esta peça DIVIDE a arte do mestre** — o *Duplicate Linked* do Blender (Enio, 2026-08-27).
///
/// # As duas leis, e porque são a MESMA
///
/// Sem esta marca, uma cópia tem arte **própria**: editar o desenho dela vira uma excepção dela, e
/// as irmãs não mudam. Com ela, a edição **sobe ao mestre** — e o passe seguinte leva-a a todas.
/// É o `Alt+D`: uma malha, vários objetos.
///
/// ⚠️ **«Arte» são os PIXELS e os DOCUMENTOS POSSUÍDOS, nunca as propriedades.** Uma cópia ligada
/// continua a ter a pose, o `tint` e os componentes dela — senão ela não era uma cópia, era o mesmo
/// objeto desenhado duas vezes. É exactamente a fronteira do Blender: partilha-se o *dado*, não o
/// *objeto*.
///
/// # ⚠️ Porque é per-PEÇA e não na raiz
///
/// Os dois consumidores — a subida da tinta (`hero_intents::texture_rebind`) e a do documento
/// (`instance_sync_docs`) — têm em mão a **peça** que o artista tocou, nunca a raiz. Uma marca na
/// raiz obrigaria os dois a subir a árvore para a encontrar, e *cada um teria a sua resposta*. É a
/// mesma razão pela qual o [`InstanceOf`] vive em toda peça.
///
/// ⛔ **Não propaga** (`instance_sync::NEVER_PROPAGATES`): o mestre não a tem, e um passe que
/// propagasse a ausência arrancá-la-ia da cópia todo o quadro.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkedArt;

/// O que uma cópia profunda produziu — e o mapa que o remap consome.
#[derive(Clone, Debug)]
pub struct DeepCopy {
    /// A raiz da cópia.
    pub root: Entity,
    /// Entidade do original → entidade da cópia, em ordem de visita.
    pub entities: BTreeMap<Entity, Entity>,
    /// `StableId` do original → `StableId` da cópia. **É esta a chave do remap**, e não os bits
    /// de entidade: as referências guardam identidade.
    pub stable_ids: BTreeMap<u64, u64>,
}

impl DeepCopy {
    /// As entidades novas, na ordem em que nasceram — o que o remap varre.
    #[must_use]
    pub fn copies(&self) -> Vec<Entity> {
        self.entities.values().copied().collect()
    }
}

/// ⭐ **Copia `src_root` e toda a descendência dele**, devolvendo o mapa de identidade.
///
/// `parent` diz onde a cópia aterra: `Some(p)` para a pendurar (o *Duplicar*, que a põe ao lado
/// do original), `None` para a deixar na raiz da cena.
///
/// ⚠️ **Ordem determinística** (pré-ordem, filhos na ordem do `Children`) — a mesma cópia tem de
/// dar os mesmos ids em qualquer máquina, senão o `physics_ecs_c9` diverge entre os 3 OS.
///
/// ⛔ Ver o cabeçalho do módulo: **isto não remapeia referência nenhuma.**
pub fn deep_copy_subtree(
    world: &mut World,
    registry: &ComponentRegistry,
    src_root: Entity,
    parent: Option<Entity>,
) -> Result<DeepCopy, RegistryError> {
    if world.get_entity(src_root).is_err() {
        return Err(RegistryError::EntityMissing(src_root));
    }
    // O original tem de ter identidade ANTES, senão o mapa nasce sem chaves.
    crate::assign_missing_stable_ids(world);

    // 1. A subárvore, em pré-ordem.
    let mut order: Vec<Entity> = Vec::new();
    let mut stack = vec![src_root];
    while let Some(e) = stack.pop() {
        order.push(e);
        if let Some(kids) = world.get::<Children>(e) {
            let mut ks: Vec<Entity> = kids.iter().copied().collect();
            // Empilhar ao contrário é o que faz a visita seguir a ordem do `Children`.
            ks.reverse();
            stack.extend(ks);
        }
    }

    // 2. Os blobs, ANTES de tocar no mundo — a extração pede `&World` e a inserção `&mut`.
    let mut snap = ComponentSnapshot::new();
    let mut blobs: Vec<Vec<(u64, Vec<u8>)>> = Vec::with_capacity(order.len());
    for &e in &order {
        extract_component_snapshot(world, e, registry, &mut snap)?;
        blobs.push(
            snap.entries
                .iter()
                .map(|c| (c.type_id, c.data.clone()))
                .collect(),
        );
    }

    // 3. As entidades novas.
    let mut entities: BTreeMap<Entity, Entity> = BTreeMap::new();
    for (i, &src) in order.iter().enumerate() {
        let dst = world.spawn_empty().id();
        for (type_id, data) in &blobs[i] {
            // ⚠️ Um id sem entrada no registo é um componente que o snapshot produziu e este
            // registo não conhece — impossível hoje (é o mesmo registo), e saltá-lo é a
            // resposta certa se algum dia deixar de ser: perder um componente é melhor que
            // abortar a cópia a meio, com metade das entidades nascidas.
            let Some(entry) = registry.get_by_id(*type_id) else {
                continue;
            };
            // ⭐⭐ **A PONTE para um documento POSSUÍDO fica de fora** — ver
            // [`ph2d_component_desc::ComponentDesc::owned_document`]. O id é opaco: copiá-lo daria
            // duas entidades a escrever no MESMO documento (duplicar uma sprite pintada devolvia um
            // sósia que apaga a tinta do original). A cópia nasce sem o elo, que é exatamente o que
            // a cópia rasa fazia — e ensinar cada documento a copiar-se é outra obra.
            //
            // ⚠️ **Quem decide é o DESCRITOR, não uma lista aqui.** Uma ponte nova declarada no
            // catálogo passa a ser saltada sem que ninguém se lembre deste ficheiro; uma lista de
            // nomes local seria a segunda resposta que envelhece.
            if entry.desc.is_some_and(|d| d.owned_document) {
                continue;
            }
            let insert = entry.insert_from_bytes;
            insert(world, dst, data)?;
        }
        entities.insert(src, dst);
    }

    // 4. A hierarquia — interna, e depois onde a raiz aterra.
    for &src in &order {
        if src == src_root {
            continue;
        }
        let Some(&dst) = entities.get(&src) else {
            continue;
        };
        let src_parent = world.get::<ChildOf>(src).map(|c| c.0);
        if let Some(p) = src_parent
            && let Some(&new_parent) = entities.get(&p)
        {
            world.entity_mut(dst).insert(ChildOf(new_parent));
        }
    }
    let root = entities[&src_root];
    // A raiz não herda a ordem do original (ver o cabeçalho).
    world.entity_mut(root).remove::<RootOrder>();
    world.entity_mut(root).remove::<SiblingOrder>();
    if let Some(p) = parent {
        world.entity_mut(root).insert(ChildOf(p));
    }

    // 5. Identidade nova, e o mapa que o remap consome.
    crate::assign_missing_stable_ids(world);
    let mut stable_ids: BTreeMap<u64, u64> = BTreeMap::new();
    for (&src, &dst) in &entities {
        if let (Some(a), Some(b)) = (world.get::<StableId>(src), world.get::<StableId>(dst)) {
            stable_ids.insert(a.0, b.0);
        }
    }

    Ok(DeepCopy {
        root,
        entities,
        stable_ids,
    })
}

/// **Reescreve o elo [`InstanceOf`] das entidades dadas** através do mapa de identidade.
///
/// O remapeador de `ph2d::ecs::InstanceOf` — a entrada dele na tabela da shell. Devolve quantos
/// mexeu.
///
/// ⚠️ **Um mestre que está FORA do que se copiou não está no mapa, e o elo fica** — que é o caso
/// normal (duplicar uma instância dá outra instância do mesmo mestre) e é o comportamento certo.
pub fn remap_instance_of(
    world: &mut World,
    entities: &[Entity],
    by_id: &BTreeMap<u64, u64>,
) -> usize {
    let mut hits = 0;
    for &e in entities {
        let Some(mut link) = world.get_mut::<InstanceOf>(e) else {
            continue;
        };
        if let Some(&new) = by_id.get(&link.master) {
            link.master = new;
            hits += 1;
        }
    }
    hits
}

#[cfg(test)]
#[path = "instantiate_tests.rs"]
mod tests;
