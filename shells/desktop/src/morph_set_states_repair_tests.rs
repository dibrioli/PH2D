//! ⭐⭐⭐ **A REPARAÇÃO das poses quando uma forma sai do conjunto** (plano 32 W11h) — irmão de
//! [`super`] pelo teto de 600 LOC.
//!
//! Enio, 2026-08-26, 4.º report: *"com um morph states com 3 shapes dentro de uma animação de
//! States, desconectei uma shape do morph state e quebrou a animação do state. (…) se o usuário
//! desconectar uma shape, coloque outra shape do conjunto em seu lugar de modo a não quebrar as
//! anims."*
//!
//! ⚠️ **Duas metades, e as duas são precisas:** o **gesto** substitui a forma nas poses (o ⊘, que é
//! explícito e desfazível), e o **consumidor** ignora um passo cuja ponta não é estado — porque o ⊘
//! não é a única rota para uma forma sair (arrastar na Hierarquia também tira).

use super::super::super::{create, upkeep};
use super::super::world;
use ph2d_ecs::{Entity, VecMorph};

use crate::vec_entities::sync;

/// **A cena do report, montada pela porta real**: `wide`/`tall`/`thin` num conjunto, com o
/// `Default` no `wide` e o `Hover` numa forma à escolha.
fn bench(
    hover: usize,
) -> (
    ph2d_ecs::SimWorld,
    ph2d_vec_scene::VecScene,
    crate::vec_entities::VecEntityMap,
    Vec<ph2d_vec_scene::VecPathId>,
    ph2d_vec_scene::VecPathId,
    ph2d_ui_state::StateSets,
) {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let host_id = scene.paths().last().unwrap().id;

    let mut states = ph2d_ui_state::StateSets::default();
    for (role, shape) in [
        (ph2d_ui_state::StateRole::Default, ids[0]),
        (ph2d_ui_state::StateRole::Hover, ids[hover]),
    ] {
        let mut st = ph2d_ui_state::UiState::new(role);
        st.objects = crate::vec_ui_state_edit::members(&sim, &scene, &map, host_id)
            .into_iter()
            .map(|id| crate::vec_ui_state_edit::capture(&sim, &scene, &map, id))
            .collect();
        for p in &mut st.objects {
            if p.id == host_id {
                p.morph_shape = Some(shape);
            }
        }
        states.set(host_id, st);
    }
    (sim, scene, map, ids, host_id, states)
}

/// A forma que um papel nomeia.
fn shape_of(
    states: &ph2d_ui_state::StateSets,
    host_id: ph2d_vec_scene::VecPathId,
    role: ph2d_ui_state::StateRole,
) -> Option<ph2d_vec_scene::VecPathId> {
    states
        .role(host_id, role)
        .and_then(|st| st.objects.iter().find(|p| p.id == host_id))
        .and_then(|p| p.morph_shape)
}

