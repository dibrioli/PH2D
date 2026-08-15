//! **ESTA SUPERFÍCIE NÃO É PAREDE** (`W-WallMaterial`) — o
//! `platform_wall_layers` do Godot, resolvido por CORPO/PEÇA em vez de por
//! camada.
//!
//! ⚠️ **A medição que decidiu a wave, antes de uma linha:** a metade irmã do item
//! (`platform_floor_layers`, *"o que me carrega"*) **já era exprimível** — a
//! sonda `measure_kinematic_carry` mostra uma plataforma horizontal a levar
//! `0,99×` com tração cheia e **`0,00×` com `WalkSurface { grip: 0 }`**, nos dois
//! modos. Já *"o que me segura"* não tinha porta nenhuma: a lei aceita parede
//! **só por inclinação**, então toda superfície íngreme com que o personagem
//! colide era escalável, e o artista não tinha como dizer o contrário sem mudar
//! a geometria ou fazê-lo cair através dela.
//!
//! O oráculo destes gates é **o percurso**: onde o personagem chega diz se a
//! parede o segurou.

#[path = "platform_water_scene.rs"]
mod scene;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, NoWallCling, PhysicsBridge, PlatformPlayer, RigidBody,
};
use ph2d_platformer::PlayerInput;
use scene::{FLOAT, floor, subject_tuned};

/// Um personagem com o pulo de parede ARMADO — sem isto a wave não teria o que
/// desligar, e o gate mediria uma cena em que ninguém se agarra de qualquer modo.
fn climber() -> PlatformPlayer {
    let base = PlatformPlayer::default();
    PlatformPlayer {
        float_height: FLOAT,
        wall_slide_speed: 1.0,
        wall_jump_height: base.jump_height.max(1.0),
        wall_jump_push: 2.0,
        ..base
    }
}

/// A cena: uma parede alta à direita e o personagem a CAIR ao lado dela,
/// mantendo a direção contra ela. `marked` põe o material na parede.
///
/// ⚠️ **O oráculo é a DESCIDA, não a subida**, e a primeira versão deste gate
/// errou aqui: ela mandava o personagem escalar, e o CONTROLE reprovou — um pulo
/// de parede empurra para LONGE (`wall_jump_push`), então escalar uma parede
/// única exige uma afinação que não é o assunto desta wave. O que a wave muda é
/// se a AMOSTRA existe, e a amostra é observável de forma directa: agarrado, ele
/// desliza ao `wall_slide_speed`; sem parede, cai livre.
fn beside(marked: bool) -> (usize, f32) {
    let mut sim = SimWorld::new();
    floor(&mut sim, -40.0);
    let wall = sim
        .world_mut()
        .spawn((
            Name::new("Wall"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.5,
                    half_y: 20.0,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(1.0, 0.0)),
        ))
        .id();
    if marked {
        sim.world_mut().entity_mut(wall).insert(NoWallCling);
    }
    let who = subject_tuned(&mut sim, true, 8.0, Some(climber()));
    let mut bridge = PhysicsBridge::new();
    let start = scene::y_of(&sim, "Subject");
    let mut clung = 0usize;
    for t in 1..=120u64 {
        bridge.set_player_input(
            who,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
        if bridge.player_view(who).is_some_and(|v| v.wall.is_some()) {
            clung += 1;
        }
    }
    (clung, start - scene::y_of(&sim, "Subject"))
}

/// SONDA: o que a VISTA diz, tique a tique.
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_the_slide() {
    for marked in [false, true] {
        let (clung, drop) = beside(marked);
        // ⚠️ A descida vai impressa mas **não é oráculo**: encostado a uma
        // parede com `drive` contra ela, o atrito de Coulomb do solver segura um
        // corpo dinâmico quase parado nos DOIS casos — o número não separa
        // *agarrado* de *não é parede*. Quem separa é a contagem.
        println!(
            "{:9} | tiques agarrado {clung:3} | (desceu {drop:6.3} m, nao e' oraculo)",
            if marked { "MARCADA" } else { "controle" }
        );
    }
}

/// **Sem marca, a parede segura — com marca, não.**
///
/// ⚠️ **A metade sem marca é o CONTROLE, e ela vem primeiro por isso:** um gate
/// que só afirmasse *"marcado não sobe"* ficaria verde numa cena em que ninguém
/// sobe por qualquer outro motivo (o pulo de parede desarmado, a parede fora de
/// alcance, o `drive` no sentido errado). É a subida do controle que dá sentido
/// à ausência dela.
#[test]
fn a_marked_surface_is_not_a_wall_and_the_unmarked_one_still_is() {
    let (clings, _) = beside(false);
    let (marked, _) = beside(true);
    assert!(
        clings > 30,
        "o CONTROLE tem de agarrar: so' {clings} tiques de 120"
    );
    assert_eq!(
        marked, 0,
        "a parede marcada nao e' parede: {marked} tiques agarrado"
    );
}

/// **A PEÇA fala por si, não o corpo que a possui** — a mesma lei que a
/// [`ph2d_physics_ecs::WalkSurface`] já tem, porque as duas viajam na mesma
/// entrada da tabela do bridge.
///
/// ⚠️ **E ele também tem controle**: sem a marca a mesma torre agarra. Sem essa
/// metade o gate ficaria verde numa cena em que a peça nem é alcançada — que foi
/// exactamente como a primeira versão desta suíte passou, medindo altura escalada
/// numa cena em que ninguém escalava.
#[test]
fn the_part_speaks_for_itself_not_the_body_that_owns_it() {
    let bare = tower(false);
    let marked = tower(true);
    assert!(bare > 30, "o CONTROLE tem de agarrar: so' {bare} tiques");
    assert_eq!(
        marked, 0,
        "a marca na PECA vale para quem encosta nela: {marked} tiques agarrado"
    );
}

/// Uma torre de DUAS formas: o tronco lá em cima e a face que o personagem de
/// facto encosta pendurada nele como PEÇA. `marked` põe o material só na peça.
fn tower(marked: bool) -> usize {
    let mut sim = SimWorld::new();
    floor(&mut sim, -40.0);
    let body = sim
        .world_mut()
        .spawn((
            Name::new("Tower"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.5,
                    half_y: 0.25,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(1.0, 20.5)),
        ))
        .id();
    let mut part = sim.world_mut().spawn((
        Name::new("Face"),
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.5,
                half_y: 20.0,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, -20.5)),
        ph2d_ecs::ChildOf(body),
    ));
    if marked {
        part.insert(NoWallCling);
    }
    let who = subject_tuned(&mut sim, true, 8.0, Some(climber()));
    let mut bridge = PhysicsBridge::new();
    let mut clung = 0usize;
    for t in 1..=120u64 {
        bridge.set_player_input(
            who,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
        if bridge.player_view(who).is_some_and(|v| v.wall.is_some()) {
            clung += 1;
        }
    }
    clung
}
