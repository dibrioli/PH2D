//! ⭐ **Os gates do ISOLAR** (W38) — e das duas vozes que faltavam.
//!
//! ⚠️ **A lei foi LIDA, não decidida:** o módulo irmão (`sculpt3d_objects::toggle_isolate`) já a
//! tinha escrita — toggle, e **nada entra na história**. O que esta linha acrescentou foi o
//! mecanismo, e ele acabou por não ser mecanismo nenhum: o `cook` já dizia *"coze a **subárvore** de
//! `root`"*, então isolar **é** cozer a partir daquele nó.

use bevy_ecs::entity::Entity;
use ph2d_ecs::SimWorld;
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};

/// `A ∪ (B − C)`, com o **grupo interno deslocado** — é o deslocamento que prova a pose da cadeia.
fn nested() -> FieldDoc {
    let ball = |x: f32, r: f32| Node {
        xform: Xform::at(x, 0.0, 0.0),
        kind: NodeKind::Leaf(Primitive::Sphere { radius: r }),
        mods: Vec::new(),
        verb: None,
    };
    FieldDoc::new(
        vec![
            ball(0.0, 0.25),
            ball(0.0, 0.2),
            ball(0.15, 0.15),
            Node {
                xform: Xform::at(1.2, -0.4, 0.3),
                kind: NodeKind::Combine {
                    op: Op::Difference(Blend::Sharp),
                    children: vec![NodeId(1), NodeId(2)],
                },
                mods: Vec::new(),
                verb: None,
            },
            Node {
                // ⚠️ **A RAIZ também sai da identidade, e isto é o gate a existir.** Com a raiz na
                // identidade, a pose LOCAL do grupo interno e a de MUNDO coincidem — e uma prova de
                // mutação que apagasse a composição da cadeia passava **verde**. *A fixture que
                // concorda é a que não prova nada.*
                xform: Xform::at(0.5, 0.25, -0.1),
                kind: NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0), NodeId(3)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(4),
    )
    .expect("o aninhado")
}

fn scene() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    crate::field3d_scene::sync_scene(&mut sim, Some(&nested()), 0.0);
    let world = sim.world_mut();
    let mut q = world.query::<(Entity, &ph2d_field_ecs::FieldObject)>();
    let root = q.iter(world).next().map(|(e, _)| e).expect("a peça");
    (sim, root)
}

fn inner_group(sim: &SimWorld, root: Entity) -> Entity {
    ph2d_field_ecs::walk(sim.world(), root)
        .into_iter()
        .map(|(e, _)| e)
        .find(|e| {
            matches!(
                sim.world()
                    .get::<ph2d_field_ecs::FieldNode>(*e)
                    .map(|n| &n.shape),
                Some(ph2d_field::NodeShape::Combine(_))
            ) && sim
                .world()
                .get::<bevy_ecs::hierarchy::ChildOf>(*e)
                .is_some()
        })
        .expect("o grupo interno")
}

fn leaves(doc: &FieldDoc) -> usize {
    doc.nodes()
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Leaf(_)))
        .count()
}

/// ⭐ **O GATE-MÃE: cozer a partir do nó isolado mostra SÓ aquela subárvore.**
#[test]
fn isolating_shows_only_that_subtree_and_the_whole_part_comes_back() {
    let (mut sim, root) = scene();
    let group = inner_group(&sim, root);
    let whole = crate::field3d_scene::sync_scene(&mut sim, None, 0.0).expect("cozinha");
    assert_eq!(leaves(&whole), 3, "a peça inteira tem três formas");

    let from = crate::field3d_scene::cook_root(sim.world(), root, Some(group.to_bits()));
    let only = ph2d_field_ecs::cook(sim.world(), from)
        .expect("não vazia")
        .expect("válida");
    assert_eq!(
        leaves(&only),
        2,
        "isolado, só as duas formas DENTRO do grupo entram — a terceira é irmã e fica de fora"
    );

    // E sem isolamento nenhum, a peça inteira — byte a byte.
    let back = ph2d_field_ecs::cook(
        sim.world(),
        crate::field3d_scene::cook_root(sim.world(), root, None),
    )
    .expect("não vazia")
    .expect("válida");
    assert_eq!(back, whole, "a peça volta EXACTAMENTE como estava");
}