/// ⭐⭐⭐ **AS TRÊS ROTAS QUE TIRAM UMA FORMA DO CONJUNTO ARRUMAM A ANIMAÇÃO** (plano 32 W11h+W11i).
///
/// Enio, 2026-08-26: *"eu tinha no morph state wide, tall e thin. criei a anim state com wide e
/// thin. Desconectei thin. Na animação tall deveria ter sido colocada no lugar de thin. Isso deve
/// acontecer para quando desconectar do morph state ou quando **deletar** a shape que participa do
/// morph ou se o usuário **mexer na hierarquia** movendo uma das shapes para fora."*
///
/// ⛔⛔ **A W11h curou UMA das três**, porque pôs a substituição dentro do gesto ⊘ — e o ⊘ é a
/// única que passa por uma função. ⇒ a arrumação mudou-se para a **derivação**
/// (`morph_machine_drive::reconcile`, tarde no quadro), e as três ficam cobertas **sem que nenhuma
/// tenha código a reagir**. *Curar no gesto cura um caminho; curar na derivação cura a pergunta.*
///
/// ⭐ **A substituta é uma que NENHUM outro estado nomeia** — pôr a do `Default` no `Hover` deixaria
/// os dois iguais, e a animação sobreviveria ao ficheiro para **morrer na tela**.
///
/// **Mutação que deve sangrar:** o `reconcile` não chamar o `repair_states`, a escolha ignorar o
/// que já está em uso, ou a substituição tocar poses que não nomeavam a forma que saiu.
#[test]
fn every_route_that_removes_a_shape_repairs_the_states_animation() {
    for route in ["⊘ disconnect", "apagar a forma", "arrastar para fora"] {
        // `Hover` no THIN (a 3.ª) — o caso exacto do report.
        let (mut sim, mut scene, mut map, ids, host_id, mut states) = bench(2);
        let host = Entity::from_bits(map[&host_id]);
        assert_eq!(
            shape_of(&states, host_id, ph2d_ui_state::StateRole::Hover),
            Some(ids[2]),
            "{route}: a fixtura perdeu a premissa"
        );

        match route {
            "⊘ disconnect" => {
                let row = crate::morph_set::graph_of(&sim, &map, host)
                    .shapes()
                    .iter()
                    .position(|s| *s == ids[2])
                    .expect("o thin esta' na lista");
                crate::morph_set::disconnect_row(&mut sim, &map, host, row);
            }
            "apagar a forma" => {
                scene.remove_path(ids[2]);
                sync(&mut sim, &mut scene, &mut map);
            }
            _ => {
                sim.world_mut()
                    .entity_mut(Entity::from_bits(map[&ids[2]]))
                    .remove::<ph2d_ecs::ChildOf>();
            }
        }

        // ⭐ E o quadro arruma — a MESMA porta para as tres rotas.
        let mut machines = crate::morph_machine_drive::MorphMachines::new();
        crate::morph_machine_drive::reconcile(&mut machines, &mut sim, &scene, &map, &mut states);

        let members = crate::morph_set::graph_of(&sim, &map, host).shapes();
        let d = shape_of(&states, host_id, ph2d_ui_state::StateRole::Default);
        let h = shape_of(&states, host_id, ph2d_ui_state::StateRole::Hover);
        assert_eq!(
            h,
            Some(ids[1]),
            "{route}: o `tall` tinha de tomar o lugar do `thin` -- ficou {h:?}"
        );
        assert_eq!(d, Some(ids[0]), "{route}: o Default nao podia ser tocado");
        assert!(
            members.contains(&h.unwrap()),
            "{route}: a substituta nem sequer e' membro: {h:?} contra {members:?}"
        );
        assert_ne!(
            d, h,
            "{route}: os dois estados ficaram na MESMA forma -- a animacao morre na tela"
        );
    }
}

/// ⛔ **E a POSE do objecto que saiu vai junto**, pelas três rotas.
///
/// Um estado grava a **sub-árvore** com a pose **LOCAL** de cada filho: a pose antiga faria o
/// `install` do próximo Show atirar a forma solta para a **origem do conjunto**, no meio de uma
/// animação que já não é sobre ela.
///
/// **Mutação que deve sangrar:** o `repair_states` não chamar o `forget_object_in_all_states`.
#[test]
fn the_pose_of_the_departed_object_goes_with_it() {
    let (mut sim, scene, map, ids, host_id, mut states) = bench(2);
    let host = Entity::from_bits(map[&host_id]);
    assert!(
        states
            .role(host_id, ph2d_ui_state::StateRole::Default)
            .is_some_and(|s| s.objects.iter().any(|p| p.id == ids[1])),
        "a fixtura tem de gravar a forma-membro -- senao nao ha' o que vazar"
    );

    sim.world_mut()
        .entity_mut(Entity::from_bits(map[&ids[1]]))
        .remove::<ph2d_ecs::ChildOf>();
    let mut machines = crate::morph_machine_drive::MorphMachines::new();
    crate::morph_machine_drive::reconcile(&mut machines, &mut sim, &scene, &map, &mut states);

    assert!(
        !states
            .role(host_id, ph2d_ui_state::StateRole::Default)
            .is_some_and(|s| s.objects.iter().any(|p| p.id == ids[1])),
        "⛔ a forma solta ficou na tabela -- o proximo Show puxa-a para a origem do conjunto"
    );
    // O CONTROLE: o hospedeiro e a que ficou continuam la'.
    let left = states
        .role(host_id, ph2d_ui_state::StateRole::Default)
        .map_or(0, |s| s.objects.len());
    assert_eq!(left, 3, "so' a que saiu podia sair (host + 2 formas)");
    let _ = host;
}

