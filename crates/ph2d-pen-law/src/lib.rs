//! **As leis puras da mão no pincel**, partilhadas pelo Painter e pelo Pixel Art (uma porta só: duas
//! cópias da mesma lei divergem em silêncio). O Pixel não depende do Painter (dono, 25/10).

use ph2d_det_math::DetMath as _;
/// Um passo da **corda** (*Pulled String*, o *Stabilizer* do Aseprite): o pincel em `anchor` só anda
/// quando a mão (`sample`) está a mais de `len`, e então fica a `len` atrás dela, na recta que os une.
/// Medido no Trial 1.3.18.6: o atraso é um comprimento fixo em px de ECRÃ, igual a 400 % e a 200 % e
/// com pincel 1 e 5 (`oracle_trial_dynamics_velocity.json`, `stabilizer_16`).
#[must_use]
pub fn pulled_string_step(anchor: [f32; 2], sample: [f32; 2], len: f32) -> [f32; 2] {
    let (dx, dy) = (sample[0] - anchor[0], sample[1] - anchor[1]);
    let d = dx.det_hypot(dy);
    if d <= len {
        return anchor;
    }
    let k = (d - len) / d;
    [anchor[0] + dx * k, anchor[1] + dy * k]
}
