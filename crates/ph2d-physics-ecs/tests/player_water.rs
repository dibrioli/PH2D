//! **UM SENSOR NÃO É CHÃO** (W-Water).
//!
//! Report do Enio (2026-08-07): *"nosso player atual não interage corretamente
//! com a água (como a jangada faz)"*. Medido antes da cura
//! (`measure_player_in_water.rs`): ele assenta em `y = 0,9023` — **exatamente a
//! `float_height` acima do topo da poça** —, ou seja **de pé sobre a água**, e
//! com `x = 0,0000` depois de cinco segundos de correnteza.
//!
//! # ⚠️ A causa não é sobre água
//!
//! O `cast_ray` do wrapper monta `QueryFilter { .., ..default() }`, e
//! `QueryFilterFlags::empty()` no rapier significa literalmente *"no filter"* ⇒
//! **um sensor responde ao raio como matéria sólida**. O `buoyancy.rs` já
//! escreve a frase do outro lado — *"um sensor não desloca fluido: um sensor é
//! um marcador, não matéria"* —, e estes gates são a mesma frase deste lado: se
//! não é matéria, **não se fica de pé em cima, não se bate a cabeça e não se
//! escorrega na parede**.
//!
//! # ⚠️ O oráculo é o CONTROLE, não um literal
//!
//! Cada gate compara o player com uma cápsula **idêntica** sem o
//! `PlatformPlayer` — mesma forma, mesma densidade, mesma poça — ou com o mesmo
//! player numa cena **sem o sensor**. Nenhum número desta família é escolhido.

#[path = "platform_water_scene.rs"]
mod water;

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, PlayerInput, RigidBody};
use water::{FLOAT, HALF_H, RADIUS, pool, subject, xy_of, y_of};

/// A meia-altura total da cápsula — o que separa o centro do pé.
const HALF_TALL: f32 = HALF_H + RADIUS;
/// O topo da poça do `platform_water_scene`.
const POOL_TOP: f32 = 0.0;

fn run(sim: &mut SimWorld, bridge: &mut PhysicsBridge, ticks: u64) {
    for t in 1..=ticks {
        bridge.dispatch(sim, true, t);
    }
}

/// **Ele tem de MOLHAR o pé.**
///
/// Um corpo que bóia tem parte de si abaixo da superfície — é o que boiar é. De
/// pé sobre a água, o pé do personagem fica em `0,9023 − 0,5 = 0,4023`, meio
/// metro de ar abaixo dele e **zero** de submersão.
#[test]
fn a_player_does_not_stand_on_the_surface_of_a_pool() {
    let mut sim = SimWorld::new();
    pool(&mut sim, 0.0);
    let _ = subject(&mut sim, true, 0.5);
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 600);

    let y = y_of(&sim, "Subject");
    let foot = y - HALF_TALL;
    assert!(
        foot < POOL_TOP,
        "o personagem tem de tocar a agua: pe em {foot:.4}, superficie em {POOL_TOP:.4} \
         (centro {y:.4}; de pe sobre a poca ele fica em float_height = {FLOAT:.3})"
    );
}

/// **E ele tem de boiar onde uma cápsula idêntica boia.**
///
/// ⚠️ **A tolerância é NOMEADA, não escolhida:** o resíduo medido é `0,127 m`, e
/// a ablação o atribui inteiro aos multiplicadores de gravidade do pulo — com
/// eles neutros a diferença é `+0,0000`. Um personagem a flutuar tem velocidade
/// vertical perto de zero, que é a janela do `peak_gravity`, então ele vive no
/// **ápice de um pulo que nunca deu** e pesa menos. Isso é a metade B da wave e
/// uma decisão de produto; aqui a barra é `0,25 m` **só** para separar *boiar
/// mais alto* de *ficar de pé em cima* (que são `0,68 m`).
#[test]
fn a_floating_player_settles_near_the_waterline_of_an_identical_capsule() {
    let mut control = SimWorld::new();
    pool(&mut control, 0.0);
    let _ = subject(&mut control, false, 0.5);
    let mut b1 = PhysicsBridge::new();
    run(&mut control, &mut b1, 600);
    let line = y_of(&control, "Subject");

    let mut sim = SimWorld::new();
    pool(&mut sim, 0.0);
    let _ = subject(&mut sim, true, 0.5);
    let mut b2 = PhysicsBridge::new();
    run(&mut sim, &mut b2, 600);
    let y = y_of(&sim, "Subject");

    assert!(
        (y - line).abs() < 0.25,
        "o player boia em {y:.4} e a capsula identica em {line:.4} (delta {:.4}) \
         -- de pe sobre a poca o delta e' 0,68",
        y - line
    );
}

/// **A correnteza ALCANÇA o personagem.**
///
/// De pé sobre a água ele era **imune** — `x = 0,0000` em cinco segundos —,
/// porque quem está no chão tem a caminhada a frear a velocidade para o alvo, e
/// o alvo de um dedo parado é zero.
///
/// ⚠️ **Este gate afirma o que a cura ENTREGA, e não mais:** o controle interno
/// é o MESMO player em água PARADA. Que ele seja levado *tanto quanto* uma
/// jangada é outra coisa, e a medição diz quanto falta — ver o gate seguinte,
/// que a pina como o defeito que ela é.
#[test]
fn a_current_reaches_a_floating_player() {
    const CURRENT: f32 = 2.0;

    let still = drift_of(true, 0.0);
    let carried = drift_of(true, CURRENT);
    assert!(
        carried - still > 0.1,
        "a correnteza tem de alcancar quem boia: parado {still:.4}, na correnteza \
         {carried:.4} (antes da cura era 0,0000 nos dois)"
    );
}

