//! **The catalog** — one table, four consumers.
//!
//! `paint` / `populate` / `event` / the seam gate all iterate THIS slice, which is
//! the structural answer to *"the fullest card rots"*: a recipe added here is born
//! painted, registered, live under the mouse and swept by the seam. The pattern is
//! the physics panel's `SECTIONS` and the timeline's `ADDPROP_BUTTONS`.
//!
//! Split per family so no file approaches the LOC cap and so a family reads as a
//! unit — the emit functions are the specification of the language we speak.

// ⚠️ **`logic` e `field` NÃO são mais módulos** (FASE A do plano 12). As seis de Logic
// saíram por serem PROGRAMAÇÃO (D3, *"não vejo o menor sentido para artistas na seção
// logic"*) e as três de Field por serem COMPOSIÇÕES que a pilha já expressa
// (`fade-by-distance ~> distance-2d`, 6e-8). As duas famílias respondem por
// `refusal::REFUSALS` (`condition` → um keyframe · `compose` → duas linhas), e o registro
// com a medição de cada corte está em `retired.rs`. O `git log` guarda o código.
mod life;
mod link;
mod physics;
mod raw;
mod shape;
mod time;
mod wave;

use crate::recipe::{Recipe, RecipeId};

/// Every recipe, in gallery order.
///
/// ⚠️ **50 → 31 na FASE A**, e o número é o RESTO de uma regra (*inerte ou programação*),
/// nunca uma cota — a meta "~21" do plano foi abandonada porque o tamanho não era o
/// defeito (a Cavalry shipa 40+ Behaviours). O que saiu, e a medição de cada corte, está
/// em [`crate::retired::RETIRED`]; quem cortar mais entra ali no MESMO commit, senão o gate
/// `every_retired_label_still_finds_its_answer` nasce vermelho.
pub const CATALOG: &[&Recipe] = &[
    // Life
    &life::SHAKE, // absorveu o `turbulence` (Detail/Roughness)
    &life::DRIFT,
    &life::JITTER,
    &life::BREATHE,
    &life::FLICKER,
    // Wave — ⚠️ NENHUM corte: a medição REFUTOU *"Ping-Pong e Pulse e Blink são a MESMA
    // pergunta"* (nenhuma contenção entre as três; são triangular · quadrada · dente com
    // decaimento). O `Cycle` com chip de forma segue defensável como PRODUTO, e é decisão
    // do Enio — não um corte por redundância, que é como o plano o apresentava.
    &wave::SWAY,
    &wave::BOUNCE,
    &wave::PING_PONG,
    &wave::BLINK,
    &wave::PULSE,
    &wave::ORBIT_X,
    &wave::ORBIT_Y,
    // Link
    &link::FOLLOW, // herdou `opposite` (Multiply −1) e `offset-copy`
    &link::DISTANCE_2D,
    &link::DISTANCE_1D,
    &link::BLEND_TWO,
    // Shape
    &shape::LIMIT, // herdou `floor-at`, `ceiling-at` e `remap-clamped` (mútua)
    &shape::REMAP,
    &shape::MULTIPLY_ADD, // herdou `invert-range`
    &shape::ABSOLUTE,
    &shape::QUANTIZE,
    // Time — ⚠️ `freeze-after` e `start-at` FICAM: a medição não achou a contenção que o
    // plano afirmava (*"o mesmo clamp em lados opostos"*).
    &time::STEPPED_TIME,
    &time::DELAY,
    &time::SPEED, // herdou `reverse-time` (Speed −1)
    &time::FREEZE_AFTER,
    &time::START_AT,
    &time::PING_PONG_TIME,
    // Physics-lite
    &physics::PENDULUM,
    &physics::THROW, // herdou `free-fall` (velocidade 0)
    &physics::WAVE_ALONG_CHAIN,
    // Raw
    &raw::CUSTOM,
];

/// Look a recipe up by its stable id.
#[must_use]
pub fn by_id(id: RecipeId) -> Option<&'static Recipe> {
    CATALOG.iter().copied().find(|r| r.id == id)
}
