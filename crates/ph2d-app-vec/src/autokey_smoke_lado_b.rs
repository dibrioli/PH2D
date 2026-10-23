//! **A cena do smoke «o lado B da junta»** (F27 Bloco A) — `PH2D_AUTOKEY_SMOKE=27`, irmã de
//! [`crate::autokey_smoke_tipos`]. Três bancadas:
//! - **Motor** (sozinha): o Auto Key grava o eixo e o alvo do motor nos quadros 12 e 24; a cena
//!   atravessa o quadro 24 (onde o eixo muda) e mede o alvo — o funil zerava a chave do artista.
//! - **Lamp** (o dono): um pino de mundo sem corpo B. «Body» arma a escolha do corpo (o Hook), em
//!   vez de deixar a junta dormente.
//! - **Hinge** (o dono): pino de mundo com corpo B; trocar World ↔ Body re-deriva a âncora de B — a
//!   ponta saltava para o centro do Plank.

use ph2d_core::{Playhead, Vec2};
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, CustomAxis, JointKind, JointWorldAnchor, MotorMode,
    PhysicsJoint, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_timeline::TimelineState;
use std::sync::Mutex;

use crate::autokey_smoke::avanca as go;
use crate::autokey_smoke_tudo::trilhas;

const QUADRO_12: f64 = 0.5;
const QUADRO_18: f64 = 0.75;
const QUADRO_24: f64 = 1.0;
const CINZA: [f32; 4] = [0.75, 0.75, 0.8, 1.0];
const COR: [f32; 4] = [0.95, 0.6, 0.2, 1.0];
/// O alvo do motor no quadro 24 (o eixo X, metros) — o que a medição tem de ler depois de cruzar.
const ALVO_24: f32 = 0.3;

/// `[Motor, Lamp, Hinge]`, as entidades-junta.
static JUNTAS: Mutex<Vec<u64>> = Mutex::new(Vec::new());
/// O alvo do Motor lido em cada quadro depois de cruzar o 24.
static VISTOS: Mutex<Vec<f32>> = Mutex::new(Vec::new());

fn juntas() -> Vec<u64> {
    JUNTAS.lock().map(|v| v.clone()).unwrap_or_default()
}

pub(crate) fn step(
    nivel: u8,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    match nivel {
        1 => {
            let todas = montar(sim);
            if let Ok(mut a) = JUNTAS.lock() {
                *a = todas.iter().map(|e| e.to_bits()).collect();
            }
            hero.set_panel_visible("timeline", true);
            hero.set_panel_visible("inspector", true);
            hero.store.bump_panel_z(ph2d_editor_core::ids::INSP_PANEL);
            hero.store
                .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_JOINT_SECTION, false);
            go(2);
            Some(format!("[autokey-smoke] lado B montado · {}", medir(sim)))
        }
        s @ 2..=90 => {
            go(s + 1);
            let j = juntas();
            let motor = j.first().copied();
            match s {
                8 => {
                    tl.flags.auto_key = true;
                    ph.pause();
                    ph.seek(QUADRO_12);
                    hero.gizmo.replace_selection(motor);
                    None
                }
                14 => {
                    mexer_motor(sim, motor, true);
                    None
                }
                22 => {
                    ph.seek(QUADRO_24);
                    None
                }
                28 => {
                    mexer_motor(sim, motor, false);
                    None
                }
                32 => {
                    motor.map(|b| format!("[autokey-smoke] trilhas do Motor: {}", trilhas(tl, b)))
                }
                36 => {
                    tl.flags.auto_key = false;
                    ph.seek(QUADRO_18);
                    None
                }
                44 => {
                    ph.seek(QUADRO_24);
                    None
                }
                // Quadro a quadro: o zero durava UM quadro (a timeline re-escreve o alvo no seguinte),
                // e um servo que lê zero nesse quadro dá um tranco.
                45..=51 => {
                    let alvo = motor
                        .and_then(|b| sim.world().get::<PhysicsJoint>(Entity::from_bits(b)))
                        .map_or(f32::NAN, |p| p.motor_target);
                    if let Ok(mut v) = VISTOS.lock() {
                        v.push(alvo);
                    }
                    None
                }
                52 => {
                    let vistos = VISTOS.lock().map(|v| v.clone()).unwrap_or_default();
                    let ok = vistos.iter().all(|a| (a - ALVO_24).abs() < 1e-4);
                    Some(format!(
                        "[autokey-smoke] q24, quadro a quadro depois de cruzar a troca de eixo: \
                         alvo {vistos:.3?} (quer {ALVO_24:.3} em todos) {}",
                        if ok {
                            "OK"
                        } else {
                            "ZERADO — a chave do artista perdeu-se"
                        }
                    ))
                }
                56 => {
                    hero.gizmo.replace_selection(j.get(1).copied());
                    None
                }
                90 => Some(format!("[autokey-smoke] fim · {}", medir(sim))),
                _ => None,
            }
        }
        _ => None,
    }
}

