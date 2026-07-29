//! **A cadernal MÓVEL dirigida por curva é um guincho com vantagem 2**
//! (W-Pulley W3) — o item que o plano trazia como *"não medido, não gateado"*.
//!
//! Irmão de `pulley.rs` (680/700 LOC) e o corte é o assunto: lá mora o guincho por
//! **MOTOR** (o `L0` encolhe) e a corda amarrada num tambor kinematic (a PONTA se
//! move); aqui, o que acontece quando o corpo kinematic carrega uma **RODLANA**.
//!
//! # A nota era verdadeira, e a medição a tornou exata
//!
//! A afirmação vive no `end`: *"a velocidade do ponto (`v`) NÃO é zerada junto …
//! um corpo kinematic é massa infinita (a corda não o move) mas TEM movimento
//! próprio, então uma bobina animada por curva vira um guincho de graça"*. Metade
//! disso já era gateada (a PONTA, 1:1, em `pulley.rs`); a metade da RODLANA não —
//! e **não é o mesmo número**: uma cadernal móvel de enlace 180° tem vantagem
//! mecânica **2**.
//!
//! Medido (`measure_pulley_kinematic_sheave`), baixando o bloco `d`:
//!
//! | `d` | cadernal MÓVEL | a PONTA (controle) | razão |
//! |---|---|---|---|
//! | 0,20 | 1,907 | 0,952 | **2,003** |
//! | 0,50 | 1,917 | 0,958 | **2,001** |
//! | 1,00 | 1,928 | 0,964 | **2,000** |
//!
//! ⚠️ **Os dois números absolutos são `cos θ` da perna, não um atraso do solver:**
//! a perna do bloco à cadernal fixa mede `(0,9; 2,8)`, cuja componente vertical é
//! `2,8/2,941 = 0,952` — exatamente o controle. É por isso que o oráculo do gate é
//! a **RAZÃO entre os dois rigs**: ela cancela o ângulo, e o que sobra é a
//! vantagem mecânica, que é a coisa afirmada.

use ph2d_physics::world::pulley::PulleyDesc;
use ph2d_physics::world::rope_route::{self, RopeWheel};
use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyHandle, ShapeDesc};
use rapier2d::dynamics::RigidBodyType;

const R: f32 = 0.2;
const BOOM_Y: f32 = 6.0;
const SPAN: f32 = 0.9;
const BLOCK_Y: f32 = 3.2;
const LOAD_Y: f32 = 3.5;

fn body(kind: RigidBodyType, x: f32, y: f32, mass: f32) -> BodyDesc {
    BodyDesc {
        body_type: kind,
        x,
        y,
        rotation: 0.0,
        density: mass / (std::f32::consts::PI * R * R),
        shape: ShapeDesc::Ball { radius: R },
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
    }
}

/// **A talha dirigida por curva**, e o controle dela.
///
/// ```text
///   morta (-0.9, BOOM)  [estatica]      cadernal FIXA (+0.9, BOOM)  [cenario]
///          \                                   / \
///           \                                 /   carga (+0.9)  [dinamica]
///            \                               /
///             cadernal MOVEL (0, BLOCK_Y)  [montada no bloco KINEMATIC]
/// ```
///
/// `mounted = false` é o CONTROLE: a cadernal móvel sai da rota e o bloco
/// kinematic passa a ser a **PONTA** da corda — o caso que `pulley.rs` já gateia.
/// É essa troca, e só ela, que muda a vantagem de 2 para 1: os dois rigs têm a
/// mesma corda, a mesma carga e as mesmas pernas.
fn rig(mounted: bool) -> (PhysicsWorld, RigidBodyHandle, RigidBodyHandle) {
    let mut w = PhysicsWorld::new();
    let dead = w.spawn_body(body(RigidBodyType::Fixed, -SPAN, BOOM_Y, 1.0));
    let block = w.spawn_body(body(
        RigidBodyType::KinematicPositionBased,
        0.0,
        BLOCK_Y,
        1.0,
    ));
    let load = w.spawn_body(body(RigidBodyType::Dynamic, SPAN, LOAD_Y, 1.0));

    let mut wheels: Vec<RopeWheel> = Vec::new();
    if mounted {
        wheels.push(RopeWheel {
            centre: [0.0, BLOCK_Y],
            body: Some(block),
            local: [0.0, 0.0],
            radius: 0.0,
            side: 1,
            id: 1,
            break_force: f32::INFINITY,
            ..RopeWheel::default()
        });
    }
    wheels.push(RopeWheel {
        centre: [SPAN, BOOM_Y],
        radius: 0.0,
        side: 1,
        id: 2,
        break_force: f32::INFINITY,
        ..RopeWheel::default()
    });

    let a = if mounted { dead } else { block };
    let wa = if mounted {
        [-SPAN, BOOM_Y]
    } else {
        [0.0, BLOCK_Y]
    };
    let wb = [SPAN, LOAD_Y];
    rope_route::resolve_sides(wa, wb, &mut wheels, &mut Vec::new());
    // O comprimento é o da ROTA em repouso ⇒ a corda nasce esticada e o 1º tique
    // não dá um puxão (o transiente mediria o piso do `L0`, não a vantagem).
    let total_length = rope_route::route(wa, wb, &wheels, &mut Vec::new())
        .expect("a rota de repouso resolve")
        .length;
    w.set_pulleys(
        vec![PulleyDesc {
            body_a: a,
            body_b: load,
            local_a: [0.0, 0.0],
            local_b: [0.0, 0.0],
            wheel_start: 0,
            wheel_count: u32::try_from(wheels.len()).expect("conta cabe"),
            id: 1,
            total_length,
            motor_rate: 0.0,
            break_force: f32::INFINITY,
        }],
        wheels,
    );
    (w, block, load)
}