/// ⛔⛔ **UM PASSO COM UMA PONTA QUE NÃO É ESTADO É IGNORADO** — a blindagem do consumidor (W11h).
///
/// O ⊘ substitui a forma que sai nas poses, mas **não é a única rota**: arrastar um membro para
/// FORA na Hierarquia tira-o do conjunto sem passar por lá, e uma pose gravada antes disso
/// continua a nomeá-lo.
///
/// ⚠️ **O dano de não guardar é COZER UM ESTRANHO:** a forma que saiu ainda existe na cena, mas o
/// `Transform` dela já é de MUNDO — e o `recook` de um conjunto lê as poses **locais** dos filhos.
/// O morfo sairia de um sítio que não é o dela nem o do conjunto.
///
/// ⇒ *não morfar é uma resposta; morfar a partir de um estranho não é.*
///
/// **Mutação que deve sangrar:** o `apply_ui_steps` largar a checagem de pertença.
#[test]
fn a_step_naming_a_non_member_is_ignored() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let host_id = scene.paths().last().unwrap().id;
    let host = Entity::from_bits(map[&host_id]);
    let mut drive = crate::preview_drive::PreviewDrive::default();

    // O CONTROLE POSITIVO primeiro: com as duas pontas membros, ele escreve.
    let good = ph2d_ui_state::MorphStep {
        id: host_id,
        from: ids[0],
        to: ids[2],
        t: 0.5,
    };
    assert_eq!(
        crate::morph_machine_drive::apply_ui_steps(&mut sim, &map, &[good], &mut drive),
        1
    );
    assert_eq!(
        sim.world().get::<VecMorph>(host).unwrap().sources,
        [ids[0], ids[2]]
    );

    // Agora a forma 1 sai pela Hierarquia -- `ChildOf` e mais nada, sem passar pelo ⊘.
    sim.world_mut()
        .entity_mut(Entity::from_bits(map[&ids[1]]))
        .remove::<ph2d_ecs::ChildOf>();
    let before = sim.world().get::<VecMorph>(host).unwrap().clone();
    let stale = ph2d_ui_state::MorphStep {
        id: host_id,
        from: ids[1],
        to: ids[2],
        t: 0.5,
    };
    assert_eq!(
        crate::morph_machine_drive::apply_ui_steps(&mut sim, &map, &[stale], &mut drive),
        0,
        "⛔ o motor aceitou cozer a partir de uma forma que ja' nao e' estado"
    );
    assert_eq!(
        sim.world().get::<VecMorph>(host).unwrap(),
        &before,
        "e o mundo tem de ficar EXACTAMENTE onde estava"
    );
}

/// ⭐⭐⭐ **DE PONTA A PONTA: tirar o `thin` e a animação continuar a MORFAR** — o gesto do Enio
/// inteiro, pela composição que o quadro corre.
///
/// ⛔ Os gates acima medem a **tabela** depois da arrumação. Este mede o que o artista vê: pedir o
/// `Hover` e a cena de facto andar de `wide` para `tall`. *Um gate sobre a tabela é cego ao que o
/// motor faz com ela.*
#[test]
fn after_removing_thin_the_hover_still_morphs_to_tall() {
    let (mut sim, mut scene, map, ids, host_id, mut states) = bench(2);
    let host = Entity::from_bits(map[&host_id]);
    let row = crate::morph_set::graph_of(&sim, &map, host)
        .shapes()
        .iter()
        .position(|s| *s == ids[2])
        .expect("o thin esta' na lista");
    crate::morph_set::disconnect_row(&mut sim, &map, host, row);
    let mut machines = crate::morph_machine_drive::MorphMachines::new();
    crate::morph_machine_drive::reconcile(&mut machines, &mut sim, &scene, &map, &mut states);

    // ⭐ E agora o quadro dos STATES: pedir o Hover e andar.
    let mut ui = crate::render_loop::ui_state_bridge::UiMachines::new();
    let mut cooked = crate::render_loop::ui_state_bridge::Cooked::default();
    crate::render_loop::ui_state_bridge::request(
        &mut ui,
        &states,
        host_id,
        ph2d_ui_state::StateRole::Hover,
    );
    crate::render_loop::ui_state_bridge::dispatch(
        &mut ui,
        &mut states,
        &mut sim,
        &mut scene,
        &map,
        0.05,
        &mut cooked,
    );
    assert_eq!(
        cooked.morph_steps.len(),
        1,
        "⛔ a transicao nao publicou passo nenhum -- a animacao morreu"
    );
    let st = cooked.morph_steps[0];
    assert_eq!(
        (st.from, st.to),
        (ids[0], ids[1]),
        "⛔ o passo tem de ir de `wide` para `tall`"
    );

    // E o motor tem de o ACEITAR (a blindagem da W11h nao pode recusar um passo legitimo).
    let mut drive = crate::preview_drive::PreviewDrive::default();
    assert_eq!(
        crate::morph_machine_drive::apply_ui_steps(&mut sim, &map, &cooked.morph_steps, &mut drive),
        1,
        "⛔ o motor RECUSOU o passo -- a blindagem esta' a morder um par legitimo"
    );
    let m = sim.world().get::<VecMorph>(host).unwrap();
    assert_eq!(m.sources, [ids[0], ids[1]]);
    assert!(
        m.t > 0.0 && m.t < 1.0,
        "e a cena tem de estar A MEIO: {}",
        m.t
    );
}

