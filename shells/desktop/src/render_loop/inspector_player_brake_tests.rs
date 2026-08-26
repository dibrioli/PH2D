//! **O FREIO, do lado da SHELL** (`W-Brake`) — irmão do `inspector_player_tests`
//! pelo teto de LOC, cortado por ASSUNTO (o precedente do
//! `inspector_player_out_tests`, ao lado).
//!
//! ⚠️ **Este é o degrau do MEIO da QUARTA condição de UI do plano 00** — *a
//! sequência leva a algum lugar* —, e a escada tem TRÊS: o seam do painel prova
//! que o clique vira um `PlayerFieldEdit` (`seam_player`), este prova que o edit
//! atravessa até a `PlayerConfig` **que a ponte lê**, e o gate da LEI
//! (`ph2d_platformer::walk::brake_tests`) prova que aquela config encurta a
//! paragem. Sem o do meio nada liga os outros dois.

use super::inspector_player::{apply_player_edit, attach_player, build_player_info};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_editor::PlayerFieldEdit;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PlatformPlayer, RigidBody};

/// **A premissa desta fixture, declarada uma vez** — todo corpo aqui é
/// `Dynamic` e vira player pela porta do Inspector, então a lei corre nele com
/// a perna ELÁSTICA. Passá-la a cada chamada seria repetir trinta vezes o que
/// é um fato do arquivo; passá-la ERRADA deixaria verdes, pelo motivo errado,
/// os gates que leem `reaction_is_live`/`push_is_live`/`spring_is_live`.
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
