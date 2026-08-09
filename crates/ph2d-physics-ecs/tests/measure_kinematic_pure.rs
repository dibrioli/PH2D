//! **SONDA — o que o TERCEIRO modo tem de fazer** (W-KinPure).
//!
//! O plano diz, em uma frase, o que o *"puro sangue"* é: *nada de reação, nada
//! de empurrão — o platformer clássico, em que o mundo físico é cenário*. Antes
//! de existir um variant, esta sonda mede as **três** colunas pelas portas do
//! produto, obtendo a terceira do jeito que ela é alcançável hoje: o modo Snap
//! com os três escalares da reação a zero.
//!
//! ⚠️ **É essa terceira coluna que decide o tamanho da wave.** Se ela já der o
//! comportamento do platformer clássico, então o modo novo **não é capacidade
//! nova** — é uma declaração de intenção com uma porta só, e a wave tem de o
//! dizer em vez de vender o contrário.
//!
//! ⚠️ **E as duas primeiras colunas são o CONTROLE.** Sem uma coluna que afunda
//! e empurra, um zero na terceira não distingue *"o modo cala o canal"* de *"a
//! cena não tem o que afundar"*.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_kinematic_pure -- --ignored --nocapture`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, GravityScale, LockRotation, PhysicsBridge, PlatformPlayer,
    PlayerMode, RigidBody,
};
use ph2d_platformer::PlayerInput;

const FLOAT_HEIGHT: f32 = 0.9;

/// As três colunas da tabela: o que o artista pode escolher hoje, e o que ele
/// poderá escolher depois desta wave.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Mode {
    /// A cápsula flutuante — o solver é dono da pose.
    Spring,
    /// O controlador com a 3ª lei viva.
    Snap,
    /// O controlador com os três escalares a zero — o `Pure` de amanhã,
    /// alcançado hoje pelos knobs.
    PureByKnobs,
}

impl Mode {
    fn name(self) -> &'static str {
        match self {
            Mode::Spring => "dinamico",
            Mode::Snap => "CINEMATICO",
            Mode::PureByKnobs => "cine + zeros",
        }
    }
    fn kinematic(self) -> bool {
        !matches!(self, Mode::Spring)
    }
    fn player(self) -> PlatformPlayer {
        let mut p = PlatformPlayer {
            float_height: FLOAT_HEIGHT,
            ..PlatformPlayer::default()
        };
        if matches!(self, Mode::PureByKnobs) {
            p.reaction_support = 0.0;
            p.reaction_movement = 0.0;
            p.reaction_push = 0.0;
        }
        p
    }
}

fn spawn_player(sim: &mut SimWorld, mode: Mode, at: Vec2) -> Entity {
    let mut e = sim.world_mut().spawn((
        Name::new("Player".to_string()),
        RigidBody {
            kind: if mode.kinematic() {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            },
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 0.3,
                radius: 0.2,
            },
            ..Collider::default()
        },
        LockRotation,
        mode.player(),
        Transform::from_translation(at),
    ));
    if mode.kinematic() {
        e.insert(PlayerMode::Kinematic);
    }
    e.id()
}

fn floor(sim: &mut SimWorld, at: Vec2, half: [f32; 2]) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Floor".to_string()),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: half[0],
                    half_y: half[1],
                },
                ..Collider::default()
            },
            Transform::from_translation(at),
        ))
        .id()
}

fn y_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).expect("pose").translation.y
}
fn x_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).expect("pose").translation.x
}

fn drive(bridge: &mut PhysicsBridge, e: Entity, drive: f32) {
    bridge.set_player_input(
        e,
        PlayerInput {
            drive,
            ..PlayerInput::default()
        },
    );
}

/// **1 — O PESO.** Uma jangada sem peso próprio (`GravityScale(0)`), então todo
/// milímetro que ela desce é do personagem. É a fixture da W-KinWeight.
fn raft_sink(mode: Mode) -> f32 {
    let mut sim = SimWorld::new();
    let raft = sim
        .world_mut()
        .spawn((
            Name::new("Raft".to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 30.0,
                    half_y: 0.25,
                },
                ..Collider::default()
            },
            GravityScale(0.0),
            LockRotation,
            Transform::from_translation(Vec2::new(0.0, -0.25)),
        ))
        .id();
    let p = spawn_player(&mut sim, mode, Vec2::new(0.0, FLOAT_HEIGHT));
    let mut bridge = PhysicsBridge::new();
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let before = y_of(&sim, raft);
    for t in 61..=180u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let _ = p;
    y_of(&sim, raft) - before
}

/// **2 — O EMPURRÃO.** A pista é LIVRE: nada de parede, para a viagem ser
/// limitada pela lei e não pela geometria.
fn crate_travel(mode: Mode) -> f32 {
    let mut sim = SimWorld::new();
    floor(&mut sim, Vec2::new(0.0, -0.5), [60.0, 0.5]);
    let krate = sim
        .world_mut()
        .spawn((
            Name::new("Crate".to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.3,
                    half_y: 0.3,
                },
                ..Collider::default()
            },
            LockRotation,
            Transform::from_translation(Vec2::new(1.5, 0.3)),
        ))
        .id();
    let p = spawn_player(&mut sim, mode, Vec2::new(0.0, FLOAT_HEIGHT));
    let mut bridge = PhysicsBridge::new();
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let before = x_of(&sim, krate);
    drive(&mut bridge, p, 1.0);
    for t in 61..=240u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    x_of(&sim, krate) - before
}

