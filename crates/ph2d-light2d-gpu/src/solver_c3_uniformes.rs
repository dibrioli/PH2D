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

impl super::C3 {
    /// Os grupos 8×8 da REGIÃO da fina deste quadro (o despacho da fina de `F` e o da de `M`).
    pub(in crate::solver) fn grupos_da_fina(&self) -> [u32; 2] {
        let (a, b) = (self.u.inicio, self.u.fim);
        [(b[0] - a[0]).div_ceil(8), (b[1] - a[1]).div_ceil(8)]
    }
}
