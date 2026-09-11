//! **CCD stops a fast body that discrete detection lets tunnel** (W-CCD).
//!
//! Discrete collision detection tests a body only at each (sub-)step's END
//! pose, so a small fast body can be on one side of thin geometry at one pose
//! and clean past it at the next — never overlapping, never colliding. That is
//! the collision a game MISSES (a bullet through a wall), distinct from the deep
//! LANDING overlap the sub-stepping default fixes (a heavy body that is already
//! inside the floor the frame it touches). CCD sweeps the body's motion between
//! poses and stops it at the first impact.
//!
//! ⛔⛔ **O QUE A BANDEIRA `ccd` COMPRA MUDOU na `rapier2d` 0.35, e este ficheiro mudou com ela.**
//!
//! Até à 0.31, `ccd` ligava ou desligava a varredura contra **tudo**, e o gate era: *bola rápida
//! contra parede FIXA fina — sem a bandeira atravessa, com ela pára*. Na 0.35 o doc do
//! `CCDSolver` diz outra coisa:
//!
//! > *«Fast dynamic bodies automatically sweep against **fixed** colliders; `ccd_enabled` upgrades
//! > to a **bullet** that also sweeps kinematic/dynamic bodies (never other bullets).»*
//!
//! ⇒ **Contra cenário FIXO a protecção passou a ser de graça** — um projéctil já não atravessa uma
//! parede estática, com ou sem bandeira. Isso é produto novo e é bom. Mas apaga o *controlo* do
//! gate antigo: ele provava que a bandeira faz diferença **usando uma parede fixa**, e essa
//! diferença deixou de existir.
//!
//! ⚠️ *Um gate cujo controlo deixou de conter o fenómeno não fica «desactualizado» — ele fica
//! VERDE-por-ausência, e o teste antigo dizia-o na mensagem dele: «a fixtura já não contém o
//! fenómeno que o gate existe para provar».* Foi essa frase que apanhou isto.
//!
//! ⇒ O ficheiro passa a ter **dois** gates, um por cada metade do que a 0.35 promete:
//!
//! 1. [`a_fast_body_no_longer_tunnels_through_fixed_scenery_even_without_the_flag`] — a metade de
//!    graça. Ela é o **oráculo do produto novo**, e teria sido a regressão mais cara de não notar.
//! 2. [`the_ccd_flag_is_what_stops_a_bullet_through_a_MOVING_wall`] — a metade que a bandeira ainda
//!    compra, medida contra a classe de alvo onde ela agora manda: um corpo que **se move**.

use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

/// Launch a small ball at `speed` m/s straight at a thin wall at x=0 and return
/// where it ends up after it has had time to cross. `ccd` toggles continuous
/// detection on the ball; everything else is identical.
///
/// Gravity is zeroed so the only motion is the horizontal launch — the tunnel
/// is a fact about detection, not a parabola that happens to clear the wall.
fn final_x_after_launch(ccd: bool, speed: f32) -> f32 {
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);

    // A thin, TALL static wall at x=0: 0.04 m thick (half 0.02), 2 m tall so the
    // ball cannot go around it. This is the "thin geometry" a fast body tunnels.
    w.spawn_body(BodyDesc {
        body_type: RigidBodyType::Fixed,
        x: 0.0,
        y: 0.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Cuboid {
            half_x: 0.02,
            half_y: 1.0,
        },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    });

    // A small ball, one metre to the LEFT, launched right at `speed` m/s. At
    // 80 m/s it moves 80/240 ≈ 0.33 m per sub-step (default 4 sub-steps) — far
    // more than the 0.04 m wall plus the ball's own 0.1 m diameter, so no
    // discrete pose ever overlaps the wall.
    let ball = w.spawn_body(BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x: -1.0,
        y: 0.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Ball { radius: 0.05 },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [speed, 0.0],
        angvel: 0.0,
        ccd,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    });

    // 30 ticks = 0.5 s: at 80 m/s the ball would travel 40 m unobstructed, so a
    // tunnelled ball is far past the wall and a stopped one has long since
    // settled against it.
    for _ in 0..30 {
        w.step();
    }
    w.body_pose(ball).expect("ball exists").translation.x
}

