#![forbid(unsafe_code)]
//! **The expression RECIPE catalog** (plano 10) — the artist-facing half of
//! expressions.
//!
//! An artist does not want `value + wiggle(2, 30)`; they want *"shake, a bit
//! slower"*. Twenty years of After Effects say that a better text editor does not
//! close that gap — the market answered with plugins that are **buttons which write
//! the expression for you**, and Apple Motion, Cavalry, Blender and Rive all
//! converged, independently, on a **catalog with knobs**. This crate is that
//! catalog.
//!
//! # The model
//!
//! A [`Recipe`] is a named, knobbed fragment that **emits text** in the VEX-lite
//! language. A [`RecipeStack`] is a list of rows applied **top to bottom**, folded
//! into ONE formula by [`RecipeStack::to_formula`].
//!
//! ```text
//!   row 1  Shake   Speed 2  Amount 30        value + wiggle(2, 30)
//!   row 2  Limit   Min -10  Max 10     ->    min(max(value + wiggle(2, 30), -10), 10)
//! ```
//!
//! ⚠️ **The recipe is the SOURCE; the text is a PROJECTION.** There is no second
//! string kept in sync, because there is no second string: the formula bar renders
//! `to_formula()`. Going the other way — parsing arbitrary text back into rows —
//! is deliberately NOT offered: a recogniser of "canonical fragments" starts LYING
//! the day someone edits one character. Re-opening a formula that is byte-identical
//! to what some stack would produce is not a parser, it is a comparison
//! ([`RecipeStack::recover`]).
//!
//! # Three kinds of row
//!
//! | [`RowKind`] | what it does |
//! |---|---|
//! | [`RowKind::Value`] | transforms the value |
//! | [`RowKind::Time`]  | **rewrites the clock the rows BELOW it read** |
//! | [`RowKind::Raw`]   | the artist's own formula, passed through |
//!
//! The Time row is what Cinema 4D calls a `Time` effector and Houdini a `shift`.
//! It costs nothing extra because we own the emitted text: a Value row is handed
//! the current clock as [`EmitCtx::clock`], and a Time row simply changes it for
//! everything after.
//!
//! ⚠️ **But a clock only reaches a recipe that SPELLS it**, and one does not:
//! `wiggle` builds `time + __seed` **inside the parser** (`ph2d-expr-parse`), so a
//! Time row cannot reach a `Shake`. That is a fact about the frozen sugar, not a
//! bug to fix here, so every recipe DECLARES which it is ([`ClockUse`]) and the UI
//! tells the truth instead of surprising the artist. `Drift` is the clock-honouring
//! sibling of `Shake` for exactly this reason.
//!
//! # What this crate is NOT
//!
//! It does not parse and it does not evaluate — it emits strings. That is what
//! keeps it a leaf that both the timeline UI and the Motion node's
//! `motion.expression` can consume. `ph2d-expr-parse` and `ph2d-expr` are
//! **dev-dependencies**, used only by the gates that prove every emitted formula
//! parses and that composing two rows equals composing their two functions.

mod catalog;
mod emit;
mod knob;
mod recipe;
mod refusal;
mod search;
mod stack;

pub use catalog::{CATALOG, by_id};
pub use emit::{EmitCtx, fmt_num, paren};
pub use knob::{Knob, KnobKind, KnobValue};
pub use recipe::{ClockUse, Family, Neutrality, Recipe, RecipeId, RowKind};
pub use refusal::{Destination, REFUSALS, Refusal};
pub use search::{SearchHit, search};
pub use stack::{RecipeStack, Row, SEED_CLOCK, SEED_VALUE};
