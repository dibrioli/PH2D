//! **A CURVA DO PULO** — o arco que o editor desenha, tirado da LEI e não de uma parábola.
//!
//! Corre [`crate::player_motor`] tique a tique, sem cena: o chão só no arranque, ar depois. A
//! integração é a da ponte (`apply_player_motor`: `v += accel·dt`, `v += boost`) seguida da do
//! mundo: por sub-passo a perna, e por iteração do solver `v += g·h`, `x += v·h`. O portão `o_arco_do_desenho_e_o_do_jogo`
//! (`ph2d-physics-ecs`) mede-o contra o player REAL. Ficha:
//! `docs/GameEngineFuncional/componentes/04_curva_do_pulo.md`.

use crate::{
    Buoyed, GroundSample, PlayerConfig, PlayerInput, PlayerState, Support, Vec2, player_motor,
};

/// Teto de tiques de UMA curva: 10 s a 60 Hz. Recurso: CPU do quadro do editor (uma curva de
/// ~100 tiques custa microssegundos); só uma gravidade nula ou para cima chega aqui.
pub const MAX_ARC_TICKS: usize = 600;

/// Quanto a curva desce abaixo do arranque, em metros, se nada a pára antes. Recurso: o ECRÃ — a
/// vista inicial tem ~11 m de altura, e 24 m são duas vistas de queda (um precipício que a câmara
/// ainda mostra); o terreno corta antes em toda cena com chão.
pub const MAX_FALL_M: f32 = 24.0;

/// Quantos pulos do ar se desenham. Recurso: o ECRÃ — com os números de partida cada um sobe 2 m,
/// e 8 já passam dos ~11 m de altura da vista inicial.
pub const AIR_JUMPS_DRAWN: u32 = 8;

/// Uma curva: o caminho do CENTRO do corpo relativo ao ponto de partida, em metros.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JumpArc {
    pub points: Vec<Vec2>,
    pub apex: Vec2,
    pub apex_time: f32,
    /// Até voltar à linha do arranque.
    pub air_time: f32,
    /// Onde e como a curva TOCA o terreno (o último ponto), se um corte o achou; a lei pura deixa `None`.
    pub touch: Option<ArcTouch>,
}

/// Como a curva parou no terreno.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TouchKind {
    /// O pé pousa: a superfície olha para cima, dentro do limite de rampa.
    Land,
    /// A cabeça ou o lado batem (teto, quina, parede): o corpo pára no ar.
    Bump,
}

/// O fim de uma curva cortada no terreno.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ArcTouch {
    /// O pé na paragem (o último ponto da curva), relativo ao arranque.
    pub at: Vec2,
    /// O ponto tocado NA superfície, relativo ao arranque.
    pub contact: Vec2,
    pub kind: TouchKind,
}

/// Todas as curvas que o canvas mostra.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JumpArcs {
    /// Botão preso, a correr à velocidade máxima. Leva a alça no ápice.
    pub full: JumpArc,
    /// Botão solto no tique seguinte ao salto (o corte).
    pub short: JumpArc,
    /// Parte parado, com a direcção presa desde o salto.
    pub standing: JumpArc,
    /// Os pulos do ar, cada um disparado no ápice do anterior.
    pub air: Vec<JumpArc>,
}

/// O mundo em que a curva corre.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ArcWorld {
    pub gravity: Vec2,
    pub dt: f32,
    /// `+1` para a direita, `-1` para a esquerda.
    pub facing: f32,
    /// Os sub-passos do mundo por tique (`PhysicsWorld::step`): a perna paga-se em cada um.
    pub substeps: u32,
    /// As iterações do solver em cada sub-passo: o rapier integra a gravidade e a posição em cada
    /// uma (`dt / (substeps · iterações)`), e é o passo que o jogo dá de verdade.
    pub solver_iterations: u32,
}

/// O «cima» de um mundo: contra a gravidade.
#[must_use]
pub fn up_of_gravity(gravity: Vec2) -> Vec2 {
    let n = (gravity[0] * gravity[0] + gravity[1] * gravity[1]).sqrt();
    if n > 0.0 && n.is_finite() {
        [-gravity[0] / n, -gravity[1] / n]
    } else {
        [0.0, 1.0]
    }
}

fn dot(a: Vec2, b: Vec2) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

/// O chão plano do arranque: a perna em repouso, sem quina.
fn rest_ground(cfg: &PlayerConfig, up: Vec2) -> GroundSample {
    GroundSample {
        distance: cfg.ride.float_height,
        normal: up,
        ground_velocity: [0.0, 0.0],
        one_way: false,
        grip: GroundSample::NEUTRAL_GRIP,
        brink: crate::Brink::NONE,
    }
}

/// Quando o botão do pulo está preso.
#[derive(Copy, Clone, Debug, PartialEq)]
enum Hold {
    /// Preso o tempo todo; com `air` pulos do ar, solta um tique em cada ápice e volta a premir.
    Full { air: u32 },
    /// Só no tique do salto.
    Tap,
}

struct Sim {
    points: Vec<Vec2>,
    /// Os índices em `points` onde cada pulo (o do chão e os do ar) começa.
    starts: Vec<usize>,
    apexes: Vec<(usize, f32)>,
    dt: f32,
}

