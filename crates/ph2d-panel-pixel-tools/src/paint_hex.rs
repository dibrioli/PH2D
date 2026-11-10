//! O hex de uma cor como o painel o mostra.

/// `#RRGGBB` (com `AA` quando a cor não é opaca) — o hex que o Aseprite mostra.
#[must_use]
pub fn hex(c: [u8; 4]) -> String {
    let [r, g, b, a] = c;
    // Um formato só nos cartões: `F0F0F5`, como o Inspector e o Vector; `PH2D_CARDS=0` tem o de antes.
    let cardinal = if ph2d_editor_core::panel::cards::ferramenta::cards_on() {
        ""
    } else {
        "#"
    };
    if a == u8::MAX {
        format!("{cardinal}{r:02X}{g:02X}{b:02X}")
    } else {
        format!("{cardinal}{r:02X}{g:02X}{b:02X}{a:02X}")
    }
}