/// ⛔⛔⛔ **O CONJUNTO PODE SER UMA PEÇA DE UM WIDGET MAIOR — e a animação vive no PAI.**
///
/// Enio, 2026-08-26, com o log ligado:
///
/// ```text
/// [morph] CLIQUE ⊘ row=2 conjunto=Some(3) chaves-da-tabela=[4] formas=[0, 1, 2]
/// [morph]   path 3 nome="Morph States 3" morph=true maquina=true pai=Some(4)
/// ```
///
/// ⇒ o conjunto era **filho** de outra forma, e o `Rec` gravou no **PAI** — que é a lei escrita do
/// `host_of_selection` (*"a forma-ancestral mais próxima cuja sub-árvore contém todas"*). A pose
/// que diz **qual forma o conjunto mostra** vivia, portanto, na tabela do PAI.
///
/// ⛔ **E toda a W11i procurava no sítio errado:** `states.get(h)` com `h` = o próprio conjunto,
/// saindo no `is_empty()` sem fazer nada. *Um conjunto de Morph States não é necessariamente o dono
/// da animação que o usa: ele pode ser uma PEÇA de um widget maior.*
///
/// ⚠️ **Nenhum gate podia ver isto**, e a razão é a fixtura: todos punham o conjunto na RAIZ, onde
/// a chave da tabela e o id do conjunto são o mesmo **por construção**. *Uma fixtura que não contém
/// o fenómeno aprova a cura errada* — e aqui aprovou quatro waves seguidas.
///
/// **Mutação que deve sangrar:** o `repair_states` voltar a procurar só em `states.get(h)`.
#[test]
fn the_set_can_be_a_child_and_the_animation_lives_in_the_parent() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let set_id = scene.paths().last().unwrap().id;

    // ⭐ A FIXTURA QUE FALTAVA: uma forma PAI, com o conjunto por baixo dela.
    let parent_id = scene.push_path(ph2d_vec_scene::rectangle([-9.0, -9.0], [-7.0, -7.0]));
    sync(&mut sim, &mut scene, &mut map);
    let set = Entity::from_bits(map[&set_id]);
    let parent = Entity::from_bits(map[&parent_id]);
    sim.world_mut()
        .entity_mut(set)
        .insert(ph2d_ecs::ChildOf(parent));

    // O `Rec` grava no PAI -- e a sub-arvore dele contem o conjunto.
    let governed = crate::vec_ui_state_edit::members(&sim, &scene, &map, parent_id);
    assert!(
        governed.contains(&set_id),
        "a fixtura perdeu a premissa: o pai tem de governar o conjunto"
    );
    let mut states = ph2d_ui_state::StateSets::default();
    for (role, shape) in [
        (ph2d_ui_state::StateRole::Default, ids[0]),
        (ph2d_ui_state::StateRole::Hover, ids[2]),
    ] {
        let mut st = ph2d_ui_state::UiState::new(role);
        st.objects = governed
            .iter()
            .map(|&id| crate::vec_ui_state_edit::capture(&sim, &scene, &map, id))
            .collect();
        for p in &mut st.objects {
            if p.id == set_id {
                p.morph_shape = Some(shape);
            }
        }
        states.set(parent_id, st);
    }
    assert_eq!(
        states.hosts().collect::<Vec<_>>(),
        vec![parent_id],
        "a tabela tem de estar sob o PAI -- e' esse o caso que o Enio reportou"
    );

    // O ⊘ do `thin`, e o quadro a arrumar.
    let row = crate::morph_set::graph_of(&sim, &map, set)
        .shapes()
        .iter()
        .position(|s| *s == ids[2])
        .expect("o thin esta' na lista");
    crate::morph_set::disconnect_row(&mut sim, &map, set, row);
    let mut machines = crate::morph_machine_drive::MorphMachines::new();
    crate::morph_machine_drive::reconcile(&mut machines, &mut sim, &scene, &map, &mut states);

    let shape_in = |states: &ph2d_ui_state::StateSets, role| {
        states
            .role(parent_id, role)
            .and_then(|st| st.objects.iter().find(|p| p.id == set_id))
            .and_then(|p| p.morph_shape)
    };
    assert_eq!(
        shape_in(&states, ph2d_ui_state::StateRole::Hover),
        Some(ids[1]),
        "⛔ o `tall` tinha de tomar o lugar do `thin` na tabela do PAI"
    );
    assert_eq!(
        shape_in(&states, ph2d_ui_state::StateRole::Default),
        Some(ids[0]),
        "o Default nao podia ser tocado"
    );
    // E a pose da forma que saiu tambem vai junto -- ela deixou a sub-arvore do PAI.
    assert!(
        !states
            .role(parent_id, ph2d_ui_state::StateRole::Default)
            .is_some_and(|s| s.objects.iter().any(|p| p.id == ids[2])),
        "⛔ a pose do `thin` ficou na tabela do pai"
    );
}

