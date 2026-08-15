//! **O TETO DE QUEDA, do lado da SHELL** (`W-Fall`) — irmão do
//! `inspector_player_tests` pelo teto de LOC, cortado por ASSUNTO (o precedente
//! exacto do `inspector_player_brake_tests`, ao lado).
//!
//! ⚠️ **Este é o degrau do MEIO da QUARTA condição de UI do plano 00** — *a
//! sequência leva a algum lugar* —, e a escada tem TRÊS: o seam do painel prova
//! que o clique vira um `PlayerFieldEdit` (`seam_player`), este prova que o edit
//! atravessa até a `PlayerConfig` **que a ponte lê**, e o gate da LEI
//! (`ph2d_physics_ecs::tests::player_terminal`) prova que aquela config faz a
//! queda parar de acelerar. Sem o do meio nada liga os outros dois.

use super::inspector_player::{apply_player_edit, build_player_info};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_editor::PlayerFieldEdit;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PlatformPlayer, RigidBody};

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

/// **O número digitado chega à CONFIG que a ponte lê — e NÃO ao vizinho.**
///
/// ⚠️ **A metade do VIZINHO é a que este gate acrescenta aos irmãos**, e ela é o
/// modo de falha que esta wave de facto produz: `max_fall_speed` e
/// `glide_fall_speed` são dois tetos da MESMA velocidade, com nomes quase
/// iguais, fiados no mesmo commit e compostos por uma porta só
/// (`ph2d_platformer::descent_ceiling`) — escrever num pelo braço do outro dá um
/// produto que **funciona** (a queda ainda trava) e mente sobre QUANDO
/// (o planeio passaria a valer com o dedo em cima).
///
/// ⚠️ **O oráculo é o `config()`, e a distinção é o desenho da escada:** afirmar
/// `p.max_fall_speed == 30.0` provaria que a escrita pousou num campo, e a ponte
/// não lê campos — ela lê a `PlayerConfig` que a porta única monta.
///
/// **Mutações que devem sangrar:** o braço `MaxFallSpeed` escrever em
/// `glide_fall_speed` (ou não escrever); o `to_config` devolver
/// `FallConfig::STARTING_POINT`.
#[test]
fn a_typed_cap_reaches_the_config_and_leaves_the_glide_alone() {
    let (mut sim, bits) = dynamic_body();
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);

    let cfg_of = |sim: &SimWorld| {
        let c = sim
            .world()
            .get::<PlatformPlayer>(ph2d_ecs::Entity::from_bits(bits))
            .copied()
            .expect("o gesto Add faz dele um player")
            .config();
        (c.fall.max_speed, c.glide.fall_speed)
    };

    // O CONTROLE: um player recém-criado nasce SEM teto — sem ele o gate ficaria
    // verde sobre um `config()` que devolvesse sempre o valor escrito.
    assert_eq!(
        cfg_of(&sim),
        (0.0, 0.0),
        "um player recem-criado nasce sem teto e sem planeio"
    );

    // O planeio primeiro, para o vizinho ter um valor que uma escrita errada
    // possa DESTRUIR — escrito depois, a mutação apenas o repetiria.
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::GlideFallSpeed(2.5));
    for typed in [8.0_f32, 30.0, 142.57] {
        apply_player_edit(&mut sim, bits, PlayerFieldEdit::MaxFallSpeed(typed));
        assert_eq!(
            cfg_of(&sim),
            (typed, 2.5),
            "o {typed} digitado tem de chegar ao teto sem tocar no planeio"
        );
    }
}

/// **Um teto negativo não sobrevive à fronteira** — e a §14 volta a mostrá-lo.
///
/// ⚠️ Uma velocidade terminal negativa não é uma direção: o zero **é** o
/// desligado desta lei (`FallConfig::armed`), então a fronteira mapeia o
/// disparate no desligado em vez de o deixar chegar ao motor. Sem o clamp a row
/// mostraria `-1` sobre um personagem que cai como se nada tivesse sido escrito.
///
/// **Mutação que deve sangrar:** tirar o `.max(0.0)` do braço `MaxFallSpeed`.
#[test]
fn a_negative_cap_is_clamped_at_the_boundary_so_the_row_never_lies() {
    let (mut sim, bits) = dynamic_body();
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::MaxFallSpeed(-1.0));

    let info = build_player_info(&sim, bits, 0.0, 0.0, None).expect("a secao continua viva");
    assert!(
        (info.max_fall_speed - 0.0).abs() < 1.0e-6,
        "a row tem de mostrar o que o motor honra: {info:?}"
    );
}
