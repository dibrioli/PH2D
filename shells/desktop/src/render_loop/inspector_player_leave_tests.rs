//! **A POLÍTICA DE PLATAFORMA, do lado da SHELL** (`W-Leave`) — irmão do
//! `inspector_player_tests` pelo teto de LOC, cortado por ASSUNTO (o precedente
//! exacto do `inspector_player_fall_tests`, ao lado).
//!
//! ⚠️ **Este é o degrau do MEIO da QUARTA condição de UI do plano 00** — *a
//! sequência leva a algum lugar* —, e a escada tem TRÊS: o seam do painel prova
//! que o clique vira um `PlayerFieldEdit`
//! (`seam_player::the_platform_lift_chip_reaches_the_bus_in_every_option`), este
//! prova que o edit atravessa até a `PlayerConfig` **que a ponte lê**, e o gate
//! da LEI (`ph2d_physics_ecs::tests::platform_leave`) prova que aquela config
//! muda o que o pulo entrega. Sem o do meio nada liga os outros dois.

use super::inspector_player::{apply_player_edit, build_player_info};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_editor::PlayerFieldEdit;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PlatformLift, PlatformPlayer, RigidBody,
};

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

/// **A política escolhida chega à CONFIG que a ponte lê — e a row a mostra de
/// volta.**
///
/// ⚠️ **O oráculo é o `config()`, e a distinção é o desenho da escada:** afirmar
/// `p.platform_lift == UpOnly` provaria que a escrita pousou num campo, e a
/// ponte não lê campos — ela lê a `PlayerConfig` que a porta única monta. É no
/// espelho `PlatformLift::law` que a wave pode falhar em silêncio.
///
/// ⚠️ **E a volta importa tanto quanto a ida:** um chip cujo estado não é lido
/// de volta pinta sempre a primeira opção, e o artista escolhe uma política que
/// a tela nega — o defeito exacto que as cinco rows de área da W-AreaTorque
/// tinham (write-only, curado em 2026-07-23).
///
/// **Mutações que devem sangrar:** o braço `PlatformLift` não escrever; o
/// `config()` devolver sempre `PlatformLift::Full`; o `build_player_info`
/// carimbar `0` em vez do `tag` autorado.
#[test]
fn the_chosen_policy_reaches_the_config_and_the_row_shows_it_back() {
    let (mut sim, bits) = dynamic_body();
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);

    // ⚠️ **Lido de volta pela INVERSA** (`of_law`), e não pela mesma `law()` que o
    // fold usa: comparar `config().jump.platform_lift == lift.law()` seria o
    // oráculo que usa a função sob teste para computar o que espera. A shell não
    // alcança a `ph2d-platformer` (nem deve), então a volta é a única leitura
    // honesta que ela tem.
    let law_of = |sim: &SimWorld| {
        PlatformLift::of_law(
            sim.world()
                .get::<PlatformPlayer>(ph2d_ecs::Entity::from_bits(bits))
                .copied()
                .expect("o gesto Add faz dele um player")
                .config()
                .jump
                .platform_lift,
        )
    };
    let shown = |sim: &SimWorld| {
        build_player_info(sim, bits, 0.0, 0.0, None)
            .expect("a secao continua viva")
            .platform_lift
    };

    // O CONTROLE: um player recém-criado nasce em `Full` — sem ele o gate
    // ficaria verde sobre um `config()` que devolvesse sempre o valor escrito.
    assert_eq!(
        law_of(&sim),
        PlatformLift::Full,
        "um player recem-criado nasce na politica que ja' shipava"
    );
    assert_eq!(shown(&sim), 0, "e a row mostra isso");

    for (tag, want) in [
        (1u8, PlatformLift::UpOnly),
        (2, PlatformLift::Nothing),
        (0, PlatformLift::Full),
    ] {
        apply_player_edit(&mut sim, bits, PlayerFieldEdit::PlatformLift(tag));
        assert_eq!(law_of(&sim), want, "o tag {tag} tem de chegar a' lei");
        assert_eq!(
            shown(&sim),
            tag,
            "e a row tem de mostrar o tag {tag} de volta"
        );
    }
}

/// **Um tag que nenhuma variante reivindica é IGNORADO, e não dobrado num
/// plausível.**
///
/// ⚠️ Dobrá-lo em `Full` seria uma escolha silenciosa: o artista veria a row
/// saltar para uma política que ele não pediu. A disciplina é a do
/// `BodyKind::from_tag` — quem chama decide o que fazer com um valor que não
/// esperava.
///
/// **Mutação que deve sangrar:** o `from_tag` devolver `Some(Full)` no `_`.
#[test]
fn a_tag_no_variant_claims_leaves_the_authored_policy_alone() {
    let (mut sim, bits) = dynamic_body();
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::PlatformLift(1));
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::PlatformLift(9));

    let p = sim
        .world()
        .get::<PlatformPlayer>(ph2d_ecs::Entity::from_bits(bits))
        .copied()
        .expect("o player continua vivo");
    assert_eq!(
        p.platform_lift,
        PlatformLift::UpOnly,
        "um tag desconhecido nao pode reescrever a politica autorada"
    );
}