fn simulate(cfg: &PlayerConfig, w: ArcWorld, run: bool, hold: Hold) -> Sim {
    let up = up_of_gravity(w.gravity);
    let side = [up[1] * w.facing, -up[0] * w.facing];
    let ground = rest_ground(cfg, up);
    let speed = if run { cfg.walk.speed } else { 0.0 };
    let mut v = [side[0] * speed, side[1] * speed];
    let mut p = [0.0_f32, 0.0_f32];
    let mut state = PlayerState::default();
    let mut sim = Sim {
        points: vec![p],
        starts: vec![0],
        apexes: Vec::new(),
        dt: w.dt,
    };
    let air_left = match hold {
        Hold::Full { air } => air,
        Hold::Tap => 0,
    };
    let mut pressed_air = 0_u32;
    let mut release_tick = false;
    // Dois tiques de chão sem botão: a memória do chão e o coyote assentam como no jogo.
    for tick in 0..MAX_ARC_TICKS + 2 {
        let warm = tick < 2;
        let first = tick == 2;
        let rising = dot(v, up);
        let held = match hold {
            _ if warm => false,
            Hold::Tap => first,
            Hold::Full { .. } => !release_tick,
        };
        let input = PlayerInput {
            drive: w.facing,
            jump: held,
            ..PlayerInput::default()
        };
        let sample = (warm || first).then_some(&ground);
        let step = player_motor(
            cfg,
            sample,
            None,
            None,
            None,
            None,
            input,
            state,
            v,
            w.gravity,
            up,
            w.dt,
            Buoyed::DRY,
            Support::default(),
        );
        state = step.state;
        if warm {
            continue;
        }
        let m = step.motor;
        let hold_g = step.gravity_hold;
        let n = w.substeps.max(1);
        let k = w.solver_iterations.max(1);
        let h = w.dt / n as f32;
        let hk = h / k as f32;
        for i in 0..2 {
            v[i] += m.accel[i] * w.dt + m.boost[i];
            for _ in 0..n {
                v[i] += hold_g[i] * h;
                for _ in 0..k {
                    v[i] += w.gravity[i] * hk;
                    p[i] += v[i] * hk;
                }
            }
        }
        if step.jump.is_some() && !first {
            sim.starts.push(sim.points.len() - 1);
        }
        sim.points.push(p);
        let now_rising = dot(v, up);
        if rising > 0.0 && now_rising <= 0.0 {
            sim.apexes.push((sim.points.len() - 1, dot(p, up)));
            // No ápice: solta um tique e volta a premir, se ainda houver pulos do ar a desenhar.
            if pressed_air < air_left {
                release_tick = true;
                pressed_air += 1;
                continue;
            }
        }
        release_tick = false;
        // Abaixo do arranque a curva continua: quem a pára é o terreno (`ph2d_physics_ecs::arc_terrain`).
        if dot(p, up) <= -MAX_FALL_M && !first {
            break;
        }
    }
    sim
}

fn arc_of(points: &[Vec2], up: Vec2, dt: f32) -> JumpArc {
    let (i, _) = points
        .iter()
        .enumerate()
        .fold((0, f32::MIN), |(bi, bh), (i, q)| {
            let h = dot(*q, up);
            if h > bh { (i, h) } else { (bi, bh) }
        });
    // O tempo no ar é até voltar à linha do arranque (a curva pode seguir abaixo dela).
    let back = (i..points.len())
        .find(|&k| k > 0 && dot(points[k], up) <= 0.0)
        .unwrap_or(points.len().saturating_sub(1));
    JumpArc {
        apex: points.get(i).copied().unwrap_or_default(),
        apex_time: i as f32 * dt,
        air_time: back as f32 * dt,
        points: points.to_vec(),
        touch: None,
    }
}

/// Uma curva só: botão preso, a correr, sem pulos do ar — a da alça.
#[must_use]
pub fn full_arc(cfg: &PlayerConfig, w: ArcWorld) -> JumpArc {
    let up = up_of_gravity(w.gravity);
    let s = simulate(cfg, w, true, Hold::Full { air: 0 });
    arc_of(&s.points, up, s.dt)
}

/// Todas as curvas do canvas.
#[must_use]
pub fn jump_arcs(cfg: &PlayerConfig, w: ArcWorld) -> JumpArcs {
    let up = up_of_gravity(w.gravity);
    let short = simulate(cfg, w, true, Hold::Tap);
    let standing = simulate(cfg, w, false, Hold::Full { air: 0 });
    let full = full_arc(cfg, w);
    let air_n = cfg.jump.air_jumps.min(AIR_JUMPS_DRAWN);
    let air = if air_n == 0 {
        Vec::new()
    } else {
        let s = simulate(cfg, w, true, Hold::Full { air: air_n });
        let mut out = Vec::new();
        for (k, &start) in s.starts.iter().enumerate().skip(1) {
            let end = s.starts.get(k + 1).copied().unwrap_or(s.points.len() - 1);
            out.push(arc_of(&s.points[start..=end], up, s.dt));
        }
        out
    };
    JumpArcs {
        full,
        short: arc_of(&short.points, up, short.dt),
        standing: arc_of(&standing.points, up, standing.dt),
        air,
    }
}

/// A velocidade que leva o ápice a `ahead` metros para o lado (o tempo até ao topo só depende da
/// vertical). `None` se o ápice é instantâneo.
#[must_use]
pub fn speed_for_apex(cfg: &PlayerConfig, w: ArcWorld, ahead: f32) -> Option<f32> {
    let t = full_arc(cfg, w).apex_time;
    (t > 0.0 && ahead.is_finite()).then(|| (ahead / t).max(0.0))
}

#[cfg(test)]
#[path = "arc_tests.rs"]
mod tests;
