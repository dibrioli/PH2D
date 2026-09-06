//! ⭐⭐⭐ **A FORMA de uma instância segue a do mestre** (ADR-0164 / F5.1) — irmão de
//! [`crate::instance_sync`] por ASSUNTO, e porque aquele ficheiro está perto do tecto de 600 LOC.
//!
//! Lá mora *«que VALOR cada peça tem»*; aqui *«que PEÇAS existem»*. São perguntas diferentes e o
//! passe de valores não podia responder à segunda: ele percorre **pares** (peça do mestre ↔ peça da
//! instância), e uma peça que só existe de um dos lados não forma par nenhum — ela é invisível para
//! aquele laço, por construção.
//!
//! # ⛔ O que estava em falta, medido por sonda em 2026-08-27
//!
//! ```text
//! A = receita vazia · a_inst = instância de A na cena
//! (o artista acrescenta uma peça a A)
//! a_inst tem 0 filho(s) depois do passe   ← esperado: 1
//! ```
//!
//! A tabela do doc 04 §2.6 promete *«adicionar peça → **materializa em todas**»*, e nada o fazia.
//! Para o artista: *«acrescentei uma peça ao componente e as cópias não mudaram»* — a mesma frase
//! que já custou três reports a esta linha, por três mecanismos diferentes.
//!
//! # As DUAS metades, e porque nenhuma pode ir sozinha
//!
//! Acrescentar sem remover deixa uma cópia com peças que a receita já não tem — *um objeto que o
//! artista apagou da biblioteca e que continua na cena*, mudo. É a mesma forma do
//! `assign_master_pieces`, que também tem de marcar **e** desmarcar.
//!
//! ⚠️ **E a fronteira que fica:** um elo cujo MESTRE inteiro desapareceu não é uma peça a mais —
//! é uma instância órfã, e a lei dela já existe (`a_dangling_link_is_left_alone`). Este passe só
//! olha para dentro de instâncias **vivas**: mestre presente, peça ausente.

use ph2d_ecs::{Children, Entity, InstanceOf, MasterRoot, SimWorld, StableId};
use std::collections::{BTreeMap, BTreeSet};

/// Quantas peças o passe criou e quantas apagou — o `0/0` é o estado normal, e é o que faz disto
/// um ponto fixo.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct StructureReport {
    pub(crate) added: usize,
    pub(crate) removed: usize,
    /// Excepções que perderam o alvo neste passe (F5.3).
    pub(crate) orphaned: usize,
    /// Excepções que voltaram a pegar porque a peça voltou.
    pub(crate) restored: usize,
    /// ⭐⭐⭐ **Peças que estavam no pai errado** (F5.12) — a terceira metade da forma.
    pub(crate) moved: usize,
}

/// ⭐⭐ **Põe a forma de cada instância a par com a do mestre dela.**
///
/// ⚠️ **Corre ANTES do passe de valores**, e é isso que dá a promessa de UM quadro: uma peça
/// materializada aqui forma par no laço seguinte e recebe os bytes do mestre no mesmo passe. Ao
/// contrário, ela ficaria com os valores do momento da cópia até alguém tocar no mestre outra vez.
pub(crate) fn reconcile(
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
) -> StructureReport {
    let mut out = StructureReport::default();
    for root in instance_roots(sim) {
        reconcile_one(sim, registry, docs, root, &mut out);
    }
    out
}

/// As raízes de instância viva, em ordem determinística — o mesmo filtro do passe de valores
/// (*a peça cujo mestre é um [`MasterRoot`]*), e pela mesma razão: os bits mudam a cada respawn.
fn instance_roots(sim: &mut SimWorld) -> Vec<Entity> {
    let by_id = stable_index(sim);
    let mut roots: Vec<(u64, Entity)> = {
        let mut q = sim.world_mut().query::<(Entity, &InstanceOf, &StableId)>();
        q.iter(sim.world())
            .filter(|(_, link, _)| {
                by_id
                    .get(&link.master)
                    .is_some_and(|&m| sim.world().get::<MasterRoot>(m).is_some())
            })
            .map(|(e, _, s)| (s.0, e))
            .collect()
    };
    roots.sort_unstable();
    roots.into_iter().map(|(_, e)| e).collect()
}

fn stable_index(sim: &mut SimWorld) -> BTreeMap<u64, Entity> {
    let mut q = sim.world_mut().query::<(Entity, &StableId)>();
    q.iter(sim.world()).map(|(e, s)| (s.0, e)).collect()
}

