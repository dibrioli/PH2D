//! **The catalog** — one table, four consumers.
//!
//! `paint` / `populate` / `event` / the seam gate all iterate THIS slice, which is
//! the structural answer to *"the fullest card rots"*: a recipe added here is born
//! painted, registered, live under the mouse and swept by the seam. The pattern is
//! the physics panel's `SECTIONS` and the timeline's `ADDPROP_BUTTONS`.
//!
//! Split per family so no file approaches the LOC cap and so a family reads as a
//! unit — the emit functions are the specification of the language we speak.

mod field;
mod life;
mod link;
mod logic;
mod physics;
mod raw;
mod shape;
mod time;
mod wave;

use crate::recipe::{Recipe, RecipeId};

/// Every recipe, in gallery order.
pub const CATALOG: &[&Recipe] = &[
    // Life
    &life::SHAKE,
    &life::TURBULENCE,
    &life::DRIFT,
    &life::JITTER,
    &life::BREATHE,
    &life::FLICKER,
    // Wave
    &wave::SWAY,
    &wave::SWAY_COSINE,
    &wave::BOUNCE,
    &wave::PING_PONG,
    &wave::RAMP_LOOP,
    &wave::BLINK,
    &wave::PULSE,
    &wave::ORBIT_X,
    &wave::ORBIT_Y,
    // Link
    &link::FOLLOW,
    &link::MIRROR,
    &link::OPPOSITE,
    &link::OFFSET_COPY,
    &link::DISTANCE_2D,
    &link::DISTANCE_1D,
    &link::MIDPOINT,
    &link::BLEND_TWO,
    &link::SWITCH,
    // Shape
    &shape::LIMIT,
    &shape::FLOOR_AT,
    &shape::CEILING_AT,
    &shape::REMAP,
    &shape::REMAP_CLAMPED,
    &shape::MULTIPLY_ADD,
    &shape::NEGATE,
    &shape::INVERT_RANGE,
    &shape::ABSOLUTE,
    &shape::QUANTIZE,
    // Time
    &time::STEPPED_TIME,
    &time::DELAY,
    &time::SPEED,
    &time::REVERSE_TIME,
    &time::FREEZE_AFTER,
    &time::START_AT,
    &time::PING_PONG_TIME,
    // Logic
    &logic::IF_GREATER,
    &logic::IF_LESS,
    &logic::IF_EQUAL,
    &logic::GATE_AND,
    &logic::GATE_OR,
    &logic::AFTER_TIME,
    // Field
    &field::FADE_BY_DISTANCE,
    &field::SCALE_BY_PROXIMITY,
    &field::GRADIENT_BY_VALUE,
    // Physics-lite
    &physics::PENDULUM,
    &physics::FREE_FALL,
    &physics::THROW,
    &physics::WAVE_ALONG_CHAIN,
    // Raw
    &raw::CUSTOM,
];

/// Look a recipe up by its stable id.
#[must_use]
pub fn by_id(id: RecipeId) -> Option<&'static Recipe> {
    CATALOG.iter().copied().find(|r| r.id == id)
}
