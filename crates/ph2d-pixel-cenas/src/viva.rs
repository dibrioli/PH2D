//! Os números da cena 12.

/// O tamanho da cena 12.
pub const SIZE: (u32, u32) = (128, 80);
/// O raio do canto do retângulo assentado, em células.
pub const CORNER: f64 = 6.0;
/// A estrela viva: a caixa (cantos inclusivos), a cor e o giro (graus, horário na tela).
pub const STAR: ((i32, i32), (i32, i32), [u8; 4], f64) =
    ((58, 8), (117, 67), [220, 60, 60, 255], 18.0);
