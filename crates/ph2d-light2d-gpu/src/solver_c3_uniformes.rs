//! Os espelhos dos uniformes da C3 (a fina e o preenchimento). Módulo-filho de `solver_c3.rs` (o tecto de
//! 700 LOC).

use bytemuck::{Pod, Zeroable};

/// O espelho de `Fina` do `c3_direta.wgsl` (48 B).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct UniformeFina {
    pub(super) tamanho: [u32; 2],
    pub(super) inicio: [u32; 2],
    pub(super) fim: [u32; 2],
    pub(super) inicio_grosso: [u32; 2],
    pub(super) fim_grosso: [u32; 2],
    pub(super) fina: u32,
    pub(super) limiar: f32,
}

/// O espelho de `Preenche` do `c3_preenche.wgsl` (48 B).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct UniformePreenche {
    pub(super) celulas: [u32; 2],
    pub(super) inicio: [u32; 2],
    pub(super) fim: [u32; 2],
    pub(super) celula: u32,
    pub(super) salto: u32,
    pub(super) fina: u32,
    pub(super) _p: [u32; 3],
}