/// A sub-árvore de `root`, ela incluída, em ordem de travessia determinística.
fn subtree(sim: &SimWorld, root: Entity) -> Vec<Entity> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        out.push(e);
        if let Some(kids) = sim.world().get::<Children>(e) {
            let mut k: Vec<Entity> = kids.iter().copied().collect();
            k.sort_by_key(|&c| ph2d_ecs::sibling_key(sim.world(), c));
            stack.extend(k.into_iter().rev());
        }
    }
    out
}

fn reconcile_one(
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
    root: Entity,
    out: &mut StructureReport,
) {
    let by_id = stable_index(sim);
    let Some(master_root) = sim
        .world()
        .get::<InstanceOf>(root)
        .and_then(|l| by_id.get(&l.master))
        .copied()
    else {
        return;
    };
    // `StableId` de cada peça do MESTRE → a entidade dela.
    let master_pieces: BTreeMap<u64, Entity> = subtree(sim, master_root)
        .into_iter()
        .filter_map(|e| sim.world().get::<StableId>(e).map(|s| (s.0, e)))
        .collect();
    // ⭐⭐⭐ **As peças que ESTA cópia recusou** (F5.10) — o *Removed GameObject* do Unity.
    //
    // ⚠️ **A decisão entra nas DUAS metades do passe, e é isso que a torna barata:** uma peça
    // recusada deixa de contar como *«a instância tem»* (logo cai no ramo das que SOBRAM, que já
    // sepulta as excepções dela e a despawna) e sai da lista das que FALTAM (logo nunca volta). ⇒ o
    // gesto escreve **um id** e o passe faz o resto, pela mesma maquinaria que o mestre a apagar já
    // usava — incluindo o `entomb`/`exhume`, que é o que faz o *Put back* devolver a excepção junto
    // com a peça.
    //
    // ⛔ **A LEI do passe não mudou:** apagar uma peça **por fora** (um `despawn` cru, sem esta
    // marca) continua a ser desfeito no quadro seguinte, e há gate a afirmá-lo. *A guarda vive no
    // gesto; a lei fica no passe.*
    let removed = sim
        .world()
        .get::<ph2d_ecs::ObjectInstance>(root)
        .map(|o| o.removed.clone())
        .unwrap_or_default();
    // `StableId` da peça do mestre → a entidade correspondente NA INSTÂNCIA.
    let mut have: BTreeMap<u64, Entity> = BTreeMap::new();
    let mut extra: Vec<Entity> = Vec::new();
    for e in subtree(sim, root) {
        let Some(link) = sim.world().get::<InstanceOf>(e).copied() else {
            // ⚠️ Uma entidade SEM elo dentro de uma instância é autoria do artista (um *Add Child*
            // sobre uma peça), e **não** uma peça a mais: ela nunca veio do mestre, logo apagá-la
            // seria apagar trabalho que ninguém pediu. *Só o que a receita deu é que a receita tira.*
            continue;
        };
        if master_pieces.contains_key(&link.master) && !removed.contains(&link.master) {
            have.insert(link.master, e);
        } else {
            extra.push(e);
        }
    }

    // ── as que FALTAM ────────────────────────────────────────────────────────────────────────
    //
    // ⚠️ **Só as de TOPO**: a cópia profunda leva a sub-árvore inteira, então materializar uma peça
    // cujo pai também falta duplicaria os descendentes. Uma peça está no topo quando o pai dela já
    // tem contrapartida.
    let linked = sim.world().get::<ph2d_ecs::LinkedArt>(root).is_some();
    let missing: Vec<(u64, Entity, Entity)> = master_pieces
        .iter()
        .filter(|(sid, _)| !have.contains_key(sid) && !removed.contains(sid))
        .filter_map(|(&sid, &m)| {
            let parent_sid = sim
                .world()
                .get::<ph2d_ecs::ChildOf>(m)
                .and_then(|c| sim.world().get::<StableId>(c.0))
                .map(|s| s.0)?;
            let host = if parent_sid == master_root_sid(sim, master_root)? {
                root
            } else {
                *have.get(&parent_sid)?
            };
            Some((sid, m, host))
        })
        .collect();
    for (_, piece, host) in missing {
        // ⚠️ **Pela PORTA da cópia profunda**, e não com uma segunda montagem: o gate
        // `only_the_instantiate_door_calls_the_deep_copy` existe porque uma 2.ª montagem esquece
        // sempre um dos passos (o remap, os documentos, o `MasterRoot`), e o defeito é mudo. Ele
        // apanhou-me a escrever exactamente isso.
        if let Some(new_root) =
            crate::instantiate::materialise_piece(sim, registry, docs, piece, host, linked)
        {
            out.added += 1;
            // ⭐ E se esta peça já cá esteve e o artista tinha uma excepção nela, ela volta agora.
            let sid = sim.world().get::<StableId>(piece).map(|s| s.0);
            if let Some(sid) = sid {
                exhume(sim, registry, root, sid, new_root, out);
            }
        }
    }

    // ── as que estão no SÍTIO ERRADO ─────────────────────────────────────────────────────────
    //
    // ⭐⭐⭐ **A TERCEIRA metade da forma** (F5.12). O passe sabia materializar o que falta e
    // despawnar o que sobra, e **não sabia mover**: uma peça que o artista arrastasse para outro
    // pai dentro da receita ficava, em toda cópia, pendurada no pai antigo — para sempre e em
    // silêncio. *A peça existe, desenha, tem os bytes certos, e só a árvore está errada.*
    //
    // ⚠️ **O `ChildOf` NÃO é componente registado**, e é isso que torna esta metade obrigatória:
    // ele nunca propaga (bem — propagar os bytes dele poria a peça da cópia debaixo do pai do
    // MESTRE) e nunca vira excepção. ⇒ a árvore não tem outro dono senão este bloco.
    //
    // ⚠️⚠️ **A ORDEM NÃO IMPORTA, e a 1.ª redacção deste comentário dizia o contrário.** Ela
    // afirmava que a pré-ordem do mestre *«é o que impede um ciclo»*; a prova de mutação inverteu a
    // travessia e **nada reprovou**. O motivo é a forma do bloco, não a ordem: os alvos são
    // calculados a partir do MESTRE **antes** de qualquer escrita, logo as atribuições são
    // independentes e o estado final é o mesmo em qualquer ordem. Um ciclo pode existir **entre
    // dois `insert`** — e ninguém o observa, porque nenhuma travessia corre no meio.
    //
    // ⇒ *é por isto que este bloco não precisa de verificação de ciclo*, e não por causa da ordem.
    // ⛔ Quem um dia trocar o «recolher e depois aplicar» por «aplicar enquanto percorre» reabre a
    // pergunta — e aí a ordem passa a ser load-bearing a sério.
    //
    // ⛔ **A RAIZ nunca entra** — ela tem elo como qualquer peça (aponta para o `MasterRoot`), e um
    // bloco que a incluísse arrastaria a cópia para dentro da biblioteca. É o `ROOT_IS_ITS_OWN` um
    // nível acima: a pose **e o lugar** da raiz são dela.
    //
    // ⚠️⚠️ **E a linha é REDUNDANTE hoje — a prova de mutação disse-o, e ela fica como CERCA.**
    // Tirá-la não muda nada porque o pai da raiz do mestre **nunca** está no `have`: ele fica
    // *acima* do mestre, e o mapa só tem peças de *dentro* dele. ⇒ quem de facto recusa é o
    // `None => continue` do `match` abaixo. A guarda fica por ser o sítio onde alguém **leria** a
    // lei, e a redundância está escrita em vez de silenciosa.
    //
    // ⛔ **E uma peça que o artista ACRESCENTOU não é tocada** — ela não está no `have` (não tem
    // elo), que é a mesma linha por que o passe não a apaga.
    let root_sid = master_root_sid(sim, master_root);
    let mut moves: Vec<(Entity, Entity)> = Vec::new();
    for m in subtree(sim, master_root) {
        let Some(sid) = sim.world().get::<StableId>(m).map(|s| s.0) else {
            continue;
        };
        let Some(&mine) = have.get(&sid) else {
            continue;
        };
        if mine == root {
            continue;
        }
        let Some(parent_sid) = sim
            .world()
            .get::<ph2d_ecs::ChildOf>(m)
            .and_then(|c| sim.world().get::<StableId>(c.0))
            .map(|s| s.0)
        else {
            continue;
        };
        // O pai da peça do mestre é a raiz do mestre ⇒ o pai da minha é a raiz da CÓPIA.
        let want = if Some(parent_sid) == root_sid {
            root
        } else {
            // ⚠️ Sem contrapartida viva o pai está a caminho do despawn (foi recusado, ou o mestre
            // apagou-o): mexer nesta peça agora seria arrumá-la para dentro de algo que morre no
            // mesmo passe.
            match have.get(&parent_sid) {
                Some(&p) => p,
                None => continue,
            }
        };
        if sim.world().get::<ph2d_ecs::ChildOf>(mine).map(|c| c.0) != Some(want) {
            moves.push((mine, want));
        }
    }
    for (child, parent) in moves {
        sim.world_mut()
            .entity_mut(child)
            .insert(ph2d_ecs::ChildOf(parent));
        out.moved += 1;
    }

    // ── as que SOBRAM ────────────────────────────────────────────────────────────────────────
    //
    // ⚠️ Recolhidas como CONJUNTO da sub-árvore de cada uma: apagar só a entidade deixaria os
    // descendentes dela pendurados num pai que já não existe.
    let doomed: BTreeSet<Entity> = extra
        .into_iter()
        .flat_map(|e| subtree(sim, e).into_iter())
        .collect();
    // ⭐⭐⭐ **A EXCEPÇÃO do artista sai da peça ANTES de a peça morrer** (F5.3) — ver
    // [`ph2d_ecs::ObjectInstance::orphans`]. Sem isto, apagar a peça no mestre e **desfazer**
    // devolvia-a com o valor do MESTRE e a chave de override intacta: a cópia perdia a excepção
    // **e ficava surda à receita para sempre**, porque o passe salta o que a instância possui.
    // ⚠️ *Foi a F5.1 que criou este buraco* — antes dela ninguém despawnava a peça.
    if !doomed.is_empty() {
        entomb(sim, registry, root, &doomed, out);
    }
    for e in doomed {
        if let Ok(em) = sim.world_mut().get_entity_mut(e) {
            em.despawn();
            out.removed += 1;
        }
    }
}