/// ⭐ **A peça isolada NÃO SALTA** — a metade que a linha nova do `cook` existe para cumprir.
///
/// ⚠️ Sem a pose da cadeia, o grupo interno (deslocado `1,2 · −0,4 · 0,3`) apareceria na origem, e
/// da cadeira isso lê como *"isolar mexeu no meu modelo"*.
#[test]
fn the_isolated_piece_stays_where_it_was() {
    let (sim, root) = scene();
    let group = inner_group(&sim, root);
    let from = crate::field3d_scene::cook_root(sim.world(), root, Some(group.to_bits()));
    let only = ph2d_field_ecs::cook(sim.world(), from)
        .expect("não vazia")
        .expect("válida");
    let top = &only.nodes()[only.root().0 as usize];
    let expected = ph2d_field_ecs::world_xform(sim.world(), group);
    for k in 0..3 {
        assert!(
            (top.xform.translation[k] - expected.translation[k]).abs() < 1e-6,
            "o eixo {k} saiu em {} e o nó está em {} no mundo",
            top.xform.translation[k],
            expected.translation[k]
        );
    }
    // ⭐ **O CONTROLE DA FIXTURE**, e ele foi escrito depois de uma prova de mutação passar verde:
    // se a pose LOCAL do nó e a de MUNDO coincidirem, apagar a composição da cadeia não muda nada e
    // este gate deixa de medir o que diz medir.
    let local = sim
        .world()
        .get::<ph2d_field_ecs::FieldPose>(group)
        .expect("o grupo tem pose")
        .xform;
    assert!(
        (local.translation[0] - expected.translation[0]).abs() > 1e-3,
        "a fixture TEM de ter a pose local ({}) diferente da de mundo ({}) — senão a composição \
         da cadeia é indistinguível de não haver composição nenhuma",
        local.translation[0],
        expected.translation[0]
    );
}

/// ⭐ **UM ISOLAMENTO PENDURADO NUMA ENTIDADE MORTA é LARGADO** — e a peça inteira volta.
///
/// ⚠️ Os bits de entidade morrem num undo (o restore respawna tudo com ids novos). Obedecer a um
/// alvo que já não existe apagaria a peça da tela **sem nada a explicar** — o modo de falha exacto
/// que este módulo já pagou cinco vezes com outro nome.
#[test]
fn an_isolation_pinned_to_a_dead_object_is_dropped() {
    let (mut sim, root) = scene();
    let group = inner_group(&sim, root);
    let bits = group.to_bits();
    assert!(ph2d_field_ecs::remove(sim.world_mut(), group), "o nó some");

    let from = crate::field3d_scene::cook_root(sim.world(), root, Some(bits));
    assert_eq!(
        from, root,
        "com o alvo morto, o cozimento tem de voltar à peça inteira"
    );
}

/// ⭐ **A LEI DO TOGGLE**, dirigida sem estado nenhum — ver [`crate::field3d_smoke::next_isolation`].
#[test]
fn the_isolation_law_toggles_swaps_and_refuses_nothing() {
    use crate::field3d_smoke::next_isolation;
    // Isolar «nada» é recusado: apagaria a cena sem nada para devolver.
    assert_eq!(next_isolation(None, None), None);
    assert_eq!(
        next_isolation(Some(7), None),
        Some(7),
        "e não sai por engano"
    );
    // Entra.
    assert_eq!(next_isolation(None, Some(7)), Some(7));
    // A MESMA porta sai — não há um «sair» separado.
    assert_eq!(next_isolation(Some(7), Some(7)), None);
    // Isolar outro TROCA: o gesto é "mostra-me este", e sair primeiro seriam dois gestos.
    assert_eq!(next_isolation(Some(7), Some(9)), Some(9));
}