fn corpo(sim: &mut SimWorld, nome: &str, kind: BodyKind, em: [f32; 2]) -> Entity {
    let (lado, cor) = if kind == BodyKind::Static {
        ([0.3, 0.3], CINZA)
    } else {
        ([0.6, 0.6], COR)
    };
    sim.world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(em[0], em[1])),
            Sprite::atlas(WHITE_TILE_KEY, lado, cor),
            Name::new(nome.to_owned()),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Ball {
                    radius: lado[0] * 0.5,
                },
                ..Collider::default()
            },
        ))
        .id()
}

fn id(sim: &mut SimWorld, e: Entity) -> u64 {
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::stable_id_of(sim.world(), e).map_or(0, |s| s.0)
}

fn junta(sim: &mut SimWorld, nome: &str, j: PhysicsJoint, em: [f32; 2], mundo: bool) -> Entity {
    let e = sim
        .world_mut()
        .spawn((
            Name::new(nome.to_owned()),
            j,
            Transform::from_translation(Vec2::new(em[0], em[1])),
        ))
        .id();
    if mundo {
        sim.world_mut().entity_mut(e).insert(JointWorldAnchor);
    }
    e
}

fn montar(sim: &mut SimWorld) -> Vec<Entity> {
    let base = corpo(sim, "Motor Base", BodyKind::Static, [-4.0, 2.0]);
    let carro = corpo(sim, "Motor Cart", BodyKind::Dynamic, [-3.0, 2.0]);
    let mut m = PhysicsJoint {
        body_a: id(sim, base),
        body_b: id(sim, carro),
        motor_enabled: true,
        motor_mode: MotorMode::Position,
        motor_target: ALVO_24,
        ..PhysicsJoint::of_kind(JointKind::Custom)
    };
    m.custom.motor_axis = CustomAxis::X;
    let motor = junta(sim, "Motor", m, [-4.0, 2.0], false);

    let lampada = corpo(sim, "Lamp", BodyKind::Dynamic, [0.0, 1.0]);
    corpo(sim, "Hook", BodyKind::Static, [1.5, 3.0]);
    let pino = PhysicsJoint {
        body_a: id(sim, lampada),
        ..PhysicsJoint::of_kind(JointKind::Pin)
    };
    let lamp = junta(sim, "Lamp Pin", pino, [0.0, 2.5], true);

    let porta = corpo(sim, "Door", BodyKind::Dynamic, [3.5, 2.0]);
    let tabua = corpo(sim, "Plank", BodyKind::Dynamic, [5.5, 2.0]);
    let dobra = PhysicsJoint {
        body_a: id(sim, porta),
        body_b: id(sim, tabua),
        ..PhysicsJoint::of_kind(JointKind::Pin)
    };
    let hinge = junta(sim, "Hinge", dobra, [4.5, 2.6], true);
    vec![motor, lamp, hinge]
}

/// Quadro 12: o eixo vira Rotação e o alvo 1,2 rad; quadro 24: volta ao X com `ALVO_24` m.
fn mexer_motor(sim: &mut SimWorld, motor: Option<u64>, q12: bool) {
    let Some(mut j) = motor.and_then(|b| {
        sim.world_mut()
            .get_mut::<PhysicsJoint>(Entity::from_bits(b))
    }) else {
        return;
    };
    j.custom.motor_axis = if q12 {
        CustomAxis::Rotation
    } else {
        CustomAxis::X
    };
    j.motor_target = if q12 { 1.2 } else { ALVO_24 };
}

fn medir(sim: &SimWorld) -> String {
    let w = sim.world();
    let nomes = ["Motor", "Lamp Pin", "Hinge"];
    juntas()
        .iter()
        .zip(nomes)
        .filter_map(|(&b, n)| {
            let e = Entity::from_bits(b);
            let j = w.get::<PhysicsJoint>(e)?;
            Some(format!(
                "{n}: mundo {} corpo-B {} ancorada {} local_b [{:.2}, {:.2}] eixo {:?} alvo {:.2}",
                w.get::<JointWorldAnchor>(e).is_some(),
                j.body_b != 0,
                j.anchored,
                j.local_b[0],
                j.local_b[1],
                j.custom.motor_axis,
                j.motor_target
            ))
        })
        .collect::<Vec<_>>()
        .join(" · ")
}
