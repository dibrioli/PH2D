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

/// ⭐⭐⭐ **O 4.º REPORT DO ENIO: desconectar uma forma QUEBRAVA a animação de States.**
///
/// > *"com um morph states com 3 shapes dentro de uma animação de States, desconectei uma shape do
/// > morph state e quebrou a animação do state. (…) se o usuário desconectar uma shape, coloque
/// > outra shape do conjunto em seu lugar de modo a não quebrar as anims."*
///
/// ⛔⛔ **MEDIDO:** com `Default = forma 0` e `Hover = forma 1`, tirar a `0` deixava a tabela em
/// `Default = Some(0)` com os membros já em `[1, 2]` — e o `Transition::morph_steps` publicava
/// **`from: 0`**, ou seja o motor a cozer a partir de uma forma que saiu do conjunto e cujo
/// `Transform` já é de MUNDO, não do referencial dele.
///
/// ⚠️ **A W11g arrumou o MUNDO** (o par desenhado) e não podia arrumar isto: a pose é dado
/// **autorado**, e reescrevê-lo só é legítimo dentro de um gesto explícito do artista — que é
/// exactamente o que o ⊘ é.
///
/// ⭐ **A substituta é uma que NENHUM outro estado nomeia**: pôr a do `Hover` no `Default` deixaria
/// os dois iguais, e a animação sobreviveria ao ficheiro para **morrer na tela**.
///
/// **Mutação que deve sangrar:** o `disconnect_row` não chamar o `replace_morph_shape_in_all_states`,
/// ou ele escolher `candidates.first()` sem olhar para o que já está em uso.
#[test]
fn disconnecting_a_shape_does_not_break_the_states_animation() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let host_id = scene.paths().last().unwrap().id;
    let host = Entity::from_bits(map[&host_id]);

    let mut states = ph2d_ui_state::StateSets::default();
    for (role, shape) in [
        (ph2d_ui_state::StateRole::Default, ids[0]),
        (ph2d_ui_state::StateRole::Hover, ids[1]),
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
    let shape_of = |s: &ph2d_ui_state::StateSets, r| {
        s.role(host_id, r)
            .and_then(|st| st.objects.iter().find(|p| p.id == host_id))
            .and_then(|p| p.morph_shape)
    };

    crate::morph_set::disconnect_row(&mut sim, &map, &mut states, host, 0);

    let members = crate::morph_set::graph_of(&sim, &map, host).shapes();
    let (d, h) = (
        shape_of(&states, ph2d_ui_state::StateRole::Default),
        shape_of(&states, ph2d_ui_state::StateRole::Hover),
    );
    assert!(
        d.is_some_and(|s| members.contains(&s)),
        "⛔ o Default continua a nomear uma forma que saiu do conjunto: {d:?} contra {members:?}"
    );
    assert_eq!(h, Some(ids[1]), "o Hover nao podia ser tocado");
    assert_ne!(
        d, h,
        "⛔ os dois estados ficaram na MESMA forma -- a animacao sobrevive ao ficheiro e morre na tela"
    );

    // ⭐ E a transicao volta a ser um morfo REAL entre dois membros.
    let a = states
        .role(host_id, ph2d_ui_state::StateRole::Default)
        .unwrap()
        .objects
        .clone();
    let b = states
        .role(host_id, ph2d_ui_state::StateRole::Hover)
        .unwrap()
        .objects
        .clone();
    let steps = ph2d_ui_state::Transition::new(&a, &b).morph_steps(0.5);
    assert_eq!(
        steps.len(),
        1,
        "a transicao deixou de publicar passo nenhum"
    );
    assert!(
        members.contains(&steps[0].from) && members.contains(&steps[0].to),
        "⛔ o passo ainda coze a partir de uma forma que nao e' estado: {:?}",
        steps[0]
    );
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
