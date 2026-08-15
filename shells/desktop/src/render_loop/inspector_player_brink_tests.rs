//! **A TRAVA DE BEIRADA, do lado da SHELL** (`W-Brink`) — irmão do
//! `inspector_player_leave_tests` pelo teto de LOC, cortado por ASSUNTO.
//!
//! ⚠️ **Este é o degrau do MEIO da QUARTA condição de UI do plano 00** — *a
//! sequência leva a algum lugar* —, e a escada tem TRÊS: o seam do painel prova
//! que o clique vira um `PlayerFieldEdit`
//! (`seam_player::the_walk_off_ledges_chips_reach_the_bus_in_every_option`),
//! este prova que o edit atravessa até a `PlayerConfig` **que a ponte lê**, e o
//! gate do PRODUTO (`ph2d_physics_ecs::tests::platform_brink`) prova que aquela
//! config faz o personagem parar na quina. Sem o do meio nada liga os outros
//! dois.

use super::inspector_player::{apply_player_edit, build_player_info};
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

/// **A trava escolhida chega à CONFIG que a ponte lê — e a row a mostra de
/// volta.**
///
/// ⚠️ **O oráculo é o `config()`, e é o desenho da escada:** afirmar
/// `p.walk_off_ledges == false` provaria que a escrita pousou num campo, e a
/// ponte não lê campos — ela lê a `PlayerConfig` que a porta única monta.
///
/// ⚠️ **E a volta importa tanto quanto a ida:** um chip cujo estado não é lido
/// de volta pinta sempre a primeira opção, e o artista arma uma trava que a tela
/// nega — o defeito exacto que as cinco rows de área tinham (write-only, curado
/// em 2026-07-23).
///
/// ⚠️ **A INVERSÃO mora numa fronteira só**, e é o que este gate protege: o chip
/// fala em índice (`0` = pode andar para fora) e a lei em capacidade (`true` =
/// pode). Um segundo `!` em qualquer outro sítio faria a row mostrar o oposto do
/// que ela acabou de escrever, sem nada a explicar.
#[test]
fn the_walk_off_chip_reaches_the_config_and_the_row_reads_it_back() {
    let (mut sim, bits) = dynamic_body();
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);

    let cfg_allows = |sim: &SimWorld| {
        sim.world()
            .get::<PlatformPlayer>(ph2d_ecs::Entity::from_bits(bits))
            .copied()
            .expect("o player existe")
            .config()
            .walk
            .walk_off_ledges
    };
    let row_index = |sim: &SimWorld| {
        build_player_info(sim, bits, 0.0, 0.0, None, SPRUNG)
            .expect("a §14 monta a info")
            .walk_off_ledges
    };

    // Nasce no mundo que ja' shipava: pode andar para fora, e a row diz `0`.
    assert!(cfg_allows(&sim), "o default e' o mundo de antes desta wave");
    assert_eq!(row_index(&sim), 0);

    apply_player_edit(&mut sim, bits, PlayerFieldEdit::WalkOffLedges(false));
    assert!(
        !cfg_allows(&sim),
        "a trava armada tem de chegar a' config que a ponte le'"
    );
    assert_eq!(row_index(&sim), 1, "e a row tem de a mostrar de volta");

    apply_player_edit(&mut sim, bits, PlayerFieldEdit::WalkOffLedges(true));
    assert!(cfg_allows(&sim), "e desarmar tambem chega la'");
    assert_eq!(row_index(&sim), 0);
}

/// **A trava AGACHADO viaja pelo mesmo caminho, e só APERTA.**
///
/// ⚠️ O oráculo é a config EFETIVA do agachar
/// ([`ph2d_platformer::walk_for`]), não o campo: é ali que o `&&` vive, e é ali
/// que um `=` deixaria o agachar LIGAR a caminhada para fora do patamar num
/// personagem cujo autor a proibiu de pé.
#[test]
fn the_crouching_chip_reaches_the_law_and_only_tightens() {
    let (mut sim, bits) = dynamic_body();
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    // O agachar tem de estar AUTORADO, senao a `walk_for` devolve a config de
    // pe' e este gate ficaria verde sobre um numero que a lei nunca le'.
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::CrouchHeight(0.25));

    let crouched_allows = |sim: &SimWorld| {
        let c = sim
            .world()
            .get::<PlatformPlayer>(ph2d_ecs::Entity::from_bits(bits))
            .copied()
            .expect("o player existe")
            .config();
        ph2d_physics_ecs::walk_for(&c.crouch, &c.ride, &c.walk, true).walk_off_ledges
    };

    assert!(crouched_allows(&sim), "o default deixa andar para fora");
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::CrouchWalkOffLedges(false));
    assert!(
        !crouched_allows(&sim),
        "agachado APERTA quem de pe' andava para fora"
    );

    // E o contrario NAO solta: com a trava de pe' armada, marcar a de agachado
    // nao devolve a liberdade.
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::WalkOffLedges(false));
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::CrouchWalkOffLedges(true));
    assert!(
        !crouched_allows(&sim),
        "agachado nao pode devolver o que de pe' foi recusado"
    );
}

/// **A row do agachar é oferecida pelo veredito da LEI, não por uma segunda
/// regra na shell** — agachar é ficar mais baixo, e *mais baixo* só existe
/// contra a altura de pé.
#[test]
fn the_crouch_armed_flag_comes_from_the_law() {
    let (mut sim, bits) = dynamic_body();
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::Add);
    let armed = |sim: &SimWorld| {
        build_player_info(sim, bits, 0.0, 0.0, None, SPRUNG)
            .expect("a §14 monta a info")
            .crouch_armed
    };
    assert!(!armed(&sim), "o agachar nasce DESLIGADO");
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::CrouchHeight(0.25));
    assert!(armed(&sim), "com altura autorada ele fica armado");
    // ⚠️ Uma altura que NAO e' menor que a de pe' nao e' um agachar — e e' a lei
    // que o diz, com a `float_height` na mao.
    apply_player_edit(&mut sim, bits, PlayerFieldEdit::CrouchHeight(9.0));
    assert!(
        !armed(&sim),
        "um agachar que SOBE nao e' um agachar, e a lei e' quem o sabe"
    );
}
