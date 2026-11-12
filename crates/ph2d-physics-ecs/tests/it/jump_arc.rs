//! **A curva do canvas é o pulo do jogo** (`ph2d_platformer::arc`, ficha GEF 04): o arco que o
//! editor desenha contra o player REAL na ponte (rapier, mola, cápsula, sub-passos) num chão plano.

use crate::platform_scene::{pose, scene};

use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{PhysicsBridge, PlatformPlayer, PlayerInput};
use ph2d_platformer::arc::{JumpArc, jump_arcs};
use ph2d_platformer::feel::Feel;

/// O que se compara de um pulo: o ápice (acima do arranque, para o lado), o tempo no ar e o alcance.
#[derive(Debug)]
struct Medido {
    apex_up: f32,
    apex_side: f32,
    air_ticks: usize,
    reach: f32,
}

/// O pouso, igual para os dois: o primeiro tique depois do ápice abaixo de [`POUSO`]. A perna do
/// jogo apara a queda (não desce à linha de partida); o desenho atravessa-a no mesmo tique.
const POUSO: f32 = 0.1;

fn pouso(ys: &[[f32; 2]]) -> usize {
    let top = ys
        .iter()
        .enumerate()
        .max_by(|a, b| a.1[1].total_cmp(&b.1[1]))
        .map_or(0, |(i, _)| i);
    (top..ys.len())
        .find(|&i| ys[i][1] < POUSO)
        .unwrap_or(ys.len() - 1)
}

fn medir(pts: &[[f32; 2]]) -> Medido {
    let top = pts
        .iter()
        .copied()
        .max_by(|a, b| a[1].total_cmp(&b[1]))
        .unwrap();
    let i = pouso(pts);
    Medido {
        apex_up: top[1],
        apex_side: top[0],
        air_ticks: i,
        reach: pts[i][0],
    }
}

fn do_arco(a: &JumpArc) -> Medido {
    medir(&a.points)
}

#[derive(Copy, Clone)]
enum Como {
    Correndo,
    Parado,
    Curto,
}

fn do_jogo(sim: &mut SimWorld, bridge: &mut PhysicsBridge, player: Entity, como: Como) -> Medido {
    let drive = 1.0;
    let mut tick = 0_u64;
    // Assenta na perna; correndo, já chega à velocidade máxima antes do salto.
    for _ in 0..60 {
        let d = if matches!(como, Como::Parado) {
            0.0
        } else {
            drive
        };
        bridge.set_player_input(
            player,
            PlayerInput {
                drive: d,
                ..PlayerInput::default()
            },
        );
        tick += 1;
        bridge.dispatch(sim, true, tick);
    }
    let (x0, y0) = pose(sim);
    let mut pts = vec![[0.0, 0.0]];
    for i in 0..200 {
        let jump = match como {
            Como::Curto => i == 0,
            _ => true,
        };
        bridge.set_player_input(
            player,
            PlayerInput {
                drive,
                jump,
                ..PlayerInput::default()
            },
        );
        tick += 1;
        bridge.dispatch(sim, true, tick);
        let (x, y) = pose(sim);
        pts.push([x - x0, y - y0]);
    }
    medir(&pts)
}

fn caso(feel: Option<Feel>, como: Como) -> (Medido, Medido) {
    let (mut sim, mut bridge, player) = scene(0.0, -20.0);
    let w = ph2d_physics_ecs::arc_world(&bridge.settings(), 1.0);
    let cfg = {
        let mut e = sim.world_mut().entity_mut(player);
        let mut p = e.get_mut::<PlatformPlayer>().unwrap();
        if let Some(f) = feel {
            p.set_feel(f);
        }
        p.config()
    };
    let arcs = jump_arcs(&cfg, w);
    let arco = match como {
        Como::Correndo => &arcs.full,
        Como::Parado => &arcs.standing,
        Como::Curto => &arcs.short,
    };
    (do_arco(arco), do_jogo(&mut sim, &mut bridge, player, como))
}

#[test]
fn o_arco_do_desenho_e_o_do_jogo() {
    let mut falhas = Vec::new();
    for feel in [
        None,
        Some(Feel::Floaty),
        Some(Feel::Snappy),
        Some(Feel::Heavy),
    ] {
        for (nome, como) in [
            ("correndo", Como::Correndo),
            ("parado", Como::Parado),
            ("curto", Como::Curto),
        ] {
            let (a, j) = caso(feel, como);
            eprintln!(
                "{feel:?} {nome}: desenho {a:?}\n{:>w$}jogo    {j:?}",
                "",
                w = 0
            );
            // Altura: 1,3 cm medido (o sub-passo do rapier e o nosso diferem na ordem). Lado do
            // ápice: um tique de corrida (6 m/s · 1/60), porque num topo achatado dois tiques empatam.
            let ok = (a.apex_up - j.apex_up).abs() < 0.02
                && (a.apex_side - j.apex_side).abs() < 0.11
                && a.air_ticks.abs_diff(j.air_ticks) <= 1
                // O pouso é o tique em que se cruza POUSO: ±1 tique, logo um tique de corrida.
                && (a.reach - j.reach).abs() < 0.11;
            if !ok {
                falhas.push(format!("{feel:?} {nome}"));
            }
        }
    }
    assert!(falhas.is_empty(), "o desenho mente em: {falhas:?}");
}

