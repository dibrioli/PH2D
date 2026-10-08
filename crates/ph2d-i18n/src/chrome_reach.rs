//! **O ALCANCE DAS BARRAS** (D10, F4–F5 — `docs/UI_New_and_Simple/vetor_nova_ui/03_plano.md` §4):
//! o radial por famílias, a paleta que mostra onde o comando mora, e o toque sem teclado. Irmão do
//! `vector_bars.rs` pela razão do `chrome_menus.rs`: duas linhas que acrescentem chaves ao mesmo
//! `match` colidem no mesmo punhado de linhas.

/// A tradução de uma chave do alcance das barras, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "chrome.bars.family.select" => "Select",
        "chrome.bars.family.draw" => "Draw",
        "chrome.bars.family.build" => "Build",
        "chrome.bars.family.carve" => "Carve",
        "chrome.bars.family.stroke" => "Stroke",
        "chrome.bars.family.colour" => "Colour",
        "chrome.palette.left_bar" => "Left bar",
        "chrome.palette.top_bar" => "Top bar",
        "chrome.palette.near_selection" => "Next to the selection",
        "chrome.touch.shift" => "Shift",
        "chrome.touch.alt" => "Alt",
        "chrome.touch.ctrl" => "Ctrl",
        "chrome.touch.tip" => {
            "Hold: held while pressed \u{00b7} Tap: lock \u{00b7} Or rest 1 finger (Shift) or 2 (Shift+Alt) while drawing"
        }
        "chrome.menu.focus_mode" => "Focus mode \u{00b7} Ctrl+Space",
        "chrome.menu.tool_row_bottom" => "Tool options at the bottom",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
