//! Shell-side driver for the ECS physics bridge (ADR-0131 W1): reads the ONE clock (`Playhead`) and
//! hands `(playing, target tick)` to [`PhysicsBridge`]. Called each frame between the timeline apply
//! and `sim_extract`, so a body's readback `Transform` renders the same frame it was computed.

use ph2d_core::Playhead;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{InputTape, PhysicsBridge};

/// The fixed tick the playhead currently sits on — `round(time / dt)`, the
/// same mapping Motion uses (`motion_tick`), so physics and motion step on
/// the identical tick grid.
fn physics_tick(playhead: &Playhead, fixed_dt: f64) -> u64 {
    (playhead.time() / fixed_dt).round().max(0.0) as u64
}

/// Step (or settle) the rigid world for this frame. Play = advance the owed
/// ticks + read poses back into `Transform`; paused = settle bodies to the
/// authored `Transform` (read-only on `Transform`, so an idle paused frame
/// produces no snapshot diff — the fixed-point rule).
///
/// `simulate` is the transport's Physics toggle
/// (`TimelineFlags::simulate_physics`, off by default). Disarmed, the world is
/// HELD rather than skipped — see [`PhysicsBridge::hold`], which explains why
/// those are different things.
// ⚠️ Nove argumentos: os pares que agrupariam bem são dois; o agrupamento honesto ainda deixaria 7.
#[allow(clippy::too_many_arguments)]
pub fn dispatch(
    bridge: &mut PhysicsBridge,
    sim: &mut SimWorld,
    playhead: &Playhead,
    fixed_dt: f64,
    doc: &mut ph2d_timeline::TimelineDoc,
    simulate: bool,
    input: ph2d_physics_ecs::PlayerInput,
    tape: &mut InputTape,
    (drive, fisica, mundo): (
        &mut ph2d_preview_drive::PreviewDrive,
        &mut crate::canais_corpo::ConducaoFisica,
        &mut crate::canais_mundo::ConducaoMundo,
    ),
) {
    // ⭐ A marca de MESTRE em dia ANTES da ponte (ADR-0164 / F4.1): as consultas da ponte filtram
    // `Without<MasterPiece>`, e um mestre novo simularia um quadro. Único ponto de entrada; idempotente.
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let target = physics_tick(playhead, fixed_dt);
    let players = hand_input_to_players(bridge, sim, input);
    // ⚠️ Grava SÓ andando para a frente (W7: um scrub para trás não reescreve a corrida), em TODO
    // tique devido e não só no alvo (gate `na_segunda_vida_sem_tecla_o_heroi_nao_anda_nem_roda`), e
    // só com player e Physics armado (W17, `measure_player_tape`): um tique que o solver não rodou
    // descreve um instante que não aconteceu.
    if simulate && players > 0 && target > bridge.last_stepped() {
        for tick in bridge.last_stepped() + 1..=target {
            tape.record(tick, input);
        }
    }
    if !simulate {
        bridge.hold(sim, target);
        return;
    }
    // ⚠️ The bridge is told where the timeline puts scene-driven bodies at each tick, the rewind
    // replay above all (`FrozenScene` here would compile and be `bake_and_scrub_agree_with_play`'s bug).
    let drive_do_tique = &mut *drive;
    let livro = Some(crate::bake::Livros {
        drive: drive_do_tique,
        fisica,
        mundo,
    });
    let mut scene = crate::bake::TimelineScene {
        doc,
        fixed_dt,
        livro,
    };
    // **A POSE DE ANTES**, para o ledger de pré-visualização saber o que foi o solver que escreveu.
    let before = body_poses(sim);
    // ⚠️ **A FITA entra aqui, e é ela que torna o replay reproduzível** (W7): o
    // laço de replay do rewind dirige os players com a entrada do TIQUE que ele
    // está refazendo, não com o dedo de agora.
    bridge.dispatch_with_scene_and_tape(sim, playhead.is_playing(), target, &mut scene, tape);
    declare_solver_writes(sim, &before, drive);
}

