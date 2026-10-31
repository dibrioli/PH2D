//! C.2: a cópia f64 que a função `#[target_feature]` do `main` inlina (código v3 no mesmo binário).
#![allow(clippy::unnecessary_cast, dead_code)]
pub type F = f64;
include!("kern.rs");
