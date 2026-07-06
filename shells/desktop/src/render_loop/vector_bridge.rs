//! Vector tool ⟷ shell bridge (ADR-0108 cutover).
//!
//! One per-frame job: reflect the [`VectorTool`](ph2d_tool_vector::VectorTool)
//! Style into the shell-held document + interaction state. The document
//! (`AppGfx.vec_scene`) and the Pen / History live on the shell (document ≠
//! tool); this bridge is the seam that reads the tool's Style panel:
//!
//! 1. **Style sync** — copy the tool's stroke / fill / width into the Pen so
//!    newly drawn paths honour the palette.
//! 2. **Recolour selected** — when a palette option was just chosen
//!    (`take_apply_to_selected`), recolour the currently selected path
//!    (undoable via the shell History).
//!
//! The concrete-tool downcast lives HERE (allowlisted:
//! `architecture_no_downcast_to_concrete_tool_in_shell`), so the central render
//! loop stays downcast-free — mirror of `painter_bridge`.

use ph2d_editor::{ToolId, ToolRegistry};
use ph2d_vec_edit::{History, PenStyle, PenTool};
use ph2d_vec_scene::{Rgba8, VecScene};

fn rgba(c: [u8; 4]) -> Rgba8 {
    Rgba8::new(c[0], c[1], c[2], c[3])
}

/// Per-frame Vector-tool plumbing. Safe to call every frame; a no-op when the
/// Vector tool is absent from the registry.
pub(super) fn dispatch(
    tools: &mut ToolRegistry,
    scene: &mut VecScene,
    pen: &mut PenTool,
    history: &mut History,
) {
    // The tool persists in the registry whether or not it is active, so its
    // Style survives tool switches (mirror of the painter bridge).
    let Some(tool) = tools
        .tool_by_id_mut(&ToolId::new("vector"))
        .and_then(|t| t.as_any_mut().downcast_mut::<ph2d_tool_vector::VectorTool>())
    else {
        return;
    };

    let stroke = tool.stroke_rgba();
    let fill = tool.fill_rgba();

    // 1. New paths honour the tool's Style.
    pen.set_style(PenStyle {
        stroke: rgba(stroke),
        stroke_w_px: tool.stroke_width_px(),
        fill: rgba(fill),
    });

    // 2. A palette pick recolours the selected path (undoable). Fill applies
    //    only to closed paths; a "None" fill (alpha 0) clears it.
    if tool.take_apply_to_selected()
        && let Some(sel) = pen.selected()
    {
        let pre = scene.clone();
        let mut changed = false;
        if let Some(path) = scene.path_mut(sel) {
            if let Some((_, w)) = path.stroke {
                path.stroke = Some((rgba(stroke), w));
                changed = true;
            }
            if path.closed {
                path.fill = if fill[3] == 0 { None } else { Some(rgba(fill)) };
                changed = true;
            }
        }
        if changed {
            history.push_undo(pre);
        }
    }
}
