//! **A cena do smoke «o Auto Key grava TODA junta»** (F26) — `PH2D_AUTOKEY_SMOKE=16..26`, irmã de
//! [`crate::autokey_smoke_juntas`]. Uma bancada parada por tipo de junta (a física não corre no
//! editor), cada uma com os campos que só ela mostra e os que nenhuma gravava antes de F26: o tipo,
//! o Active, o Collide Connected, o lado B no mundo, o alvo/taxa do motor, os comprimentos e a §13
//! (a roldana). No quadro 12 tudo muda, no 24 volta, e a cena pára no 18. O número da cena escolhe
//! a junta que fica seleccionada (o Inspector à frente, na secção dela).

use ph2d_core::{Playhead, Vec2};
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_physics_ecs::{
    AxisMode, BodyKind, Collider, ColliderShape, CustomAxis, JointKind, JointWorldAnchor,
    MotorMode, PhysicsJoint, PulleyWheel, RigidBody, WestonAxle, WrapSide,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_timeline::TimelineState;
use std::sync::Mutex;

use crate::autokey_smoke::avanca as go;
use crate::autokey_smoke_tudo::trilhas;

const QUADRO_12: f64 = 0.5;
const QUADRO_24: f64 = 1.0;
const QUADRO_18: f64 = 0.75;
const CINZA: [f32; 4] = [0.75, 0.75, 0.8, 1.0];
const COR: [f32; 4] = [0.95, 0.6, 0.2, 1.0];

/// As bancadas, pela ordem do número da cena (`=16` é a primeira).
const BANCADAS: [(&str, JointKind, [f32; 2]); 10] = [
    ("Pin", JointKind::Pin, [-5.0, 3.2]),
    ("Spring", JointKind::Spring, [-2.0, 3.2]),
    ("Rope", JointKind::Rope, [1.0, 3.2]),
    ("Weld", JointKind::Weld, [4.0, 3.2]),
    ("Slider", JointKind::Slider, [-5.0, 1.0]),
    ("Rod", JointKind::Rod, [-2.0, 1.0]),
    ("Wheel", JointKind::Wheel, [1.0, 1.0]),
    ("Pulley", JointKind::Pulley, [4.0, 1.0]),
    ("Custom", JointKind::Custom, [-5.0, -1.2]),
    ("Type Change", JointKind::Pin, [-2.0, -1.2]),
];
/// A roldana da Pulley é a 11.ª entidade (`=24`).
const RODA: usize = 10;

static ALVOS: Mutex<Vec<u64>> = Mutex::new(Vec::new());

fn alvos() -> Vec<u64> {
    ALVOS.lock().map(|v| v.clone()).unwrap_or_default()
}

/// O índice da entidade que `modo` (`cena − 16`) deixa seleccionada: as oito primeiras bancadas,
/// a roldana (`=24`), o Custom e a troca de tipo.
fn da_cena(modo: u8) -> usize {
    match modo {
        m @ 0..=7 => usize::from(m),
        8 => RODA,
        m => usize::from(m) - 1,
    }
}

pub(crate) fn step(
    nivel: u8,
    modo: u8,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    match nivel {
        1 => {
            let todos = montar(sim);
            if let Ok(mut a) = ALVOS.lock() {
                *a = todos.iter().map(|e| e.to_bits()).collect();
            }
            // A partida é o estado a que o quadro 24 volta: cada campo muda de facto no 12.
            mexer(sim, &alvos(), false);
            hero.set_panel_visible("timeline", true);
            hero.set_panel_visible("inspector", true);
            hero.store.bump_panel_z(ph2d_editor_core::ids::INSP_PANEL);
            for s in [
                ph2d_editor_core::ids::INSP_LIVE_JOINT_SECTION,
                ph2d_editor_core::ids::INSP_LIVE_WHEEL_SECTION,
            ] {
                hero.store.set_collapsed(s, false);
            }
            go(2);
            Some(format!(
                "[autokey-smoke] {} juntas montadas · {}",
                todos.len(),
                medir(sim)
            ))
        }
        s @ 2..=90 => {
            go(s + 1);
            let todos = alvos();
            match s {
                8 => {
                    tl.flags.auto_key = true;
                    ph.pause();
                    ph.seek(QUADRO_12);
                    seleccionar(hero, &todos);
                    None
                }
                14 => {
                    mexer(sim, &todos, true);
                    Some(format!("[autokey-smoke] q12: {}", medir(sim)))
                }
                22 => {
                    ph.seek(QUADRO_24);
                    None
                }
                28 => {
                    mexer(sim, &todos, false);
                    None
                }
                32 => Some(format!(
                    "[autokey-smoke] q24: tudo de volta · {}",
                    todos
                        .iter()
                        .zip(nomes())
                        .map(|(&b, n)| format!("{n}: {}", trilhas(tl, b)))
                        .collect::<Vec<_>>()
                        .join(" | ")
                )),
                // A junta da foto fica seleccionada ANTES do seek: o Inspector segue o documento
                // que a timeline escreve, sem a selecção mudar.
                34 => {
                    let i = da_cena(modo);
                    hero.gizmo.replace_selection(todos.get(i).copied());
                    None
                }
                36 => {
                    ph.seek(QUADRO_18);
                    None
                }
                46 => {
                    let pistas: Vec<String> = todos
                        .iter()
                        .zip(nomes())
                        .map(|(&b, n)| format!("{n}: {}", trilhas(tl, b)))
                        .collect();
                    Some(format!(
                        "[autokey-smoke] q18 (a meio): {} · trilhas {}",
                        medir(sim),
                        pistas.join(" | ")
                    ))
                }
                90 => Some(format!("[autokey-smoke] fim · {}", medir(sim))),
                _ => None,
            }
        }
        _ => None,
    }
}

fn nomes() -> impl Iterator<Item = &'static str> {
    BANCADAS
        .iter()
        .map(|b| b.0)
        .chain(std::iter::once("Pulley Wheel"))
}

