//! **O som do sprite** (P13 §4): faixas que referem um asset de áudio do projeto, NUNCA as
//! amostras. Tocar e o relógio moram na `ph2d-pixel-audio`.

/// Uma faixa sob os quadros: começa em `inicio_ms` (negativo = já a meio no quadro 0).
#[derive(Debug, Clone)]
pub struct FaixaDeAudio {
    /// A referência ao asset (o caminho ou nome no projeto).
    pub asset: String,
    pub inicio_ms: i32,
    /// Ganho linear (1 = como está).
    pub ganho: f32,
    pub mudo: bool,
}

// A igualdade é por BITS (o documento é `Eq`, e o formato grava os bits).
impl PartialEq for FaixaDeAudio {
    fn eq(&self, o: &Self) -> bool {
        self.asset == o.asset
            && self.inicio_ms == o.inicio_ms
            && self.ganho.to_bits() == o.ganho.to_bits()
            && self.mudo == o.mudo
    }
}
impl Eq for FaixaDeAudio {}
