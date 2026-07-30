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
//!   row 1  Shake   Speed 2  Amount 0.3       value + wiggle(2, 0.3)
//!   row 2  Limit   Min -1   Max 1      ->    min(max(value + wiggle(2, 0.3), -1), 1)
//! ```
//!
//! # The scale a number is written in
//!
//! ⚠️ **A knob that carries a VALUE is authored so that `1` is about one object.**
//! The catalog is property-agnostic — the same `Shake` drives a position, a
//! rotation, a scale or an opacity — so it cannot convert units, and it does not
//! need to: a metre is roughly the size of a thing, a radian is roughly a big turn,
//! and `1` is full scale and full opacity. One magnitude reads sensibly in all four,
//! and that is why there is **no per-property table** here to keep in sync with
//! `PropKind`.
//!
//! The ceiling is the canvas. At the project default of **100 px/m** a 4K canvas is
//! `4096 / 100 = 40.96 m` across, so [`CANVAS_M`] is what a value RANGE is allowed to
//! reach and a DEFAULT stays two orders below it — an amount that starts at a third
//! of a canvas is not a strong default, it is an object you cannot see.
//!
//! ⚠️ The first smoke of the modal reported *"Shake: changing the parameters did not
//! change the animation"*. Measured, the formula responded perfectly (amount 30 → 120
//! quadrupled its span); what did not respond was the SCREEN, because `30` meant
//! **30 metres = 3000 px** and the object had left the frame — every value of every
//! knob looked the same, which is to say looked like nothing. Fourteen of the
//! forty-seven value recipes did that at their own defaults. The gate that keeps them
//! home is `no_recipe_flings_the_object_off_a_4k_canvas`.
//!
//! ⚠️ **The recipe is the SOURCE; the text is a PROJECTION.** There is no second
//! string kept in sync, because there is no second string: the formula bar renders
//! `to_formula()`. Going the other way — parsing arbitrary text back into rows —
//! is deliberately NOT offered: a recogniser of "canonical fragments" starts LYING
//! the day someone edits one character. Re-opening a formula that is byte-identical
//! to what some stack would produce is not a parser, it is a comparison
//! — and it is NOT built. ⚠️ Two doc-comments cited a `RecipeStack::recover` that has
//! never existed (grepped): the sheet lives in the panel's state for as long as the card
//! is open, and closing it keeps the FORMULA, not the rows.
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

/// The widest a **value** knob may be dragged, in the driven property's units.
///
/// A 4K canvas at the project's default 100 px/m is `4096 / 100 = 40.96 m` across;
/// this is that canvas, rounded to a number a person can read. Nothing forbids
/// TYPING past it — the number box clamps only its arrows and its drag — so this is
/// where the artist's thumb stops, not where the model does.
pub const CANVAS_M: f32 = 40.0; // LITERAL-PX-OK: 4096 px / 100 px-por-metro, arredondado

mod catalog;
mod emit;
mod knob;
mod recipe;
mod refusal;
pub mod retired;
mod search;
mod stack;

pub use catalog::{CATALOG, by_id};
pub use emit::{EmitCtx, fmt_num, paren};
pub use knob::{Knob, KnobKind, KnobValue};
pub use recipe::{ClockUse, Combine, Family, Neutrality, Recipe, RecipeId, RowKind};
pub use refusal::{Destination, REFUSALS, Refusal};
pub use retired::{Answer, RETIRED, Retired, retired_by_id};
pub use search::{SearchHit, search};
pub use stack::{RecipeStack, Row, SEED_CLOCK, SEED_VALUE};
