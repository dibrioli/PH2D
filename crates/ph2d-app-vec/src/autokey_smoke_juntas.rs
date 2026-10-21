//! **A cena do smoke «o Auto Key grava a junta»** (F25) — `PH2D_AUTOKEY_SMOKE=14|15`, irmã de
//! [`crate::autokey_smoke_bloco2`]. Mexe sozinha e pára no quadro 18.
//!
//! Duas bancadas paradas (a física não corre no editor): uma DOBRADIÇA (Pin) entre um poste e um
//! braço, com limites ±0,4 rad, motor desligado e quebra desligada; e uma MOLA (Spring) entre um
//! gancho e uma bola. No quadro 12: os limites abrem para ±1,4 rad (o arco desenhado abre), o motor
//! liga com mais força, a quebra liga; a mola fica dura e amortecida. No 24 tudo volta. `=14` acaba
//! com a dobradiça seleccionada, `=15` com a mola — o Inspector à frente, na secção da junta.

use ph2d_core::{Playhead, Vec2};
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_timeline::TimelineState;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use crate::autokey_smoke::avanca as go;
use crate::autokey_smoke_tudo::trilhas;

const QUADRO_12: f64 = 0.5;
const QUADRO_24: f64 = 1.0;
const QUADRO_18: f64 = 0.75;
const CINZA: [f32; 4] = [0.75, 0.75, 0.8, 1.0];
const LARANJA: [f32; 4] = [0.95, 0.6, 0.2, 1.0];
const VERDE: [f32; 4] = [0.4, 0.85, 0.55, 1.0];
/// O poste da dobradiça e o gancho da mola.
const POSTE: [f32; 2] = [-2.5, 1.0];
const GANCHO: [f32; 2] = [2.5, 2.0];
/// O limite (±), a força do motor, a quebra: no início e no quadro 12.
const LIMITE: [f32; 2] = [0.4, 1.4];
const FORCA: [f32; 2] = [50.0, 400.0];
const QUEBRA: [f32; 2] = [500.0, 2000.0];
/// A mola: rigidez e amortecimento, no início e no quadro 12.
const RIGIDEZ: [f32; 2] = [20.0, 200.0];
const AMORTECE: [f32; 2] = [0.5, 5.0];

static PIN: AtomicU64 = AtomicU64::new(0);
static MOLA: AtomicU64 = AtomicU64::new(0);

#[allow(clippy::too_many_arguments)]
pub(crate) fn step(
    nivel: u8,
    // `0` = `=14` (acaba na dobradiça) · `1` = `=15` (acaba na mola).
    modo: u8,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    match nivel {
        1 => {
            let (pin, mola) = montar(sim);
            PIN.store(pin.to_bits(), Relaxed);
            MOLA.store(mola.to_bits(), Relaxed);
            hero.set_panel_visible("timeline", true);
            hero.set_panel_visible("inspector", true);
            hero.store.bump_panel_z(ph2d_editor_core::ids::INSP_PANEL);
            hero.store
                .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_JOINT_SECTION, false);
            go(2);
            Some(format!(
                "[autokey-smoke] dobradiça {:#x} (limites ±{}, motor desligado, quebra desligada) · \
                 mola {:#x} (rigidez {}, amortecimento {})",
                pin.to_bits(),
                LIMITE[0],
                mola.to_bits(),
                RIGIDEZ[0],
                AMORTECE[0]
            ))
        }
        s @ 2..=90 => {
            go(s + 1);
            let (pb, mb) = (PIN.load(Relaxed), MOLA.load(Relaxed));
            match s {
                8 => {
                    tl.flags.auto_key = true;
                    ph.pause();
                    ph.seek(QUADRO_12);
                    hero.gizmo.replace_selection(Some(pb));
                    None
                }
                14 => {
                    dobradica(sim, pb, 1)?;
                    Some(format!(
                        "[autokey-smoke] q12: dobradiça ← limites ±{} · motor ligado força {} · quebra \
                         ligada {}",
                        LIMITE[1], FORCA[1], QUEBRA[1]
                    ))
                }
                18 => {
                    hero.gizmo.replace_selection(Some(mb));
                    None
                }
                22 => {
                    mola(sim, mb, 1)?;
                    Some(format!(
                        "[autokey-smoke] q12: mola ← rigidez {} amortecimento {}",
                        RIGIDEZ[1], AMORTECE[1]
                    ))
                }
                28 => {
                    ph.seek(QUADRO_24);
                    hero.gizmo.replace_selection(Some(pb));
                    None
                }
                33 => {
                    dobradica(sim, pb, 0)?;
                    None
                }
                37 => {
                    hero.gizmo.replace_selection(Some(mb));
                    None
                }
                41 => {
                    mola(sim, mb, 0)?;
                    Some(format!(
                        "[autokey-smoke] q24: tudo de volta · dobradiça {} · mola {}",
                        trilhas(tl, pb),
                        trilhas(tl, mb)
                    ))
                }
                // A junta da foto fica seleccionada ANTES do seek: o Inspector tem de seguir o
                // documento que a timeline escreve, sem a selecção mudar (`sync_joint_fields`).
                44 => {
                    hero.gizmo
                        .replace_selection(Some(if modo == 0 { pb } else { mb }));
                    None
                }
                46 => {
                    ph.seek(QUADRO_18);
                    None
                }
                56 => Some(format!(
                    "[autokey-smoke] q18 (a meio): {} · dobradiça {} · mola {}",
                    medir(sim, pb, mb),
                    trilhas(tl, pb),
                    trilhas(tl, mb)
                )),
                90 => Some(format!("[autokey-smoke] fim · {}", medir(sim, pb, mb))),
                _ => None,
            }
        }
        _ => None,
    }
}