fn seleccionar(hero: &mut HeroScreen, todos: &[u64]) {
    hero.gizmo.replace_selection(todos.first().copied());
    for &b in todos.iter().skip(1) {
        hero.gizmo.add_to_selection(b);
    }
}

fn corpo(sim: &mut SimWorld, nome: String, kind: BodyKind, em: [f32; 2]) -> Entity {
    let (lado, cor) = if kind == BodyKind::Static {
        ([0.2, 0.2], CINZA)
    } else {
        ([0.5, 0.5], COR)
    };
    sim.world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(em[0], em[1])),
            Sprite::atlas(WHITE_TILE_KEY, lado, cor),
            Name::new(nome),
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

/// As dez bancadas e a roldana; devolve as onze entidades pela ordem de [`BANCADAS`].
fn montar(sim: &mut SimWorld) -> Vec<Entity> {
    let mut out = Vec::new();
    for (nome, kind, p) in BANCADAS {
        // A polia pendura as duas pontas, uma de cada lado, abaixo da roldana (senão a rota
        // degenera e o canvas diz «no route»).
        let (pa, pb) = if kind == JointKind::Pulley {
            ([p[0] - 0.4, p[1] - 0.8], [p[0] + 1.4, p[1] - 0.8])
        } else {
            (p, [p[0] + 1.0, p[1] - 0.8])
        };
        let a = corpo(sim, format!("{nome} A"), BodyKind::Static, pa);
        let b = corpo(sim, format!("{nome} B"), BodyKind::Dynamic, pb);
        let (body_a, body_b) = (id(sim, a), id(sim, b));
        let j = PhysicsJoint {
            body_a,
            body_b,
            ..inicial(kind)
        };
        let e = sim
            .world_mut()
            .spawn((
                Name::new(nome.to_owned()),
                j,
                // A corda da polia nasce na ponta A (a roldana fica longe dela).
                Transform::from_translation(Vec2::new(pa[0], pa[1])),
            ))
            .id();
        out.push(e);
    }
    // A roldana da Pulley, entre as duas pontas e acima delas.
    let polia = BANCADAS[7].2;
    let rope = id(sim, out[7]);
    let roda = sim
        .world_mut()
        .spawn((
            Name::new("Pulley Wheel"),
            PulleyWheel {
                rope,
                radius: 0.3,
                ..PulleyWheel::default()
            },
            Transform::from_translation(Vec2::new(polia[0] + 0.5, polia[1] + 0.6)),
        ))
        .id();
    out.push(roda);
    out
}

/// O estado de partida de cada tipo.
fn inicial(kind: JointKind) -> PhysicsJoint {
    let mut j = PhysicsJoint::of_kind(kind);
    match kind {
        JointKind::Pin => {
            j.limits_enabled = true;
            j.limit_min = -0.4;
            j.limit_max = 0.4;
            j.motor_enabled = true;
            j.motor_mode = MotorMode::Velocity;
        }
        JointKind::Slider => {
            j.limits_enabled = true;
            j.motor_enabled = true;
        }
        _ => {}
    }
    j
}