/// ⭐⭐⭐ **A cópia RECUSA estas peças** (F5.10) — o *Removed GameObject* do Unity. Devolve quantas.
///
/// # ⚠️ Ela escreve uma DECISÃO e não apaga nada
///
/// Quem apaga é o passe: uma peça marcada aqui cai, no quadro seguinte, no ramo das que **sobram**
/// — que já sepulta as excepções dela ([`entomb`]) e despawna a sub-árvore inteira — e sai da lista
/// das que **faltam**, logo nunca mais volta. ⛔ Despawnar aqui saltaria o sepultador: a excepção
/// daquela peça ficaria **nem viva nem enterrada**, invisível ao cartão e a bloquear a receita.
///
/// ⚠️ **A chave é a peça do MESTRE**, e não a entidade clicada: a cópia é respawnada a cada Ctrl+Z
/// com bits novos, e o elo é o que sobrevive.
///
/// ⚠️ **Uma peça sem raiz de instância é saltada** — a partição do gesto já garante que só peças de
/// cópia chegam aqui, e uma segunda pergunta com outra resposta seria a lei em dois sítios.
pub(crate) fn refuse_pieces(sim: &mut SimWorld, pieces: &[u64]) -> usize {
    let mut done = 0;
    for &bits in pieces {
        let e = Entity::from_bits(bits);
        let Some(master_piece) = sim.world().get::<InstanceOf>(e).map(|l| l.master) else {
            continue;
        };
        let Some(root) = crate::instance_verbs::instance_root_of(sim, e) else {
            continue;
        };
        let mut inst = sim
            .world()
            .get::<ph2d_ecs::ObjectInstance>(root)
            .cloned()
            .unwrap_or_default();
        if inst.removed.insert(master_piece) {
            sim.world_mut().entity_mut(root).insert(inst);
            done += 1;
        }
    }
    done
}

