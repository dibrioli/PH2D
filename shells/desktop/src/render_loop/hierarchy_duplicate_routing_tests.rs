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
    let root = ph2d_field_ecs::spawn_doc(world, &crate::field3d_smoke::scene(1), "Model");
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
