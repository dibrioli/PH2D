//! **A SENSAÇÃO do pulo** — Floaty · Snappy · Heavy: seis gravidades de uma vez, sem mexer na
//! altura que o artista puxou (a altura re-resolve-se pelo ápice, [`crate::arc::height_for_apex`]).
//! Valores e porquê: `docs/GameEngineFuncional/componentes/04_curva_do_pulo.md` §Valores.

use crate::JumpConfig;
use crate::arc::{ArcWorld, full_arc, height_for_apex};
use crate::PlayerConfig;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Feel {
    Floaty,
    Snappy,
    Heavy,
}

/// As seis gravidades que uma sensação escreve.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FeelGravity {
    pub takeoff_gravity: f32,
    pub takeoff_speed: f32,
    pub peak_gravity: f32,
    pub peak_speed: f32,
    pub fall_gravity: f32,
    pub cut_gravity: f32,
}

impl Feel {
    pub const ALL: [Self; 3] = [Self::Floaty, Self::Snappy, Self::Heavy];

    #[must_use]
    pub const fn gravity(self) -> FeelGravity {
        match self {
            // Topo longo, descida mansa, corte suave (Mario).
            Self::Floaty => FeelGravity {
                takeoff_gravity: 1.0,
                takeoff_speed: 0.0,
                peak_gravity: 0.45,
                peak_speed: 2.0,
                fall_gravity: 1.4,
                cut_gravity: 2.5,
            },
            // Saída rápida, janela curta no topo, queda e corte fortes (Celeste).
            Self::Snappy => FeelGravity {
                takeoff_gravity: 1.6,
                takeoff_speed: 4.0,
                peak_gravity: 0.6,
                peak_speed: 1.2,
                fall_gravity: 2.6,
                cut_gravity: 5.0,
            },
            // Sem flutuação no topo, cai como pedra (Hollow Knight).
            Self::Heavy => FeelGravity {
                takeoff_gravity: 1.3,
                takeoff_speed: 0.0,
                peak_gravity: 1.0,
                peak_speed: 0.8,
                fall_gravity: 3.2,
                cut_gravity: 4.0,
            },
        }
    }
}

fn gravity_of(j: &JumpConfig) -> FeelGravity {
    FeelGravity {
        takeoff_gravity: j.takeoff_gravity,
        takeoff_speed: j.takeoff_speed,
        peak_gravity: j.peak_gravity,
        peak_speed: j.peak_speed,
        fall_gravity: j.fall_gravity,
        cut_gravity: j.cut_gravity,
    }
}

/// A sensação cujos seis números batem EXACTAMENTE com os do pulo; `None` se o artista os mexeu.
#[must_use]
pub fn feel_of(j: &JumpConfig) -> Option<Feel> {
    let g = gravity_of(j);
    Feel::ALL.into_iter().find(|f| f.gravity() == g)
}

/// Escreve a sensação e re-resolve `jump_height` (e `air_jump_height`, na mesma proporção) para o
/// ápice do pulo inteiro ficar onde estava.
pub fn apply_feel(cfg: &mut PlayerConfig, feel: Feel, w: ArcWorld) {
    let up_apex = |c: &PlayerConfig| {
        let a = full_arc(c, w).apex;
        let up = crate::arc::up_of_gravity(w.gravity);
        a[0] * up[0] + a[1] * up[1]
    };
    let before = up_apex(cfg);
    let old_h = cfg.jump.jump_height;
    let g = feel.gravity();
    let j = &mut cfg.jump;
    j.takeoff_gravity = g.takeoff_gravity;
    j.takeoff_speed = g.takeoff_speed;
    j.peak_gravity = g.peak_gravity;
    j.peak_speed = g.peak_speed;
    j.fall_gravity = g.fall_gravity;
    j.cut_gravity = g.cut_gravity;
    if let Some(h) = height_for_apex(cfg, w, before) {
        cfg.jump.jump_height = h;
        if old_h > 0.0 {
            cfg.jump.air_jump_height *= h / old_h;
        }
    }
}