fn corpo(
    sim: &mut SimWorld,
    nome: &str,
    kind: BodyKind,
    shape: ColliderShape,
    lado: [f32; 2],
    cor: [f32; 4],
    em: [f32; 2],
) -> Entity {
    sim.world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(em[0], em[1])),
            Sprite::atlas(WHITE_TILE_KEY, lado, cor),
            Name::new(nome.to_owned()),
            RigidBody { kind },
            Collider {
                shape,
                ..Collider::default()
            },
        ))
        .id()
}

/// A junta prende os corpos pela IDENTIDADE estável deles (`StableId`), como o painel faz.
fn junta(
    sim: &mut SimWorld,
    nome: &str,
    a: Entity,
    b: Entity,
    em: [f32; 2],
    j: PhysicsJoint,
) -> Entity {
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let id = |e| ph2d_ecs::stable_id_of(sim.world(), e).map_or(0, |s| s.0);
    let (body_a, body_b) = (id(a), id(b));
    sim.world_mut()
        .spawn((
            Name::new(nome.to_owned()),
            PhysicsJoint {
                body_a,
                body_b,
                ..j
            },
            Transform::from_translation(Vec2::new(em[0], em[1])),
        ))
        .id()
}

/// As duas bancadas; devolve `(dobradiça, mola)`.
fn montar(sim: &mut SimWorld) -> (Entity, Entity) {
    let pino = ColliderShape::Ball { radius: 0.1 };
    let poste = corpo(
        sim,
        "Poste",
        BodyKind::Static,
        pino,
        [0.2, 0.2],
        CINZA,
        POSTE,
    );
    let braco = corpo(
        sim,
        "Braco",
        BodyKind::Dynamic,
        ColliderShape::Cuboid {
            half_x: 0.9,
            half_y: 0.1,
        },
        [1.8, 0.2],
        LARANJA,
        [POSTE[0] + 0.9, POSTE[1]],
    );
    let pin = junta(
        sim,
        "Dobradica",
        poste,
        braco,
        POSTE,
        PhysicsJoint {
            kind: JointKind::Pin,
            limits_enabled: true,
            limit_min: -LIMITE[0],
            limit_max: LIMITE[0],
            motor_enabled: false,
            motor_speed: 1.2,
            motor_max_force: FORCA[0],
            break_enabled: false,
            break_force: QUEBRA[0],
            ..PhysicsJoint::default()
        },
    );
    let gancho = corpo(
        sim,
        "Gancho",
        BodyKind::Static,
        pino,
        [0.2, 0.2],
        CINZA,
        GANCHO,
    );
    let bola = corpo(
        sim,
        "Bola",
        BodyKind::Dynamic,
        ColliderShape::Ball { radius: 0.3 },
        [0.6, 0.6],
        VERDE,
        [GANCHO[0], GANCHO[1] - 1.8],
    );
    let mola = junta(
        sim,
        "Mola",
        gancho,
        bola,
        GANCHO,
        PhysicsJoint {
            kind: JointKind::Spring,
            rest_length: 1.2,
            stiffness: RIGIDEZ[0],
            damping: AMORTECE[0],
            ..PhysicsJoint::default()
        },
    );
    (pin, mola)
}

fn dobradica(sim: &mut SimWorld, bits: u64, n: usize) -> Option<()> {
    let mut j = sim
        .world_mut()
        .get_mut::<PhysicsJoint>(Entity::from_bits(bits))?;
    j.limit_min = -LIMITE[n];
    j.limit_max = LIMITE[n];
    j.motor_enabled = n == 1;
    j.motor_max_force = FORCA[n];
    j.break_enabled = n == 1;
    j.break_force = QUEBRA[n];
    Some(())
}

fn mola(sim: &mut SimWorld, bits: u64, n: usize) -> Option<()> {
    let mut j = sim
        .world_mut()
        .get_mut::<PhysicsJoint>(Entity::from_bits(bits))?;
    j.stiffness = RIGIDEZ[n];
    j.damping = AMORTECE[n];
    Some(())
}

fn medir(sim: &SimWorld, pb: u64, mb: u64) -> String {
    let w = sim.world();
    let p = w.get::<PhysicsJoint>(Entity::from_bits(pb));
    let m = w.get::<PhysicsJoint>(Entity::from_bits(mb));
    let pin = p.map_or_else(
        || "sem dobradiça".to_owned(),
        |j| {
            format!(
                "limites {:.3}..{:.3} · motor {} força {:.1} · quebra {} {:.1}",
                j.limit_min,
                j.limit_max,
                j.motor_enabled,
                j.motor_max_force,
                j.break_enabled,
                j.break_force
            )
        },
    );
    let mola = m.map_or_else(
        || "sem mola".to_owned(),
        |j| format!("rigidez {:.2} amortecimento {:.3}", j.stiffness, j.damping),
    );
    format!("{pin} · {mola}")
}
