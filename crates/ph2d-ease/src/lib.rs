#![forbid(unsafe_code)]
//! **A tabela de easing e a ORDEM da cascata — uma conta, duas línguas** (ciclo 17, doc 126 §3).
//!
//! Antes desta crate o `motion.stagger` tinha a sua tabela `f32` e uma cópia WGSL à mão, o
//! `ph2d-anim` outra em `f64`, e o `value.time` nenhuma. Aqui vive a lei: [`ease`] e
//! [`order::raw_at`] em Rust, e [`wgsl_lib!`] com as MESMAS operações, na mesma ordem, para o
//! dispositivo. Quem consome é o nó; esta folha não conhece o grafo.
//!
//! **A escada das famílias é wire format** (um documento guarda o NÚMERO): `0..7` e `8 = Custom`
//! são as do ciclo 2, ao bit; `9..14` foram apendadas no ciclo 17.
//!
//! **Transcendentais no dispositivo sem divergir da CPU:** `sin` e `exp2` são polinómios de grau
//! fixo (Horner) com coeficientes dados por BITS ([`f32::from_bits`] aqui, `bitcast<f32>` no WGSL)
//! — nenhum dos lados passa por um parser decimal nem pela biblioteca matemática da plataforma.
//! Erro medido contra `f64`: `sin` `1,8e-7`, `exp2` `1,0e-7` relativo (doc 126 §5).

pub mod order;
mod wgsl;

/// `0` — identidade; ignora a direção.
pub const LINEAR: i32 = 0;
/// `1` — `t²`.
pub const QUAD: i32 = 1;
/// `2` — `t³`.
pub const CUBIC: i32 = 2;
/// `3` — `t⁴`.
pub const QUART: i32 = 3;
/// `4` — `t⁵`.
pub const QUINT: i32 = 4;
/// `5` — circular (`sqrt`).
pub const CIRC: i32 = 5;
/// `6` — antecipação/ultrapassagem, com o [`Shape::overshoot`].
pub const BACK: i32 = 6;
/// `7` — quatro parábolas.
pub const BOUNCE: i32 = 7;
/// `8` — a curva DESENHADA. Vive no nó (uma LUT/um `ph2d-curve`); aqui devolve `t`.
pub const CUSTOM: i32 = 8;
/// `9` — `1 − cos(πt/2)`.
pub const SINE: i32 = 9;
/// `10` — `2^(10t−10)` (Penner; `0` em `t = 0`).
pub const EXPO: i32 = 10;
/// `11` — elástico de Penner/GSAP, amplitude `1`, com o [`Shape::period`].
pub const ELASTIC: i32 = 11;
/// `12` — `floor(t·n)/n`; o `Out` reflectido é o «salto no início» do CSS.
pub const STEPS: i32 = 12;
/// `13` — a resposta ao degrau de uma mola, em forma FECHADA; ignora a direção.
pub const SPRING: i32 = 13;
/// `14` — o `cubic-bezier(x1, y1, x2, y2)` do CSS; ignora a direção.
pub const BEZIER: i32 = 14;
/// A última família da escada.
pub const LAST: i32 = BEZIER;

/// As famílias em que a direção (In · Out · In-Out) muda a curva.
pub const USES_DIRECTION: &[i32] = &[
    QUAD, CUBIC, QUART, QUINT, CIRC, BACK, BOUNCE, SINE, EXPO, ELASTIC, STEPS,
];

/// **Os nomes dos params de uma curva de easing num nó** — os mesmos em todo nó que a usa
/// (`motion.stagger`, `value.time`), para o editor gráfico ler e escrever pela mesma lista.
pub mod nomes {
    /// A família (`0..=LAST`).
    pub const CURVE: &str = "ease_curve";
    /// A direção (`0` In · `1` Out · `2` In-Out).
    pub const DIR: &str = "ease_dir";
    pub const OVERSHOOT: &str = "overshoot";
    pub const PERIOD: &str = "period";
    pub const STEPS: &str = "steps";
    pub const DAMPING: &str = "damping";
    /// As alças do `Bezier`, `[x1, y1, x2, y2]`.
    pub const BEZIER: [&str; 4] = ["bez_x1", "bez_y1", "bez_x2", "bez_y2"];
}

