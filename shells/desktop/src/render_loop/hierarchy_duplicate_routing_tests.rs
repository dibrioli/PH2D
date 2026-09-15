//! O gate do **roteamento** de `Duplicate` — ver [`super::DuplicateKind`].
//!
//! ⚠️ Irmão de [`super`] pelo teto de 600 LOC da shell, e o corte é por ASSUNTO: lá fica o dreno
//! das intenções da Hierarquia; aqui, a prova de que cada tipo de entidade vai para a porta que
//! sabe duplicá-la.
//!
//! ⚠️ Ele existe por uma prova de mutação que **passou**: os gates do módulo 3D chamavam a porta
//! de duplicar diretamente, então apagar o braço daqui não reprovava nada. *A costura não-testada
//! é a causa nº 1 da `DIRETIVA_IMPLEMENTACAO` §1.*

use super::{DuplicateKind, duplicate_kind};

/// ⭐ **Cada tipo de entidade vai para quem sabe duplicá-la.**
#[test]
fn a_field_node_never_goes_to_the_generic_arm() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let world = sim.world_mut();

    // Um nó de modelagem 3D, criado pela porta de produção.
    let root = ph2d_field_ecs::spawn_doc(world, &ph2d_app_field3d::smoke::scene(1), "Model");
    assert_eq!(
        duplicate_kind(world, root),
        DuplicateKind::Field,
        "um nó de campo no braço genérico sai como um sósia sem geometria"
    );

    // Uma entidade comum continua a ir para o braço genérico — senão o roteamento passaria a
    // reclamar tudo, e o gate acima ficaria verde por reclamar de mais.
    let plain = world
        .spawn((
            ph2d_ecs::Name::new("Sprite"),
            ph2d_ecs::Transform::default(),
        ))
        .id();
    assert_eq!(duplicate_kind(world, plain), DuplicateKind::Entity);

    // E um path vetorial vai para o dono da geometria dele.
    let path = world
        .spawn((
            ph2d_ecs::Name::new("Path"),
            ph2d_ecs::Transform::default(),
            ph2d_ecs::VecPathRef(7),
        ))
        .id();
    assert_eq!(duplicate_kind(world, path), DuplicateKind::VecPath);
}

/// ⭐⭐⭐ **DUPLICAR UMA LUZ FUNCIONA, e a nota de `docs/Render3d/05` §25.8 dizia o contrário.**
///
/// Ela lia *«o `duplicate` exige um pai, e uma luz é raiz — declarado e testado»*, e o que estava
/// testado era a porta de MODELAGEM ([`ph2d_field_ecs::duplicate`]), que uma luz **nunca visita**:
/// o [`duplicate_kind`] pergunta por `FieldNode` e uma luz tem `FieldLight`, logo ela cai no braço
/// GENÉRICO — a cópia profunda do ADR-0164, que leva **todo componente registado**, e os dois que
/// fazem a luz (`FieldLight`, `FieldPose`) estão registados.
///
/// ⚠️ *Uma ausência afirmada pela porta ERRADA é um palpite com cara de medição.*
///
/// ⛔ Este gate é o que impede a nota de voltar: ele afirma o roteamento **e** o que a cópia leva —
/// porque cair no braço genérico só é a resposta certa enquanto os componentes forem registados,
/// e um `register_default` apagado devolveria exactamente o *«sósia que não desenha nada»* que o
/// [`DuplicateKind`] existe para evitar.
#[test]
fn duplicar_uma_luz_da_uma_segunda_luz() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let luz = ph2d_field_ecs::add_light(
        sim.world_mut(),
        [1.0, 2.0, 3.0],
        ph2d_field_ecs::FieldLight {
            intensity: 4.0,
            color: [0.25, 0.5, 0.75],
        },
    );
    assert_eq!(
        duplicate_kind(sim.world(), luz),
        DuplicateKind::Entity,
        "uma luz não é um nó de campo: ela tem de ir ao braço da cópia profunda"
    );

    let registry = crate::init::build_component_registry();
    let mut vec_scene = ph2d_vec_scene::VecScene::default();
    let mut vec_entities = ph2d_vec_entities::entities::VecEntityMap::default();
    let mut docs = ph2d_app_components::instance_docs::OwnedDocs {
        vec_scene: &mut vec_scene,
        vec_entities: &mut vec_entities,
    };
    let copia = ph2d_app_components::instantiate::duplicate_subtree(
        &mut sim, &registry, luz, &mut docs, [0.0, 0.0],
    )
    .expect("a cópia de uma luz");

    assert_ne!(copia, luz, "a cópia é uma entidade nova");
    let a = *sim
        .world()
        .get::<ph2d_field_ecs::FieldLight>(copia)
        .expect("a cópia tem de LEVAR a lâmpada — sem ela é uma linha sobre coisa nenhuma");
    assert_eq!(
        a,
        ph2d_field_ecs::FieldLight {
            intensity: 4.0,
            color: [0.25, 0.5, 0.75],
        },
        "a cópia tem de levar a INTENSIDADE e a COR, não só o componente"
    );
    assert!(
        sim.world().get::<ph2d_field_ecs::FieldPose>(copia).is_some(),
        "sem a pose a cópia não tem onde estar, e o gizmo não a agarra"
    );
    assert!(
        sim.world().get::<ph2d_ecs::Transform>(copia).is_some(),
        "sem o `Transform` a cópia não é raiz da Hierarquia e a linha dela não aparece"
    );

    // ⭐ E as duas são luzes da cena: o que o traçado lê é a POPULAÇÃO, não uma delas.
    let mut q = sim.world_mut().query::<&ph2d_field_ecs::FieldLight>();
    assert_eq!(
        q.iter(sim.world()).count(),
        2,
        "duplicar tem de deixar DUAS luzes no mundo"
    );
}
