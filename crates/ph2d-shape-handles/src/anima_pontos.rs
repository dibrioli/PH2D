//! **O movimento dos indicadores dos pontos**: o ponto sob o ponteiro CRESCE suave e o alvo da fusão PULSA. Tudo é
//! função pura do relógio do quadro (`agora_ms`), que os testes fixam; sem relógio a tinta desenha o fim.

use crate::pontos::{self as p, Ponto};
use bevy_ecs::prelude::Resource;
use bevy_ecs::world::World;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

/// A duração do crescer, em ms.
pub const CRESCE_MS: f64 = 140.0;
/// O período do pulso da fusão, em ms.
pub const PULSO_MS: f64 = 900.0;

/// `ease-out` cúbico em `[0, 1]`: arranca depressa e pousa macio.
#[must_use]
pub fn suave(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// O pulso em `[0, 1]` (0 no início de cada período, 1 a meio).
#[must_use]
pub fn pulso(agora_ms: f64) -> f32 {
    let fase = (agora_ms.rem_euclid(PULSO_MS) / PULSO_MS) as f32;
    0.5 - 0.5 * (fase * std::f32::consts::TAU).cos()
}

/// O ponto sob o ponteiro e desde quando, e o relógio do último quadro.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct AnimaPontos {
    pub sob: Option<(Ponto, f64)>,
    pub agora_ms: f64,
}

impl AnimaPontos {
    /// Quanto o ponto `pt` já cresceu, em `[0, 1]` (outro ponto = 0).
    #[must_use]
    pub fn crescimento(&self, pt: Ponto) -> f32 {
        match self.sob {
            Some((q, desde)) if q == pt => suave(((self.agora_ms - desde) / CRESCE_MS) as f32),
            _ => 0.0,
        }
    }
}

/// Avança o relógio dos indicadores. `true` = ainda está a mexer (crescer ou pulso da fusão): o quadro seguinte
/// tem de vir.
pub fn passo(
    world: &mut World,
    cam: &Camera2d,
    win: WindowSize,
    ponteiro: (f32, f32),
    agora_ms: f64,
) -> bool {
    let Some(alvo) = crate::editando(world) else {
        return false;
    };
    let Some((pose, m, o)) = p::do_alvo(world, alvo) else {
        return false;
    };
    let ecra = p::no_ecra(&pose, &m, &o, cam, win);
    let est = crate::tinta_pontos::estados(world, &ecra, ponteiro);
    let mut a = world
        .get_resource::<AnimaPontos>()
        .copied()
        .unwrap_or_default();
    a.sob = match (est.sob, a.sob) {
        (Some(pt), Some((q, desde))) if q == pt => Some((q, desde)),
        (Some(pt), _) => Some((pt, agora_ms)),
        (None, _) => None,
    };
    a.agora_ms = agora_ms;
    world.insert_resource(a);
    est.fusao.is_some() || a.sob.is_some_and(|(_, d)| agora_ms - d < CRESCE_MS)
}