/// ⭐ **ALGUÉM DIZ QUE O GRUPO NASCEU** (W38, a outra metade).
///
/// ⚠️ A W31 fez o gesto e deixou-o **mudo**: a Hierarquia ganha uma linha nova, o objeto escolhido
/// passa a estar um nível abaixo, e nada na tela explica porquê. O aviso diz **quantos** entraram,
/// que é o que distingue *"criei um grupo com esta forma"* de *"embrulhei as três"*.
#[test]
fn a_born_group_says_so() {
    let _ = ph2d_panel_model3d::drain_intents();
    crate::field3d_notice::clear();
    let _ = crate::field3d_notice::drain();

    let (mut sim, root) = scene();
    let leaf = ph2d_field_ecs::walk(sim.world(), root)
        .into_iter()
        .map(|(e, _)| e)
        .find(|e| {
            matches!(
                sim.world()
                    .get::<ph2d_field_ecs::FieldNode>(*e)
                    .map(|n| &n.shape),
                Some(ph2d_field::NodeShape::Leaf(_))
            ) && sim
                .world()
                .get::<bevy_ecs::hierarchy::ChildOf>(*e)
                .is_some_and(|c| c.0 == root)
        })
        .expect("uma folha filha da raiz");

    ph2d_panel_model3d::state::push_intent_for_test(ph2d_panel_model3d::ModelIntent::ApplyOp {
        slot: 0,
    });
    crate::field3d_scene::sync_scene_and_birth(
        &mut sim,
        None,
        &[leaf],
        0.0,
        &crate::field3d_scene::no_drawing(),
    );

    let said = crate::field3d_notice::drain();
    assert!(
        said.iter().any(|m| m.contains("Group created")),
        "criar um grupo tem de DIZER que ele nasceu; o módulo disse {said:?}"
    );
}

// ─────────────────────── W39: a escultura da CENA (sem disco) ───────────────────────

/// ⭐ **A escultura da cena só é oferecível quando há cena esculpida** — a lei da W34 aplicada à
/// única forma cuja disponibilidade não é constante.
///
/// ⚠️ **Ela mede o CATÁLOGO desde a W100**, e não mais a fileira do painel: a fileira virou um botão
/// que abre a paleta, e a filtragem mudou-se para lá. A lei é a mesma; o sujeito é que deixou de ser
/// uma lista de chips.
#[test]
fn the_scene_sculpture_button_appears_only_when_there_is_one() {
    use crate::field3d_shapes::{SHAPES, available, slot_of};

    let cena = slot_of("panel.model3d.add.sculpt_scene").expect("a escultura da cena existe");
    // ⚠️ **O perfil fica LIGADO nos dois lados**, de propósito: este gate mede a filtragem da
    // escultura da cena, e deixar a outra a variar junto mediria as duas ao mesmo tempo — um gate
    // que muda de significado quando outra feature entra.
    assert!(
        !available(&SHAPES[cena], false, true),
        "sem escultura na cena, ela não é oferecível"
    );
    assert!(
        available(&SHAPES[cena], true, true),
        "com escultura na cena, ela é"
    );
    // ⚠️ **E o CONTROLE**: nenhuma outra forma muda de disponibilidade com essa condição — sem ele,
    // um `available` que devolvesse `live_sculpt` para tudo passaria a metade de cima.
    let mudam = SHAPES
        .iter()
        .filter(|s| available(s, false, true) != available(s, true, true))
        .count();
    assert_eq!(mudam, 1, "só a escultura da cena depende de haver uma");
}

/// ⚠️ **As duas esculturas são duas formas distintas, e cada uma faz a sua coisa.**
///
/// ⭐ **Este gate mudou de sujeito na W100, e a mudança é o ganho:** ele defendia duas constantes
/// derivadas do fim da lista (`SHAPES.len() - 2` e `- 1`), onde um `-2` trocado por `-1` faria o
/// botão do arquivo e o da cena serem o **mesmo** slot — o diálogo abriria no botão errado, sem erro
/// nenhum. Hoje cada linha traz o próprio [`crate::field3d_shapes::Make`], então a colisão que ele
/// defendia **não é exprimível**; o que sobra medir é que as duas continuam a existir e a ser
/// diferentes.
#[test]
fn the_two_sculpture_slots_are_distinct_and_in_range() {
    use crate::field3d_shapes::{Make, SHAPES, slot_of};
    let arquivo = slot_of("panel.model3d.add.sculpt").expect("a do arquivo existe");
    let cena = slot_of("panel.model3d.add.sculpt_scene").expect("a da cena existe");
    assert_ne!(arquivo, cena);
    assert!(matches!(SHAPES[arquivo].make, Make::Sculpt));
    assert!(matches!(SHAPES[cena].make, Make::SculptScene));
}

