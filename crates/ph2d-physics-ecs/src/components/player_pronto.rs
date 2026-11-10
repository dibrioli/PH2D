//! **Os números de nascença das habilidades** do Platform Player — irmão de `player.rs` pelo tecto de LOC.

/// **O número com que cada habilidade nasce**, para que marcar a caixa já dê um movimento no primeiro
/// Play (a lei de antes nascia com `0`, que desligava). Valores das cenas de smoke da física
/// (`ph2d-physics-smoke-scenes`: parede 3 e 2, beirada 0,6, nado 4, planeio 2), o dash do
/// `physics_smoke_out` (14) e o agachar a 0,6 da perna de partida; o teto de queda (10 m/s) só
/// corta quedas longas: o pulo de 2 m com Fall Gravity 2 aterra a ~12,5 m/s.
pub(super) struct Pronto {
    pub wall_slide_speed: f32,
    pub wall_jump_height: f32,
    pub dash_speed: f32,
    pub crouch_height: f32,
    pub swim_speed: f32,
    pub ledge_grab: f32,
    pub glide_fall_speed: f32,
    pub max_fall_speed: f32,
}

pub(super) const PRONTO: Pronto = Pronto {
    wall_slide_speed: 3.0,
    wall_jump_height: 2.0,
    dash_speed: 14.0,
    crouch_height: 0.3,
    swim_speed: 4.0,
    ledge_grab: 0.6,
    glide_fall_speed: 2.0,
    max_fall_speed: 10.0,
};

/// A habilidade com a caixa desmarcada chega à lei com o número que a desarma.
pub(super) fn on(ligada: bool, v: f32) -> f32 {
    if ligada { v } else { 0.0 }
}
