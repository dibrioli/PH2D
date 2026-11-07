//! Os números da cena 13.

/// O tamanho da cena 13.
pub const SIZE: (u32, u32) = (160, 110);
/// O lado de cada amostra assentada e o passo da grelha, em células.
const LADO: i32 = 30;
const PASSO: i32 = 36;
/// A engrenagem viva: a caixa (cantos inclusivos) e a cor.
pub const GEAR: ((i32, i32), (i32, i32), [u8; 4]) = ((112, 30), (155, 73), [200, 70, 50, 255]);

/// A caixa da amostra `i` da grelha (cantos inclusivos).
#[must_use]
pub const fn caixa(i: usize) -> ((i32, i32), (i32, i32)) {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "índice de uma grelha 3 × 3"
    )]
    let (c, l) = ((i % 3) as i32, (i / 3) as i32);
    let a = (4 + c * PASSO, 4 + l * PASSO);
    (a, (a.0 + LADO - 1, a.1 + LADO - 1))
}