/// O player REAL na ponte: segura o pulo, e no ápice do do chão solta um tique e volta a premir
/// (o pulo do ar). Devolve a subida de cada um, medida do próprio arranque.
fn subidas_do_jogo(feel: Option<Feel>, air_h: f32) -> (f32, f32, f32, f32) {
    let (mut sim, mut bridge, player) = scene(0.0, -20.0);
    let (h, ah) = {
        let mut e = sim.world_mut().entity_mut(player);
        let mut p = e.get_mut::<PlatformPlayer>().unwrap();
        if let Some(f) = feel {
            p.set_feel(f);
        }
        p.air_jumps = 1;
        p.air_jump_height = air_h;
        (p.jump_height, p.air_jump_height)
    };
    let mut tick = 0_u64;
    let mut passo = |sim: &mut SimWorld, bridge: &mut PhysicsBridge, jump: bool| {
        bridge.set_player_input(
            player,
            PlayerInput {
                jump,
                ..PlayerInput::default()
            },
        );
        tick += 1;
        bridge.dispatch(sim, true, tick);
        pose(sim).1
    };
    for _ in 0..60 {
        passo(&mut sim, &mut bridge, false);
    }
    let y0 = pose(&sim).1;
    let (mut topo, mut antes) = (y0, y0);
    loop {
        let y = passo(&mut sim, &mut bridge, true);
        if y < antes {
            break;
        }
        topo = topo.max(y);
        antes = y;
    }
    let y1 = passo(&mut sim, &mut bridge, false);
    let mut topo_ar = y1;
    for _ in 0..200 {
        topo_ar = topo_ar.max(passo(&mut sim, &mut bridge, true));
    }
    (topo - y0, h, topo_ar - y1, ah)
}

/// A alça escreve o ápice pedido, ao centímetro, em qualquer sensação: cima = `jump_height`
/// (é o ápice), lado = `speed`. O desenho é o jogo (gate acima), então o desenho basta.
#[test]
fn a_alca_poe_o_apice_onde_foi_pedido() {
    let mut falhas = Vec::new();
    for feel in [
        None,
        Some(Feel::Floaty),
        Some(Feel::Snappy),
        Some(Feel::Heavy),
    ] {
        let (mut sim, bridge, player) = scene(0.0, -20.0);
        let w = ph2d_physics_ecs::arc_world(&bridge.settings(), 1.0);
        let mut e = sim.world_mut().entity_mut(player);
        let mut p = e.get_mut::<PlatformPlayer>().unwrap();
        if let Some(f) = feel {
            p.set_feel(f);
        }
        p.pull_apex(2.0, 2.6, w);
        let a = p.jump_arcs(w).full.apex;
        if (a[1] - 2.6).abs() >= 0.02 || (a[0] - 2.0).abs() >= 0.02 {
            falhas.push(format!("{feel:?}: ápice {a:?}"));
        }
    }
    assert!(falhas.is_empty(), "a alça erra o alvo [2, 2.6]: {falhas:?}");
}

/// ⭐ GEF 04, «altura que mente»: o campo Jump Height (e o Air Jump Height) é o ápice REAL do
/// jogo, em qualquer sensação — a lei resolve o arranque pelas fases de gravidade.
#[test]
fn o_pulo_do_jogo_sobe_a_jump_height() {
    let mut falhas = Vec::new();
    for feel in [
        None,
        Some(Feel::Floaty),
        Some(Feel::Snappy),
        Some(Feel::Heavy),
    ] {
        let (sobe, h, sobe_ar, ah) = subidas_do_jogo(feel, 1.5);
        eprintln!("{feel:?}: chão {sobe:.4} (Jump Height {h}) · ar {sobe_ar:.4} (Air {ah})");
        if (sobe - h).abs() >= 0.02 || (sobe_ar - ah).abs() >= 0.02 {
            falhas.push(format!("{feel:?}: chão {sobe:.3}/{h} ar {sobe_ar:.3}/{ah}"));
        }
    }
    assert!(falhas.is_empty(), "o campo mente: {falhas:?}");
}

// ── GEF 04: a curva pousa no TERRENO ─────────────────────────────────────────────────────────────