/// ⚠️ **A chave da cena e o prefixo que o resolvedor procura têm de casar.** São dois literais, e
/// dois literais divergem: o dia em que a chave virasse `live:sculpt` o resolvedor voltaria a
/// tentar lê-la **do disco** e o aviso mandaria o artista procurar um arquivo que nunca existiu.
#[test]
fn the_scene_key_is_the_one_the_resolver_recognises() {
    assert!(
        crate::field3d_import::SCENE_KEY.starts_with(crate::field3d_import::SCENE_PREFIX),
        "a chave `{}` tem de começar pelo prefixo `{}`",
        crate::field3d_import::SCENE_KEY,
        crate::field3d_import::SCENE_PREFIX
    );
}

/// ⭐ **UMA CHAVE `scene:` NÃO É LIDA DO DISCO** — ela é **pedida** ao shell.
///
/// ⚠️ Sem esta metade, reabrir um projeto cuja peça usa a escultura da cena dava *"Sculpture
/// scene:sculpt is missing"* — um aviso a mandar o artista procurar um arquivo que **nunca
/// existiu**. O caminho certo é a caixa de correio: quem tem a escultura viva é o shell.
#[test]
fn a_scene_key_is_asked_for_instead_of_being_read_from_disk() {
    crate::field3d_reload::forget_tried();
    let _ = crate::field3d_smoke::take_scene_sculpt_request();

    let doc = FieldDoc::new(
        vec![Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Sampled {
                key: crate::field3d_import::SCENE_KEY.to_string(),
            },
            mods: Vec::new(),
            verb: None,
        }],
        NodeId(0),
    )
    .expect("uma escultura da cena");

    let failed = crate::field3d_reload::resolve_missing(&doc);
    assert!(
        failed.is_empty(),
        "uma chave da cena não pode ser reportada como arquivo em falta; disse {failed:?}"
    );
    assert!(
        crate::field3d_smoke::take_scene_sculpt_request(),
        "…ela tem de PEDIR a escultura ao shell"
    );
}

// ─────────────── W44: «isolado» é um ESTADO — ele diz-se, e sai-se de qualquer sítio ───────────────

/// Arma o módulo e devolve-o ao repouso no fim — ver [`crate::field3d_view`].
///
/// ⚠️ Só possível desde a W42: enquanto a bandeira do pill travava ligada, armar num gate
/// contaminava todos os que corressem depois.
fn armed<R>(f: impl FnOnce() -> R) -> R {
    crate::field3d_smoke::set_armed_by_panel(true);
    let out = f();
    crate::field3d_smoke::forget_isolation();
    crate::field3d_smoke::set_armed_by_panel(false);
    let _ = crate::field3d_smoke::with_smoke(|_| ());
    out
}

/// ⭐⭐ **A LEI DA TECLA** — e ela não é a do chip.
///
/// ⚠️ As duas mexem no mesmo campo e respondem a perguntas diferentes: o chip está **numa linha**
/// (*"mostra-me ESTE"* ⇒ troca) e a tecla é **global** (*"dentro ou fora"* ⇒ sai). A linha que as
/// separa é a terceira deste gate, e é a que um `next_isolation` reaproveitado leria ao contrário.
#[test]
fn the_key_law_is_a_global_in_or_out_never_a_swap() {
    use crate::field3d_smoke::{key_isolation, next_isolation};
    assert_eq!(key_isolation(None, Some(7)), Some(7), "entra no escolhido");
    assert_eq!(key_isolation(Some(7), Some(7)), None, "e a mesma tecla sai");
    assert_eq!(
        key_isolation(Some(7), Some(9)),
        None,
        "com OUTRO escolhido a tecla SAI — ela é global; trocar é o que o chip da linha faz"
    );
    assert_ne!(
        key_isolation(Some(7), Some(9)),
        next_isolation(Some(7), Some(9)),
        "as duas leis TÊM de divergir aqui: se convergissem, uma delas estaria a mentir sobre o \
         gesto que a chamou"
    );
    assert_eq!(
        key_isolation(Some(7), None),
        None,
        "⭐ A PORTA DE SAÍDA: sem nada escolhido a tecla ainda devolve a peça — é este caso que o \
         chip da fileira não consegue exprimir"
    );
    assert_eq!(key_isolation(None, None), None, "e não há o que isolar");
}

