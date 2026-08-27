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
    /// ⭐⭐ A subárvore está **dentro de outra RECEITA** (auditoria §1.1, 2026-08-27).
    ///
    /// ⚠️ **Não é simetria decorativa com a de cima — é a única das quatro portas do §1.1 cujo
    /// dano NÃO se cura sozinho**, e o mecanismo tem nome: `master_root_of` pára na raiz **mais
    /// próxima**, então um `MasterRoot` aninhado **ENCURTA a sub-árvore de edição**. Seleccionar a
    /// receita exterior deixa de acender o que está debaixo da interior ⇒ a instância irmã fica
    /// invisível **mesmo com a receita seleccionada**, que é o estado que o modo de edição existe
    /// para tornar alcançável. As outras três portas (Duplicate, Add Child, arrastar) produzem
    /// coisas que a marca derivada volta a acender no quadro seguinte; esta não.
    ///
    /// ⛔ Recusar é a resposta de HOJE, e a boa é a F5 (aninhamento de receitas) — a mesma
    /// fronteira que o [`Self::InsideAnInstance`] declara.
    InsideAMaster,
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
    // ⭐⭐ **Dentro de outra RECEITA, não** — ver [`VerbRefusal::InsideAMaster`]: um `MasterRoot`
    // aninhado encurta a sub-árvore de edição e deixa a instância irmã invisível para sempre.
    // (A própria entidade já saiu acima, então isto pergunta pelos ANCESTRAIS.)
    if ph2d_ecs::master_root_of(sim.world(), entity).is_some() {
        return Err(VerbRefusal::InsideAMaster);
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
    // ⚠️ **A cópia que fica no lugar tem arte PRÓPRIA** (`Own`): o gesto é *«guarda isto na
    // biblioteca»*, e o que o artista pintar nela a seguir é dela. Quem quiser o `Alt+D` pede-o
    // pelo nome, com o *Instantiate Linked*.
    let instance = match crate::instantiate::instantiate_master(
        sim,
        registry,
        entity,
        parent,
        docs,
        crate::instantiate::ArtLink::Own,
    ) {
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

/// ⭐⭐ **Quantas instâncias deste mestre já existem** — o `n` da cascata.
///
/// Conta as RAÍZES (a peça cujo `master` é o próprio mestre), e não as peças: um crachá de duas
/// peças não é duas cópias.
fn instances_of(sim: &mut SimWorld, master_id: u64) -> usize {
    let mut q = sim.world_mut().query::<&InstanceOf>();
    q.iter(sim.world())
        .filter(|link| link.master == master_id)
        .count()
}

/// ⭐⭐⭐ **UMA CÓPIA NUNCA ATERRA EM CIMA DO QUE VEIO** (report do Enio, 2026-08-26 → 27).
///
/// O *Instantiate* punha a cópia na pose do mestre — exactamente por cima dele e das irmãs. Duas
/// formas idênticas sobrepostas não são um estado que o artista desfaça com o olho: *«mudei o
/// mestre»* e *«mudei a cópia que está em cima dele»* passam a ser o mesmo gesto na tela, e foi
/// isso que fez a propagação **parecer morta quando estava viva** (o §14 do handoff — a cena 2, com
/// a receita LONGE das cópias, propaga).
///
/// ⚠️ **A lei é a que o verbo VETORIAL já tinha** (`vec_component_edit::cascade_offset`): um passo
/// de TELA por cópia, cascateado. Um passo de mundo fixo seria invisível com o zoom afastado e
/// atiraria a cópia para fora do ecrã com o zoom perto.
///
/// ⚠️ **A 1.ª cópia fica no ZERO, e isso não é uma segunda regra** — é o que a CONTAGEM já diz: ela
/// conta as instâncias que já existem **menos a que acabou de nascer**, e para a primeira isso é
/// zero. É por isso que o *Criar componente* deixa a cópia exactamente onde a seleção estava sem
/// precisar de um ramo próprio. ⛔ *Uma afirmação que mutação nenhuma mata é uma afirmação sobre
/// nada* — a versão anterior deste doc dizia *«o Criar componente NÃO cascateia»*, e cascatear
/// ali era um no-op.
fn cascade(sim: &mut SimWorld, instance: Entity, master_id: u64, step: [f32; 2]) {
    // `- 1`: a instância que acabou de nascer já está contada.
    let n = instances_of(sim, master_id).saturating_sub(1) as f32;
    if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(instance) {
        t.translation.x += step[0] * n;
        t.translation.y += step[1] * n;
    }
}

/// **Qual dos verbos o menu pediu.**
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Verb {
    /// *Make Component* — a seleção vira receita.
    Make,
    /// *Instantiate* — mais uma cópia da receita, com **arte própria**.
    Place,
    /// ⭐ *Instantiate Linked* — mais uma cópia que **divide a arte** da receita (Enio,
    /// 2026-08-27). É o `Alt+D` do Blender: editar a tinta ou o desenho dela sobe à receita e
    /// chega a todas. Ver [`crate::instantiate::ArtLink`].
    PlaceLinked,
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
    // O passo da cascata, em unidades de MUNDO — ver [`cascade`].
    place_step: [f32; 2],
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
            // ⚠️ **Braço por braço, e sem `_`**: a recusa nova (`InsideAMaster`) chegou aqui com um
            // catch-all que dizia *«Inside an instance»* — a frase errada sobre a coisa errada. Um
            // `match` exaustivo é o que obriga a próxima recusa a escolher a sua voz.
            Err(VerbRefusal::InsideAMaster) => {
                toasts.push(Toast::warning(
                    "Inside a component — components cannot nest yet",
                ));
                false
            }
            Err(VerbRefusal::InsideAnInstance | VerbRefusal::NotAnInstance) => {
                toasts.push(Toast::warning(
                    "Inside an instance — detach it first, or edit the master",
                ));
                false
            }
        },
        // ⚠️ *Instantiate* pede a RECEITA, e a linha pode ser a instância que ficou no lugar dela:
        // o aviso NOMEIA a saída, senão o artista fica a clicar na linha errada.
        Verb::Place | Verb::PlaceLinked => {
            // ⭐⭐⭐ **Qual das duas leis** — ver [`crate::instantiate::ArtLink`]. O verbo é que a
            // escolhe, e não uma preferência escondida: *«esta cópia é minha»* e *«esta cópia é a
            // mesma coisa noutro sítio»* são dois pedidos diferentes, e o artista faz um deles.
            let link = if verb == Verb::PlaceLinked {
                crate::instantiate::ArtLink::Shared
            } else {
                crate::instantiate::ArtLink::Own
            };
            // ⭐⭐⭐ **A cópia herda o PAI da receita** (auditoria §1.3, 2026-08-27).
            //
            // Este ramo passava `None` LITERAL enquanto os outros dois chamadores da cópia profunda
            // — o `make_master` (:70) e o `duplicate_subtree` — derivam o pai da fonte. ⇒ com uma
            // receita aninhada num grupo deslocado, o *Instantiate* largava a cópia na RAIZ da cena
            // com a pose LOCAL do mestre: medido, mundo (9,3) escala 2× ⇒ (0.5,0) escala 1×. O que
            // se perdia não era a translação, era o transform de mundo INTEIRO do pai — a cópia
            // saía noutro sítio, **com outro tamanho e outro ângulo**.
            //
            // ⚠️ **E a metade pior era o `ChildOf` que faltava, não a pose:** a cópia nº1 (a que o
            // *Make Component* deixa no lugar) fica dentro do grupo e as nº2..n ficavam na raiz da
            // cena ⇒ **mover o grupo passava a mover uma instância e não as outras**, e arrastar a
            // perdida de volta à mão não curava. *Uma discordância entre irmãos, não um erro do
            // motor de cópia.*
            let parent = sim.world().get::<ph2d_ecs::ChildOf>(entity).map(|c| c.0);
            match crate::instantiate::instantiate_master(sim, registry, entity, parent, docs, link)
            {
                Ok(inst) => {
                    // ⭐ A cópia não aterra em cima do mestre nem das irmãs — ver [`cascade`].
                    if let Some(id) = sim.world().get::<StableId>(entity).map(|s| s.0) {
                        cascade(sim, inst, id, place_step);
                    }
                    // ⚠️ **O toast diz QUAL das duas** — os dois itens do menu ficam um debaixo do
                    // outro e a diferença entre eles só se vê no gesto SEGUINTE (pintar, mover um
                    // nó). Uma confirmação igual para os dois deixaria o artista sem saber qual
                    // clicou.
                    toasts.push(Toast::success(if verb == Verb::PlaceLinked {
                        "Instantiated linked — its art follows the component both ways"
                    } else {
                        "Instantiated"
                    }));
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
fn instance_root_of(sim: &mut SimWorld, clicked: Entity) -> Option<Entity> {
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

/// ⚠️ **Onde a cópia ATERRA é outro assunto** — e o ficheiro acima estava no tecto de 600 LOC.
/// Aqui os gates dos verbos; lá a pergunta dos dois reports do Enio sobre a pose da cópia nova.
#[cfg(test)]
#[path = "instance_place_tests.rs"]
mod place_tests;
