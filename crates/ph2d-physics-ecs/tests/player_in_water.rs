//! **O PERSONAGEM CINEMÁTICO NA ÁGUA** — os gates da W-KinFluid.
//!
//! Report do Enio (2026-08-09): *"testou o kinemátic na água? ou ele não
//! funciona lá?"*. Não funcionava: medido, numa poça funda de 4 s o controle
//! boiava `1,1072 m` acima do ponto de largada, o player dinâmico `1,0893`, e o
//! **cinemático afundava `139,67 m`** — queda livre, com o multiplicador de
//! queda por cima.
//!
//! # ⚠️ O ORÁCULO é o modo DINÂMICO, e a primeira versão dele estava errada
//!
//! O primeiro número que eu reportei foi *"assenta a `0,4140`, três milímetros do
//! dinâmico"* — e ele era **um instante de uma oscilação**: com a poça da fixture
//! o player BOBEIA `1,44 m` de amplitude entre o terceiro e o sexto segundo, e o
//! `t = 4 s` calhou de o apanhar perto do dinâmico. *Uma amostra única de um
//! sistema que oscila não é um repouso.*
//!
//! ⚠️ **E a oscilação não é desta wave:** o corpo DINÂMICO faz `1,4408` na mesma
//! cena contra os `1,4394` do cinemático — eles concordam na quarta decimal, e a
//! cápsula solta (sem lei de player nenhuma) faz `0,8097`. O que bobeia é o
//! *player* na água, nos dois modos, e isso é anterior a esta wave.
//!
//! Por isso o oráculo é a **PARIDADE ENTRE MODOS**: a mesma cena, o mesmo tique,
//! e a espécie do corpo a não ser uma pergunta que a água faça. Um literal de
//! linha d'água seria um espelho da fórmula; a paridade é uma propriedade.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test player_in_water`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    AreaBuoyancy, AreaDrag, BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge,
    PlatformPlayer, PlayerInput, PlayerMode, RigidBody,
};

/// A cápsula das fixtures do player, e a poça 4× mais densa que ela.
const HALF_H: f32 = 0.3;
const RADIUS: f32 = 0.2;
const FLOAT: f32 = 0.9;
const FLUID: f32 = 4.0;
const START: f32 = 1.5;

/// A poça funda: nada ao alcance do sensor de chão.
///
/// ⚠️ **O arrasto não é enfeite** — empuxo sem resistência é uma mola sem
/// amortecimento, e a fixture irmã já carregava essa frase. Medido nesta cena:
/// com `AreaDrag 0` a amplitude do cinemático na segunda metade é **2,90 m**,
/// contra **1,44** com o `0,6` daqui.
fn pool(sim: &mut SimWorld, drag: f32) {
    let mut e = sim.world_mut().spawn((
        Name::new("Pool"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 3.0,
            },
            ..Collider::default()
        },
        AreaBuoyancy(FLUID),
        Transform::from_translation(Vec2::new(0.0, -3.0)),
    ));
    if drag > 0.0 {
        e.insert(AreaDrag(drag));
    }
}

fn player(sim: &mut SimWorld, kinematic: bool) -> Entity {
    let mut e = sim.world_mut().spawn((
        Name::new("Subject"),
        RigidBody {
            kind: if kinematic {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            },
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: HALF_H,
                radius: RADIUS,
            },
            density: 1.0,
            ..Collider::default()
        },
        LockRotation,
        PlatformPlayer {
            float_height: FLOAT,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(0.0, START)),
    ));
    if kinematic {
        e.insert(PlayerMode::Kinematic);
    }
    e.id()
}

fn y_of(sim: &SimWorld) -> f32 {
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == "Subject" {
            return t.translation.y;
        }
    }
    panic!("o sujeito tem de existir");
}

