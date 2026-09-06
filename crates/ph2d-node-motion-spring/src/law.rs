//! ⭐⭐⭐ **A MOLA EM DOIS NÚMEROS QUE UM ARTISTA SABE PENSAR** (ciclo 2, W1 — doc 105).
//!
//! **O problema, medido:** a mola tinha **três** params, e dois deles são constantes de física —
//! `tension = 8`, `friction = 1,5`. *Nenhum artista consegue prever o que `friction 1,5` faz*, e
//! a única forma de a afinar é tentativa e erro. É o animador com mais poder por knob do grupo e
//! o que menos se deixa dirigir.
//!
//! ⛔⛔ **E a hipótese óbvia — «falta a MASSA» — está REFUTADA duas vezes, antes de uma linha de
//! código.** **(a)** Matematicamente ela é absorvida: `m·x″ + c·x′ + k·x = 0` dividido por `m` dá
//! `x″ + (c/m)·x′ + (k/m)·x = 0`, então a mola tem **dois** graus de liberdade e um param de
//! massa só reescalaria os outros dois. **(b)** E ela já existe onde faz sentido: o
//! [`super::channel::inv_mass_at`] lê a coluna `inv_mass` que o `motion.pin_constraint` escreve —
//! massa é **por elemento**, não um knob. *Um terceiro knob redundante teria sido a primeira
//! coisa que eu construiria sem medir.*
//!
//! ## A reparametrização: **duração** e **salto**
//!
//! É a mesma equação escrita nos dois números que a pergunta do artista tem — *quanto tempo* e
//! *quanto salta* —, e é a forma para que o estado da arte convergiu (o `spring(duration:bounce:)`
//! do SwiftUI, o `visualDuration`/`bounce` do Framer Motion). ⚠️ **Não é uma aproximação:** a
//! conversão é fechada e o gate `the_time_mode_reproduces_the_physics_mode` prova-o correndo as
//! duas leis.
//!
//! ```text
//! ω₀ = √tension                    (frequência natural)
//! ζ  = friction / (2·√tension)     (razão de amortecimento)
//!
//! tension  = (2π / duration)²
//! friction = 4π·(1 − bounce) / duration          se bounce ≥ 0   (ζ = 1 − bounce)
//! friction = 4π / (duration·(1 + bounce))        se bounce <  0   (ζ = 1/(1 + bounce))
//! ```
//!
//! - `bounce = 0` ⇒ ζ = 1, **criticamente amortecida**: chega e pára, sem passar do alvo.
//! - `bounce > 0` ⇒ sub-amortecida, salta. `bounce → 1` ⇒ oscila quase sem parar.
//! - `bounce < 0` ⇒ sobre-amortecida, arrasta-se.
//!
//! ⚠️ **Os dois ramos encontram-se em `bounce = 0`**: os dois dão `friction = 4π/duration`, então
//! a função é contínua ali — sem isso o slider teria um degrau no meio, no ponto de omissão.
//!
//! ## Por que ela vive numa PORTA
//!
//! Ela é lida pela CPU (`eval`) **e** pelo dispositivo (o WGSL do [`super::kernel`]), e é por
//! isso que a fórmula está escrita aqui em prosa executável: as duas rotas têm de fazer as
//! MESMAS operações na MESMA ordem, que é a disciplina de bit-identidade desta casa. O gate
//! `the_wgsl_says_what_the_rust_says` compara os dois textos termo a termo.

/// O modo do param `mode` — a escada é o wire format, **append-only**.
pub const MODE_PHYSICS: f32 = 0.0;
pub const MODE_TIME: f32 = 1.0;

/// `2π`, e `4π` — escritos uma vez para as duas rotas os poderem citar.
const TWO_PI: f32 = std::f32::consts::TAU;
const FOUR_PI: f32 = 4.0 * std::f32::consts::PI;

/// O piso da duração. ⚠️ **É um recurso de REPRESENTAÇÃO, não um gosto:** `tension = (2π/d)²`, e
/// a `d = 0,01` isso já dá `394 784` — o sub-passo estável (`√(0,05/tension)`) fica em `0,0004 s`
/// e o tecto de `MAX_STEPS` corta a integração. A `0,05` a tensão é `15 791` e o passo pedido é
/// `0,0018 s`: **56 sub-passos** num quadro de 60 fps, ainda dentro dos 64.
pub const MIN_DURATION: f32 = 0.05;
/// O tecto do salto. ⚠️ `bounce = 1` daria `friction = 0` — uma mola que **nunca** assenta; e
/// `bounce = −1` daria uma divisão por zero. Os dois lados param a um passo do slider.
pub const MAX_BOUNCE: f32 = 0.99;

/// ⭐⭐⭐ **O QUE O INTEGRADOR CONSEGUE INTEGRAR — derivado das constantes DELE, não escolhido.**
///
/// O passo mais fino que a mola alguma vez toma é `MAX_DT / MAX_STEPS`; acima da estabilidade
/// desse passo ela **explode**, por mais sub-passos que peça. ⇒ a conversão não pode pedir ao
/// integrador uma mola que ele não sabe integrar, e o tecto **sai da aritmética**:
///
/// | | |
/// |---|---|
/// | passo mínimo | `MAX_DT / MAX_STEPS` = `0,1 / 64` = **`1,5625 ms`** |
/// | tecto da rigidez | `STABLE / passo²` = `0,05 / 2,44e-6` = **`20 480`** |
/// | tecto do atrito | `FRICTION_STABLE / passo` = `1,0 / 1,5625e-3` = **`640`** |
///
/// ⚠️ **Ele só morde no modo `Time`:** no `Physics` os sliders param em `tension 60` e
/// `friction 20`, duas ordens de grandeza abaixo — e o caminho de omissão devolve os params
/// **tal e qual**, sem passar por aqui.
const MIN_SUB_DT: f32 = crate::MAX_DT / crate::MAX_STEPS as f32;
const TENSION_CEIL: f32 = crate::STABLE / (MIN_SUB_DT * MIN_SUB_DT);
const FRICTION_CEIL: f32 = crate::FRICTION_STABLE / MIN_SUB_DT;

/// ⭐ **A ÚNICA PORTA:** `(tension, friction)` a partir do que o artista autorou.
///
/// No modo `Physics` devolve os dois params tal e qual — **byte a byte** o que sempre fez, e é
/// isso que mantém todo grafo autorado intacto.
#[must_use]
pub fn physics_of(
    mode: f32,
    tension: f32,
    friction: f32,
    duration: f32,
    bounce: f32,
) -> (f32, f32) {
    if mode < 0.5 {
        return (tension, friction);
    }
    let d = duration.max(MIN_DURATION);
    let b = bounce.clamp(-MAX_BOUNCE, MAX_BOUNCE);
    let w = TWO_PI / d;
    let k = w * w;
    // ⚠️ Os dois ramos encontram-se em `b = 0` (os dois dão `4π/d`): a função é contínua no
    // ponto de omissão, e o slider não tem degrau a meio.
    let c = if b >= 0.0 {
        FOUR_PI * (1.0 - b) / d
    } else {
        FOUR_PI / (d * (1.0 + b))
    };
    // ⚠️ **O tecto do INTEGRADOR, e não do gosto** — ver [`TENSION_CEIL`]. Sem ele um
    // `Duration` curto com `Bounce` muito negativo pede uma mola que nenhum número de
    // sub-passos integra, e ela explode.
    (k.min(TENSION_CEIL), c.min(FRICTION_CEIL))
}

#[cfg(test)]
#[path = "law_tests.rs"]
mod tests;
