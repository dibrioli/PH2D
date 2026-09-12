//! **O pedaço que o Trim aponta** (plano 38) — o TIPO que o realce desenha e o clique apaga. A lei
//! (o que é uma fronteira, o que sobra) mora em `ph2d_vec_scene::trim_tool`; o gesto na shell
//! (`vec_trim.rs`).
//!
//! ⚠️ **Desceu da shell em 2026-09-12** (`line/render-loop`, A9 da auditoria de arquitectura): o
//! campo `vec_trim_hit` da `App` tinha o TIPO na shell, e um campo assim não pode juntar-se ao
//! [`crate::state::VecState`].

use ph2d_vec_scene::VecPathId;

/// **O pedaço que o cursor está a apontar.** É o que o realce desenha e o que o clique apaga — a
/// mesma resposta, pela mesma porta.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrimHit {
    pub path: VecPathId,
    /// O índice do contorno na ordem canónica (`trim_tool::contours_of`): `0` = primário.
    pub contour: usize,
    pub de: f64,
    pub ate: f64,
}