/// ⭐ O que o artista mexe no painel, por tipo (`q12` = o estado novo; senão volta ao de partida).
fn mexer(sim: &mut SimWorld, todos: &[u64], q12: bool) {
    let w = sim.world_mut();
    let par = |a: f32, b: f32| if q12 { b } else { a };
    for (i, &bits) in todos.iter().enumerate() {
        let e = Entity::from_bits(bits);
        if i == RODA {
            if let Some(mut r) = w.get_mut::<PulleyWheel>(e) {
                r.radius = par(0.3, 0.6);
                r.radius_out = par(0.0, 0.15);
                r.wrap = if q12 { WrapSide::Over } else { WrapSide::Auto };
                r.motor_speed = par(0.0, 2.0);
                r.break_enabled = q12;
                r.break_force = par(500.0, 1500.0);
            }
            // ⚠️ Sem a talha de Weston: com uma roldana só ela não tem por onde voltar e a rota
            // degenera («no route») — o canal dela prova-se em `canais_junta_tests`.
            continue;
        }
        if BANCADAS[i].1 == JointKind::Rope {
            if q12 {
                w.entity_mut(e).insert(JointWorldAnchor);
            } else {
                w.entity_mut(e).remove::<JointWorldAnchor>();
            }
        }
        let Some(mut j) = w.get_mut::<PhysicsJoint>(e) else {
            continue;
        };
        match (i, BANCADAS[i].1) {
            (9, _) => {
                j.kind = if q12 {
                    JointKind::Slider
                } else {
                    JointKind::Pin
                }
            }
            (_, JointKind::Pin) => {
                j.motor_mode = if q12 {
                    MotorMode::Position
                } else {
                    MotorMode::Velocity
                };
                j.motor_target = par(0.0, 1.2);
                j.collide_connected = q12;
            }
            (_, JointKind::Spring) => {
                j.rest_length = par(1.2, 0.6);
                j.active = !q12;
            }
            (_, JointKind::Rope) => j.max_length = par(2.0, 1.0),
            (_, JointKind::Weld) => {
                j.soft = q12;
                j.stiffness = par(20.0, 200.0);
            }
            (_, JointKind::Slider) => {
                j.limit_min = par(-0.5, -1.5);
                j.limit_max = par(0.5, 1.5);
                j.motor_speed = par(0.5, 2.0);
            }
            (_, JointKind::Rod) => {
                j.max_length = par(1.5, 0.8);
                j.break_enabled = q12;
                j.break_force = par(500.0, 2000.0);
            }
            (_, JointKind::Wheel) => {
                j.stiffness = par(400.0, 100.0);
                j.motor_speed = par(0.0, 3.0);
            }
            (_, JointKind::Pulley) => {
                j.break_enabled = q12;
                j.break_force = par(500.0, 2000.0);
            }
            (_, JointKind::Custom) => {
                let x = j.custom.axis_mut(CustomAxis::X);
                x.mode = if q12 {
                    AxisMode::Limited
                } else {
                    AxisMode::Free
                };
                x.min = par(-0.5, -1.5);
                x.max = par(0.5, 1.5);
                j.custom.motor_axis = if q12 {
                    CustomAxis::Rotation
                } else {
                    CustomAxis::X
                };
            }
        }
    }
}

fn medir(sim: &SimWorld) -> String {
    let w = sim.world();
    let mut out = Vec::new();
    for (i, (&bits, nome)) in alvos().iter().zip(nomes()).enumerate() {
        let e = Entity::from_bits(bits);
        if i == RODA {
            if let Some(r) = w.get::<PulleyWheel>(e) {
                out.push(format!(
                    "{nome} raio {:.3} saída {:.3} weston {} lado {:?} motor {:.2} quebra {} {:.0}",
                    r.radius,
                    r.radius_out,
                    w.get::<WestonAxle>(e).is_some(),
                    r.wrap,
                    r.motor_speed,
                    r.break_enabled,
                    r.break_force
                ));
            }
            continue;
        }
        let Some(j) = w.get::<PhysicsJoint>(e) else {
            continue;
        };
        let x = j.custom.axis(CustomAxis::X);
        out.push(format!(
            "{nome}[{:?}] activa {} colide {} mundo {} alvo {:.2} taxa {:.2} repouso {:.2} máx {:.2} \
             limites {:.2}..{:.2} rigidez {:.0} mole {} quebra {} {:.0} eixoX {:?} {:.2}..{:.2} motor-eixo {:?}",
            j.kind,
            j.active,
            j.collide_connected,
            w.get::<JointWorldAnchor>(e).is_some(),
            j.motor_target,
            j.motor_speed,
            j.rest_length,
            j.max_length,
            j.limit_min,
            j.limit_max,
            j.stiffness,
            j.soft,
            j.break_enabled,
            j.break_force,
            x.mode,
            x.min,
            x.max,
            j.custom.motor_axis
        ));
    }
    out.join(" · ")
}