fn y(w: &PhysicsWorld, h: RigidBodyHandle) -> f32 {
    w.body_pose(h).expect("corpo vivo").translation.y
}

/// Baixa o bloco `travel` m em `ticks` tiques; devolve `carga andada / bloco
/// andado` — a razão adimensional que o gate compara.
fn haul_ratio(mounted: bool, travel: f32, ticks: u32) -> f32 {
    let (mut w, block, load) = rig(mounted);
    let (b0, l0) = (y(&w, block), y(&w, load));
    for t in 0..ticks {
        let frac = (t + 1) as f32 / ticks as f32;
        w.set_next_kinematic_pose(block, 0.0, BLOCK_Y - travel * frac, 0.0);
        w.step();
    }
    (y(&w, load) - l0) / -(y(&w, block) - b0)
}

/// **Uma cadernal móvel dirigida por curva ergue a carga ao DOBRO da taxa que a
/// mesma corda ergue quando o corpo kinematic é a PONTA.**
///
/// ⚠️ **O oráculo é o CONTROLE, nunca um literal.** Os dois números absolutos são
/// `cos θ` da perna (0,952 na geometria desta fixture), então cravar "1,907"
/// pinaria o ângulo do rig junto com a física — qualquer ajuste de fixture
/// quebraria o gate por um motivo que não é o que ele alega. A razão entre os dois
/// rigs cancela o ângulo e deixa só a vantagem mecânica.
///
/// ⚠️ **Nenhum "2" é escrito no produto** — a magnitude do Jacobiano do eixo
/// montado *é* a vantagem, e ela cai da geometria do enlace. É a mesma frase que o
/// `mounted_axle` já afirma sobre a talha estática; aqui ela é medida com o eixo
/// **dirigido**, que é o caso que o `end` promete e ninguém tinha conferido.
///
/// Mutação: zerar a velocidade do ponto para corpo não-dinâmico no `end` (a linha
/// que a nota do kernel defende) ⇒ a cadernal dirigida **para de içar** e a razão
/// cai a ~0.
#[test]
fn a_kinematic_sheave_hauls_at_twice_the_endpoints_rate() {
    for travel in [0.2_f32, 0.5, 1.0] {
        let mounted = haul_ratio(true, travel, 90);
        let endpoint = haul_ratio(false, travel, 90);
        assert!(
            endpoint > 0.5,
            "o CONTROLE tem de içar: a {travel} m ele deu {endpoint:.4} — sem isso a \
             razão abaixo não pode falhar pelo motivo que alega"
        );
        let advantage = mounted / endpoint;
        assert!(
            (advantage - 2.0).abs() < 0.02,
            "a cadernal MÓVEL tinha de içar o DOBRO da ponta (vantagem 2 de um \
             enlace de 180°); a {travel} m deu {advantage:.4} \
             (móvel {mounted:.4} · ponta {endpoint:.4})"
        );
    }
}

/// **E a carga não pode arrastar a cadernal dirigida** — é isso que faz dela um
/// guincho em vez de dois corpos negociando.
///
/// `inv_m = 0` para corpo não-dinâmico (a linha que o `end` já tem por causa da
/// PAREDE, medida em 5,8× de frouxidão), então o impulso da corda não o acelera. O
/// bloco é mandado ficar PARADO com uma carga pendurada nele.
#[test]
fn the_load_cannot_move_the_driven_sheave() {
    let (mut w, block, load) = rig(true);
    let (b0, l0) = (y(&w, block), y(&w, load));
    for _ in 0..120 {
        w.set_next_kinematic_pose(block, 0.0, BLOCK_Y, 0.0);
        w.step();
    }
    let moved = y(&w, block) - b0;
    assert!(
        moved.abs() < 1.0e-5,
        "a carga arrastou a cadernal dirigida {moved:.6} m — um corpo kinematic é \
         massa infinita para a corda"
    );
    // E a fixture contém o fenômeno: havia de fato carga puxando.
    assert!(
        (y(&w, load) - l0).abs() < 0.05,
        "a carga tinha de ficar pendurada (a corda a segura); ela andou {:.4} m",
        y(&w, load) - l0
    );
}
