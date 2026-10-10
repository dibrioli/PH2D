//! **O QUADRO — a fidelidade ao Excalidraw** (MiroClone W9, `docs/MiroClone/07_fidelidade_excalidraw.md`):
//! as palavras da cena 11.

/// A tradução de uma chave da cena 11 do quadro, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "board.smoke.fidelity.title" => {
            "Like Excalidraw: your file, arrows that stop before the shape, heads that shrink"
        }
        "board.smoke.fidelity.from" => "Start here",
        "board.smoke.fidelity.to" => "Drop inside",
        "board.smoke.fidelity.l1" => {
            "1. Your bv.excalidraw (top left): the three arrows are straight and stop just before the shapes. Compare it with docs/MiroClone/bv_excalidraw.png, the picture Excalidraw draws."
        }
        "board.smoke.fidelity.l2" => {
            "2. Open it yourself: right-click this tab, Open an Excalidraw file…, pick docs/MiroClone/bv.excalidraw. A new tab opens with the same drawing."
        }
        "board.smoke.fidelity.l3" => {
            "3. Short arrows (10 to 60 long, bottom left): the head shrinks to half the arrow. Before, an arrow shorter than 24 had no head."
        }
        "board.smoke.fidelity.l4" => {
            "4. Draw: press A, drag from Start here and drop inside Drop inside. The arrow stops a few pixels before the box. Hold Ctrl while dropping to leave it loose."
        }
        "board.smoke.fidelity.l5" => {
            "5. This box came from a file with the plain letter. Press Sketch ↔ Final (top bar, nothing selected): its words take the hand letter, like the whole board."
        }
        _ => return None,
    })
}
