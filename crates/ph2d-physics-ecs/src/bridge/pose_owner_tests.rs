//! Os gates da porta única de posse da pose (W-KinMove).
use super::*;
use ph2d_ecs::SimWorld;

fn player_cfg() -> PlatformPlayer {
    PlatformPlayer::default()
}

/// **Os quatro casos, e a lei que os une:** a pose tem exatamente um dono, e o
/// [`Support`] sai da MESMA resposta.
#[test]
fn every_body_has_exactly_one_pose_owner_and_the_support_follows_it() {
    let mut sim = SimWorld::new();
    let plain = sim.world_mut().spawn(()).id();
    let scene_body = sim.world_mut().spawn(()).id();
    let baked = sim.world_mut().spawn(player_cfg()).id();
    let driven = sim
        .world_mut()
        .spawn((player_cfg(), PlayerMode::Kinematic))
        .id();
    let w = sim.world();

    // Dinâmico: o solver, sempre — inclusive um player com o modo escrito.
    assert_eq!(
        pose_owner(w, plain, BodyKind::Dynamic),
        PoseOwner::Solver,
        "um corpo dinamico e' do solver"
    );
    assert_eq!(
        pose_owner(w, driven, BodyKind::Dynamic),
        PoseOwner::Solver,
        "⚠️ e o modo NAO sobrepoe o corpo: a falha e' a SEGURA"
    );
    assert_eq!(
        pose_owner(w, driven, BodyKind::Dynamic).support(),
        Support::Spring
    );

    // Cinemático sem player: a cena (o bake, uma curva).
    assert_eq!(
        pose_owner(w, scene_body, BodyKind::Kinematic),
        PoseOwner::Scene
    );
    // ⚠️ **E um player ASSADO também** — é a resposta do §8 do plano: o
    // discriminador é o componente, nunca o `BodyKind`.
    assert_eq!(
        pose_owner(w, baked, BodyKind::Kinematic),
        PoseOwner::Scene,
        "player assado (sem PlayerMode) continua dirigido pela CENA"
    );

    // Cinemático COM o modo: o player.
    assert_eq!(
        pose_owner(w, driven, BodyKind::Kinematic),
        PoseOwner::Player
    );
    assert_eq!(
        pose_owner(w, driven, BodyKind::Kinematic).support(),
        Support::Snap,
        "e a lei dele nao tem perna elastica"
    );

    // Estático: da cena, e nunca sai.
    assert_eq!(pose_owner(w, plain, BodyKind::Static), PoseOwner::Scene);
}

/// **`flows_out` e `driven_by_scene` PARTICIONAM** — nenhum corpo é reclamado
/// pelos dois estágios, e nenhum fica sem nenhum.
///
/// ⚠️ É o invariante que o doc do [`BodyKind::solver_owns_pose`] já enunciava
/// para dois donos, afirmado agora para três: escrito como duas perguntas
/// separadas, ele voltaria a poder ser quebrado por um `if` esquecido.
#[test]
fn the_two_stages_partition_the_owners() {
    for owner in [PoseOwner::Solver, PoseOwner::Scene, PoseOwner::Player] {
        assert_ne!(
            owner.flows_out(),
            owner.driven_by_scene(),
            "{owner:?} tem de ser reclamado por exatamente um estagio"
        );
    }
}

/// **`PlayerMode` faz o round-trip pela fronteira da UI** — e um tag que este
/// build não conhece é RECUSADO, não dobrado num modo qualquer.
#[test]
fn the_mode_survives_the_ui_boundary_and_an_unknown_tag_is_refused() {
    for m in [PlayerMode::Dynamic, PlayerMode::Kinematic] {
        assert_eq!(PlayerMode::from_tag(m.tag()), Some(m));
    }
    assert_eq!(PlayerMode::from_tag(2), None);
    assert_eq!(PlayerMode::default(), PlayerMode::Dynamic);
}
