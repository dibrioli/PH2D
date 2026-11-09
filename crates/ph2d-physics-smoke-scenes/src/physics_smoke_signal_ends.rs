//! **A CENA 121 — QUEM OUVE A MOEDA** (GEF02 onda 1): a caixa de um nome de sinal diz se alguém
//! está na outra ponta. «Coin A» grita `coin_taken` e o «Score» ouve-o (`1 listens`); «Coin B»
//! grita `coin_takn`, com a letra trocada de propósito, e ninguém o ouve (`nobody listens`, em
//! aviso). `PH2D_PHYSICS_SMOKE_SELECT=<nome>` escolhe outro objecto para as fotos.

use ph2d_core::Vec2;
use ph2d_ecs::signal_actions::{SignalAction, SignalActions, SignalVerb};
use ph2d_ecs::{Entity, Name, Transform, World};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PlatformPlayer, RigidBody, SignalOnHit,
};
use ph2d_physics_scene::spawn_floor;
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// O nome que o «Score» ouve, e o erro de uma letra da segunda moeda.
pub const OUVIDO: &str = "coin_taken";
pub const TROCADO: &str = "coin_takn";

fn moeda(world: &mut World, nome: &str, x: f32, sinal: &str) -> Entity {
    world
        .spawn((
            Name::new(nome),
            Transform::from_translation(Vec2::new(x, 1.2)),
            Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], [0.98, 0.80, 0.20, 1.0]),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                is_sensor: true,
                ..Collider::default()
            },
            SignalOnHit(sinal.into()),
        ))
        .id()
}

/// **A geometria da cena 121**: devolve `(Coin A, Coin B, Score)`.
pub fn build_signal_ends_scene(world: &mut World) -> (Entity, Entity, Entity) {
    spawn_floor(world);
    world.spawn((
        Name::new("Player"),
        Transform::from_translation(Vec2::new(-4.0, 1.5)),
        Sprite::atlas(WHITE_TILE_KEY, [0.4, 1.0], [0.35, 0.65, 0.95, 1.0]),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 0.3,
                radius: 0.2,
            },
            ..Collider::default()
        },
        LockRotation,
        PlatformPlayer::default(),
    ));
    let a = moeda(world, "Coin A", -1.0, OUVIDO);
    let b = moeda(world, "Coin B", 2.0, TROCADO);
    let score = world
        .spawn((
            Name::new("Score"),
            Transform::from_translation(Vec2::new(3.5, 3.0)),
            Sprite::atlas(WHITE_TILE_KEY, [1.2, 0.5], [0.55, 0.85, 0.45, 1.0]),
            SignalActions(vec![SignalAction {
                on: OUVIDO.into(),
                target: "Coin A".into(),
                verb: SignalVerb::Destroy,
                ..Default::default()
            }]),
        ))
        .id();
    (a, b, score)
}

/// **A cena 121** — nasce PAUSADA e com a «Coin A» escolhida.
pub fn physics_smoke_signal_ends(ctx: &mut crate::SceneCtx<'_>) {
    let (a, _, _) = build_signal_ends_scene(ctx.world);
    ctx.want.camera_center = Some([0.5, 1.0]);
    ctx.want.camera_height_world = Some(9.0);
    ctx.want.select = Some(a.to_bits());
    if let Ok(nome) = std::env::var("PH2D_PHYSICS_SMOKE_SELECT") {
        let mut q = ctx.world.query::<(Entity, &Name)>();
        ctx.want.select = q
            .iter(ctx.world)
            .find(|(_, n)| n.0 == nome.trim())
            .map(|(e, _)| e.to_bits());
    }
    println!(
        "[physics-smoke 121] WHO HEARS THE COIN, PAUSED. Coin A is selected: its On Hit box says \
         \"coin_taken\" and, at the right edge of the box, \"1 listens\" (the Score object reacts to it). \
         Select Coin B: its On Hit says \"coin_takn\" (one letter missing) and the box says \"nobody \
         listens\" in the warning color. Select Score: its On box says \"1 sends\". Type the missing \
         letter in Coin B (coin_taken): Coin B now says \"1 listens\" and Score says \"2 send\"."
    );
}

#[cfg(test)]
#[test]
fn a_cena_ensina_1_listens_e_nobody_listens() {
    use ph2d_signal_census::{Hint, Role, census};
    let mut w = World::new();
    build_signal_ends_scene(&mut w);
    let c = census(&w);
    assert_eq!(c.hint(OUVIDO, Role::Emit), Hint::Count(1));
    assert_eq!(c.hint(TROCADO, Role::Emit), Hint::Nobody);
    assert_eq!(c.hint(OUVIDO, Role::Listen), Hint::Count(1));
}
