//! A cena 24 (`PH2D_GAME_SMOKE=24`, ficha GEF 05): o Top-Down Player montado pelas peças do produto, e a
//! pergunta da ficha — a imagem GIRA com a pose do corpo quando Facing ≠ None?

use ph2d_app_game::smoke::topdown;
use ph2d_ecs::{Entity, GlobalTransform, Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, PhysicsBridge, PlayerInput, RigidBody, TopDownPlayer};
use ph2d_render::{RenderInstance, Sprite};
use ph2d_topdown::rotation::{self, RotationMode};

fn por_nome(sim: &mut SimWorld, nome: &str) -> Option<Entity> {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w).find(|(_, n)| n.0 == nome).map(|(e, _)| e)
}

/// A cena tem o herói de fábrica com o mover de vista de cima, corpo Kinematic com forma, Facing = Move, e as
/// três paredes estáticas.
#[test]
fn a_cena_24_monta_o_heroi_de_vista_de_cima_pelas_pecas() {
    let mut sim = SimWorld::new();
    let heroi = crate::comp::montar(&mut sim, topdown::CENA);
    assert_eq!(por_nome(&mut sim, topdown::HEROI), Some(heroi));
    let w = sim.world();
    let td = w.get::<TopDownPlayer>(heroi).expect("o Top-Down Player");
    assert_eq!(
        rotation::from_wire(td.rotation_mode),
        RotationMode::ToMovement
    );
    assert_eq!(
        w.get::<RigidBody>(heroi).map(|r| r.kind),
        Some(BodyKind::Kinematic)
    );
    assert!(ph2d_physics_ecs::tem_forma(w, heroi), "o corpo tem forma");
    assert_eq!(
        w.get::<Sprite>(heroi).map(|s| s.source),
        Some(ph2d_render::SpriteSource::Atlas {
            key: ph2d_render::HERO_SHEET_KEY
        })
    );
    for (nome, _, _) in topdown::PAREDES {
        let e = por_nome(&mut sim, nome).unwrap_or_else(|| panic!("{nome}"));
        assert_eq!(
            sim.world().get::<RigidBody>(e).map(|r| r.kind),
            Some(BodyKind::Static),
            "{nome}"
        );
    }
}

/// O ângulo com que o renderer desenha `e` (a base do `GlobalTransform`; o herói está na raiz).
fn angulo_desenhado(sim: &SimWorld, e: Entity) -> f32 {
    let t = *sim.world().get::<Transform>(e).expect("pose");
    let (_, b) = RenderInstance::pose_de(&GlobalTransform::from_transform(t));
    b[1].atan2(b[0])
}

/// ⭐ **A imagem gira com o corpo**: ↗ ⇒ 45°, ↑ ⇒ 90°, → ⇒ 0°, no `Transform` e na base que o renderer desenha.
/// (Mutação: o herói com Facing = None ⇒ fica a 0° e reprova.)
#[test]
fn com_facing_move_a_imagem_desenhada_vira_para_onde_anda() {
    let mut sim = SimWorld::new();
    let heroi = crate::comp::montar(&mut sim, topdown::CENA);
    let mut bridge = PhysicsBridge::new();
    let mut tick = 0_u64;
    let mut anda = |sim: &mut SimWorld, drive: f32, drive_y: f32| {
        for _ in 0..30 {
            tick += 1;
            let input = PlayerInput {
                drive,
                drive_y,
                ..PlayerInput::default()
            };
            bridge.set_player_input(heroi, input);
            bridge.dispatch(sim, true, tick);
        }
        let t = sim.world().get::<Transform>(heroi).expect("pose");
        (t.translation, t.rotation, angulo_desenhado(sim, heroi))
    };
    let inicio = topdown::INICIO;
    for (drive, drive_y, graus) in [(1.0, 1.0, 45.0), (0.0, 1.0, 90.0), (1.0, 0.0, 0.0)] {
        let (onde, rot, desenhado) = anda(&mut sim, drive, drive_y);
        println!(
            "[topdown-giro] dedo ({drive}, {drive_y}) ⇒ em ({:.2}, {:.2}), Transform.rotation {:.2}°, desenhado {:.2}°",
            onde.x,
            onde.y,
            rot.to_degrees(),
            desenhado.to_degrees()
        );
        assert!(
            (rot.to_degrees() - graus).abs() < 0.5,
            "o Transform devia olhar a {graus}° e olha a {:.3}°",
            rot.to_degrees()
        );
        assert!(
            (desenhado.to_degrees() - graus).abs() < 0.5,
            "o renderer devia desenhar a {graus}° e desenha a {:.3}°",
            desenhado.to_degrees()
        );
    }
    let fim = sim
        .world()
        .get::<Transform>(heroi)
        .expect("pose")
        .translation;
    assert!(
        fim.x > inicio[0] + 1.0 && fim.y > inicio[1] + 1.0,
        "ele andou: de {inicio:?} a ({}, {})",
        fim.x,
        fim.y
    );
}