/// **3 — A CAMINHADA e o PULO.** A lei de intenção não pode ver a diferença: se
/// vir, o modo novo não é *"os dois canais calados"*, é outro personagem.
fn walk_and_jump(mode: Mode) -> (f32, f32) {
    let mut sim = SimWorld::new();
    floor(&mut sim, Vec2::new(0.0, -0.5), [60.0, 0.5]);
    let p = spawn_player(&mut sim, mode, Vec2::new(0.0, FLOAT_HEIGHT));
    let mut bridge = PhysicsBridge::new();
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let x0 = x_of(&sim, p);
    drive(&mut bridge, p, 1.0);
    for t in 61..=180u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let walked = x_of(&sim, p) - x0;
    let ground = y_of(&sim, p);
    bridge.set_player_input(
        p,
        PlayerInput {
            drive: 1.0,
            jump: true,
            ..PlayerInput::default()
        },
    );
    let mut apex = ground;
    for t in 181..=300u64 {
        bridge.dispatch(&mut sim, true, t);
        apex = apex.max(y_of(&sim, p));
    }
    (walked, apex - ground)
}

/// **4 — A PLATAFORMA MÓVEL (K7).** Ser LEVADO não é influenciar: um platformer
/// clássico anda em cima de uma plataforma. Se o modo novo perdesse isto, teria
/// calado o canal errado.
fn carried_by_platform(mode: Mode) -> (f32, f32) {
    let mut sim = SimWorld::new();
    let plat = sim
        .world_mut()
        .spawn((
            Name::new("Platform".to_string()),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 4.0,
                    half_y: 0.25,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, -0.25)),
        ))
        .id();
    let p = spawn_player(&mut sim, mode, Vec2::new(0.0, FLOAT_HEIGHT));
    let mut bridge = PhysicsBridge::new();
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let before = x_of(&sim, p);
    let plat_before = x_of(&sim, plat);
    // A plataforma é dirigida pela CENA, um tique de cada vez.
    for t in 61..=180u64 {
        let x = x_of(&sim, plat);
        sim.world_mut()
            .get_mut::<Transform>(plat)
            .expect("pose")
            .translation
            .x = x + 2.0 / 60.0;
        bridge.dispatch(&mut sim, true, t);
    }
    (x_of(&sim, p) - before, x_of(&sim, plat) - plat_before)
}

/// **4b — A MESMA pergunta com uma plataforma cuja velocidade é do SOLVER.**
///
/// ⚠️ Discriminador: se aqui a conta fechar e na `carried_by_platform` não, o
/// defeito é de como uma pose escrita pela CENA vira velocidade, não da lei.
fn carried_by_drifting_platform(mode: Mode) -> (f32, f32) {
    let mut sim = SimWorld::new();
    let plat = sim
        .world_mut()
        .spawn((
            Name::new("Platform".to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 8.0,
                    half_y: 0.25,
                },
                ..Collider::default()
            },
            GravityScale(0.0),
            LockRotation,
            ph2d_physics_ecs::InitialVelocity {
                linvel: [2.0, 0.0],
                angvel: 0.0,
            },
            Transform::from_translation(Vec2::new(0.0, -0.25)),
        ))
        .id();
    let p = spawn_player(&mut sim, mode, Vec2::new(0.0, FLOAT_HEIGHT));
    let mut bridge = PhysicsBridge::new();
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let before = x_of(&sim, p);
    let plat_before = x_of(&sim, plat);
    for t in 61..=180u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    (x_of(&sim, p) - before, x_of(&sim, plat) - plat_before)
}

/// **5 — O personagem é SÓLIDO?** Um caixote lançado contra ele tem de parar.
/// *Cenário* quer dizer que o mundo não o obedece, não que ele seja um fantasma
/// — e esta coluna é o que impede a wave de "consertar" isso por engano.
fn crate_blocked_by_player(mode: Mode) -> f32 {
    let mut sim = SimWorld::new();
    floor(&mut sim, Vec2::new(0.0, -0.5), [60.0, 0.5]);
    let p = spawn_player(&mut sim, mode, Vec2::new(0.0, FLOAT_HEIGHT));
    let krate = sim
        .world_mut()
        .spawn((
            Name::new("Crate".to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.3,
                    half_y: 0.3,
                },
                ..Collider::default()
            },
            LockRotation,
            ph2d_physics_ecs::InitialVelocity {
                linvel: [-6.0, 0.0],
                angvel: 0.0,
            },
            Transform::from_translation(Vec2::new(3.0, 0.3)),
        ))
        .id();
    let mut bridge = PhysicsBridge::new();
    for t in 1..=180u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let _ = p;
    // Onde o caixote parou. À esquerda do personagem (x < 0) = atravessou.
    x_of(&sim, krate)
}

#[test]
#[ignore = "sonda"]
fn measure_what_the_third_mode_would_change() {
    println!(
        "\n  modo           jangada(m)   caixote(m)   andou(m)  pulo(m)  levado/plat     drift/plat    caixote"
    );
    for mode in [Mode::Spring, Mode::Snap, Mode::PureByKnobs] {
        let sink = raft_sink(mode);
        let travel = crate_travel(mode);
        let (walked, jump) = walk_and_jump(mode);
        let (carried, platform) = carried_by_platform(mode);
        let (drift, drift_plat) = carried_by_drifting_platform(mode);
        let blocked = crate_blocked_by_player(mode);
        println!(
            "  {:<13}  {sink:>9.4}  {travel:>10.4}  {walked:>9.4} {jump:>7.4}  {carried:>8.4}/{platform:<6.4}  {drift:>7.4}/{drift_plat:<6.4}  {blocked:>8.4}",
            mode.name()
        );
    }
    println!(
        "\n  A coluna 'caixote para em' e' o x FINAL de um caixote lancado contra\n  \
         o personagem, que esta' em x=0: um numero POSITIVO quer dizer que ele\n  \
         foi barrado, e negativo que atravessou.\n"
    );
}
