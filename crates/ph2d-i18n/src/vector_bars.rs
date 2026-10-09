//! **AS BARRAS DO VETOR** (D10 — `docs/UI_New_and_Simple/vetor_nova_ui/`): a coluna ①, a fila ②,
//! a barra ④ e a dica ⑤. Irmão do `vector.rs` pela razão do `chrome_menus.rs`: duas linhas que
//! acrescentem chaves ao mesmo `match` colidem no mesmo punhado de linhas.
//!
//! ⚠️ **As dicas (`panel.vector.hint.*`) dizem só o que o código FAZ** — cada oração foi medida no
//! despacho do gesto (handoff da linha UIUX de 07/10). Uma dica que ensina o que o app não faz é
//! pior que nenhuma.

/// A tradução de uma chave das barras do vetor, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "panel.vector.mode.eyedropper" => "Eyedropper",
        "panel.vector.hint.select" => {
            "Click: pick \u{00b7} Shift+click: add \u{00b7} Drag: move, scale or rotate"
        }
        "panel.vector.hint.node" => {
            "Click: point \u{00b7} Shift+click: add \u{00b7} Drag empty: marquee \u{00b7} Alt on a handle: break it"
        }
        "panel.vector.hint.pen" => {
            "Click: corner \u{00b7} Drag: curve \u{00b7} Alt: free handle \u{00b7} Esc: finish"
        }
        "panel.vector.hint.pencil" => "Drag: draw \u{00b7} Right-click or Esc: cancel",
        "panel.vector.hint.pencil_quickshape" => {
            "Drag: draw \u{00b7} Hold at the end: perfect shape \u{00b7} Right-click or Esc: cancel"
        }
        "panel.vector.hint.shape" => {
            "Drag: size \u{00b7} Shift: even \u{00b7} Alt: from the centre \u{00b7} Esc: cancel"
        }
        "panel.vector.hint.text" => "Click: place the cursor \u{00b7} Type \u{00b7} Esc: finish",
        "panel.vector.hint.build" => {
            "Shift+click: pick shapes \u{00b7} Drag: merge regions \u{00b7} Alt: remove \u{00b7} Esc: cancel"
        }
        "panel.vector.hint.connect" => "Press on a shape and drag to another: connect them",
        "panel.vector.hint.pickblend" => {
            "Click a closed shape: add it \u{00b7} Click again: remove it"
        }
        "panel.vector.hint.fillet" => "Drag a corner: round it",
        "panel.vector.hint.chamfer" => "Drag a corner: cut it straight",
        "panel.vector.hint.width" => {
            "Click the line: add a stop \u{00b7} Drag a stop: width \u{00b7} Right-click: delete it"
        }
        "panel.vector.hint.trim" => "Click the red piece: remove it",
        "panel.vector.hint.bucket" => "Click the lit region: fill it",
        "panel.vector.hint.cut" => "Click and drag a blade like the Pen \u{00b7} Esc: finish",
        "panel.vector.hint.frame" => {
            "Drag: a frame \u{00b7} Shift: even \u{00b7} Alt: from the centre"
        }
        "panel.vector.hint.eyedropper" => {
            "Click a shape: take its fill and stroke \u{00b7} the selection gets them too"
        }
        // As legendas por cima dos chips da fila (curtas: o gate de elisão mede-as).
        "panel.vector.bar.sub.fill" => "FILL",
        "panel.vector.bar.sub.stroke" => "LINE",
        "panel.vector.bar.sub.width" => "W",
        "panel.vector.bar.sub.fidelity" => "FID",
        "panel.vector.bar.sub.stabilizer" => "STAB",
        "panel.vector.bar.sub.segments" => "SEG",
        "panel.vector.bar.fill" => "Fill colour",
        "panel.vector.bar.stroke" => "Stroke colour",
        "panel.vector.bar.width" => "Stroke width",
        "panel.vector.bar.swap" => "Swap fill and stroke",
        "panel.vector.bar.default" => "Default colours",
        "panel.vector.bar.focus" => "Edit the other colour",
        "panel.vector.bar.symmetry" => "Symmetry",
        "panel.vector.bar.snap" => "Snap",
        "panel.vector.bar.snap_on" => "Snap \u{2713}",
        "panel.vector.bar.snap_off" => "Snap",
        "panel.vector.bar.snap.shapes" => "Snap to shapes",
        "panel.vector.bar.snap.path" => "Snap to paths",
        "panel.vector.bar.snap.crossings" => "Snap to crossings",
        "panel.vector.bar.snap.guides" => "Snap to guides",
        "panel.vector.bar.path" => "Path",
        "panel.vector.bar.readout.xy" => "x {x}  y {y}",
        "panel.vector.bar.readout.snap_on" => "snap on",
        "panel.vector.bar.readout.snap_off" => "snap off",
        "panel.vector.bar.readout.zoom" => "{z} %",
        "panel.vector.key.shift" => "Shift+{l}",
        "panel.vector.key.ctrl_alt" => "Ctrl+Alt+{l}",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