/// Uma caixa estática de topo em `top`, de `x0` a `x1`, com `h` de altura.
fn caixa(sim: &mut SimWorld, x0: f32, x1: f32, top: f32, h: f32) -> Entity {
    use ph2d_core::Vec2;
    use ph2d_ecs::{Name, Transform};
    use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
    sim.world_mut()
        .spawn((
            Name::new("Terreno"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: (x1 - x0) * 0.5,
                    half_y: h * 0.5,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new((x0 + x1) * 0.5, top - h * 0.5)),
        ))
        .id()
}

/// Uma caixa-sensor (gatilho): o corpo atravessa-a.
fn sensor(sim: &mut SimWorld, x0: f32, x1: f32, top: f32, h: f32) {
    let e = caixa(sim, x0, x1, top, h);
    let mut c = sim
        .world_mut()
        .get_mut::<ph2d_physics_ecs::Collider>(e)
        .unwrap();
    c.is_sensor = true;
}

/// Assenta o player 60 tiques (`correndo` ou parado). Devolve o tique corrente.
fn assenta(sim: &mut SimWorld, bridge: &mut PhysicsBridge, player: Entity, correndo: bool) -> u64 {
    for tick in 1..=60 {
        bridge.set_player_input(
            player,
            PlayerInput {
                drive: if correndo { 1.0 } else { 0.0 },
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(sim, true, tick);
    }
    60
}

/// A curva desenhada (cortada no terreno) e o pulo real: pontos relativos ao centro do arranque.
fn terreno(extra: impl Fn(&mut SimWorld, f32), correndo: bool) -> (JumpArc, Vec<[f32; 2]>) {
    // O x em que o player fica depois de assentar: corrida de ensaio sem terreno extra.
    let x_assentado = {
        let (mut sim, mut bridge, player) = scene(0.0, 0.0);
        assenta(&mut sim, &mut bridge, player, correndo);
        pose(&sim).0
    };
    let (mut sim, mut bridge, player) = scene(0.0, 0.0);
    extra(&mut sim, x_assentado);
    let mut tick = assenta(&mut sim, &mut bridge, player, correndo);
    let (x0, y0) = pose(&sim);
    let settings = bridge.settings();
    let w = ph2d_physics_ecs::arc_world(&settings, 1.0);
    let (mut arcs, fh) = {
        let p = sim.world().get::<PlatformPlayer>(player).unwrap();
        (p.jump_arcs(w), p.float_height)
    };
    let chao = [x0, y0 - fh];
    ph2d_physics_ecs::arc_terrain::cut_on_terrain(
        sim.world(),
        &settings,
        player,
        chao,
        [0.0, 1.0],
        &mut arcs,
    );
    let arco = if correndo { arcs.full } else { arcs.standing };
    let mut pts = vec![[0.0, 0.0]];
    for _ in 0..240 {
        bridge.set_player_input(
            player,
            PlayerInput {
                drive: 1.0,
                jump: true,
                ..PlayerInput::default()
            },
        );
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
        let (x, y) = pose(&sim);
        pts.push([x - x0, y - y0]);
    }
    (arco, pts)
}

/// O pouso do jogo sobre a altura `h` (relativa): o primeiro tique depois do ápice abaixo de
/// `h + POUSO`, e a altura em que o corpo ASSENTA logo depois (a da superfície que o pegou).
fn pouso_real(pts: &[[f32; 2]], h: f32) -> (usize, f32, f32) {
    let top = pts
        .iter()
        .enumerate()
        .max_by(|a, b| a.1[1].total_cmp(&b.1[1]))
        .map_or(0, |(i, _)| i);
    let i = (top..pts.len())
        .find(|&i| pts[i][1] < h + POUSO)
        .unwrap_or(pts.len() - 1);
    // Assenta: 20 tiques depois (a mola acaba de apanhar o corpo; ele continua a correr).
    (i, pts[i][0], pts[(i + 20).min(pts.len() - 1)][1])
}

fn confere_pouso(nome: &str, arco: &JumpArc, pts: &[[f32; 2]], h_esperada: f32) {
    let t = arco
        .touch
        .unwrap_or_else(|| panic!("{nome}: a curva não tocou o terreno"));
    let tique = arco.points.len() - 1;
    let (i, x, assenta) = pouso_real(pts, t[1]);
    eprintln!(
        "{nome}: desenho toca {t:?} no tique {tique} · jogo {i} x {x:.3} assenta {assenta:.3}"
    );
    assert!(
        (t[1] - h_esperada).abs() < 0.02,
        "{nome}: tocou a altura errada {t:?}"
    );
    assert!(
        (assenta - t[1]).abs() < 0.03,
        "{nome}: o jogo assenta em {assenta}, a curva em {}",
        t[1]
    );
    assert!(tique.abs_diff(i) <= 1, "{nome}: tique {tique} × jogo {i}");
    assert!((t[0] - x).abs() < 0.11, "{nome}: x {} × jogo {x}", t[0]);
}

/// Põe o player `dy` metros acima de onde a cena o pôs (antes do primeiro tique).
fn ergue(sim: &mut SimWorld, dy: f32) {
    let mut q = sim
        .world_mut()
        .query::<(&ph2d_ecs::Name, &mut ph2d_ecs::Transform)>();
    for (n, mut t) in q.iter_mut(sim.world_mut()) {
        if n.as_str() == "Player" {
            t.translation.y += dy;
        }
    }
}

/// (a) Arranque numa plataforma 1 m acima do chão, que acaba à frente: a curva DESCE ao chão de
/// baixo e pousa onde o player real pousa. Corrida e parado.
#[test]
fn a_curva_desce_ao_chao_de_baixo_e_pousa_onde_o_jogo_pousa() {
    for correndo in [true, false] {
        let (arco, pts) = terreno(
            |sim, x| {
                caixa(sim, x - 8.0, x + 0.6, 1.5, 1.0);
                ergue(sim, 1.0);
                // Uma zona-sensor atravessada pela curva: não a pára (o jogo atravessa-a).
                sensor(sim, x + 2.0, x + 5.0, 4.0, 3.5);
            },
            correndo,
        );
        confere_pouso(&format!("desce correndo={correndo}"), &arco, &pts, -1.0);
    }
}

/// (b) Uma plataforma 1 m mais alta à frente, que o pulo alcança: a curva pousa em cima dela.
#[test]
fn a_curva_pousa_na_plataforma_mais_alta() {
    for correndo in [true, false] {
        let ahead = if correndo { 3.0 } else { 1.5 };
        let (arco, pts) = terreno(
            |sim, x| {
                caixa(sim, x + ahead, x + 20.0, 1.5, 1.0);
            },
            correndo,
        );
        confere_pouso(&format!("sobe correndo={correndo}"), &arco, &pts, 1.0);
    }
}

/// (c) Uma parede à frente: a curva pára NA parede (o corpo encosta nela), onde o jogo encosta.
#[test]
fn a_curva_para_na_parede() {
    let face = 2.0;
    let (arco, pts) = terreno(
        |sim, x| {
            caixa(sim, x + face, x + face + 1.0, 10.0, 9.5);
        },
        true,
    );
    let t = arco.touch.expect("a curva não tocou a parede");
    let real = pts.iter().map(|p| p[0]).fold(f32::MIN, f32::max);
    eprintln!("parede: desenho {t:?} · jogo encosta em x {real:.3}");
    // A cápsula (raio 0,2) encosta na face: o centro fica a 0,2 dela.
    assert!(
        (t[0] - (face - 0.2)).abs() < 0.02,
        "a curva atravessa a parede: {t:?}"
    );
    assert!((t[0] - real).abs() < 0.11, "desenho {} × jogo {real}", t[0]);
    assert!(t[1] > 0.0, "a parede pega-o no ar: {t:?}");
}

/// Sonda: o custo de UMA chamada do desenho (lei + corte no terreno) na cena (a), com Air Jumps 1.
/// `PH2D_RELEASE="mede velocidade" bash scripts/ph2d-run.sh cargo test --release -p ph2d-physics-ecs custo_das_curvas_no_terreno -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime o custo"]
fn custo_das_curvas_no_terreno() {
    let (mut sim, bridge, player) = scene(0.0, 0.0);
    caixa(&mut sim, -8.0, 0.6, 1.5, 1.0);
    sensor(&mut sim, 2.0, 5.0, 4.0, 3.5);
    ergue(&mut sim, 1.0);
    let settings = bridge.settings();
    let w = ph2d_physics_ecs::arc_world(&settings, 1.0);
    sim.world_mut()
        .get_mut::<PlatformPlayer>(player)
        .unwrap()
        .air_jumps = 1;
    let (x0, y0) = pose(&sim);
    let chao = [x0, y0 - crate::platform_scene::FLOAT_HEIGHT];
    let mut best = f64::MAX;
    for _ in 0..5 {
        let t = std::time::Instant::now();
        for _ in 0..200 {
            let p = sim.world().get::<PlatformPlayer>(player).unwrap();
            let mut arcs = p.jump_arcs(w);
            ph2d_physics_ecs::arc_terrain::cut_on_terrain(
                sim.world(),
                &settings,
                player,
                chao,
                [0.0, 1.0],
                &mut arcs,
            );
            std::hint::black_box(arcs);
        }
        best = best.min(t.elapsed().as_secs_f64() / 200.0);
    }
    eprintln!(
        "lei + corte (3 formas, Air Jumps 1): {:.1} µs por chamada (mínimo de 5 blocos)",
        best * 1e6
    );
}
