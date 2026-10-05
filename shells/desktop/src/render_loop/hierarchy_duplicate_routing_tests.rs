//! O gate do **roteamento** de `Duplicate` — ver [`super::DuplicateKind`].
//!
//! ⚠️ Irmão de [`super`] pelo teto de 600 LOC da shell, e o corte é por ASSUNTO: lá fica o dreno
//! das intenções da Hierarquia; aqui, a prova de que cada tipo de entidade vai para a porta que
//! sabe duplicá-la.
//!
//! ⚠️ Ele existe por uma prova de mutação que **passou**: os gates de um módulo chamavam a porta
//! de duplicar diretamente, então apagar o braço daqui não reprovava nada. *A costura não-testada
//! é a causa nº 1 da `DIRETIVA_IMPLEMENTACAO` §1.* (O braço do nó de modelagem 3D saiu com o
//! módulo — ADR-0179; a lei ficou com o path vetorial.)

use super::{DuplicateKind, duplicate_kind};

/// ⭐ **Cada tipo de entidade vai para quem sabe duplicá-la.**
#[test]
fn a_vector_path_never_goes_to_the_generic_arm() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let world = sim.world_mut();

    // Um path vetorial vai para o dono da geometria dele — no braço genérico sairia um sósia sem
    // geometria (ou dois donos do mesmo path).
    let path = world
        .spawn((
            ph2d_ecs::Name::new("Path"),
            ph2d_ecs::Transform::default(),
            ph2d_ecs::VecPathRef(7),
        ))
        .id();
    assert_eq!(
        duplicate_kind(world, path),
        DuplicateKind::VecPath,
        "um path vetorial no braço genérico sai como um sósia sem geometria"
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
}