/// **As chaves de i18n dos nomes das famílias**, pela ordem da escada — quem as traduz é quem
/// pinta (esta folha não depende da tabela de textos).
pub const CHAVES_FAMILIA: [&str; 15] = [
    "anim.easing.family.linear",
    "anim.easing.family.quad",
    "anim.easing.family.cubic",
    "anim.easing.family.quart",
    "anim.easing.family.quint",
    "anim.easing.family.circ",
    "anim.easing.family.back",
    "anim.easing.family.bounce",
    "editor.ease.family.custom",
    "anim.easing.family.sine",
    "anim.easing.family.expo",
    "anim.easing.family.elastic",
    "editor.ease.family.steps",
    "editor.ease.family.spring",
    "editor.ease.family.bezier",
];

/// As chaves dos nomes das três direções (`0` In · `1` Out · `2` In-Out).
pub const CHAVES_DIRECAO: [&str; 3] = [
    "anim.easing.mode.in",
    "anim.easing.mode.out",
    "anim.easing.mode.in_out",
];

/// Os números de forma que algumas famílias leem. Os defaults são os das referências.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shape {
    /// `Back`: o `c1` de Penner (`1,70158` = 10 % de ultrapassagem).
    pub overshoot: f32,
    /// `Elastic`: o período, em fração da animação (`0,3` = Penner/GSAP).
    pub period: f32,
    /// `Steps`: o número de degraus (arredondado, mínimo `1`).
    pub steps: f32,
    /// `Spring`: a razão de amortecimento `ζ` (`< 1` oscila, `1` crítico, `> 1` lento).
    pub damping: f32,
    /// `Bezier`: `[x1, y1, x2, y2]`, `x` presos a `0..1` (CSS Easing L1).
    pub bezier: [f32; 4],
}

impl Shape {
    /// Os defaults — `ease` do CSS no `Bezier`.
    pub const DEFAULT: Shape = Shape {
        overshoot: 1.70158,
        period: 0.3,
        steps: 5.0,
        damping: 0.3,
        bezier: [0.25, 0.1, 0.25, 1.0],
    };
}

impl Default for Shape {
    fn default() -> Self {
        Self::DEFAULT
    }
}

const S0: f32 = f32::from_bits(0x3fc9_0fda);
const S1: f32 = f32::from_bits(0xbf25_5ddc);
const S2: f32 = f32::from_bits(0x3da3_33d8);
const S3: f32 = f32::from_bits(0xbb99_1c14);
const S4: f32 = f32::from_bits(0x391e_4867);
const E0: f32 = f32::from_bits(0x3f31_7218);
const E1: f32 = f32::from_bits(0x3e75_fe20);
const E2: f32 = f32::from_bits(0x3d63_4fb5);
const E3: f32 = f32::from_bits(0x3c1e_2545);
const E4: f32 = f32::from_bits(0x3aa6_5267);
const E5: f32 = f32::from_bits(0x395a_68ef);
/// `ln 1000` — a mola decai a `1/1000` em `t = 1`.
const LN1000: f32 = f32::from_bits(0x40dd_0c55);
const INV_LN2: f32 = f32::from_bits(0x3fb8_aa3b);
const INV_TAU: f32 = f32::from_bits(0x3e22_f983);

/// `sin(π/2 · x)` para `x ∈ [−1, 1]`.
#[must_use]
pub fn sinq(x: f32) -> f32 {
    let u = x * x;
    x * ((((S4 * u + S3) * u + S2) * u + S1) * u + S0)
}

/// `sin(2π · turns)`, para qualquer `turns` finito.
#[must_use]
pub fn sin_turn(turns: f32) -> f32 {
    let u = turns - (turns + 0.5).floor();
    let v = 4.0 * u;
    let w = if v > 1.0 {
        2.0 - v
    } else if v < -1.0 {
        -2.0 - v
    } else {
        v
    };
    sinq(w)
}

/// `cos(2π · turns)`.
#[must_use]
pub fn cos_turn(turns: f32) -> f32 {
    sin_turn(turns + 0.25)
}

/// `2^x`, com a escala inteira presa a `[−126, 127]` (o expoente normal do `f32`).
#[must_use]
pub fn exp2(x: f32) -> f32 {
    let k = x.floor();
    let fr = x - k;
    let p = 1.0 + fr * (((((E5 * fr + E4) * fr + E3) * fr + E2) * fr + E1) * fr + E0);
    #[expect(clippy::cast_possible_truncation, reason = "preso a [-126, 127] antes")]
    let ki = k.clamp(-126.0, 127.0) as i32;
    #[expect(clippy::cast_sign_loss, reason = "ki + 127 >= 1")]
    let escala = f32::from_bits(((ki + 127) as u32) << 23);
    p * escala
}