/// ⛔⛔ **A ARRUMAÇÃO NÃO TOCA A TABELA DE QUEM NÃO GOVERNA O CONJUNTO.**
///
/// A varredura procura a pose do conjunto em **toda** tabela — porque ela pode viver na do PAI — e
/// é por isso que a guarda de posse existe: sem ela, arrumar UM conjunto varreria as tabelas de
/// **widgets que nada têm com ele**, e apagaria poses de objectos que legitimamente saíram e voltam
/// à sub-árvore deles. *Varrer é a metade fácil; saber onde parar é a que protege autoria.*
///
/// **Mutação que deve sangrar:** largar o `if !live.contains(&h) { continue }`.
#[test]
fn the_repair_never_touches_a_table_that_does_not_govern_the_set() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let set_id = scene.paths().last().unwrap().id;
    let set = Entity::from_bits(map[&set_id]);

    // ⭐ UM WIDGET ALHEIO, com uma pose de um objecto que ja' saiu da sub-arvore DELE.
    let widget = scene.push_path(ph2d_vec_scene::rectangle([20.0, 20.0], [22.0, 22.0]));
    let stranger = scene.push_path(ph2d_vec_scene::rectangle([30.0, 30.0], [32.0, 32.0]));
    sync(&mut sim, &mut scene, &mut map);
    let mut states = ph2d_ui_state::StateSets::default();
    let mut st = ph2d_ui_state::UiState::new(ph2d_ui_state::StateRole::Default);
    st.objects = vec![
        crate::vec_ui_state_edit::capture(&sim, &scene, &map, widget),
        crate::vec_ui_state_edit::capture(&sim, &scene, &map, stranger),
    ];
    states.set(widget, st);
    assert_eq!(
        states
            .role(widget, ph2d_ui_state::StateRole::Default)
            .unwrap()
            .objects
            .len(),
        2,
        "a fixtura tem de conter o fenomeno: uma pose de quem nao esta' na sub-arvore"
    );

    // Arrumar o CONJUNTO nao pode mexer no widget alheio.
    crate::morph_set::disconnect_row(&mut sim, &map, set, 0);
    let mut machines = crate::morph_machine_drive::MorphMachines::new();
    crate::morph_machine_drive::reconcile(&mut machines, &mut sim, &scene, &map, &mut states);

    assert_eq!(
        states
            .role(widget, ph2d_ui_state::StateRole::Default)
            .unwrap()
            .objects
            .len(),
        2,
        "⛔ a arrumacao de UM conjunto apagou uma pose de um widget que nada tem com ele"
    );
}