/// ⭐⭐ **A metade DE GRAÇA: cenário fixo já não é atravessado, com ou sem a bandeira.**
///
/// Era exactamente o contrário até à `rapier2d` 0.31 — a `200 m/s` a bola discreta saía do outro
/// lado em `x ≈ 99`. Hoje pára nos dois casos, e a bandeira não é o que a segura.
///
/// ⚠️ **Este gate é o oráculo do produto novo, e ele vale mais do que o que substituiu:** um dia
/// em que uma subida devolva o túnel contra cenário fixo, é ele que acorda — e essa é a regressão
/// que um jogo sente primeiro (a bala que sai pela parede).
#[test]
fn a_fast_body_no_longer_tunnels_through_fixed_scenery_even_without_the_flag() {
    const SPEED: f32 = 200.0;

    let discrete_x = final_x_after_launch(false, SPEED);
    assert!(
        discrete_x < 0.0,
        "uma bola rapida SEM a bandeira `ccd` atravessou a parede fixa e acabou em x={discrete_x}. \
         A rapier 0.35 varre todo corpo dinamico rapido contra colliders FIXOS por omissao (doc do \
         `CCDSolver`), entao isto e' a protecao de graca a desaparecer -- e e' a regressao que um \
         jogo sente primeiro."
    );

    let ccd_x = final_x_after_launch(true, SPEED);
    assert!(
        ccd_x < 0.0,
        "com a bandeira ligada tambem tinha de parar, e acabou em x={ccd_x}"
    );
}

/// ⭐⭐ **A metade que a BANDEIRA ainda compra: o alvo que se MOVE.**
///
/// Na 0.35 a bandeira promove o corpo a *bullet*, e um bullet varre também alvos **cinemáticos e
/// dinâmicos** — que é a classe onde a varredura por omissão não vai. É contra essa classe que o
/// controlo do gate volta a existir.
///
/// ⚠️ A parede é **cinemática e parada**: cinemática para sair da classe «fixo» (que é varrida de
/// graça) e parada para o fenómeno medido ser o túnel, e não um encontro. *O controlo tem de
/// mudar a classe do ALVO, não a velocidade da bola — mexer na velocidade só reescreveria o mesmo
/// gate com outro número.*
///
/// Mutação: trocar `.ccd_enabled(desc.ccd)` por `.ccd_enabled(false)` no `spawn_body` faz a bola
/// com bandeira atravessar também, e a asserção do «parou» fica VERMELHA.
#[test]
fn the_ccd_flag_is_what_stops_a_bullet_through_a_moving_wall() {
    const SPEED: f32 = 200.0;

    let launch = |ccd: bool| -> f32 {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        // A MESMA parede fina, mas CINEMÁTICA — fora da classe que a 0.35 varre de graça.
        w.spawn_body(BodyDesc {
            body_type: RigidBodyType::KinematicPositionBased,
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            density: 1.0,
            shape: ShapeDesc::Cuboid {
                half_x: 0.02,
                half_y: 1.0,
            },
            restitution: 0.0,
            friction: 0.5,
            layer: 0,
            is_sensor: false,
            gravity_scale: 1.0,
            linvel: [0.0, 0.0],
            angvel: 0.0,
            ccd: false,
            lock_rotation: false,
            lock_x: false,
            lock_y: false,
            mass_override: None,
            dominance: 0,
            material: Default::default(),
            damping: None,
            one_way: false,
            effector: None,
            offset: [0.0, 0.0],
        });
        let ball = w.spawn_body(BodyDesc {
            body_type: RigidBodyType::Dynamic,
            x: -1.0,
            y: 0.0,
            rotation: 0.0,
            density: 1.0,
            shape: ShapeDesc::Ball { radius: 0.05 },
            restitution: 0.0,
            friction: 0.5,
            layer: 0,
            is_sensor: false,
            gravity_scale: 1.0,
            linvel: [SPEED, 0.0],
            angvel: 0.0,
            ccd,
            lock_rotation: false,
            lock_x: false,
            lock_y: false,
            mass_override: None,
            dominance: 0,
            material: Default::default(),
            damping: None,
            one_way: false,
            effector: None,
            offset: [0.0, 0.0],
        });
        for _ in 0..30 {
            w.step();
        }
        w.body_pose(ball).expect("ball exists").translation.x
    };

    let discrete_x = launch(false);
    assert!(
        discrete_x > 0.5,
        "SEM a bandeira, a bola tinha de atravessar a parede CINEMATICA (a varredura de graca da \
         0.35 so' alcanca colliders FIXOS), e ela acabou em x={discrete_x}. Se isto deixar de ser \
         verdade, a fixtura ja' nao contem o fenomeno -- e nesse caso a bandeira `ccd` passou a nao \
         comprar NADA, o que e' uma decisao de produto (um knob morto), nao um teste a afinar."
    );

    let ccd_x = launch(true);
    assert!(
        ccd_x < 0.0,
        "COM a bandeira, o corpo vira um *bullet* e varre tambem alvos cinematicos/dinamicos, \
         entao ele tinha de parar do lado de ca' -- e acabou em x={ccd_x}"
    );
}
