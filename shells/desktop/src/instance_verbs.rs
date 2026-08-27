//! ⭐ **Os VERBOS de instância que faltavam** (ADR-0164 / plano F4.5) — *Criar componente*,
//! *Destacar* e *Aplicar ao mestre*.
//!
//! O *Redefinir* já vive em [`crate::instance_sync::revert_all_overrides`] (é o *Revert to Master*
//! do menu) e o *Instanciar* em [`crate::instantiate`]. Aqui ficam os três que fecham a tabela do
//! [doc 04 §112-119].
//!
//! # ⚠️ Cada um responde uma RAZÃO, nunca um `bool`
//!
//! É a lei que o `Refusal` da porta de instanciar já paga: *duas recusas que devolvem o mesmo
//! `None` produzem o mesmo aviso inútil*. Quem tem UI escolhe as palavras; estas funções respondem
//! o FATO.
//!
//! [doc 04 §112-119]: https://github.com/dibrioli/PH2D/blob/main/docs/Components/04_decisao_arquitetura.md

use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{Children, Entity, InstanceOf, MasterRoot, ObjectInstance, SimWorld, StableId};

/// **Por que um verbo de instância foi recusado.**
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum VerbRefusal {
    /// A entidade não é (nem pertence a) uma instância.
    NotAnInstance,
    /// A subárvore já é uma receita.
    AlreadyAMaster,
    /// A subárvore está **dentro** de uma instância — transformá-la em receita partiria o elo
    /// da instância que a contém. É o caso que a F5 (aninhamento) abre.
    InsideAnInstance,
}

// ⛔ **Havia um quarto (`MasterIsGone`) e ele saiu**: nenhum verbo o constrói, porque a pergunta
// *«o mestre ainda existe?»* já é respondida pelo `NotAnInstance` — a raiz de uma instância é *a
// peça cujo mestre é um `MasterRoot`*, e um mestre apagado deixa de a satisfazer. *Uma variante que
// nada constrói é uma resposta que o artista nunca lê.*

/// ⭐⭐ **CRIAR COMPONENTE** — a seleção vira **receita**, e uma **instância fica no lugar dela**.
///
/// Devolve `(mestre, instância)`.
///
/// # ⚠️ A receita fica ESCONDIDA, e a decisão é de produto
///
/// Sem isto o artista faz o gesto e vê **dois objetos empilhados** — um que cai e outro que não —,
/// que se lê como um defeito. Escondida, o canvas fica exatamente como estava (é o que o Unity
/// faz: o prefab **asset** não está na cena) e a Hierarquia ganha uma linha, que é onde a receita
/// se edita até a biblioteca existir (F6/F7).
///
/// ⚠️⚠️ **E isso obrigou a `Visibility` a ser da RAIZ da instância** (`ROOT_IS_ITS_OWN`): sem essa
/// metade, o `hidden` da receita propagava e **todas as instâncias nasciam invisíveis** — o gesto
/// apagaria da tela o objeto que o artista acabou de transformar em componente.
///
/// # ⛔ As duas recusas
///
/// - a subárvore **já é** uma receita (o gesto não tem o que fazer);
/// - ela está **dentro** de uma instância — fazer dela uma receita partiria o elo da instância que
///   a contém, e a resposta certa é a da F5 (aninhamento), não um mestre a meio de uma cópia.
pub(crate) fn make_master(
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    entity: Entity,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
) -> Result<(Entity, Entity), VerbRefusal> {
    if sim.world().get::<MasterRoot>(entity).is_some() {
        return Err(VerbRefusal::AlreadyAMaster);
    }
    // ⚠️ A pergunta é sobre a entidade **e os ancestrais dela**: uma peça no meio de uma cópia
    // também está «dentro de uma instância».
    if belongs_to_an_instance(sim, entity) {
        return Err(VerbRefusal::InsideAnInstance);
    }
    let parent = sim.world().get::<ph2d_ecs::ChildOf>(entity).map(|c| c.0);

    sim.world_mut().entity_mut(entity).insert(MasterRoot);
    // ⚠️ A instância nasce ANTES de a receita ser marcada? Não: ela é a cópia da receita, e a
    // cópia leva o `MasterRoot` que a porta de instanciar depois tira. A ordem é esta.
    let instance = match crate::instantiate::instantiate_master(sim, registry, entity, parent, docs)
    {
        Ok(e) => e,
        Err(_) => {
            // Desfaz a marcação: um gesto que falha a meio deixaria uma receita que o artista não
            // pediu — e a subárvore dele desapareceria da tela, porque uma receita não se desenha.
            sim.world_mut().entity_mut(entity).remove::<MasterRoot>();
            ph2d_ecs::assign_master_pieces(sim.world_mut());
            return Err(VerbRefusal::AlreadyAMaster);
        }
    };
    // ⚠️ **A POSE já veio, e uma prova de mutação foi precisa para o dizer.** A 1.ª versão
    // reescrevia aqui o `Transform` da receita na instância *«porque a pose é `InstanceLocal` e o
    // sync nunca a traria»* — verdade sobre o sync, e **irrelevante**: a cópia profunda leva o
    // `Transform` verbatim, então a instância **nasce** no lugar. A mutação que apagava a linha
    // não matou gate nenhum, e foi assim que ela se revelou morta.
    //
    //
    // ⚠️⚠️ **E a `Visibility` SAIU daqui em 2026-08-26.** A 1.ª versão escondia a raiz da receita
    // com `Visibility { hidden: true }` e depois devolvia à instância o valor que a seleção tinha.
    // A premissa era **falsa**: `Visibility` é per-entidade neste motor e **não desce aos
    // descendentes** (o `sim_extract` diz-no pelo nome), então uma receita que fosse um GRUPO
    // continuava a desenhar as peças — o artista via **dois objetos empilhados**, que é o defeito
    // que a nota dizia ter evitado. ⇒ hoje quem não desenha uma receita é o extract, pela marca
    // **derivada** `MasterPiece`, e este gesto não toca em autoria nenhuma de visibilidade.
    Ok((entity, instance))
}

