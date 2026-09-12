//! **Os gates da §14 do inspector do player** — o sujeito é o painel, que vive na
//! [`ph2d_app_physics::inspector::player`]; o que eles exercitam é a **PORTA DE
//! PRODUÇÃO desta shell** (`component_attach::attach_by_name` sobre o registo do
//! `init`), e é por isso que eles moram aqui.
//!
//! ⚠️ **Um `#[cfg(test)]` é invisível do outro lado da fronteira de crate** (HOWTO §2),
//! e o `attach_player` — o helper que os 29 gates partilham — atravessa a porta real
//! de propósito: *«um atalho de teste que constrói o componente por outro caminho é a
//! segunda porta que diverge»*, diz o doc dele, que veio junto. ⛔ Por isso ele NÃO
//! ficou na crate com um `insert` à mão.

use super::*;
use ph2d_app_physics::inspector::player::{apply_player_edit, build_player_info};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_editor_core::PlayerFieldEdit;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PlatformPlayer, RigidBody};

const SPRUNG: ph2d_physics_ecs::PlayerLiveness = ph2d_physics_ecs::PlayerLiveness::SPRING;

const CAPSULE: ColliderShape = ColliderShape::Capsule {
    half_height: 0.3,
    radius: 0.2,
};

fn dynamic_body() -> (SimWorld, u64) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Name::new("Hero"),
            Transform::from_translation(Vec2::new(0.0, 1.0)),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: CAPSULE,
                ..Collider::default()
            },
        ))
        .id();
    (sim, e.to_bits())
}

/// **O número digitado chega à CONFIG que a ponte lê** — não só ao campo.
///
/// ⚠️ **O oráculo é o `config()`, e a distinção é o desenho da escada:** afirmar
/// `p.brake_scale == 0.25` provaria que a escrita pousou num campo, e a ponte não
/// lê campos — ela lê a `PlayerConfig` que a porta única monta. Um degrau novo
/// que entrasse no componente e não na tradução ficaria verde num gate de campo.
///
/// ⚠️ **E o degrau SEGUINTE tem dono noutro crate:** *"e aí ele pára mais
/// curto"* é `ph2d_platformer::walk::brake_tests`, que mede a distância pela
/// porta do produto. Esta shell não depende da lei, e escrever aqui uma segunda
/// medição exigiria uma dep nova para responder o que já está respondido.
///
/// **Mutação que deve sangrar:** o braço `BrakeScale` do `apply_player_edit`
/// escrever noutro campo (ou não escrever).
#[test]
fn a_typed_brake_reaches_the_config_the_bridge_reads() {
    let (mut sim, bits) = dynamic_body();
    attach_player(&mut sim, bits);

    let brake_of = |sim: &SimWorld| {
        sim.world()
            .get::<PlatformPlayer>(ph2d_ecs::Entity::from_bits(bits))
            .copied()
            .expect("o gesto Add faz dele um player")
            .config()
            .walk
            .brake_scale
    };

    // O CONTROLE: um player recém-criado nasce no NEUTRO — sem ele o gate ficaria
    // verde sobre um `config()` que devolvesse sempre o valor escrito.
    assert!(
        (brake_of(&sim) - 1.0).abs() < 1.0e-6,
        "um player recem-criado nasce com o freio neutro: {}",
        brake_of(&sim)
    );

    for typed in [0.0_f32, 0.25, 2.0] {
        apply_player_edit(&mut sim, bits, PlayerFieldEdit::BrakeScale(typed));
        assert!(
            (brake_of(&sim) - typed).abs() < 1.0e-6,
            "o {typed} digitado tem de chegar a' config: {}",
            brake_of(&sim)
        );
    }
}

/// **Um freio negativo não sobrevive à fronteira** — e a §14 volta a mostrá-lo.
///
/// ⚠️ Esta é a metade de FRONTEIRA de uma defesa em DUAS camadas: o consumidor
/// (`walk::brake_scale`) tem o piso load-bearing, e este clamp existe para o
/// número que o artista relê ser o número que o motor honra. Sem ele a row
/// mostraria `-1` sobre um personagem que freia como se fosse `0`.
///
/// **Mutação que deve sangrar:** tirar o `.max(0.0)` do braço `BrakeScale`.
#[test]
fn a_negative_brake_is_clamped_at_the_boundary_so_the_row_never_lies() {
    let (mut sim, bits) = dynamic_body();
    attach_player(&mut sim, bits);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::BrakeScale(-1.0));

    let info =
        build_player_info(&sim, bits, 0.0, 0.0, None, SPRUNG).expect("a secao continua viva");
    assert!(
        (info.brake_scale - 0.0).abs() < 1.0e-6,
        "a row tem de mostrar o que o motor honra: {info:?}"
    );
}