/// A média e a amplitude de `y` na **segunda metade** de seis segundos.
///
/// ⚠️ A primeira metade contém a entrada na água, que não é oscilação — medi-la
/// junto misturaria o transiente com o regime.
fn settle_stats(kinematic: bool, drag: f32) -> (f32, f32) {
    let mut sim = SimWorld::new();
    pool(&mut sim, drag);
    let who = player(&mut sim, kinematic);
    let mut bridge = PhysicsBridge::new();
    bridge.set_player_input(who, PlayerInput::default());
    let (mut lo, mut hi, mut sum, mut n) = (f32::MAX, f32::MIN, 0.0f64, 0u32);
    for t in 1..=360u64 {
        bridge.dispatch(&mut sim, true, t);
        if t > 180 {
            let y = y_of(&sim);
            lo = lo.min(y);
            hi = hi.max(y);
            sum += f64::from(y);
            n += 1;
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    ((sum / f64::from(n)) as f32, hi - lo)
}

/// **A ÁGUA EXISTE PARA O MODO CINEMÁTICO, e ela faz com ele o que faz com o
/// dinâmico** — o gate que a wave serve.
///
/// ⚠️ **Nasceu VERMELHO com `−138,17` contra `+0,41`.** O empuxo e o arrasto de
/// uma zona chegam por `apply_impulse` no corpo, e um corpo cinemático tem massa
/// infinita para o solver: é a mesma frase que explicava por que o `move_shape`
/// não empurrava um caixote antes da W-KinPush, agora do outro lado — ele não
/// RECEBIA.
///
/// ⚠️ **As duas metades são precisas:** a média diz *onde ele vive* e a amplitude
/// diz *como ele se move*. Só a média deixaria passar uma lei que o pusesse no
/// lugar certo por dois caminhos errados que se cancelam; só a amplitude deixaria
/// passar uma que o afogasse suavemente.
#[test]
fn the_kinematic_player_floats_like_the_dynamic_one() {
    let (dyn_mean, dyn_amp) = settle_stats(false, 0.6);
    let (kin_mean, kin_amp) = settle_stats(true, 0.6);

    assert!(
        (kin_mean - dyn_mean).abs() < 0.1,
        "o cinemático tem de viver onde o dinâmico vive: {kin_mean:.4} contra {dyn_mean:.4}"
    );
    assert!(
        (kin_amp - dyn_amp).abs() < 0.1,
        "e mover-se como ele: amplitude {kin_amp:.4} contra {dyn_amp:.4}"
    );
    // E o número absoluto, para o gate não passar com os DOIS afogados.
    assert!(
        kin_mean > 0.0,
        "os dois têm de estar ACIMA da superfície (y = 0), e o cinemático está em {kin_mean:.4}"
    );
}

/// **CONTROLE: sem poça, ele CAI** — o gate acima não pode passar sobre um
/// personagem que simplesmente parou de obedecer à gravidade.
///
/// ⚠️ A ablação é pela ENTRADA (a poça deixa de existir), nunca por
/// instrumentação: é a mesma cena, o mesmo corpo, o mesmo relógio.
#[test]
fn without_a_pool_the_kinematic_player_still_falls() {
    let mut sim = SimWorld::new();
    let who = player(&mut sim, true);
    let mut bridge = PhysicsBridge::new();
    bridge.set_player_input(who, PlayerInput::default());
    for t in 1..=240u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let y = y_of(&sim);
    // Queda livre de 4 s são ~78 m de gravidade nominal; o multiplicador de
    // queda da lei do pulo a aumenta. O que se afirma é a ORDEM, não o número.
    assert!(
        y < START - 50.0,
        "sem água ele tem de cair, e caiu só até {y:.4}"
    );
}

/// **O MEIO é o que o freia — e o gate mede a ablação do knob do artista.**
///
/// ⚠️ Sem o arrasto o empuxo é um oscilador conservativo: a amplitude não decai,
/// porque não há nada que consuma energia. Medido nesta fixture: `2,90 m` com
/// `AreaDrag 0` contra `1,44` com `0,6` — e a diferença é o termo que a lei
/// integra, não uma propriedade do solver (que nem toca num corpo cinemático).
#[test]
fn the_mediums_drag_is_what_damps_the_bobbing() {
    let (_, loose) = settle_stats(true, 0.0);
    let (_, damped) = settle_stats(true, 0.6);
    assert!(
        loose > damped * 1.5,
        "sem arrasto ele tem de oscilar bem mais: {loose:.4} contra {damped:.4}"
    );
}
