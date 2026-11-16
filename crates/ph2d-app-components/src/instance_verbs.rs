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
use ph2d_ecs::{Entity, InstanceOf, MasterRoot, Name, ObjectInstance, SimWorld, StableId};

/// **Por que um verbo de instância foi recusado.**
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VerbRefusal {
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
    if belongs_to_an_instance(sim, entity) && instance_root_of(sim, entity) != Some(entity) {
        return Err(VerbRefusal::InsideAnInstance);
    }
    let parent = sim.world().get::<ph2d_ecs::ChildOf>(entity).map(|c| c.0);
    // ⭐⭐ **O que este objeto ERA** — se ele já seguia um mestre, o gesto faz uma VARIANTE, e daí
    // sai o nome (ver o bloco do fim). Lido ANTES, porque a marcação e a cópia mexem nos dois.
    let base_link = sim
        .world()
        .get::<ph2d_ecs::InstanceOf>(entity)
        .map(|l| l.master);
    let original_name = sim.world().get::<Name>(entity).map(|n| n.0.clone());

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
    if let Some(base_id) = base_link {
        let base_name = master_named(sim, base_id).unwrap_or_else(|| "Prefab".to_string());
        // ⚠️ **O nome é só um NOME** (Enio, 2026-09-01): o sufixo `Variant` é o idioma do Unity e
        // serve de rótulo do chip. ⛔ **Nada de propriedades aqui** — o mecanismo (`Size = Big`) foi
        // recusado duas vezes e está adiado para o fim do plano. *Foi pôr a declaração no nome que
        // obrigou todo gesto a reescrevê-lo, e custou seis reports.*
        let vname =
            ph2d_unique_name::unique_name_excluding(sim, &format!("{base_name} Variant"), entity);
        sim.world_mut().entity_mut(entity).insert(Name::new(vname));
        if let Some(original) = original_name {
            let cname = ph2d_unique_name::unique_name_excluding(sim, &original, instance);
            sim.world_mut()
                .entity_mut(instance)
                .insert(Name::new(cname));
        }
    }
    // ⛔ **O espelho MORREU aqui em 2026-09-01.** Ele existia porque a declaração vivia no `Name`
    // e todo gesto que mexesse no elo tinha de a reescrever — hoje a declaração é um componente da
    // RECEITA, e mudar o elo de uma cópia não tem nome nenhum a acertar.
    Ok((entity, instance))
}