/// **E ela ainda NÃO o leva como leva uma jangada** — o número, pinado.
///
/// ⚠️ **Gate de DEFEITO, escrito para ficar VERMELHO no dia em que a metade B
/// chegar.** Com a cura do sensor o personagem bóia e a correnteza o alcança,
/// mas ele viaja **menos de 1%** do que uma cápsula idêntica viaja, porque
/// continua governado pela lei do **AR**: sem chão, a caminhada usa o controle
/// aéreo, e um dedo parado pede velocidade **zero** — ele nada contra a
/// corrente sem querer.
///
/// É a MESMA doença do `peak_gravity` (que o faz boiar 12,7 cm mais alto): duas
/// leis escritas para um corpo em voo balístico, aplicadas a um corpo que
/// flutua. **Uma causa, dois sintomas** — e a cura é uma decisão de produto (o
/// que é *nadar*), não um conserto.
#[test]
fn the_air_brake_still_fights_the_current_and_this_is_its_number() {
    const CURRENT: f32 = 2.0;

    let raft_like = drift_of(false, CURRENT);
    let player = drift_of(true, CURRENT);
    let ratio = player / raft_like;
    assert!(
        ratio < 0.10,
        "se o player passou a viajar como a capsula ({ratio:.3} de {raft_like:.4}), \
         a metade B chegou -- reescreva este gate em vez de afrouxar a barra"
    );
}

/// Quanto o sujeito anda em 2 s de correnteza.
///
/// ⚠️ **2 s, e não 5:** com cinco segundos a cápsula solta SAI da poça pelo lado
/// (medido: 30,59 m contra os 20 m de meia-largura) e deixa de ser controle — o
/// que ela mediria a partir dali é queda livre no vazio.
fn drift_of(player: bool, current: f32) -> f32 {
    let mut sim = SimWorld::new();
    pool(&mut sim, current);
    let _ = subject(&mut sim, player, 0.5);
    let mut bridge = PhysicsBridge::new();
    run(&mut sim, &mut bridge, 120);
    xy_of(&sim, "Subject").0
}

/// **Um volume de gatilho não é um TETO.**
///
/// A outra face do MESMO raio: o sensor de teto (W10) também é um `cast_ray`.
///
/// ⚠️ **A primeira versão deste gate nasceu VERDE, e a fixture era o defeito:**
/// ela punha uma laje sensora **inteira** sobre a cabeça, e uma laje inteira não
/// tem quina para a assistência desviar — nada era observável, e o gate não
/// podia falhar pelo motivo que alegava. A correção de quina é um DESVIO
/// LATERAL, então o que a exercita é uma laje que cobre **parte** da cabeça, e
/// o que ela produz é `x` a mexer-se. O oráculo é o **mesmo pulo sem o sensor**.
#[test]
fn a_trigger_volume_is_not_a_ceiling() {
    /// O quanto a laje cobre da cabeça — o suficiente para a quina existir.
    const CLIP: f32 = 0.15;
    /// A face de baixo da laje, acima do repouso.
    const UNDER: f32 = FLOAT + 1.2;

    fn jump_drift(with_sensor: bool) -> f32 {
        let mut sim = SimWorld::new();
        water::floor(&mut sim, 0.0);
        if with_sensor {
            let half_x = 2.0;
            sim.world_mut().spawn((
                Name::new("Trigger"),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    is_sensor: true,
                    shape: ColliderShape::Cuboid {
                        half_x,
                        half_y: 0.5,
                    },
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(RADIUS - CLIP + half_x, UNDER + 0.5)),
            ));
        }
        // ⚠️ A assistência de quina tem de estar ARMADA, senão não há desvio a
        // observar e o gate volta a não poder falhar.
        let who: Entity = water::subject_tuned(
            &mut sim,
            true,
            FLOAT,
            Some(ph2d_physics_ecs::PlatformPlayer {
                float_height: FLOAT,
                corner_reach: 0.3,
                air_acceleration: 0.0,
                ..ph2d_physics_ecs::PlatformPlayer::default()
            }),
        );
        let mut bridge = PhysicsBridge::new();
        run(&mut sim, &mut bridge, 60);
        let (x0, _) = xy_of(&sim, "Subject");
        bridge.set_player_input(
            who,
            PlayerInput {
                jump: true,
                ..PlayerInput::default()
            },
        );
        for t in 61..=180u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        xy_of(&sim, "Subject").0 - x0
    }

    let free = jump_drift(false);
    let under = jump_drift(true);
    assert!(
        (under - free).abs() < 0.005,
        "um gatilho sobre a cabeca nao pode desviar o pulo: deriva livre {free:.4}, \
         sob o sensor {under:.4} (delta {:.4})",
        under - free
    );
}
