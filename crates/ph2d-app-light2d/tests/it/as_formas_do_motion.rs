//! ⭐ **A cena 8 — as formas do Motion** (W5e): a lâmpada e o chão sobre os objetos do smoke do
//! Motion, marcados com «bloqueia a luz» quando nascem. Que as formas do Motion desenham para a luz a
//! ÁREA certa prova-o o teste de contrato da família (`ph2d-app-motion`, `motion_shape_luz_tests.rs`);
//! que a luz trata essa área como a da imagem gêmea, a cena 7 (`o_traco_do_flip`, discos exatos).

use bevy_ecs::world::World;
use ph2d_app_light2d::smoke;
use ph2d_ecs::{MotionGenerator, Name, Transform};
use ph2d_light2d_ecs::BloqueiaLuz;
use ph2d_light2d_ecs::coleta::cena_de_luz;

/// ⭐ A cena 8 ensina as formas do Motion: a lâmpada e o chão; os objetos Motion (os do smoke do Motion)
/// recebem o «bloqueia a luz» quando nascem — antes, nada a marcar. O censo do roteador mora aqui (a
/// ÚLTIMA cena).
#[test]
fn a_cena_8_ensina_as_formas_do_motion() {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 8));
    assert!(!smoke::encenar(&mut World::new(), 9), "não há cena 9");
    assert_eq!(smoke::ROUTERS[0].max_level, 8, "o maior nível do `match`");
    assert_eq!(cena_de_luz(&w).luzes, 2, "a lâmpada e a luz ambiente");
    assert_eq!(smoke::marcar_motion(&mut w), 0, "antes do smoke do Motion");
    let pilha = w
        .spawn((Name::new("Pile"), Transform::IDENTITY, MotionGenerator))
        .id();
    assert_eq!(smoke::marcar_motion(&mut w), 1);
    assert!(w.get::<BloqueiaLuz>(pilha).is_some(), "a Pile bloqueia");
    assert_eq!(smoke::marcar_motion(&mut w), 0, "uma vez só");
    assert_eq!(
        cena_de_luz(&w).outros.len(),
        1,
        "a coleta vê o objeto Motion"
    );
}