/// ⭐ **DESTACAR** — corta o vínculo; os objetos continuam exatamente iguais, só deixam de seguir.
///
/// Devolve quantas peças foram soltas.
///
/// ⚠️ **É a instância INTEIRA, mesmo clicando numa peça** — e não uma peça de cada vez. Uma
/// instância com metade das peças ligadas não é nada que se saiba nomear: o sync propagaria a
/// metade que ficou e o artista veria um objeto que obedece pela metade. *Unity chama-lhe Unpack, e
/// também não tem meia-instância.*
///
/// ⚠️ **Irreversível pelo verbo** (o Ctrl+Z desfaz, como qualquer gesto): religar exigiria saber de
/// que peça do mestre cada objeto veio, e é exatamente essa informação que o gesto apaga.
pub(crate) fn detach(sim: &mut SimWorld, clicked: Entity) -> Result<usize, VerbRefusal> {
    let root = instance_root_of(sim, clicked).ok_or(VerbRefusal::NotAnInstance)?;
    let mut n = 0;
    for e in subtree(sim, root) {
        if sim.world().get::<InstanceOf>(e).is_some() {
            sim.world_mut().entity_mut(e).remove::<InstanceOf>();
            n += 1;
        }
    }
    sim.world_mut().entity_mut(root).remove::<ObjectInstance>();
    Ok(n)
}

/// ⭐⭐ **APLICAR AO MESTRE** — empurra as excepções desta instância para dentro da receita.
///
/// Devolve quantos componentes foram escritos no mestre. O escopo é o que se clicou (uma peça ⇒ as
/// dela; a raiz ⇒ todas), pela razão do *Revert*.
///
/// # ⚠️ Ele muta a RECEITA, e é isso que se quer
///
/// Depois de aplicar, o override deixa de existir — e o passe de sync vê **o mestre mexer-se** e
/// leva o valor a **todas as outras instâncias**. É o *Apply* do Unity: *«o que eu fiz aqui passa a
/// ser o padrão»*.
///
/// ⚠️ **A ordem é: escrever no mestre, DEPOIS limpar a chave.** Ao contrário, o passe que corre no
/// meio veria a instância sem excepção e diferente da receita, e achataria a edição que o gesto
/// existe para promover.
pub(crate) fn apply_to_master(
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    echo: &mut crate::instance_sync::MasterEcho,
    clicked: Entity,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
) -> Result<usize, VerbRefusal> {
    let root = instance_root_of(sim, clicked).ok_or(VerbRefusal::NotAnInstance)?;
    let scope = (root != clicked)
        .then(|| sim.world().get::<InstanceOf>(clicked).map(|l| l.master))
        .flatten();
    let by_id = stable_index(sim);
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
    if keys.is_empty() {
        return Ok(0);
    }
    // De que entidade da instância veio cada peça do mestre.
    let mine: std::collections::BTreeMap<u64, Entity> = subtree(sim, root)
        .into_iter()
        .filter_map(|e| sim.world().get::<InstanceOf>(e).map(|l| (l.master, e)))
        .collect();

    let mut n = 0;
    for key in keys {
        let (Some(&master_piece), Some(&inst_piece)) =
            (by_id.get(&key.piece), mine.get(&key.piece))
        else {
            continue;
        };
        // ⭐⭐ **Um DOCUMENTO aplica-se por CONTEÚDO** (F4.6b), e não pelos bytes do componente: o
        // `insert_from_bytes` escreveria o **id** do `VecPathRef` da instância no mestre, e as duas
        // passariam a apontar para o mesmo path — editar uma mexeria na outra.
        if key.type_id == ph2d_ecs::scene::stable_type_id(crate::instance_sync_docs::VEC_PATH) {
            if crate::instance_sync_docs::apply_one(sim, docs, inst_piece, master_piece) {
                crate::instance_sync::revert_override(sim, echo, root, key);
                n += 1;
            }
            continue;
        }
        let Some(entry) = registry.get_by_id(key.type_id) else {
            continue;
        };
        let value = (entry.serialize)(sim.world(), inst_piece).unwrap_or_default();
        match value {
            Some(bytes) => {
                if (entry.insert_from_bytes)(sim.world_mut(), master_piece, &bytes).is_err() {
                    continue;
                }
            }
            // ⚠️ A ausência também é uma excepção: o artista **tirou** o componente da cópia, e
            // aplicar isso é tirá-lo da receita.
            None => (entry.remove)(sim.world_mut(), master_piece),
        }
        crate::instance_sync::revert_override(sim, echo, root, key);
        n += 1;
    }
    Ok(n)
}