const T0: f32 = f32::from_bits(0x3f7f_ffc7);
const T1: f32 = f32::from_bits(0xbeaa_96f1);
const T2: f32 = f32::from_bits(0x3e4a_e2ad);
const T3: f32 = f32::from_bits(0xbe07_a5b6);
const T4: f32 = f32::from_bits(0x3da3_6d02);
const T5: f32 = f32::from_bits(0xbd0a_17ae);
const T6: f32 = f32::from_bits(0x3be0_1636);
const PI: f32 = f32::from_bits(0x4049_0fdb);
const HALF_PI: f32 = f32::from_bits(0x3fc9_0fdb);
const UM_TERCO: f32 = f32::from_bits(0x3eaa_aaab);

/// ⭐ **`atan2(y, x)` em VOLTAS** (`(−0,5, 0,5]`) — polinómio de grau 13 sobre `[0, 1]` (erro
/// medido `3,5e-7` rad) e a redução por octante, escrito igual no WGSL (ciclo 18: a matiz do OKLCH
/// tem de ser a mesma na placa e na CPU).
#[must_use]
pub fn atan_turn(y: f32, x: f32) -> f32 {
    let (ax, ay) = (x.abs(), y.abs());
    let mx = ax.max(ay);
    if mx == 0.0 {
        return 0.0;
    }
    let z = ax.min(ay) / mx;
    let u = z * z;
    let mut r = z * ((((((T6 * u + T5) * u + T4) * u + T3) * u + T2) * u + T1) * u + T0);
    if ay > ax {
        r = HALF_PI - r;
    }
    if x < 0.0 {
        r = PI - r;
    }
    if y < 0.0 {
        r = -r;
    }
    r * INV_TAU
}

/// ⭐ **A raiz cúbica** — o palpite pelos BITS e quatro passos de Newton, escrito igual no WGSL
/// (o WGSL não tem `cbrt`, e o `pow` dele não promete nada).
#[must_use]
pub fn cbrt(x: f32) -> f32 {
    if x == 0.0 {
        return 0.0;
    }
    let a = x.abs();
    let mut y = f32::from_bits(a.to_bits() / 3 + 0x2a51_4067);
    for _ in 0..4 {
        y = (y + y + a / (y * y)) * UM_TERCO;
    }
    if x < 0.0 { -y } else { y }
}

/// `x − floor(x)` — a parte fracionária que o WGSL escreve igual (o `rem_euclid` do Rust não é a
/// mesma conta).
#[must_use]
pub fn fract(x: f32) -> f32 {
    x - x.floor()
}

/// O `easeOutBounce` canónico — quatro parábolas.
fn bounce_out(t: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;
    if t < 1.0 / D1 {
        N1 * t * t
    } else if t < 2.0 / D1 {
        let t = t - 1.5 / D1;
        N1 * t * t + 0.75
    } else if t < 2.5 / D1 {
        let t = t - 2.25 / D1;
        N1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / D1;
        N1 * t * t + 0.984375
    }
}

/// O número de degraus do `Steps`.
fn steps_n(s: &Shape) -> f32 {
    s.steps.round().max(1.0)
}

/// A forma `In` de uma família em `t ∈ [0, 1]`; `Out` e `In-Out` saem dela por reflexão em
/// [`ease`].
#[must_use]
pub fn ease_in(curve: i32, t: f32, s: &Shape) -> f32 {
    match curve {
        QUAD => t * t,
        CUBIC => t * t * t,
        QUART => t * t * t * t,
        QUINT => t * t * t * t * t,
        CIRC => 1.0 - (1.0 - t * t).max(0.0).sqrt(),
        BACK => {
            let c1 = s.overshoot;
            let c3 = c1 + 1.0;
            c3 * t * t * t - c1 * t * t
        }
        BOUNCE => 1.0 - bounce_out(1.0 - t),
        SINE => {
            if t <= 0.0 {
                0.0
            } else if t >= 1.0 {
                1.0
            } else {
                1.0 - sinq(1.0 - t)
            }
        }
        EXPO => {
            if t <= 0.0 {
                0.0
            } else if t >= 1.0 {
                1.0
            } else {
                exp2(10.0 * t - 10.0)
            }
        }
        ELASTIC => {
            if t <= 0.0 {
                0.0
            } else if t >= 1.0 {
                1.0
            } else {
                let p = s.period.max(0.05);
                let q = t - 1.0;
                -(exp2(10.0 * q) * sin_turn((q - p * 0.25) / p))
            }
        }
        STEPS => {
            if t >= 1.0 {
                1.0
            } else {
                let n = steps_n(s);
                (t * n).floor() / n
            }
        }
        _ => t,
    }
}

