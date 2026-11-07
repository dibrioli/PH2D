//! Mudar Width num bloco repetido: a colisão acompanha? Medido pelo que o motor faz, não pelo número do componente:
//! uma bola cai ao lado do bloco e só pára se a colisão chegou até lá.

use ph2d_app_sprite_anim::tiled::set_tiled_side;
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, TiledDraw, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// O bloco tem uma cópia de 2 × 1 m no centro, e a bola cai em `x = 2,5`: fora dele, dentro do bloco com 3 cópias.
const AUTOMATICA: ColliderShape = ColliderShape::Cuboid {
    half_x: 1.0,
    half_y: 0.5,
};

fn cena(forma: ColliderShape) -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let bloco = w
        .spawn((
            Sprite::atlas(WHITE_TILE_KEY, [2.0, 1.0], [1.0; 4]),
            TiledDraw { tile: [2.0, 1.0] },
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: forma,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    let bola = w
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(2.5, 3.0)),
        ))
        .id();
    (sim, bloco, bola)
}

/// A bola pousou no topo do bloco?
fn pousou(sim: &mut SimWorld, bola: Entity, de: u64, ate: u64, bridge: &mut PhysicsBridge) -> bool {
    for tick in de..=ate {
        bridge.dispatch(sim, true, tick);
    }
    let y = sim.world().get::<Transform>(bola).unwrap().translation.y;
    (0.5..1.0).contains(&y)
}

fn lados(sim: &SimWorld, bloco: Entity) -> ColliderShape {
    sim.world().get::<Collider>(bloco).unwrap().shape
}

#[test]
fn sem_mudar_a_largura_a_bola_passa_ao_lado() {
    let (mut sim, _, bola) = cena(AUTOMATICA);
    assert!(!pousou(&mut sim, bola, 1, 240, &mut PhysicsBridge::new()));
}

#[test]
fn a_forma_automatica_acompanha_a_largura() {
    let (mut sim, bloco, bola) = cena(AUTOMATICA);
    assert!(set_tiled_side(sim.world_mut(), bloco, 0, 3.0));
    assert_eq!(
        lados(&sim, bloco),
        ColliderShape::Cuboid {
            half_x: 3.0,
            half_y: 0.5
        }
    );
    assert!(pousou(&mut sim, bola, 1, 240, &mut PhysicsBridge::new()));
}

/// O motor já a correr: o bloco que cresce a meio tem de apanhar a bola que ainda cai.
#[test]
fn a_forma_automatica_acompanha_com_o_jogo_a_correr() {
    let (mut sim, bloco, bola) = cena(AUTOMATICA);
    let mut bridge = PhysicsBridge::new();
    assert!(!pousou(&mut sim, bola, 1, 5, &mut bridge));
    assert!(set_tiled_side(sim.world_mut(), bloco, 0, 3.0));
    assert!(pousou(&mut sim, bola, 6, 240, &mut bridge));
}

#[test]
fn a_forma_que_o_artista_mudou_fica_como_esta() {
    let minha = ColliderShape::Cuboid {
        half_x: 0.7,
        half_y: 0.5,
    };
    let (mut sim, bloco, bola) = cena(minha);
    assert!(set_tiled_side(sim.world_mut(), bloco, 0, 3.0));
    assert_eq!(lados(&sim, bloco), minha);
    assert!(!pousou(&mut sim, bola, 1, 240, &mut PhysicsBridge::new()));
}

#[test]
fn a_bola_nao_acompanha_a_largura() {
    let minha = ColliderShape::Ball { radius: 1.0 };
    let (mut sim, bloco, _) = cena(minha);
    assert!(set_tiled_side(sim.world_mut(), bloco, 0, 3.0));
    assert_eq!(lados(&sim, bloco), minha);
}
