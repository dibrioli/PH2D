//! **A cena 29** (MiroClone W24): a barra de estilo foge das gavetas, dos painéis e da barra curta.

/// A tradução de uma chave da cena 29, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "board.smoke.w24.board" => "Bars",
        "board.smoke.w24.frame" => "The style bar keeps clear",
        "board.smoke.w24.l1" => {
            "1. The box on the left is selected and the shapes drawer is open: the box\u{2019}s bar \
             stays clear of it. On the left bar, open the others one at a time: Sticky note, Pen, \
             Templates. The bar moves aside and never covers an open drawer."
        }
        "board.smoke.w24.l2" => {
            "2. Frames is open on the right. Select the box on the right, then open Comments and \
             Minimap too (bottom right): its bar keeps clear of every panel."
        }
        "board.smoke.w24.l3" => {
            "3. Make the window very narrow: the bar breaks into more rows and never goes over the \
             left bar."
        }
        _ => return None,
    })
}
