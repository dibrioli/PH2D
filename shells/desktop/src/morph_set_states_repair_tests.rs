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
