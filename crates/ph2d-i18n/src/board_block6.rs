//! **A cena 28** (MiroClone W23, bloco 6): o laço, o QuickShape da caneta, as menções e mandar a
//! selecção para outro quadro — os nomes dos frames e as lições, pela ordem real.

/// A tradução de uma chave da cena 28, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "board.smoke.b6.board" => "Block 6",
        "board.smoke.b6.plan" => "Plan",
        "board.smoke.b6.f1" => "1 \u{b7} Lasso",
        "board.smoke.b6.f2" => "2 \u{b7} Draw here",
        "board.smoke.b6.f3" => "3 \u{b7} Mentions",
        "board.smoke.b6.f4" => "4 \u{b7} Send to another board",
        "board.smoke.b6.idea" => "Idea",
        "board.smoke.b6.edge_most" => "Mostly inside",
        "board.smoke.b6.edge_half" => "Half outside",
        "board.smoke.b6.l1" => {
            "1. Pen button (left bar) \u{2192} Lasso. Draw a loop around the frame. The notes inside \
             are selected; \u{201c}Mostly inside\u{201d} too, \u{201c}Half outside\u{201d} not."
        }
        "board.smoke.b6.l2" => {
            "2. Pen \u{2192} draw a box, HOLD STILL: a rectangle (Ctrl+Z: the stroke back). Sketch is \
             on: bottom right, left = Rectangle tool, right = pen. Select one, press the wavy button: \
             smooth. Again: hand-drawn."
        }
        "board.smoke.b6.l3" => {
            "3. Rui mentioned you: the message at the top, the dot on Comments. Click Show. \
             Reply with @ and pick a name from the list."
        }
        "board.smoke.b6.l4" => {
            "4. Select the two boxes and the arrow, right-click one \u{2192} Move to board\u{2026} \
             \u{2192} Plan. They land in Plan; the message at the top has Undo."
        }
        "board.smoke.b6.launch" => "Launch plan",
        "board.smoke.b6.rui_says" => "{name} can you check the dates?",
        "board.smoke.b6.a" => "Design",
        "board.smoke.b6.b" => "Build",
        _ => return None,
    })
}
