//! **As guias do documento** (P13 §4): pontos de fuga, horizonte e a grelha iso 2:1, em células
//! do sprite. Só o dado; `direcao_guiada` e o overlay moram na `ph2d-pixel-guia`.

/// A grelha isométrica 2:1 (dimétrica), ancorada em `origem`; o losango tem `2·passo × passo`.
#[derive(Debug, Clone, Copy)]
pub struct Iso2x1 {
    pub origem: [f32; 2],
    /// O lado vertical do losango, em células (≥ 1).
    pub passo: u16,
}

impl Iso2x1 {
    /// O losango de omissão: 16×8 células.
    pub const PASSO: u16 = 8;
}

impl Default for Iso2x1 {
    fn default() -> Self {
        Self {
            origem: [0.0; 2],
            passo: Self::PASSO,
        }
    }
}

/// Perspectiva de 1, 2 ou 3 pontos e/ou isometria. Vazia = sem guias.
#[derive(Debug, Clone, Copy, Default)]
pub struct Guias {
    pub pontos_de_fuga: [Option<[f32; 2]>; 3],
    /// A linha do horizonte (y em células).
    pub horizonte: Option<f32>,
    pub iso: Option<Iso2x1>,
}

fn bits(p: [f32; 2]) -> [u32; 2] {
    p.map(f32::to_bits)
}

// A igualdade é por BITS (o documento é `Eq`, e o formato grava os bits).
impl PartialEq for Iso2x1 {
    fn eq(&self, o: &Self) -> bool {
        bits(self.origem) == bits(o.origem) && self.passo == o.passo
    }
}
impl Eq for Iso2x1 {}

impl PartialEq for Guias {
    fn eq(&self, o: &Self) -> bool {
        self.pontos_de_fuga.map(|p| p.map(bits)) == o.pontos_de_fuga.map(|p| p.map(bits))
            && self.horizonte.map(f32::to_bits) == o.horizonte.map(f32::to_bits)
            && self.iso == o.iso
    }
}
impl Eq for Guias {}
