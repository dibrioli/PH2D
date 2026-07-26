//! The VEX-lite parser moved to the shared `ph2d-expr-parse` leaf crate (ADR-0144),
//! so `motion.expression` and the timeline's property expressions parse through ONE
//! door — two parsers for one IR would drift in silence. Kept as a thin re-export so
//! the existing `parse::parse` call site (`lib.rs`) resolves unchanged; the grammar,
//! `wiggle` sugar, and the tests now live in `ph2d-expr-parse`.

pub(crate) use ph2d_expr_parse::parse;
