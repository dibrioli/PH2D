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
        PoseOwner::Player(PlayerMode::Kinematic)
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
    for owner in [
        PoseOwner::Solver,
        PoseOwner::Scene,
        PoseOwner::Player(PlayerMode::Kinematic),
        PoseOwner::Player(PlayerMode::Pure),
    ] {
        assert_ne!(
            owner.flows_out(),
            owner.driven_by_scene(),
            "{owner:?} tem de ser reclamado por exatamente um estagio"
        );
    }
}

/// **O PURO SANGUE é o MESMO controlador** (W-KinPure) — ele escreve a própria
/// pose e a perna dele é o corpo, exatamente como o Snap.
///
/// ⚠️ Só a pergunta *"o mundo ouve?"* os separa, e ela é a ÚNICA coisa que este
/// gate exige que difira: se um dia o `Pure` deixasse de escrever a própria
/// pose, ele teria virado outro personagem, não o mesmo com dois canais
/// calados.
#[test]
fn the_pure_mode_moves_like_snap_and_differs_only_in_what_it_owes() {
    let mut sim = SimWorld::new();
    let pure = sim.world_mut().spawn((player_cfg(), PlayerMode::Pure)).id();
    let owner = pose_owner(sim.world(), pure, BodyKind::Kinematic);

    assert_eq!(owner, PoseOwner::Player(PlayerMode::Pure));
    assert!(owner.writes_own_pose(), "o puro sangue e' o controlador");
    assert_eq!(owner.support(), Support::Snap, "e a perna dele e' o corpo");
    assert!(owner.flows_out());

    assert!(
        !owner.transmits(),
        "o mundo e' CENARIO: nada do que ele faz volta"
    );
    assert!(
        pose_owner(sim.world(), pure, BodyKind::Dynamic).transmits(),
        "⚠️ e num corpo DINAMICO o modo nao cala nada: a ponte esta' a simular \
         a mola, e o `PlayerMode` nao sobrepoe o corpo"
    );
}

/// **`PlayerMode` faz o round-trip pela fronteira da UI** — e um tag que este
/// build não conhece é RECUSADO, não dobrado num modo qualquer.
#[test]
fn the_mode_survives_the_ui_boundary_and_an_unknown_tag_is_refused() {
    for m in [PlayerMode::Dynamic, PlayerMode::Kinematic, PlayerMode::Pure] {
        assert_eq!(PlayerMode::from_tag(m.tag()), Some(m));
    }
    assert_eq!(PlayerMode::from_tag(3), None);
    assert_eq!(PlayerMode::default(), PlayerMode::Dynamic);
}

/// **O VEREDITO que a §14 pinta sai desta porta, e o caso que ele existe para
/// não errar é o player ASSADO** (auditoria de 2026-08-15).
///
/// ⚠️ **A shell respondia às mesmas quatro perguntas a partir do `PlayerMode`**,
/// e um bake não escreve `PlayerMode` nenhum — `default()` é `Dynamic`, então a
/// cópia dela dizia *"mola viva, mundo ouve"* sobre um corpo cuja pose vem de
/// uma CURVA e cuja lei o `drive_players` nem chega a correr. Doze cards de
/// controles inertes, com a suíte inteira verde.
///
/// ⚠️ E a metade da PRESENÇA é o que impede a cura de virar *"a §14 sumiu"*: o
/// dinâmico e o cinemático continuam a ter tudo o que de facto leem.
#[test]
fn the_liveness_is_what_the_law_reads_and_a_baked_player_reads_nothing() {
    let mut sim = SimWorld::new();
    let plain = sim.world_mut().spawn(()).id();
    let baked = sim.world_mut().spawn(player_cfg()).id();
    let dynamic = sim
        .world_mut()
        .spawn((player_cfg(), PlayerMode::Dynamic))
        .id();
    let kinematic = sim
        .world_mut()
        .spawn((player_cfg(), PlayerMode::Kinematic))
        .id();
    let pure = sim.world_mut().spawn((player_cfg(), PlayerMode::Pure)).id();
    let w = sim.world();

    // ⚠️ O CASO DA WAVE: o mesmo componente, o mesmo `BodyKind`, e a lei não corre.
    assert_eq!(
        liveness(w, baked, BodyKind::Kinematic),
        PlayerLiveness::INERT,
        "um player ASSADO e' dirigido pela CENA -- nenhum dos knobs dele e' lido"
    );
    // Sem o componente não há personagem — mas a §14 continua OFERECIDA (o
    // botão que o cria mora fora deste veredito).
    assert_eq!(liveness(w, plain, BodyKind::Dynamic), PlayerLiveness::INERT);

    // A presença: cada modo lê exatamente o que a lei dele lê.
    assert_eq!(
        liveness(w, dynamic, BodyKind::Dynamic),
        PlayerLiveness::SPRING,
        "o dinamico tem perna de mola e transmite pelo solver"
    );
    assert_eq!(
        liveness(w, kinematic, BodyKind::Kinematic),
        PlayerLiveness::SNAP,
        "o cinematico pousa, e o empurrao lateral e' a UNICA via dele"
    );
    let pure = liveness(w, pure, BodyKind::Kinematic);
    assert!(pure.law_runs, "o puro sangue e' o mesmo controlador");
    assert!(!pure.spring, "e a perna dele e' o corpo");
    assert!(
        !pure.transmits && !pure.pushes,
        "mas o mundo e' CENARIO: os tres escalares da 3a lei sao mortos"
    );

    // ⚠️ E o modo NÃO sobrepõe o corpo, pela mesma razão do `pose_owner`: com o
    // corpo DINÂMICO a ponte está a simular a mola, e a falha é a SEGURA.
    assert_eq!(
        liveness(w, kinematic, BodyKind::Dynamic),
        PlayerLiveness::SPRING
    );
}