/// ⭐⭐ **Devolve uma peça recusada** (F5.10) — o *Put back* do cartão. `true` se saiu alguma.
///
/// ⚠️ **Ela só apaga a decisão.** Quem materializa a peça, quem lhe traz os bytes da receita e quem
/// **exuma** a excepção que o artista tinha nela é o passe estrutural, no quadro seguinte — e é por
/// isso que o *Put back* devolve a peça **como ela estava**, sem uma linha de código sobre poses.
pub(crate) fn restore_piece(sim: &mut SimWorld, root_bits: u64, piece: u64) -> bool {
    let root = Entity::from_bits(root_bits);
    let Some(mut inst) = sim.world().get::<ph2d_ecs::ObjectInstance>(root).cloned() else {
        return false;
    };
    if !inst.removed.remove(&piece) {
        return false;
    }
    sim.world_mut().entity_mut(root).insert(inst);
    true
}

fn master_root_sid(sim: &SimWorld, master_root: Entity) -> Option<u64> {
    sim.world().get::<StableId>(master_root).map(|s| s.0)
}

/// ⭐⭐ **Guarda os bytes de cada override cuja peça vai morrer**, na raiz da instância.
///
/// ⚠️ **Só os que têm override.** Uma peça que a instância não possui não tem excepção nenhuma a
/// perder — guardar-lhe os bytes seria uma cópia do mestre a envelhecer num sítio que ninguém lê.
fn entomb(
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    root: Entity,
    doomed: &BTreeSet<Entity>,
    out: &mut StructureReport,
) {
    let Some(mut inst) = sim.world().get::<ph2d_ecs::ObjectInstance>(root).cloned() else {
        return;
    };
    let mut wrote = false;
    for &e in doomed {
        let Some(link) = sim.world().get::<InstanceOf>(e).copied() else {
            continue;
        };
        // ⚠️ O `range` sobre a chave ordenada dá as entradas desta peça sem varrer as outras —
        // é para isto que `OverrideKey` ordena por `piece` antes de `type_id`.
        let keys: Vec<ph2d_ecs::OverrideKey> = inst
            .overrides
            .iter()
            .filter(|k| k.piece == link.master)
            .copied()
            .collect();
        for key in keys {
            let Some(entry) = registry.get_by_id(key.type_id) else {
                continue;
            };
            // ⚠️ A AUSÊNCIA também é a excepção (o artista tirou o componente da cópia), e por
            // isso o `unwrap_or_default` do serialize não pode ser confundido com «não havia»:
            // o que se guarda é o `Option`, achatado em bytes vazios para a ausência.
            let bytes = (entry.serialize)(sim.world(), e)
                .unwrap_or_default()
                .unwrap_or_default();
            // ⭐⭐⭐ **O NOME sai daqui, e só daqui** — ver [`ph2d_ecs::OrphanOverride`]: a peça
            // ainda está viva (o `despawn` é o laço a seguir) e o `Name` dela é o do mestre.
            // Depois deste instante não há onde o ir buscar, e o painel fica sem poder dizer
            // *«quais»*.
            let piece_name = sim
                .world()
                .get::<ph2d_ecs::Name>(e)
                .map(|n| n.0.clone())
                .unwrap_or_default();
            inst.orphans
                .insert(key, ph2d_ecs::OrphanOverride { bytes, piece_name });
            inst.overrides.remove(&key);
            out.orphaned += 1;
            wrote = true;
        }
    }
    if wrote {
        sim.world_mut().entity_mut(root).insert(inst);
    }
}