/// `x(τ)` da mola — o degrau `0 → 1` com velocidade inicial nula, decaimento `1/1000` em `τ = 1`.
fn spring_x(tau: f32, damping: f32) -> f32 {
    let z = damping.clamp(0.05, 4.0);
    let decay = exp2(-LN1000 * tau * INV_LN2);
    if z < 0.999 {
        let w = LN1000 / z;
        let wd = w * (1.0 - z * z).sqrt();
        let turns = wd * tau * INV_TAU;
        1.0 - decay * (cos_turn(turns) + (LN1000 / wd) * sin_turn(turns))
    } else if z <= 1.001 {
        1.0 - decay * (1.0 + LN1000 * tau)
    } else {
        let r = (z * z - 1.0).sqrt();
        let w = LN1000 / (z - r);
        let r2 = -w * (z + r);
        let r1 = -LN1000;
        let e2 = exp2(r2 * tau * INV_LN2);
        1.0 - (r2 * decay - r1 * e2) / (r2 - r1)
    }
}

/// A mola normalizada: `x(t)/x(1)`, exacta nas pontas.
#[must_use]
pub fn spring(t: f32, damping: f32) -> f32 {
    if t <= 0.0 {
        0.0
    } else if t >= 1.0 {
        1.0
    } else {
        spring_x(t, damping) / spring_x(1.0, damping)
    }
}

/// O `cubic-bezier` do CSS: `x(s) = t` por 24 bissecções (o `x` é monótono com `x1, x2 ∈ [0,1]`),
/// depois `y(s)`. Contagem fixa: a CPU e o dispositivo fazem as MESMAS 24.
#[must_use]
pub fn bezier(t: f32, b: [f32; 4]) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    let x1 = b[0].clamp(0.0, 1.0);
    let x2 = b[2].clamp(0.0, 1.0);
    let ax = 1.0 + 3.0 * x1 - 3.0 * x2;
    let bx = 3.0 * x2 - 6.0 * x1;
    let cx = 3.0 * x1;
    let mut lo = 0.0_f32;
    let mut hi = 1.0_f32;
    for _ in 0..24 {
        let mid = 0.5 * (lo + hi);
        let x = ((ax * mid + bx) * mid + cx) * mid;
        if x < t {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let s = 0.5 * (lo + hi);
    let ay = 1.0 + 3.0 * b[1] - 3.0 * b[3];
    let by = 3.0 * b[3] - 6.0 * b[1];
    let cy = 3.0 * b[1];
    ((ay * s + by) * s + cy) * s
}

/// **A porta única**: a família `curve` com a direção `dir` (`0` In · `1` Out · `2` In-Out) em
/// `t ∈ [0, 1]`. `Linear`, `Spring` e `Bezier` ignoram a direção; a `Custom` devolve `t` (quem
/// tem a curva desenhada é o nó, e ele a lê ANTES de chamar aqui).
#[must_use]
pub fn ease(curve: i32, dir: i32, t: f32, s: &Shape) -> f32 {
    if curve == LINEAR || curve == CUSTOM {
        return t;
    }
    if curve == SPRING {
        return spring(t, s.damping);
    }
    if curve == BEZIER {
        return bezier(t, s.bezier);
    }
    match dir {
        1 => 1.0 - ease_in(curve, 1.0 - t, s),
        2 => {
            if t < 0.5 {
                ease_in(curve, 2.0 * t, s) * 0.5
            } else {
                1.0 - ease_in(curve, 2.0 - 2.0 * t, s) * 0.5
            }
        }
        _ => ease_in(curve, t, s),
    }
}

#[cfg(test)]
mod tests;
