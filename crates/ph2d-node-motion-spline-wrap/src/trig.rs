//! ⚠️ **Espelho VERBATIM do `ph2d-node-motion-distribute-curve/src/trig.rs`** — o
//! idioma de mirror em crate-folha que este repo corre (SEIS crates carregam esta
//! mesma aproximação hoje: look-at, distribute-curve, path e as três de rig).
//!
//! ⚠️ E aqui o espelho é **load-bearing, não conveniência**: este nó e o
//! `motion.distribute_curve` são os dois que põem coisas ao longo de uma CURVA, e
//! por isso os dois que têm de concordar sobre o que *"seguir a curva"* significa.
//! Uma segunda aproximação daria dois ângulos para a mesma tangente, e a diferença
//! apareceria como duas metades de uma cena a rodar de formas ligeiramente
//! diferentes — o defeito que ninguém liga a um `atan2`.

//! `atan2` without a transcendental — the Rajan rational approximation of `atan` on `[0,1]`, folded
//! across the eight octants (~0.0015 rad error, multiply/add/compare only, HR-5). A leaf-local copy
//! of `motion.path`'s: the shared vocabulary is the BEHAVIOUR, not a shared symbol.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

/// `atan2(y, x)` in radians. Returns 0 at the origin — which is also the answer for a fully
/// degenerate curve (four coincident control points), where there is no direction to report.
pub(crate) fn atan2_approx(y: f32, x: f32) -> f32 {
    let (ax, ay) = (x.abs(), y.abs());
    let hi = ax.max(ay);
    if hi == 0.0 {
        return 0.0;
    }
    let a = ax.min(ay) / hi;
    let mut r = FRAC_PI_4 * a - a * (a - 1.0) * (0.2447 + 0.0663 * a);
    if ay > ax {
        r = FRAC_PI_2 - r;
    }
    if x < 0.0 {
        r = PI - r;
    }
    if y < 0.0 {
        r = -r;
    }
    r
}

/// Radians → degrees. The app authors angles in **degrees** — the one authored-angle unit — and the
/// `rot` column is in them.
pub(crate) fn deg(rad: f32) -> f32 {
    rad * (180.0 / PI)
}

/// A senoide parabólica corrigida em `phase` CICLOS, em `[-1, 1]` — **espelho verbatim** do
/// `ph2d-node-motion-bend/src/trig.rs`, e pela mesma razão que o `atan2` acima é espelho do
/// `distribute-curve`: o eixo do embrulho ([`super::taper::DIRECTION`]) e a direção da dobra
/// respondem à MESMA pergunta (*"que ângulo é este?"*), e duas aproximações dariam dois
/// quadros locais ligeiramente diferentes para o mesmo número autorado.
fn sin_cycles(phase: f32) -> f32 {
    let f = phase - phase.floor();
    let p = if f < 0.5 {
        let u = f * 2.0;
        4.0 * u * (1.0 - u)
    } else {
        let u = (f - 0.5) * 2.0;
        -4.0 * u * (1.0 - u)
    };
    const Q: f32 = 0.225;
    Q * (p * p.abs() - p) + p
}

/// `(cos, sin)` de `phase` ciclos. `cos(x) = sin(x + ¼ de ciclo)`.
pub(crate) fn cos_sin_cycles(phase: f32) -> (f32, f32) {
    (sin_cycles(phase + 0.25), sin_cycles(phase))
}