/// ⭐⭐ **Repõe a excepção quando a peça VOLTA** (o `Ctrl+Z` no mestre).
///
/// ⚠️ **A chave é a mesma porque o `StableId` sobrevive ao respawn do undo** — é a propriedade que
/// o id compra, e é ela que faz *«volta a pegar»* ser verdade em vez de uma esperança.
fn exhume(
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    root: Entity,
    piece_sid: u64,
    inst_piece: Entity,
    out: &mut StructureReport,
) {
    let Some(mut inst) = sim.world().get::<ph2d_ecs::ObjectInstance>(root).cloned() else {
        return;
    };
    let keys: Vec<ph2d_ecs::OverrideKey> = inst
        .orphans
        .keys()
        .filter(|k| k.piece == piece_sid)
        .copied()
        .collect();
    if keys.is_empty() {
        return;
    }
    for key in keys {
        let Some(orphan) = inst.orphans.remove(&key) else {
            continue;
        };
        let bytes = orphan.bytes;
        if let Some(entry) = registry.get_by_id(key.type_id) {
            if bytes.is_empty() {
                (entry.remove)(sim.world_mut(), inst_piece);
            } else {
                let _ = (entry.insert_from_bytes)(sim.world_mut(), inst_piece, &bytes);
            }
        }
        inst.overrides.insert(key);
        out.restored += 1;
    }
    sim.world_mut().entity_mut(root).insert(inst);
}

#[cfg(test)]
#[path = "instance_structure_tests.rs"]
mod tests;

/// ⚠️ **A RECUSA de uma peça é outro assunto** — ali a lei de que a forma segue a receita, aqui o
/// que acontece quando o artista diz não. O precedente do corte é o `instance_variant_verb_tests`.
#[cfg(test)]
#[path = "instance_refuse_tests.rs"]
mod refuse_tests;

/// ⚠️ **E QUEM É PAI DE QUEM é o terceiro assunto** (F5.12) — a metade da forma que o passe não
/// sabia. Ficheiro próprio pela mesma razão dos dois acima: o `instance_structure_tests` está a
/// **538** de 600 linhas, e um corte por assunto é mais barato que um corte por tamanho.
#[cfg(test)]
#[path = "instance_reparent_tests.rs"]
mod reparent_tests;