/// A pose de todo corpo rígido, agora — a fotografia contra a qual se lê **o que o solver
/// escreveu** neste quadro (uma varredura por quadro, só com a simulação armada).
fn body_poses(sim: &SimWorld) -> Vec<(Entity, ph2d_ecs::Transform)> {
    let Some(mut q) = sim
        .world()
        .try_query::<(Entity, &ph2d_ecs::Transform, &ph2d_physics_ecs::RigidBody)>()
    else {
        return Vec::new();
    };
    q.iter(sim.world()).map(|(e, t, _)| (e, *t)).collect()
}

/// **O que o solver mexeu é pré-visualização, não autoria** (`ph2d-preview-drive`): a corrida
/// colapsa em UM passo de undo quando pára. ⚠️ Declara só quem MUDOU, senão o `settle` nunca a acaba.
fn declare_solver_writes(
    sim: &SimWorld,
    before: &[(Entity, ph2d_ecs::Transform)],
    drive: &mut ph2d_preview_drive::PreviewDrive,
) {
    use ph2d_preview_drive::Driven;
    for &(entity, was) in before {
        let Some(now) = sim.world().get::<ph2d_ecs::Transform>(entity) else {
            continue; // o corpo saiu da cena neste quadro
        };
        if *now != was {
            drive.driven(entity, Driven::SolverPose(was), Driven::SolverPose(*now));
        }
    }
}

/// **Entrega o dedo do jogador a todo player da cena** (W3): a MESMA entrada a todos (um teclado),
/// e antes do `hold` (armar o Physics não engole a 1.ª tecla; entregar não é GRAVAR, W17). Devolve
/// quantos players a cena tem, que o chamador usa para decidir se o tique descreve uma corrida.
fn hand_input_to_players(
    bridge: &mut PhysicsBridge,
    sim: &mut SimWorld,
    input: ph2d_physics_ecs::PlayerInput,
) -> usize {
    // ⭐⭐⭐ **Pela PORTA partilhada** ([`ph2d_physics_ecs::for_each_keyboard_driven`]) desde
    // 2026-09-15 — report do dono: *«nada se move»*.
    //
    // ⛔⛔ Aqui estava um `try_query::<(Entity, &PlatformPlayer)>()` escrito à mão, e ele era a
    // metade da entrega que **não conhecia o mover de vista de cima**. A outra metade (a fita,
    // `take_taped_input`) conhecia-o — logo o componente estava ligado ao replay e desligado do
    // teclado, e os dois defeitos somavam-se: sem player contado, `players == 0`, e a fita nem
    // sequer gravava o tique. Ver o cabeçalho do módulo da porta.
    let mut n = 0;
    ph2d_physics_ecs::for_each_keyboard_driven(sim.world(), |entity| {
        bridge.set_player_input(entity, input);
        n += 1;
    });
    n
}

/// **What to say when a joint gives way** (W-J7) — one line per break, ready for
/// a toast.
///
/// A break is the one thing this module does that the artist did not ask for in
/// the frame it happens, so it announces itself. The overlay already shows WHERE
/// (the burst) and THAT it is broken (the red), and neither can carry the number:
/// a moment later the joint reads a load of zero, because it is not holding
/// anything. The load AT the break lives only in the event.
///
/// Returns messages rather than pushing them, so the caller owns the toast deck
/// and this stays testable without one.
pub fn break_reports(bridge: &PhysicsBridge, sim: &SimWorld) -> Vec<String> {
    bridge
        .joint_breaks()
        .iter()
        .map(|b| {
            let name = sim
                .world()
                .get::<ph2d_ecs::Name>(b.joint)
                .map_or_else(|| "Joint".to_string(), |n| n.0.to_string());
            // The FORCE is the number in the message even when the torque is what
            // crossed: a joint that reads a torque at all is a hinge, and its
            // linear reaction is reported next to it — printing whichever
            // threshold fired would need the wrapper to say which, and the
            // artist's question is *how hard was it pulled*.
            ph2d_i18n::tr_with(
                "app.physics.dispatch.broke_at_n",
                &[("name", &name), ("force_0", &format!("{:.0}", b.force))],
            )
        })
        .collect()
}