/// ⭐⭐ **A PEÇA ISOLADA TEM SEMPRE VOLTA** — a lei que o chip prometia e não cumpria.
///
/// ⚠️ A razão escrita para o isolamento ser um *toggle* era *"um «sair» separado seria a porta que o
/// artista não acha quando a cena some"*. Mas o único gesto de sair vivia num **chip da fileira de
/// ações**, e essa fileira só é pintada quando o escolhido se destaca da peça: com a **raiz**
/// escolhida ela desaparece inteira. *A porta existia e escondia-se exactamente no caso em que a
/// cena some.*
#[test]
fn an_isolated_part_always_has_a_way_back() {
    armed(|| {
        let (mut sim, root) = scene();
        let group = inner_group(&sim, root);
        assert!(
            crate::field3d_smoke::toggle_isolate(Some(group.to_bits())),
            "isolou"
        );

        // O artista escolhe a RAIZ — e a fileira que tinha o chip desaparece.
        crate::field3d_scene::sync_scene_and_birth(
            &mut sim,
            None,
            &[root],
            0.0,
            &crate::field3d_scene::no_drawing(),
        );
        assert!(
            ph2d_panel_model3d::state::current().acts.is_empty(),
            "o controle do gate: com a raiz escolhida a fileira de ações não é pintada"
        );

        // A tecla responde na mesma.
        crate::field3d_smoke::ask_isolate_key();
        crate::field3d_scene::sync_scene_and_birth(
            &mut sim,
            None,
            &[root],
            0.0,
            &crate::field3d_scene::no_drawing(),
        );
        assert_eq!(
            crate::field3d_smoke::isolated(),
            None,
            "sem a tecla, uma peça isolada com a raiz escolhida NÃO tem gesto nenhum que a devolva"
        );
    });
}

/// ⭐⭐ **O ISOLAMENTO DIZ-SE, E NÃO PELA SELEÇÃO.**
///
/// ⚠️ O único sinal anterior era o `active` do chip *Isolate*, e ele compara o nó isolado com o
/// **escolhido**: isolar `A` e escolher `B` apagava-o. Metade da peça fora de vista, por decisão de
/// alguém, e nada na tela a dizê-lo. *Um estado da VISTA não se anuncia por um controle da SELEÇÃO.*
#[test]
fn the_isolation_announces_itself_whatever_is_selected() {
    armed(|| {
        let (mut sim, root) = scene();
        let group = inner_group(&sim, root);
        let other = ph2d_field_ecs::walk(sim.world(), root)
            .into_iter()
            .map(|(e, _)| e)
            .find(|e| *e != group && *e != root)
            .expect("a fixture tem mais nós");
        assert!(crate::field3d_smoke::toggle_isolate(Some(group.to_bits())));

        for (what, sel) in [("outro nó", vec![other]), ("a raiz", vec![root])] {
            crate::field3d_scene::sync_scene_and_birth(
                &mut sim,
                None,
                &sel,
                0.0,
                &crate::field3d_scene::no_drawing(),
            );
            let snap = ph2d_panel_model3d::state::current();
            assert!(
                snap.isolated.is_some(),
                "com {what} escolhido o painel deixou de dizer que há um isolamento em curso — e é \
                 aí que o artista precisa de o ler"
            );
        }

        // …e cala-se quando a peça inteira volta.
        crate::field3d_smoke::forget_isolation();
        crate::field3d_scene::sync_scene_and_birth(
            &mut sim,
            None,
            &[root],
            0.0,
            &crate::field3d_scene::no_drawing(),
        );
        assert_eq!(
            ph2d_panel_model3d::state::current().isolated,
            None,
            "sem isolamento o painel não pode anunciar um — a metade da AUSÊNCIA"
        );
    });
}

/// ⚠️ **Um isolamento pendurado numa entidade MORTA não se anuncia** — a irmã da lei do cozimento.
///
/// Os bits morrem num undo, e o mundo novo realoca-os. Anunciar o nome resolvido por bits mortos
/// diria ou um nome que já não está na Hierarquia, ou — pior — o de outro nó que os herdou.
#[test]
fn a_dead_isolation_is_not_announced() {
    armed(|| {
        let (mut sim, root) = scene();
        let group = inner_group(&sim, root);
        assert!(crate::field3d_smoke::toggle_isolate(Some(group.to_bits())));
        assert!(ph2d_field_ecs::remove(sim.world_mut(), group), "o nó some");

        crate::field3d_scene::sync_scene_and_birth(
            &mut sim,
            None,
            &[root],
            0.0,
            &crate::field3d_scene::no_drawing(),
        );
        assert_eq!(
            ph2d_panel_model3d::state::current().isolated,
            None,
            "o cozimento já larga o alvo morto; a VOZ tem de largar com ele"
        );
    });
}