/// **Qual dos verbos o menu pediu.**
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Verb {
    /// *Make Component* — a seleção vira receita.
    Make,
    /// *Instantiate* — mais uma cópia da receita.
    Place,
    /// *Detach from Master* — corta o vínculo.
    Detach,
    /// *Apply to Master* — a excepção vira o padrão.
    Apply,
}

/// ⭐ **O dreno dos quatro** — resolve a entidade, corre o verbo e **responde ao artista**.
///
/// Devolve `true` quando alguma coisa mudou.
///
/// ⚠️ **Todo caminho negativo fala.** A tabela daquele menu é PLANA (ela não sabe o que a linha é),
/// então os quatro itens aparecem em toda linha; um item que come o clique em silêncio é pior que
/// um ausente. *É a mesma lei que o report do Enio sobre o Revert pagou.*
#[allow(clippy::too_many_arguments)] // um verbo, o mundo, o registo, o eco, os documentos e a voz
pub(crate) fn drain(
    verb: Verb,
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    echo: &mut crate::instance_sync::MasterEcho,
    entity_bits: u64,
    toasts: &mut ph2d_editor::ToastQueue,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
) -> bool {
    use ph2d_editor::Toast;
    let entity = Entity::from_bits(entity_bits);
    match verb {
        Verb::Make => match make_master(sim, registry, entity, docs) {
            Ok(_) => {
                toasts.push(Toast::success(
                    "Made a component — an instance took its place",
                ));
                true
            }
            Err(VerbRefusal::AlreadyAMaster) => {
                toasts.push(Toast::info("This is already a component"));
                false
            }
            Err(_) => {
                toasts.push(Toast::warning(
                    "Inside an instance — detach it first, or edit the master",
                ));
                false
            }
        },
        // ⚠️ *Instantiate* pede a RECEITA, e a linha pode ser a instância que ficou no lugar dela:
        // o aviso NOMEIA a saída, senão o artista fica a clicar na linha errada.
        Verb::Place => {
            match crate::instantiate::instantiate_master(sim, registry, entity, None, docs) {
                Ok(_) => {
                    toasts.push(Toast::success("Instantiated"));
                    true
                }
                Err(crate::instantiate::Refusal::WouldNestInItself) => {
                    toasts.push(Toast::warning("That would put the component inside itself"));
                    false
                }
                Err(_) => {
                    toasts.push(Toast::warning("Not a component — pick the master row"));
                    false
                }
            }
        }
        Verb::Detach => match detach(sim, entity) {
            Ok(n) => {
                toasts.push(Toast::success(format!("Detached {n} piece(s) from master")));
                true
            }
            Err(_) => {
                toasts.push(Toast::warning("Not part of an instance"));
                false
            }
        },
        Verb::Apply => match apply_to_master(sim, registry, echo, entity, docs) {
            Ok(0) => {
                toasts.push(Toast::info("Nothing overridden here"));
                false
            }
            Ok(n) => {
                toasts.push(Toast::success(format!("Applied {n} change(s) to master")));
                true
            }
            Err(_) => {
                toasts.push(Toast::warning("Not part of an instance"));
                false
            }
        },
    }
}

// ── as travessias partilhadas ──────────────────────────────────────────────────────────────

/// `StableId → entidade`, do mundo inteiro.
fn stable_index(sim: &mut SimWorld) -> std::collections::BTreeMap<u64, Entity> {
    let mut q = sim.world_mut().query::<(Entity, &StableId)>();
    q.iter(sim.world()).map(|(e, s)| (s.0, e)).collect()
}

/// A raiz da instância a que `clicked` pertence — a peça cujo mestre é um [`MasterRoot`].
///
/// ⚠️ Sobe por `ChildOf`, nunca pelo elo: o `InstanceOf` de uma peça aponta para a peça do MESTRE,
/// e subir por ele sairia da instância e ia parar à receita.
fn instance_root_of(sim: &mut SimWorld, clicked: Entity) -> Option<Entity> {
    let by_id = stable_index(sim);
    sim.world().get::<InstanceOf>(clicked)?;
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
fn belongs_to_an_instance(sim: &mut SimWorld, entity: Entity) -> bool {
    instance_root_of(sim, entity).is_some()
}

/// A subárvore de `root`, ela incluída.
fn subtree(sim: &SimWorld, root: Entity) -> Vec<Entity> {
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

#[cfg(test)]
#[path = "instance_verbs_tests.rs"]
mod tests;
