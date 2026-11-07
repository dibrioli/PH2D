//! Os números da cena 15.

/// O tamanho da cena 15.
pub const SIZE: (u32, u32) = (128, 80);
/// Onde o «F» está (o canto da caixa dele).
pub const FIGURE_AT: (i32, i32) = (16, 24);
/// O nome da camada de baixo (a das figuras).
pub const NAME: &str = "Figures";
/// A inclinação da moldura solta (graus).
pub const SKEW: i32 = 20;
/// O losango verde: o centro e o raio (Manhattan).
pub const BLOB: ((i32, i32), i32) = ((92, 40), 12);

/// O losango verde.
#[must_use]
pub fn in_blob(x: i32, y: i32) -> bool {
    let ((cx, cy), r) = BLOB;
    (x - cx).abs() + (y - cy).abs() <= r
}