/// O `Name` da entidade cujo `StableId` é `id` — o mesmo do construtor do cartão.
pub fn master_named(sim: &mut SimWorld, id: u64) -> Option<String> {
    let mut q = sim.world_mut().query::<(&ph2d_ecs::StableId, &Name)>();
    q.iter(sim.world())
        .find(|(s, _)| s.0 == id)
        .map(|(_, n)| n.0.clone())
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

/// ⭐⭐ **APLICAR AO MESTRE** — empurra as excepções desta instância para dentro da receita
/// **directa**, que é a mais externa da escada.
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
/// # ⭐⭐⭐ Ele é UM DEGRAU da escada, e é sempre o mais EXTERNO
///
/// Com receitas aninhadas há mais de uma resposta a *«a que mestre?»*, e a escolha vive em
/// [`crate::instance_apply_deep::apply_to_level`] — esta função é o degrau **1**, que é
/// exactamente o *«Apply All aplica sempre ao mais externo»* do Unity.
///
/// ⛔ **Ela não tem lei própria**: escrever aqui uma segunda travessia daria duas respostas a *«o
/// que é aplicar?»*, e a que envelhece é a que o menu chama.
pub(crate) fn apply_to_master(
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    echo: &mut crate::instance_sync::MasterEcho,
    clicked: Entity,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
) -> Result<usize, VerbRefusal> {
    if crate::instance_added::is_added(sim, clicked) {
        return crate::instance_added::promote(sim, registry, docs, clicked)
            .map(|p| p.pieces)
            .map_err(|_| VerbRefusal::NotAnInstance);
    }
    let root = instance_root_of(sim, clicked).ok_or(VerbRefusal::NotAnInstance)?;
    let outermost = sim
        .world()
        .get::<InstanceOf>(root)
        .map(|l| l.master)
        .ok_or(VerbRefusal::NotAnInstance)?;
    crate::instance_apply_deep::apply_to_level(sim, registry, echo, clicked, outermost, docs)
        .map(|a| a.changed)
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
pub enum Verb {
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
    /// ⭐⭐ *Remove from Library* — a receita deixa de ser receita (report do Enio, 2026-08-30:
    /// *«não dá para tirar um asset da biblioteca»*). A lei das duas metades vive em
    /// [`crate::instance_unmake`], que é onde ela se lê inteira.
    Unmake,
    /// ⭐⭐⭐ *Edit Prefab* — **abrir a receita a partir de uma CÓPIA dela** (F4.6c, 2026-09-07).
    ///
    /// # A receita não está no canvas, e até aqui só a biblioteca a alcançava
    ///
    /// Uma receita é `MasterPiece` sem `MasterEditing`, logo o `off_canvas` esconde-a da cena **e**
    /// da Hierarquia. O único caminho para ela era o *Edit Prefab* do **cartão** do navegador de
    /// assets — que exige saber o nome dela e ter aquele painel aberto. ⚠️ **E três recusas deste
    /// módulo mandavam o artista para lá** (*«edit it in the prefab»*), o que faz da ausência deste
    /// verbo um buraco que o próprio app já nomeava.
    ///
    /// ⭐ Ele **selecciona, e mais nada** — o `MasterEditing` é derivado da selecção
    /// ([`crate::master_editing`]), então pôr a selecção na raiz do mestre acende o
    /// canvas, arma o gizmo, enche o Inspector, e cada peça mexida chega a todas as cópias no mesmo
    /// quadro. *O verbo que faltava não era um modo nem uma janela: era um acesso.*
    Edit,
}

/// ⭐ **O dreno dos quatro** — resolve a entidade, corre o verbo e **responde ao artista**.
///
/// Devolve `true` quando alguma coisa mudou.
///
/// ⚠️ **Todo caminho negativo fala.** A tabela daquele menu é PLANA (ela não sabe o que a linha é),
/// então os quatro itens aparecem em toda linha; um item que come o clique em silêncio é pior que
/// um ausente. *É a mesma lei que o report do Enio sobre o Revert pagou.*
#[allow(clippy::too_many_arguments)] // um verbo, o mundo, o registo, o eco, os documentos e a voz
pub fn drain(
    verb: Verb,
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    echo: &mut crate::instance_sync::MasterEcho,
    entity_bits: u64,
    toasts: &mut ph2d_editor_core::ToastQueue,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
    // O passo da cascata, em unidades de MUNDO — ver [`cascade`].
    place_step: [f32; 2],
    select_out: &mut Option<u64>,
) -> bool {
    use ph2d_editor_core::Toast;
    let entity = Entity::from_bits(entity_bits);
    match verb {
        Verb::Make => {
            match make_master(sim, registry, entity, docs) {
                // ⭐ **A voz diz QUAL dos dois aconteceu** (F5): uma receita que nasce de uma copia
                // continua ligada a' base, e o artista tem de saber disso antes de a editar - senao ele
                // muda a variante e ve a mudanca sumir no quadro em que a base for editada.
                Ok((master, instance)) => {
                    let variant = sim.world().get::<ph2d_ecs::InstanceOf>(master).is_some();
                    // ⭐ A cópia é o objecto que o artista continua a ver — ver o doc de `select_out`.
                    *select_out = Some(instance.to_bits());
                    toasts.push(Toast::success(if variant {
                    ph2d_i18n::tr("app.components.instance_verbs.made_a_variant_it_still_follows_its_base")
                } else {
                    ph2d_i18n::tr("app.components.instance_verbs.made_a_prefab_an_instance_took_its_place")
                }));
                    true
                }
                Err(VerbRefusal::AlreadyAMaster) => {
                    toasts.push(Toast::info(ph2d_i18n::tr(
                        "app.components.instance_verbs.this_is_already_a_prefab",
                    )));
                    false
                }
                // ⚠️ **Braço por braço, e sem `_`**: a recusa nova (`InsideAMaster`) chegou aqui com um
                // catch-all que dizia *«Inside an instance»* — a frase errada sobre a coisa errada. Um
                // `match` exaustivo é o que obriga a próxima recusa a escolher a sua voz.
                Err(VerbRefusal::InsideAMaster) => {
                    toasts.push(Toast::warning(ph2d_i18n::tr(
                        "app.components.instance_verbs.inside_a_prefab_prefabs_cannot_nest_yet",
                    )));
                    false
                }
                Err(VerbRefusal::InsideAnInstance | VerbRefusal::NotAnInstance) => {
                    toasts.push(Toast::warning(
                    ph2d_i18n::tr("app.components.instance_verbs.inside_an_instance_detach_it_first_or_edit_the_p"),
                ));
                    false
                }
            }
        }
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
            let subject = crate::instance_verbs_walk::master_subject(sim, entity);
            let parent = sim.world().get::<ph2d_ecs::ChildOf>(subject).map(|c| c.0);
            match crate::instantiate::instantiate_master(sim, registry, subject, parent, docs, link)
            {
                Ok(inst) => {
                    if let Some(id) = sim.world().get::<StableId>(subject).map(|s| s.0) {
                        cascade(sim, inst, id, place_step);
                    }
                    // ⭐ **A cópia que acabou de nascer é o que o artista quer na mão** — e é ela
                    // que a QUEDA precisa de reposicionar (a pose escreve-se depois do verbo, que
                    // é o precedente do `cascade` logo acima).
                    *select_out = Some(inst.to_bits());
                    // ⚠️ **O toast diz QUAL das duas** — os dois itens do menu ficam um debaixo do
                    // outro e a diferença entre eles só se vê no gesto SEGUINTE (pintar, mover um
                    // nó). Uma confirmação igual para os dois deixaria o artista sem saber qual
                    // clicou.
                    toasts.push(Toast::success(if verb == Verb::PlaceLinked {
                        ph2d_i18n::tr("app.components.instance_verbs.instantiated_linked_its_art_follows_the_prefab_b")
                    } else {
                        ph2d_i18n::tr("app.components.instance_verbs.instantiated")
                    }));
                    true
                }
                Err(crate::instantiate::Refusal::WouldNestInItself) => {
                    toasts.push(Toast::warning(ph2d_i18n::tr(
                        "app.components.instance_verbs.that_would_put_the_prefab_inside_itself",
                    )));
                    false
                }
                Err(_) => {
                    toasts.push(Toast::warning(ph2d_i18n::tr(
                        "app.components.instance_verbs.not_a_prefab_pick_the_prefab_row",
                    )));
                    false
                }
            }
        }
        Verb::Detach => match detach(sim, entity) {
            Ok(n) => {
                toasts.push(Toast::success(ph2d_i18n::tr_with(
                    "app.components.instance_verbs.detached_piece_s_from_the_prefab",
                    &[("n", &n)],
                )));
                true
            }
            Err(_) => {
                toasts.push(Toast::warning(ph2d_i18n::tr(
                    "app.components.instance_verbs.not_part_of_an_instance",
                )));
                false
            }
        },
        Verb::Apply => match apply_to_master(sim, registry, echo, entity, docs) {
            Ok(0) => {
                toasts.push(Toast::info(ph2d_i18n::tr(
                    "app.components.instance_verbs.nothing_overridden_here",
                )));
                false
            }
            Ok(n) => {
                toasts.push(Toast::success(ph2d_i18n::tr_with(
                    "app.components.instance_verbs.applied_change_s_to_the_prefab",
                    &[("n", &n)],
                )));
                true
            }
            Err(_) => {
                toasts.push(Toast::warning(ph2d_i18n::tr(
                    "app.components.instance_verbs.not_part_of_an_instance",
                )));
                false
            }
        },
        // ⭐⭐ **Tirar da biblioteca.** ⚠️ **A voz diz QUAL das duas metades aconteceu** — elas
        // deixam a cena em estados diferentes (uma não muda nada do que se vê, a outra faz
        // aparecer um objecto), e um artista que lesse a mesma frase nas duas concluiria que a
        // segunda inventou uma cópia.
        // ⭐⭐⭐ **Abrir a receita a partir de uma cópia** — a lei vive em [`crate::instance_open`],
        // como a dos outros verbos com lei própria; aqui só o roteamento.
        Verb::Edit => crate::instance_open::open_prefab(sim, entity, toasts, select_out),
        Verb::Unmake => match crate::instance_unmake::unmake_master(sim, entity) {
            Ok(crate::instance_unmake::Unmade::Dissolved { copies }) => {
                toasts.push(Toast::success(ph2d_i18n::tr_with("app.components.instance_verbs.removed_from_library_copy_ies_are_now_independen", &[("copies", &copies)])));
                true
            }
            Ok(crate::instance_unmake::Unmade::Returned { root_bits }) => {
                // ⭐ Ela **apareceu**: a selecção segue-a, senão o artista fica com um objecto novo
                // na cena e nenhuma pista de qual é.
                *select_out = Some(root_bits);
                toasts.push(Toast::success(
                    ph2d_i18n::tr("app.components.instance_verbs.removed_from_library_it_had_no_copies_so_it_came"),
                ));
                true
            }
            Err(_) => {
                toasts.push(Toast::warning(ph2d_i18n::tr(
                    "app.components.instance_verbs.that_is_not_in_the_library",
                )));
                false
            }
        },
    }
}

// ⚠️ **As travessias partilhadas mudaram-se** para o irmão `instance_verbs_walk` — ver o
// cabeçalho de lá. Elas são re-exportadas aqui para os chamadores (e os gates) continuarem a
// escrevê-las como sempre.
pub use crate::instance_verbs_walk::{
    belongs_to_an_instance, entity_for_stable_id, instance_root_of, is_a_recipe_given_piece,
    stable_index, subtree,
};

#[cfg(test)]
#[path = "instance_verbs_tests.rs"]
mod tests;

/// ⚠️ **Onde a cópia ATERRA é outro assunto** — e o ficheiro acima estava no tecto de 600 LOC.
/// Aqui os gates dos verbos; lá a pergunta dos dois reports do Enio sobre a pose da cópia nova.
#[cfg(test)]
#[path = "instance_place_tests.rs"]
mod place_tests;

#[cfg(test)]
#[path = "instance_apply_tests.rs"]
mod apply_tests;

/// ⭐ **A CASCATA** — onde a cópia POUSA. Irmão por responsabilidade, e o corte que o tecto de LOC
/// impôs; o mecanismo (e o censo que os fez nascer) está no cabeçalho de lá.
#[cfg(test)]
#[path = "instance_verbs_cascade_tests.rs"]
mod cascade_tests;
