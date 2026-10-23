//! **A lei do valor do `PlatformPlayer`** — o que o cartão (§14) e o script (`ph2d.set`) aceitam
//! (doc CodeEditor 08 §3.5). O irmão de [`super::player`] pelo tecto de LOC; mora com o TIPO porque
//! o cartão aplica em `ph2d-app-physics` e o script em `ph2d-app-components`, e as duas famílias
//! não se vêem.

use super::player::PlatformPlayer;
use crate::{MAX_CORNER_SAMPLES, MAX_WALL_SAMPLES, RideConfig};

/// Uma fração da 3ª lei em `[0, 1]`; não-finito vira ZERO (um `NaN.clamp` é `NaN`).
fn fraction(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Uma contagem de raios em `[1, max]`.
fn samples(n: u16, max: usize) -> u16 {
    n.clamp(1, u16::try_from(max).unwrap_or(u16::MAX))
}

impl PlatformPlayer {
    /// ⭐ **O saneamento que o `apply_player_edit` faz e o script herda.** Idempotente.
    ///
    /// ⚠️ Os pisos `max(0)` são de grandezas sem sentido negativo (distâncias, velocidades,
    /// tempos, multiplicadores); os tectos são MEDIDOS (a mola e o amortecimento acima de
    /// `1/dt²` passam do alvo; as frações devolveriam mais do que recebem). O `ledge_offset_y` é
    /// uma DIREÇÃO e não tem piso.
    pub fn sanitize(&mut self) {
        for v in [
            &mut self.float_height,
            &mut self.cling_distance,
            &mut self.speed,
            &mut self.acceleration,
            &mut self.brake_scale,
            &mut self.air_acceleration,
            &mut self.jump_height,
            &mut self.air_jump_height,
            &mut self.takeoff_gravity,
            &mut self.takeoff_speed,
            &mut self.peak_gravity,
            &mut self.peak_speed,
            &mut self.fall_gravity,
            &mut self.cut_gravity,
            &mut self.coyote_time,
            &mut self.jump_buffer,
            &mut self.corner_reach,
            &mut self.corner_lookahead,
            &mut self.lift_momentum,
            &mut self.wall_slide_speed,
            &mut self.wall_jump_height,
            &mut self.wall_jump_push,
            &mut self.wall_jump_lockout,
            &mut self.wall_reach,
            &mut self.wall_grab_stamina,
            &mut self.dash_speed,
            &mut self.dash_time,
            &mut self.dash_cooldown,
            &mut self.crouch_height,
            &mut self.crouch_speed,
            &mut self.swim_speed,
            &mut self.swim_acceleration,
            &mut self.swim_enter,
            &mut self.ledge_grab,
            &mut self.ledge_reach_y,
            &mut self.ledge_span,
            &mut self.ledge_speed,
            &mut self.glide_fall_speed,
            &mut self.max_fall_speed,
        ] {
            *v = v.max(0.0);
        }
        self.spring_strength = self
            .spring_strength
            .clamp(0.0, RideConfig::MAX_SPRING_STRENGTH);
        self.spring_damping = self.spring_damping.clamp(0.0, RideConfig::MAX_DAMPING);
        self.max_slope_deg = self.max_slope_deg.clamp(0.0, 90.0);
        self.foot_spread = self.foot_spread.clamp(0.0, 1.0);
        self.wall_spread = self.wall_spread.clamp(0.0, 1.0);
        self.air_jumps = self.air_jumps.min(u32::from(u16::MAX));
        self.corner_samples = samples(self.corner_samples, MAX_CORNER_SAMPLES);
        self.foot_samples = samples(self.foot_samples, MAX_WALL_SAMPLES);
        self.wall_samples = samples(self.wall_samples, MAX_WALL_SAMPLES);
        self.reaction_support = fraction(self.reaction_support);
        self.reaction_movement = fraction(self.reaction_movement);
        self.reaction_push = fraction(self.reaction_push);
    }
}
