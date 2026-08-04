//! A sonda da cena 85 + o gate que mantém a mensagem dela honesta (W6).
//!
//! ⚠️ **A sonda mede a cena que SHIPA** (`build_reaction_scene`), não uma cópia
//! dela — uma segunda montagem divergiria no dia em que a rigidez das molas
//! mudasse, e a mensagem passaria a descrever uma cena que ninguém abre.

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};

/// Monta a cena e simula `secs` segundos pela PORTA REAL (a ponte).
fn run(secs: f32, drive: f32) -> SimWorld {
    run_walking(secs, drive, secs)
}

/// Anda por `walk_secs` e depois PARA, seguindo até `secs`.
///
/// ⚠️ A jangada tem 6 m e o personagem anda a 6 m/s: andar os 4 s inteiros o
/// leva 24 m para fora dela, e a medição vira *"nao ha' ninguem em cima"*. A
/// primeira versao deste gate fez isso e mediu 0,00° de inclinacao sobre uma
/// jangada correta.
fn run_walking(secs: f32, drive: f32, walk_secs: f32) -> SimWorld {
    let mut sim = SimWorld::new();
    build_reaction_scene(sim.world_mut());
    let players: Vec<Entity> = {
        let mut q = sim.world_mut().query::<(Entity, &Name)>();
        q.iter(sim.world())
            .filter(|(_, n)| n.as_str().starts_with("Hero"))
            .map(|(e, _)| e)
            .collect()
    };
    let mut bridge = PhysicsBridge::new();
    let walk_ticks = (walk_secs * 60.0) as u64;
    for t in 0..=(secs * 60.0) as u64 {
        for &p in &players {
            bridge.set_player_input(
                p,
                PlayerInput {
                    drive: if t <= walk_ticks { drive } else { 0.0 },
                    ..PlayerInput::default()
                },
            );
        }
        bridge.dispatch(&mut sim, true, t);
    }
    sim
}

fn pose(sim: &mut SimWorld, name: &str) -> (f32, f32) {
    let e = {
        let mut q = sim.world_mut().query::<(Entity, &Name)>();
        q.iter(sim.world())
            .find(|(_, n)| n.as_str() == name)
            .map(|(e, _)| e)
            .unwrap_or_else(|| panic!("{name} tem de existir"))
    };
    let t = ph2d_ecs::world_transform(sim.world(), e).expect("transform");
    (t.translation.y, t.rotation.to_degrees())
}

/// **A sonda.** `cargo test -p ph2d-host-desktop probe_smoke_85 -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição"]
fn probe_smoke_85() {
    let mut sim = run(4.0, 0.0);
    let live = pose(&mut sim, "RaftLive");
    let ghost = pose(&mut sim, "RaftGhost");
    eprintln!(
        "parado 4 s -> viva y={:.3} rot={:.2}deg · fantasma y={:.3} rot={:.2}deg",
        live.0, live.1, ghost.0, ghost.1
    );

    let mut walked = run_walking(4.0, -1.0, 0.35);
    let lw = pose(&mut walked, "RaftLive");
    let gw = pose(&mut walked, "RaftGhost");
    eprintln!(
        "andando 4 s -> viva y={:.3} rot={:.2}deg · fantasma y={:.3} rot={:.2}deg",
        lw.0, lw.1, gw.0, gw.1
    );

    for secs in [0.5_f32, 1.0, 2.0, 4.0, 8.0] {
        let mut s = run(secs, 0.0);
        let r = pose(&mut s, "RaftLive");
        let h = pose(&mut s, "Hero0");
        let g = pose(&mut s, "RaftGhost");
        eprintln!(
            "t={secs:>4.1}s  jangada viva y={:>7.3}  heroi y={:>7.3} (folga {:>6.3})  fantasma y={:>7.3}",
            r.0,
            h.0,
            h.0 - r.0,
            g.0
        );
    }
}

/// ⚠️ **A cena faz o que a mensagem dela promete.**
///
/// A mensagem manda o artista julgar *"a da esquerda AFUNDA, a da direita não se
/// mexe"* e *"ande até a borda: ela INCLINA"*. Um smoke cuja cena não produz o
/// fenômeno faz o artista reprovar uma feature que funciona — ou, pior, aprovar
/// uma que não.
#[test]
fn the_scene_shows_what_its_message_promises() {
    let mut sim = run(4.0, 0.0);
    let (live_y, _) = pose(&mut sim, "RaftLive");
    let (ghost_y, _) = pose(&mut sim, "RaftGhost");
    assert!(
        live_y < ghost_y - 0.1,
        "a jangada VIVA tem de afundar sob o fantasma: viva {live_y:.3} contra {ghost_y:.3}"
    );

    // E andando para a borda ela INCLINA — a metade do torque.
    // ⚠️ 0,35 s a 6 m/s são ~2,1 m: a borda de uma jangada de 3 m de meia-largura.
    let mut walked = run_walking(4.0, -1.0, 0.35);
    let (_, live_rot) = pose(&mut walked, "RaftLive");
    let (_, ghost_rot) = pose(&mut walked, "RaftGhost");
    assert!(
        live_rot.abs() > 1.0,
        "andando para a borda a jangada viva tem de INCLINAR: {live_rot:.2} graus"
    );
    assert!(
        ghost_rot.abs() < 0.5,
        "e a do fantasma nao: {ghost_rot:.2} graus"
    );
}

/// A mensagem NOMEIA os dois knobs que ela manda o artista mexer.
///
/// ⚠️ Um roteiro que cita um controle por um nome que a UI não usa é um roteiro
/// que faz o artista procurar o que não existe e reportar a feature como
/// ausente.
#[test]
fn the_message_names_the_controls_the_panel_paints() {
    for label in ["Weight on Ground", "Push on Ground"] {
        assert!(
            REACTION_SMOKE_MESSAGE.contains(label),
            "a mensagem da cena 85 tem de nomear o controle {label:?}"
        );
        assert!(
            ph2d_panel_inspector::player_row_labels().contains(&label),
            "e a §14 tem de PINTAR uma row com esse rotulo: {label:?}"
        );
    }
}