/// ⭐ **Quantas linhas dizem `ISO`** — e é isto que os dois gates abaixo sempre quiseram medir.
///
/// ⚠️ **Eles perguntavam ao mapa INTEIRO** (`badges.len() == 1`, `is_empty()`), e isso valia
/// enquanto o isolamento e o vínculo eram os únicos selos. A W97 pôs o **verbo** em toda linha que
/// participa da receita, e a premissa dissolveu: o mapa passou a ter `BSE`/`UNI`/`SUB` de propósito.
///
/// ⛔ **Não é baixar a barra — é dizer com precisão o que a barra sempre foi.** A afirmação é *«uma
/// linha, e só uma, diz que está isolada»*, e ela continua a apanhar o defeito que a escreveu
/// (selar todas) e o da irmã (selar uma entidade morta). *Uma mudança de modelo obriga a
/// re-perguntar o que cada gate ainda mede.*
fn isolados() -> Vec<u64> {
    crate::field3d_scene::link_badges()
        .into_iter()
        .filter(|(_, b)| *b == crate::field3d_scene::acts::ISOLATE_BADGE)
        .map(|(e, _)| e)
        .collect()
}

/// ⭐⭐⭐ **A HIERARQUIA DIZ QUAL LINHA ESTÁ ISOLADA** (2026-08-25).
///
/// ⚠️ O painel do MODEL já o dizia desde a W44 — mas a Hierarquia, que é onde o artista olha quando
/// pergunta *"por que só isto aparece?"*, não dizia nada. ⛔ *Um estado que esconde trabalho e não
/// se anuncia onde a ausência se vê é uma armadilha, não uma feature.*
///
/// ⚠️ **As duas metades**: o selo aparece na linha certa **e** cai quando a peça inteira volta. Sem
/// a segunda, um selo pousado acusaria um isolamento que já não existe.
#[test]
fn the_hierarchy_says_which_row_is_isolated() {
    armed(|| {
        let (mut sim, root) = scene();
        let group = inner_group(&sim, root);
        assert!(crate::field3d_smoke::toggle_isolate(Some(group.to_bits())));
        crate::field3d_scene::sync_scene_and_birth(
            &mut sim,
            None,
            &[root],
            0.0,
            &crate::field3d_scene::no_drawing(),
        );
        let badges = crate::field3d_scene::link_badges();
        assert_eq!(
            badges.get(&group.to_bits()).copied(),
            Some(crate::field3d_scene::acts::ISOLATE_BADGE),
            "a linha isolada tem de o dizer na Hierarquia"
        );
        assert_eq!(
            isolados(),
            vec![group.to_bits()],
            "e só ela — selar as outras seria dizer que TODAS estão isoladas: {badges:?}"
        );

        crate::field3d_smoke::forget_isolation();
        crate::field3d_scene::sync_scene_and_birth(
            &mut sim,
            None,
            &[root],
            0.0,
            &crate::field3d_scene::no_drawing(),
        );
        assert!(
            isolados().is_empty(),
            "com a peça inteira de volta o selo tem de cair — senão ele acusa um isolamento que \
             já não existe"
        );
    });
}

/// ⭐⭐ **UM SELO PENDURADO NUMA ENTIDADE MORTA NÃO SE PINTA** — a irmã da lei do anúncio.
///
/// ⚠️ Os bits morrem num undo e o mundo novo realoca-os. Um selo resolvido por bits mortos marcaria
/// uma linha que já não é aquela — ou, pior, a de outro nó que os herdou. ⇒ o selo pergunta à
/// **travessia**, exactamente como o [`crate::field3d_scene::panel`] faz para o nome.
#[test]
fn a_badge_pinned_to_a_dead_object_is_not_painted() {
    armed(|| {
        let (mut sim, root) = scene();
        // Uns bits que ninguém tem: o isolamento fica pendurado no nada.
        assert!(crate::field3d_smoke::toggle_isolate(Some(u64::MAX)));
        crate::field3d_scene::sync_scene_and_birth(
            &mut sim,
            None,
            &[root],
            0.0,
            &crate::field3d_scene::no_drawing(),
        );
        assert!(
            isolados().is_empty(),
            "um isolamento pendurado numa entidade morta não pode selar linha nenhuma"
        );
    });
}
